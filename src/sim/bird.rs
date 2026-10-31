use bevy_ecs::prelude::*;

use super::flags::{Collect, SYMBOL_DIN, StoryBits};

/// The stage string is `Bm1` (`d_stage.cpp:716`, `fpcNm_NPC_BM1_e`). `daNpc_Bm1_c` is the Rito
/// NPC family - Quill, Skett, Akoot, Basht, Bisht, Hoskit, Ilari, Pashli, Namali and Kogoli
/// (`d_a_npc_bm1.cpp:2340-2473`).
pub const BIRD_ACTOR: &str = "Bm1";

/// `decideType` (`d_a_npc_bm1.cpp:2343-2361`) accepts the parameter's low byte for
/// `fpcNm_NPC_BM1_e` only for 0..=4; every other value returns false, and `_create`
/// (`:4075-4079`) then answers `cPhs_ERROR_e`, so retail never creates the actor.
const TYPE_MASK: u32 = 0xFF;
const TYPE_MAX: u8 = 4;

/// `decideType`'s condition byte (`d_a_npc_bm1.cpp:2435-2448`) is `(param >> 8) & 0xFF`.
const COND_SHIFT: u32 = 8;
const COND_MASK: u32 = 0xFF;

/// `dSv_event_flag_c::UNK_0001` (`d_save_event_flag.inc:13`), read by `init_PST_0`
/// (`d_a_npc_bm1.cpp:548`).
const EVENT_BIT_0001: u16 = 0x0001;
/// `dSv_event_flag_c::UNK_1102` (`d_save_event_flag.inc:132`), read by `init_PST_3`
/// (`d_a_npc_bm1.cpp:577`).
const EVENT_BIT_1102: u16 = 0x1102;
/// `dSv_event_flag_c::UNK_1A80` (`d_save_event_flag.inc:197`), read by `chk_appCnd`
/// (`d_a_npc_bm1.cpp:522`, `:529`, `:537`).
const EVENT_BIT_1A80: u16 = 0x1A80;
/// `dSv_event_flag_c::UNK_1E80` (`d_save_event_flag.inc:225`), read by `init_PST_4`
/// (`d_a_npc_bm1.cpp:587`).
const EVENT_BIT_1E80: u16 = 0x1E80;
/// `dSv_event_flag_c::UNK_1F40` (`d_save_event_flag.inc:234`), read by `init_PST_2`
/// (`d_a_npc_bm1.cpp:566`).
const EVENT_BIT_1F40: u16 = 0x1F40;

/// `init_PST_0` .. `init_PST_4` (`d_a_npc_bm1.cpp:546-596`) - the per-type creation gate. A
/// false result fails `createInit` (`:684-686`), and `_create` (`:4099-4101`) then answers
/// `cPhs_ERROR_e`.
fn type_ok(kind: u8, collect: &Collect, stories: &StoryBits) -> bool {
    match kind {
        0 => !stories.test(EVENT_BIT_0001),
        1 => true,
        2 => !stories.test(EVENT_BIT_1F40),
        3 => !collect.is_symbol(SYMBOL_DIN) && stories.test(EVENT_BIT_1102),
        4 => !stories.test(EVENT_BIT_1E80),
        _ => false,
    }
}

/// `chk_appCnd` (`d_a_npc_bm1.cpp:511-545`) - the gate the condition byte selects. `mSpawnCondition`
/// is 1, 2, 3 or 4 for the condition bytes 0, 1, 2 and 3 (`d_a_npc_bm1.cpp:2435-2448`) and 0 for
/// every other byte, and `mSpawnCondition` 0 always spawns.
fn cond_ok(cond: u8, collect: &Collect, stories: &StoryBits, day: bool) -> bool {
    match cond {
        0 => !collect.is_symbol(SYMBOL_DIN),
        1 => collect.is_symbol(SYMBOL_DIN) && !stories.test(EVENT_BIT_1A80),
        2 => collect.is_symbol(SYMBOL_DIN) && stories.test(EVENT_BIT_1A80) && day,
        3 => collect.is_symbol(SYMBOL_DIN) && stories.test(EVENT_BIT_1A80) && !day,
        _ => true,
    }
}

