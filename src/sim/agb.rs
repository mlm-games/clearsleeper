//! The AGB cursor - `fpcNm_AGB_e`, `d_a_agb.cpp`, the actor behind the stage string `Agb`.
//!
//! `daAgb_c` is the Tingle Tuner pointer. Every behaviour past `resetCursor` hangs off
//! `mDoGaC_GbaLink()` - the Game Boy Advance link cable - which this sim has no resource for, so
//! the only reachable outcome of `daAgb_Execute` (`:1658-1807`) is the un-linked arm and that is
//! what is transcribed:
//!
//! * `field_0x679 = false` (`:1661`), `CPad_GET_ERROR_STATUS(mDoGaC_getPortNo()) = 1` (`:1667`)
//!   and the `field_0x684.remove()` particle teardown (`:1726`) all have no modelled reader.
//! * `else if (i_this->field_0x680) { ... resetCursor(true); offActive(); ... }` (`:1708-1727`)
//!   runs on the FIRST tick only, because `daAgb_Create` seeds `field_0x680 = true` (`:1932`).
//!   `resetCursor` (`:739-773`) copies the player's `current.pos` and `home.pos`, zeroes the three
//!   `shape_angle` components, and calls `offFree()` (`:741`) / `setFollowTarget(false)` (`:742`).
//! * `modeProcCall` (`:1645-1655`) dispatches `mMode`, which `fopAcM_ct` leaves at 0 = `MODE_MOVE`,
//!   and `daAgb_Create` never writes - so `modeMove` (`:1436-1633`) is the whole per-tick body.
//! * In `modeMove`, `field_0x65c`, `field_0x66b`, `field_0x67d`, `field_0x67f` and `field_0x673`
//!   are written only by `GbaItemUse` (`:858-1104`), `FlagsRecv` (`:785-831`) and the
//!   `field_0x680` teardown (`:1713-1726`), every one of which is inside a `mDoGaC_GbaLink()` or
//!   `GbaItemUse` arm, so all five stay 0 / false.
//! * `field_0x673 && !r26 && field_0x65c == 0` (`:1484`) is therefore false, so `onActive()`
//!   (`:1485`) never fires and `isActive()` stays false. That gates off the `onFree()` at `:1615`,
//!   the attention lock-on at `:1563-1577`, and the `CursorMove` tail.
//! * `!isFree()` (`:1580`) holds every tick, so `getFollowTarget() == 0` (`:1589`) holds, and the
//!   non-`fpcNm_NPC_KAM_e` arm assigns `current.pos = player->current.pos` (`:1593`). That is the
//!   entire simulated effect of this actor.
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE. `daAgb_c` contains no `dCcD_Src*` literal
//! at all - `mAcch` / `mCrrPos` are `dBgS` ground-and-wall probes (RULE 3b), not actor colliders -
//! and the profile `Group` is `fopAc_ACTOR_e` (`:1967`). Neither `combat::HitPoints` nor
//! `enemy::Hurt` is attached; attaching them would let `combat::control` despawn an actor retail
//! cannot damage.
//!
//! `daAgb_IsDelete` (`:1849-1852`) is the bare "accepts a delete request" form of RULE 6b, never a
//! self-destruct. `daAgb_Create` (`:1896-1944`) returns `cPhs_ERROR_e` from exactly one place,
//! `if (!fopAcM_entrySolidHeap(a_this, createHeap_CB, 0x500))` (`:1903-1905`), which is heap work,
//! so `spawn` never refuses.

use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::player::Player;

/// `OBJNAME("Agb", fpcNm_AGB_e, 255, 0)` (`d_stage.cpp:1237`) is the only `fpcNm_AGB_e` row in the
/// table, so the single stage string is the whole claim. The object archive is `"Agb"`
/// (`dComIfG_getObjectRes("Agb", dRes_INDEX_AGB_BDL_AGBCURSOR_e)`, `d_a_agb.cpp:1871`), which
/// happens to match.
pub const AGB_ACTORS: [&str; 1] = ["Agb"];

/// `fopAc_CULLBOX_0_e` (`d_a_agb.cpp:1968`) resolves through `l_cullSizeBox[0]`
/// (`f_op_actor_mng.cpp:558`); this translation unit calls neither `fopAcM_setCullSizeBox` nor
/// `fopAcM_SetMin` / `SetMax`, so that table entry is the whole box.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// The AGB cursor. Unit struct: every field `daAgb_c` owns is written by the un-transcribed link
/// arms and read by `daAgb_Draw` / `mDoMtx_stack_c` (`:1778-1783`), so none survives RULE 12b.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Agb;

impl Agb {
    /// `daAgb_Create` (`:1896-1944`) - `fopAcM_ct` (`:1898`), `dComIfG_resLoad` (`:1900`),
    /// `dComIfGp_setAgb` (`:1902`), `fopAcM_entrySolidHeap` (`:1903`), the `mCrrPos` / `mAcch`
    /// setup (`:1907-1915`), the `TestDataManager` GBA DMA slots (`:1917-1920`),
    /// `fopAcM_setStageLayer` (`:1935`) and `eventInfo.setEventName("DEFAULT_AGB_USE")` (`:1936`)
    /// are all archive, heap, DMA, collision or event-name setup. The profile's `Parameters`
    /// field is `0` (`d_a_agb.cpp:1962`), and nothing in the translation unit reads
    /// `fopAcM_GetParam`, so there is no parameter to decode.
    pub fn spawn(_parameters: u32) -> Self {
        Agb
    }

    /// `fopAc_CULLBOX_0_e` (`d_a_agb.cpp:1968`) -> `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_agb(name: &str) -> bool {
    AGB_ACTORS.contains(&name)
}

pub fn control(players: Query<&Player>, mut cursors: Query<&mut Actor, With<Agb>>) {
    let Some(player) = players.iter().next() else {
        return;
    };
    for mut actor in &mut cursors {
        // `if (getFollowTarget() == 0) { if (fopAcM_GetName(player) == fpcNm_NPC_KAM_e) {
        // onFree(); } else { current.pos = player->current.pos; home.pos = player->current.pos;
        // } }` (`d_a_agb.cpp:1589-1595`). `getFollowTarget()` is false from `resetCursor`'s
        // `setFollowTarget(false)` (`:742`) and nothing in the un-linked arms sets it again, and
        // `home.pos` has no counterpart in this sim's `Actor`, so the surviving half is the
        // position copy. `fpcNm_NPC_KAM_e` is `d_a_npc_kamome` (`c_dylink.cpp:196`), the bird that
        // carries Link in the cutscene; the player of a normal stage is never it, so the `onFree()`
        // arm is not the live one.
        actor.pos = player.pos;
    }
}