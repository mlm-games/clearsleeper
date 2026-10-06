//! Warp lights - `fpcNm_WARPLIGHT_e`, `d_a_warpls.cpp`, stage strings `Ywarp00` and `Ysdls00`
//! (`d_stage.cpp:906-907`, both rows bound to the one profile name `fpcNm_WARPLIGHT_e`). These
//! are the MODEL names the two rows select, not a filename: `m_arcname[]` is
//! `{ "Ywarp00", "Ysdls00" }` (`d_a_warpls.cpp:14`).
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE. The class (`d_a_warpls.h:11-58`) holds
//! no `dCcD` member of any kind - `mPhs`, `mpModel`, `mpBrkAnm`, `mpBckAnm`, `mpEmitter`, the
//! switch and event scalars, nothing else - and the profile's `Group` is `fopAc_ACTOR_e`
//! (`d_a_warpls.cpp:401`). No collider is ever registered and neither `ChkAtHit` nor `ChkTgHit`
//! appears anywhere in the file, so it cannot be hit and cannot hurt. Neither
//! `combat::HitPoints` nor `enemy::Hurt` is attached.
//!
//! `daWarpls_IsDelete { return TRUE; }` (`:375-377`) only ACCEPTS a delete request per RULE 6b;
//! it is not a self-destruct. `_create` returns `cPhs_ERROR_e` only when
//! `fopAcM_entrySolidHeap` fails (`:157-159`), which is a resource allocation failure rather than
//! a refusal, so there is no `refused` flag.
//!
//! One simulation-visible quirk is transcribed rather than smoothed over: `Type_Tower` writes
//! `scale.z = 2.0f; scale.x = 2.0f` (`:99-100`), and both `check_warp_link` and
//! `check_warp_distance` compare against `m_warp_distance * scale.x` (`:297`, `:312`). A tower
//! warp light therefore has TWICE the trigger radius of a dungeon one.

use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::area::StageName;
use super::event::{ActiveEvent, EventList, StageEvents};
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::player::Player;

/// `d_stage.cpp:906-907` binds `Ywarp00` and `Ysdls00` to `fpcNm_WARPLIGHT_e`, the profile of
/// `d_a_warpls.cpp:387-402`.
pub const WARPLS_ACTORS: [&str; 2] = ["Ywarp00", "Ysdls00"];

/// `daWarpls_c::m_warp_distance = 112.5f` (`d_a_warpls.cpp:19`).
pub const WARP_DISTANCE: f32 = 112.5;

/// `mStartupDelayTimer = 10` when the light is created with its switch off
/// (`d_a_warpls.cpp:133`).
pub const STARTUP_DELAY: u8 = 10;

/// The `eventNames` table `dComIfGp_evmng_getEventIdx(eventNames[mEvtType], 0xFF)` indexes
/// (`d_a_warpls.cpp:76-80`, `:138`).
pub const EVENT_TOWER_UP: &str = "TOWER_WARP_U";
pub const EVENT_TOWER_DOWN: &str = "TOWER_WARP_D";
pub const EVENT_DUNGEON: &str = "DUNGEON_WARP";

/// `fopAcM_setCullSizeBox(this, -250.0f, 0.0f, -250.0f, 250.0f, 2600.0f, 250.0f)`
/// (`d_a_warpls.cpp:140`). The profile's `Cull Type` is `fopAc_CULLBOX_CUSTOM_e` (`:401`), so
/// this pair IS the box.
pub const CULL_MIN: [f32; 3] = [-250.0, 0.0, -250.0];
pub const CULL_MAX: [f32; 3] = [250.0, 2600.0, 250.0];

/// `fopAcM_setCullSizeFar(this, 1.0f)` (`d_a_warpls.cpp:141`).
pub const CULL_FAR: f32 = 1.0;

/// `!strcmp(dComIfGp_getStartStageName(), "Siren")` (`d_a_warpls.cpp:82`).
pub const SIREN_STAGE: &str = "Siren";

/// `if (room_no == 7)` / `else if (room_no == 17)` (`d_a_warpls.cpp:85`, `:87`).
const SIREN_TOWER_UP_ROOM: i8 = 7;
const SIREN_TOWER_DOWN_ROOM: i8 = 17;