/// Whether retail creates this `Bm1` at all. `_create` (`d_a_npc_bm1.cpp:4060-4104`) fails with
/// `cPhs_ERROR_e` when the type byte is out of range (`:4077`), when `chk_appCnd` refuses
/// (`:684-686`), or when the per-type init refuses (`:4099-4101`), and a failed `_create` leaves
/// no actor behind.
///
/// Not modelled: the path byte `(param >> 16) & 0xFF` (`:696-704`), which makes retail fail the
/// creation when the named path does not exist, and the flight and idle actions, which are driven
/// by the HIO table this port does not carry.
pub fn spawn_ok(parameters: u32, collect: &Collect, stories: &StoryBits, day: bool) -> bool {
    let kind = (parameters & TYPE_MASK) as u8;
    if kind > TYPE_MAX {
        return false;
    }
    let cond = ((parameters >> COND_SHIFT) & COND_MASK) as u8;
    type_ok(kind, collect, stories) && cond_ok(cond, collect, stories, day)
}

/// `daNpc_Bm1_c` is an NPC, not an enemy. It creates `dNpc_cyl_src` (`d_a_npc_bm1.cpp:774`),
/// whose `SrcObjAt Type` is `0` and whose radius and height are both `0`
/// (`d_npc.cpp:956-984`), so a Rito has no attack collider and can never hurt the player. No NPC
/// in the reference sets `health` or `max_health`, so neither [`super::combat::HitPoints`] nor
/// [`super::enemy::Hurt`] belongs on one either - a Rito cannot be locked on to and cannot be
/// killed.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bird {
    /// `mSpecificType` (`d_a_npc_bm1.h:341`), the parameter's low byte.
    pub kind: u8,
    /// `m881` (`d_a_npc_bm1.h:289`), set by `setTelescopeDemo()` (`:65`). Aryll's scope raises it
    /// when the scope's message mode opens (`d_a_npc_ls1.cpp:1814`).
    telescope_demo: bool,
    /// `m882` (`d_a_npc_bm1.h:290`), set by `setFocus()` (`:64`). Aryll raises it when the zoom
    /// reaches the focus threshold (`d_a_npc_ls1.cpp:1861`).
    focus: bool,
    /// `m880` (`d_a_npc_bm1.h:288`), the surprise Aryll's scope conversation waits for through
    /// `getOdoroki()` (`:61`, read at `d_a_npc_ls1.cpp:1874`). It is only ever set, by the status
    /// 3 arm of `setStt` (`d_a_npc_bm1.cpp:3035`), so it latches.
    odoroki: bool,
}

impl Bird {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: (parameters & TYPE_MASK) as u8,
            telescope_demo: false,
            focus: false,
            odoroki: false,
        }
    }

    /// `setTelescopeDemo()` (`d_a_npc_bm1.h:65`).
    pub fn set_telescope_demo(&mut self) {
        self.telescope_demo = true;
    }

    /// `setFocus()` (`d_a_npc_bm1.h:64`).
    pub fn set_focus(&mut self) {
        self.focus = true;
    }

    /// `getOdoroki()` (`d_a_npc_bm1.h:61`).
    pub fn get_odoroki(&self) -> bool {
        self.odoroki
    }
}

pub fn is_bird(name: &str) -> bool {
    name == BIRD_ACTOR
}

/// `d_wait` leaving for `lookup` (`d_a_npc_bm1.cpp:3090-3096`) and `lookup`'s status 3
/// (`:3098-3104`, `:3033-3037`): once the scope has both raised the telescope demo and focused
/// on him, the Rito turns to look and the surprise bit latches. `d_wait` also raises
/// `fopAcM_SetStatusMap(this, 7)` (`:3093`), the no-draw status the port has no shell reader for.
pub fn control(mut birds: Query<&mut Bird>) {
    for mut bird in &mut birds {
        if bird.telescope_demo && bird.focus && !bird.odoroki {
            bird.odoroki = true;
        }
    }
}