/// `daWarpls_WarpType_e { Type_Tower, Type_Dungeon }` (`d_a_warpls.cpp:21`).
const WARP_TYPE_TOWER: u8 = 0;

/// `daWarpls_EvtType_e { EvtType_TowerUp, EvtType_TowerDown, EvtType_Dungeon }`
/// (`d_a_warpls.cpp:22`).
const EVT_TYPE_TOWER_UP: u8 = 0;
const EVT_TYPE_TOWER_DOWN: u8 = 1;
const EVT_TYPE_DUNGEON: u8 = 2;

/// `daWarpls_EvtState_e { EvtState_None, EvtState_Activation, EvtState_Warping }`
/// (`d_a_warpls.cpp:23`).
const EVT_STATE_NONE: u8 = 0;
const EVT_STATE_ACTIVATION: u8 = 1;
const EVT_STATE_WARPING: u8 = 2;

/// `mSwitchNo == 0xFF` (`d_a_warpls.cpp:113`, `:240`, `:267`). `daWarpls_prm::getSwitchNo`
/// masks with `0xFF` (`d_a_warpls.h:47`), so `0xFF` is reachable and is retail's "no switch"
/// sentinel. See the RULE 11b note on `warp_switch`.
const SWITCH_NONE: u8 = 0xFF;

/// `Type_Tower`: `scale.z = 2.0f; scale.x = 2.0f` (`d_a_warpls.cpp:99-100`).
const TOWER_SCALE: f32 = 2.0;

/// `fopAcM_seStart(this, JA_SE_OBJ_WARP_EFF_SUS, 0)` (`d_a_warpls.cpp:248`).
const SE_WARP_EFF_SUS: u32 = 0x7025;
/// `fopAcM_seStart(this, JA_SE_OBJ_WARP_EFF_APPEAR, 0)` (`d_a_warpls.cpp:332`).
const SE_WARP_EFF_APPEAR: u32 = 0x6967;
/// `fopAcM_seStart(this, JA_SE_LK_WAPR_EFF_WARP, 0)` (`d_a_warpls.cpp:209`).
const SE_WAPR_EFF_WARP: u32 = 0x285D;

/// `eventInfo.checkCommandDemoAccrpt()` (`d_a_warpls.cpp:195`) has no counterpart here: the demo
/// actor registry that `dDemo_setDemoData` feeds does not exist in this sim. The gate is
/// transcribed and fed `false`, which selects the non-demo path - the same treatment
/// `andsw0.rs` gives its identical `DEMO_ACCEPT: bool = false`.
pub const DEMO_ACCEPT: bool = false;

/// `daWarpls_prm::getType` = `(fopAcM_GetParam(this) >> 0x1C) & 0xFF` (`d_a_warpls.h:50`),
/// stored in `mWarpType` by `_create` (`:153`).
fn get_type(parameters: u32) -> u8 {
    ((parameters >> 0x1C) & 0xFF) as u8
}

/// `daWarpls_prm::getSwitchNo` = `fopAcM_GetParam(this) & 0xFF` (`d_a_warpls.h:47`), stored in
/// `mSwitchNo` (`:94`).
fn get_switch_no(parameters: u32) -> u8 {
    (parameters & 0xFF) as u8
}

/// `daWarpls_prm::getEvId` = `(fopAcM_GetParam(this) >> 0x10) & 0xFF` (`d_a_warpls.h:44`).
fn get_ev_id(parameters: u32) -> u8 {
    ((parameters >> 0x10) & 0xFF) as u8
}

/// `mActivationEvtIdx = dComIfGp_evmng_getEventIdx(NULL, daWarpls_prm::getEvId(this))`
/// (`d_a_warpls.cpp:137`). A `NULL` name makes retail resolve the id through the stage
/// event-info table, which is what `StageEvents` models here, so the slot is found by
/// `StageEvents::name(evId)` then `EventList::event_index`, exactly as `andsw0::event_slot`
/// does for the identical call in `d_a_andsw0.cpp:335`. `None` is retail's `-1`.
fn activation_slot(
    stage_events: Option<&StageEvents>,
    list: &EventList,
    ev_id: u8,
) -> Option<usize> {
    stage_events
        .and_then(|events| events.name(ev_id))
        .and_then(|name| list.event_index(name))
}

/// `if (!strcmp(dComIfGp_getStartStageName(), "Siren")) { ... } else { mEvtType =
/// EvtType_Dungeon; }` (`d_a_warpls.cpp:82-92`).
fn event_type(stage: &str, room: i8) -> u8 {
    if stage == SIREN_STAGE {
        match room {
            SIREN_TOWER_UP_ROOM => EVT_TYPE_TOWER_UP,
            SIREN_TOWER_DOWN_ROOM => EVT_TYPE_TOWER_DOWN,
            _ => EVT_TYPE_DUNGEON,
        }
    } else {
        EVT_TYPE_DUNGEON
    }
}

/// `eventNames[mEvtType]` (`d_a_warpls.cpp:76-80`, `:138`).
fn event_name(evt_type: u8) -> &'static str {
    match evt_type {
        EVT_TYPE_TOWER_UP => EVENT_TOWER_UP,
        EVT_TYPE_TOWER_DOWN => EVENT_TOWER_DOWN,
        _ => EVENT_DUNGEON,
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Warpls {
    /// `mWarpType` (`d_a_warpls.cpp:153`). Read at `:97` to select the archive table and at
    /// `:51` / `:62` / `:113` for the BRK and BCK presence.
    warp_type: u8,
    /// `mSwitchNo` (`d_a_warpls.cpp:94`), read by `fopAcM_isSwitch` at `:112`, `:174`, `:218`
    /// and `:254` and compared against `0xFF` at `:113`, `:240`, `:267`.
    switch_no: u8,
    /// `mEvtType` (`d_a_warpls.cpp:86`, `:91`). Read at `:138` and `:228`.
    evt_type: u8,
    /// `mEvtState` (`d_a_warpls.cpp:136`), read and written at `:197`, `:204`, `:220`, `:230`,
    /// `:263` and `:269`.
    evt_state: u8,
    /// `mWarpActive` (`:124`, `:133`), read and written at `:247`, `:263`, `:282`, `:291`,
    /// `:320` and `:337`.
    warp_active: bool,
    /// `mSkipActivationEvt` (`d_a_warpls.cpp:125`), read at `:263`.
    skip_activation_evt: bool,
    /// `mPlayerStartedInWarp` (`:145`), read and written at `:256`, `:258`, `:291`.
    player_started_in_warp: bool,
    /// `mStartupDelayTimer` (`d_a_warpls.cpp:126`, `:133`), decremented at `:176-178`.
    startup_delay: u8,
    /// `mActivationEvtIdx` (`d_a_warpls.cpp:137`), tested against `-1` at `:240` and `:263`.
    activation_idx: Option<usize>,
    /// `mWarpingEvtIdx = dComIfGp_evmng_getEventIdx(eventNames[mEvtType], 0xFF)` (`:138`).
    warping_idx: Option<usize>,
    /// Set on the first tick so `CreateInit` runs once, as `andsw0::Andsw0::create_init` does.
    entered: bool,
}

impl Warpls {
    /// `_create` (`:150-163`) then the parameter-independent half of `CreateInit`. The profile's
    /// `Parameters` field is `0` (`d_a_warpls.cpp:395`) and every read of the parameter is a
    /// `daWarpls_prm` accessor, so the whole u32 is decoded here. `mSceneNo` is deliberately not
    /// stored: `daWarpls_prm::getSceneNo` (`d_a_warpls.h:44` region) feeds only
    /// `dLib_setNextStageBySclsNum(mSceneNo, fopAcM_GetRoomNo(this))` at `:208`, which has no
    /// resource here - per RULE 7 an unread field is dead code.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            warp_type: get_type(parameters),
            switch_no: get_switch_no(parameters),
            evt_type: EVT_TYPE_DUNGEON,
            evt_state: EVT_STATE_NONE,
            warp_active: false,
            skip_activation_evt: false,
            player_started_in_warp: false,
            startup_delay: 0,
            activation_idx: None,
            warping_idx: None,
            entered: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn cull_far(&self) -> f32 {
        CULL_FAR
    }

    /// `fopAcM_isSwitch(this, mSwitchNo)` with the REAL index passed through, per RULE 11b:
    /// `super::flags::Switches` models only the in-memory 0..128 array, while retail routes
    /// `i_no >= 0x80` to the per-zone `dSv_zoneBit_c` arrays (`d_save.h:983-984`). A
    /// `mSwitchNo` of `0xFF` therefore reads false here even when its zone bit is set; that is
    /// the gap in the save model, not in this actor. Retail's own `|| mSwitchNo == 0xFF` term is
    /// kept separately at each site that has one.
    fn switch_on(&self, switches: &Switches) -> bool {
        switches.test(self.switch_no)
    }

    /// `Type_Tower`: `scale.z = 2.0f; scale.x = 2.0f` (`d_a_warpls.cpp:98-100`). `scale.y` is
    /// never written by this actor and stays at the 1.0 RULE 2 guarantees.
    fn apply_scale(&self, actor: &mut Actor) {
        if self.warp_type == WARP_TYPE_TOWER {
            actor.scale.x = TOWER_SCALE;
            actor.scale.z = TOWER_SCALE;
        }
    }

    /// `CreateInit` (`:75-147`).
    fn create_init(
        &mut self,
        actor: &mut Actor,
        stage: &str,
        switches: &Switches,
        list: &EventList,
        stage_events: Option<&StageEvents>,
        player: &Player,
        parameters: u32,
    ) {
        self.apply_scale(actor);

        self.evt_type = event_type(stage, actor.room);

        let is_switch_on = self.switch_on(switches);
        if is_switch_on || self.switch_no == SWITCH_NONE {
            self.warp_active = true;
            self.skip_activation_evt = true;
            self.startup_delay = 0;
        } else {
            self.startup_delay = STARTUP_DELAY;
        }

        self.evt_state = EVT_STATE_NONE;
        self.activation_idx = activation_slot(stage_events, list, get_ev_id(parameters));
        self.warping_idx = list.event_index(event_name(self.evt_type));

        if self.check_warp_distance(actor, player) {
            self.player_started_in_warp = true;
        }
    }

    /// `check_warp_distance()` (`:304-316`). The leading
    /// `if (player != daPy_getPlayerActorClass()) return FALSE;` (`:306`) tests that the player
    /// actor is Link's own class rather than a form change; `super::player::Player` carries no
    /// actor class, so that half is omitted and the distance test is transcribed whole.
    fn check_warp_distance(&self, actor: &Actor, player: &Player) -> bool {
        self.within(actor, player)
    }

    /// `(player->current.pos - current.pos).absXZ() < m_warp_distance * scale.x`
    /// (`:310-312`).
    fn within(&self, actor: &Actor, player: &Player) -> bool {
        let flat = (player.pos.x - actor.pos.x).hypot(player.pos.z - actor.pos.z);
        flat < WARP_DISTANCE * actor.scale.x
    }

    /// `check_warp_link()` (`:289-301`).
    fn check_warp_link(&self, actor: &Actor, player: &Player) -> bool {
        !self.player_started_in_warp && self.warp_active && self.within(actor, player)
    }

    /// `warp_eff_start()` (`:319-334`).
    fn warp_eff_start(&mut self, events: &mut SimEvents) {
        if self.warp_active {
            return;
        }
        events.list.push(SimEvent::Se(SE_WARP_EFF_APPEAR));
        self.warp_active = true;
    }

    /// `setStatus()` (`:246-250`).
    fn set_status(&self, events: &mut SimEvents) {
        if self.warp_active {
            events.list.push(SimEvent::Se(SE_WARP_EFF_SUS));
        }
    }

    /// `checkOrder()` (`:194-214`).
    fn check_order(
        &mut self,
        actor: &Actor,
        player: &Player,
        active: &mut ActiveEvent,
        events: &mut SimEvents,
        switch_on: bool,
        demo_accepted: bool,
    ) {
        if !demo_accepted {
            self.demo(actor, player, switch_on);
            return;
        }
        if let Some(slot) = self.activation_idx
            && active.start_check(slot)
            && self.evt_state == EVT_STATE_ACTIVATION
        {
            self.evt_state = EVT_STATE_NONE;
            self.warp_eff_start(events);
        }
        if let Some(slot) = self.activation_idx
            && active.end_check(slot)
        {
            // `dComIfGp_event_reset()` (`:201`) drops the running event.
            let _ = active.end(slot);
        }
        if let Some(slot) = self.warping_idx
            && active.start_check(slot)
            && self.evt_state == EVT_STATE_WARPING
        {
            self.evt_state = EVT_STATE_NONE;
        }
        if let Some(slot) = self.warping_idx
            && active.end_check(slot)
        {
            events.list.push(SimEvent::Se(SE_WAPR_EFF_WARP));
        }
    }

    /// `demo()` (`:253-286`).
    fn demo(&mut self, actor: &Actor, player: &Player, switch_on: bool) {
        if self.player_started_in_warp {
            if !self.check_warp_distance(actor, player) {
                self.player_started_in_warp = false;
            }
            return;
        }

        if self.evt_state == EVT_STATE_NONE
            && !self.skip_activation_evt
            && switch_on
            && !self.warp_active
            && self.activation_idx.is_some()
        {
            self.evt_state = EVT_STATE_ACTIVATION;
        }

        if switch_on || self.switch_no == SWITCH_NONE {
            if self.check_warp_link(actor, player) && self.evt_state == EVT_STATE_NONE {
                self.evt_state = EVT_STATE_WARPING;
            }
        } else {
            self.warp_active = false;
        }
    }

    /// `eventOrder()` (`:217-243`).
    fn event_order(
        &mut self,
        actor: &Actor,
        player: &Player,
        list: &EventList,
        active: &mut ActiveEvent,
        entity: Entity,
        events: &mut SimEvents,
        switch_on: bool,
    ) {
        if self.evt_state == EVT_STATE_ACTIVATION {
            self.order(list, active, self.activation_idx, entity);
            return;
        }
        if self.evt_state == EVT_STATE_WARPING {
            if self.evt_type == EVT_TYPE_TOWER_UP && !self.check_warp_distance(actor, player) {
                self.evt_state = EVT_STATE_NONE;
            } else {
                self.order(list, active, self.warping_idx, entity);
            }
            return;
        }
        if self.activation_idx.is_none() && (self.switch_no == SWITCH_NONE || switch_on) {
            self.warp_eff_start(events);
        }
    }

    /// `fopAcM_orderOtherEventId(...)` (`:221`, `:232`, `:235`). Bug class 5: re-ordering every tick
    /// resets `Stage::Order` and livelocks the demo, so the order is only issued when the slot is
    /// idle.
    fn order(&self, list: &EventList, active: &mut ActiveEvent, slot: Option<usize>, entity: Entity) {
        let Some(slot) = slot else {
            return;
        };
        if active.stage_of(slot).is_none() {
            active.order(list, slot, entity);
        }
    }
}

pub fn is_warpls(name: &str) -> bool {
    WARPLS_ACTORS.contains(&name)
}

pub fn control(
    stage: Res<StageName>,
    players: Query<&Player>,
    switches: Res<Switches>,
    list: Res<EventList>,
    stage_events: Option<Res<StageEvents>>,
    mut active: ResMut<ActiveEvent>,
    mut events: ResMut<SimEvents>,
    mut lights: Query<(Entity, &mut Actor, &mut Warpls)>,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    let stage_ref = stage_events.as_deref();
    for (entity, mut actor, mut light) in &mut lights {
        if !light.entered {
            light.entered = true;
            let parameters = actor.parameters;
            light.create_init(
                &mut actor,
                &stage.0,
                &switches,
                &list,
                stage_ref,
                player,
                parameters,
            );
        }
        let switch_on = light.switch_on(&switches);

        // `_execute()` (`:173-191`).
        if light.startup_delay > 0 {
            light.startup_delay -= 1;
        }
        light.check_order(
            &actor,
            player,
            &mut active,
            &mut events,
            switch_on,
            DEMO_ACCEPT,
        );
        light.event_order(
            &actor,
            player,
            &list,
            &mut active,
            entity,
            &mut events,
            switch_on,
        );
        light.set_status(&mut events);
    }
}
