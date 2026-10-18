use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, Stage, StageEvents};
use super::flags::EventFlags;
use super::medli::{Medli, STTS_CAM_TAG_IN, STTS_FLY};

/// The stage string is `TagMd` (`d_stage.cpp:580`, `fpcNm_TAG_ETC_e`); `daTag_Etc_c` has no
/// `_arcname` - it never calls `dComIfG_resLoad` (`d_a_tag_etc.cpp:101-130`) and holds no model.
pub const TAGETC_ACTORS: [&str; 1] = ["TagMd"];

/// `enum Action { ACT_WAIT, ACT_SEARCH, ACT_HUNT, ACT_READY, ACT_EVENT }`
/// (`d_a_tag_etc.cpp:11-17`) - the decomp's own order, which `l_action[]` (`:207-213`) mirrors.
const ACT_WAIT: u8 = 0; // <- d_a_tag_etc.cpp:12
const ACT_SEARCH: u8 = 1; // <- d_a_tag_etc.cpp:13
const ACT_HUNT: u8 = 2; // <- d_a_tag_etc.cpp:14
const ACT_READY: u8 = 3; // <- d_a_tag_etc.cpp:15
const ACT_EVENT: u8 = 4; // <- d_a_tag_etc.cpp:16

/// `getType2()`: `fopAcM_GetParam(this) >> 8 & 0xF` (`d_a_tag_etc.cpp:26`), narrowed into the `u8`
/// return type of `:25`.
const TYPE2_SHIFT: u32 = 0x8; // <- d_a_tag_etc.cpp:26
const TYPE2_MASK: u32 = 0xF; // <- d_a_tag_etc.cpp:26

/// `getEventNo()`: `fopAcM_GetParam(this) >> 24` (`d_a_tag_etc.cpp:21`), narrowed into the `u8`
/// return type of `:20`. It is the `eventInfoIdx` of `dComIfGp_evmng_getEventIdx(NULL,
/// stageEVNTListIndex)` (`:107-108`).
const EVENT_NO_SHIFT: u32 = 24; // <- d_a_tag_etc.cpp:21

/// `rangeCheck`'s `delta.absXZ() < (scale.x * 100.0f)` and `delta.y <= (scale.y * 100.0f)`
/// (`d_a_tag_etc.cpp:38`). `dStage_actorInit` never writes `fopAcM_prm_class::scale`, so a
/// stage-placed actor is exactly 1.0 (RULE 2) and both products fold to the bare literals.
const RANGE_XZ: f32 = 100.0; // <- d_a_tag_etc.cpp:38 (scale.x * 100.0f, scale.x == 1.0)
const RANGE_Y: f32 = 100.0; // <- d_a_tag_etc.cpp:38 (scale.y * 100.0f, scale.y == 1.0)

/// `field_0x29A = 15` (`d_a_tag_etc.cpp:95`), the `short field_0x29A` (`d_a_tag_etc.h:16`)
/// countdown `demoProc` drains before it calls `cutEnd` (`:74-77`).
const DEMO_HOLD: i16 = 15; // <- d_a_tag_etc.cpp:95

/// `dComIfGp_evmng_getMyStaffId("TAG_ETC_D", NULL, 0)` (`d_a_tag_etc.cpp:68`) and its two
/// `dComIfGp_evmng_cutEnd(staffIdx)` calls (`:77`, `:82`).
const STAFF_NAME: &str = "TAG_ETC_D"; // <- d_a_tag_etc.cpp:68

/// `fopAc_CULLBOX_6_e` (`d_a_tag_etc.cpp:267`) resolves through `l_cullSizeBox[6]`
/// (`f_op_actor_mng.cpp:564`) - a real box, so `cull()` is derivable.
const CULL_MIN: [f32; 3] = [-150.0, 0.0, -150.0]; // <- f_op_actor_mng.cpp:564 (fopAc_CULLBOX_6_e)
const CULL_MAX: [f32; 3] = [150.0, 300.0, 150.0]; // <- f_op_actor_mng.cpp:564 (fopAc_CULLBOX_6_e)

/// The companion-camera marker - `daTag_Etc_c` (`d_a_tag_etc.h:9-21`), the actor behind the stage
/// string `TagMd`. It finds the `Md1` companion, keeps the companion's camera-tag bit latched
/// while it is close by and flying, and holds the `TAG_ETC_D` staff cut for 15 ticks once the
/// companion is no longer flying.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daTag_Etc_c` registers no `dCcD` collider
/// of any kind - the class holds only `mAction`, three padding bytes, `mMedliPID`, `mEventIdx` and
/// `field_0x29A` (`d_a_tag_etc.h:14-17`) - and the profile `Group` is `fopAc_ACTOR_e` (`:266`).
/// Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `daTag_Etc_c::draw()` (`:202-204`) is `return TRUE` with no draw call and the class holds no
/// model, so retail renders nothing at all: no `actor_views` branch is derivable.
///
/// `daTag_Etc_IsDelete` (`:229-232`) is the bare "accepts a delete request" form of RULE 6b, and
/// `create` (`:101-130`) always returns `cPhs_COMPLEATE_e`, so there is no spawn-then-despawn path.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagEtc {
    /// `mAction` (`d_a_tag_etc.h:14`), the `l_action[]` index `execute` dispatches on (`:207-214`).
    pub action: u8,
    /// `mMedliPID` (`d_a_tag_etc.h:15`), the `fpc_ProcID` `action_search` latches at `:147` and
    /// that `fopAcM_SearchByID` re-reads at `:67`, `:169` and `:186`.
    /// `fpcM_ERROR_PROCESS_ID_e` is `-1` (`f_pc_manager.h:11`), which is what `:157`, `:175` and
    /// `:188` store; `None` is that `-1`.
    pub medli: Option<Entity>,
    /// `mEventIdx` (`d_a_tag_etc.h:16`), the `getEventIdx(NULL, getEventNo())` slot; `None` is its
    /// `-1` (`:109`).
    pub event_slot: Option<usize>,
    /// `field_0x29A` (`d_a_tag_etc.h:17`), the `demoProc` hold counter (`:74-75`).
    pub hold: i16,
    /// `getType2()` (`:26`), the only discriminator every action switches on.
    pub kind: u8,
    /// One-shot guard so the `mEventIdx` lookup of `create` (`:107-108`) and the `current.angle`
    /// stores (`:124-125`) happen once.
    pub entered: bool,
}

impl TagEtc {
    /// `daTag_Etc_c::create` (`:101-130`), minus `fopAcM_ct` (`:105`), minus the `mEventIdx` lookup
    /// (`:107-108`, deferred to the first tick because it needs the stage event list), and minus the
    /// attention block (`:126-128`), which is the attention/catch request state RULE 3b lists as
    /// absent. What survives is the parameter decode; the shape-angle zeroing of `:122-123` is
    /// carried by [`TagEtc::create_init`], which runs on the first tick for the same reason.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            // `create` cannot pick the action until `mEventIdx` resolves, so the struct starts in
            // `ACT_WAIT` and `create_init` re-decides on the first tick.
            action: ACT_WAIT,
            medli: None,
            event_slot: None,
            hold: 0,
            kind: ((parameters >> TYPE2_SHIFT) & TYPE2_MASK) as u8,
            entered: false,
        }
    }

    /// `fopAc_CULLBOX_6_e` (`d_a_tag_etc.cpp:267`) -> `l_cullSizeBox[6]`
    /// (`f_op_actor_mng.cpp:564`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daTag_Etc_c::rangeCheck` (`:30-42`): `delta = pActor->current.pos - current.pos`, a target
    /// below the marker fails outright, and the marker claims it only inside the scaled box.
    /// `cXyz::absXZ` (`c_xyz.h:126`) is the XZ-only distance.
    fn range_check(&self, here: Vec3, there: Vec3) -> bool {
        let delta = there - here;
        if delta.y < 0.0 {
            return false;
        }
        let flat = delta.x * delta.x + delta.z * delta.z;
        flat < RANGE_XZ * RANGE_XZ && delta.y <= RANGE_Y
    }

    /// `daTag_Etc_c::otherCheck` (`:45-61`): for `type2 == 0` the target must be flying, every
    /// other value passes. `checkStatusFly` is `cLib_checkBit<u32>(m30F0, daMdStts_FLY)`
    /// (`d_a_npc_md.h:60`) with `daMdStts_FLY = 0x00000010` (`:47`).
    fn other_check(&self, flying: bool) -> bool {
        match self.kind {
            0 => flying,
            _ => true,
        }
    }

    /// `daTag_Etc_c::demoInitProc` (`:88-98`), whose only arm is `type2 == 0`.
    /// `dComIfGp_event_setItemPartner(pActor)` (`:94`) writes the running event's `mPtItem`
    /// (`d_event.h:113`), and `getPtI()` has no reader anywhere in the decomp, so it is a dead
    /// store and is dropped. `field_0x29A = 15` survives.
    fn demo_init(&mut self) {
        self.hold = DEMO_HOLD;
    }

    /// `daTag_Etc_c::demoProc` (`:64-85`). `getMyStaffId("TAG_ETC_D", NULL, 0)` (`:68`) returns the
    /// first staff of that name on any event in PLAY / CLOSE / UNK3
    /// (`d_event_manager.cpp:396-422`); its `-1` disables the whole body. `cutEnd` is
    /// `dEvent_manager_c::cutEnd` (`:549-555`), which sets the current cut's flag id -
    /// `ActiveEvent::end_staff_cut`. For `type2 == 0` the countdown only runs while the tracked
    /// `Md1` is gone or not flying; every other `type2` cuts immediately.
    fn demo_proc(
        &mut self,
        flying: Option<bool>,
        list: &EventList,
        active: &mut ActiveEvent,
        flags: &mut EventFlags,
    ) {
        if active.staff_cut(list, STAFF_NAME).is_none() {
            return;
        }
        match self.kind {
            0 => {
                if flying != Some(true) {
                    if self.hold > 0 {
                        self.hold -= 1;
                    } else {
                        active.end_staff_cut(list, flags, STAFF_NAME);
                    }
                }
            }
            _ => active.end_staff_cut(list, flags, STAFF_NAME),
        }
    }

    /// `daTag_Etc_action_wait` (`:133-135`) is a bare `return TRUE`: it stores and reads nothing.
    fn action_wait(&mut self) {}

    /// `daTag_Etc_action_search` (`:138-152`):
    /// `actor = fopAcM_searchFromName("Md1", 0, 0)` (`:142`) - the only arm is `type2 == 0` -
    /// then latch its id and move to `ACT_HUNT`.
    fn action_search(&mut self, found: Option<Entity>) {
        if self.kind == 0
            && let Some(entity) = found
        {
            self.medli = Some(entity);
            self.action = ACT_HUNT;
        }
    }

    /// `daTag_Etc_action_hunt` (`:185-200`). A lost target drops straight back to `ACT_SEARCH`;
    /// otherwise the `Md1`'s camera-tag bit follows the two range checks.
    /// `onBitCamTagIn` / `offBitCamTagIn` are `cLib_onBit` / `cLib_offBit<u32>(m30F0,
    /// daMdStts_CAM_TAG_IN)` (`d_a_npc_md.h:62-63`) with `daMdStts_CAM_TAG_IN = 0x00000020`
    /// (`:48`), so this returns the bit value the caller must install.
    fn action_hunt(&mut self, here: Vec3, tracked: Option<(Vec3, bool)>) -> Option<bool> {
        let Some((pos, flying)) = tracked else {
            self.medli = None;
            self.action = ACT_SEARCH;
            return None;
        };
        Some(self.range_check(here, pos) && self.other_check(flying))
    }

    /// `daTag_Etc_action_ready` (`:168-182`). The demo-accept test is
    /// `eventInfo.checkCommandDemoAccrpt()` (`:170`) = `mCommand == dEvtCmd_INDEMO_e`
    /// (`f_op_actor.h:177`), a latch `dEvt_control_c::demoCheck` sets on both ordered actors when
    /// the event starts (`d_event.cpp:314-316`) and `demoEnd` clears (`:331-337`) - so, exactly as
    /// `htobi3.rs` argues, "this actor's event is in `Stage::Play` or `Stage::Close`", the two
    /// states `dEvent_manager_c::getMyStaffId` scans (`d_event_manager.cpp:412`).
    /// `eventInfo.onCondition(dEvtCnd_UNK2_e)` (`:171`) is dropped for `htobi3.rs`'s reason: nothing
    /// in the decomp ever calls `chkCondition(dEvtCnd_UNK2_e)`, and `beforeProc` clears `mCondition`
    /// on every `fopAc_Execute` (`f_op_actor.cpp:176`).
    ///
    /// NOTE: no `setActio(ACT_READY)` exists anywhere in `d_a_tag_etc.cpp`, so the vendored build
    /// never enters this action. It is ported verbatim because the decomp contains it.
    fn action_ready(
        &mut self,
        here: Vec3,
        tracked: Option<(Vec3, bool)>,
        list: &EventList,
        active: &mut ActiveEvent,
        flags: &mut EventFlags,
        entity: Entity,
    ) {
        if self.demo_running(active) {
            if self.kind == 0 {
                self.demo_init();
            }
            self.action = ACT_EVENT;
            self.action_event(tracked.map(|(_, flying)| flying), list, active, flags);
            return;
        }
        let held = tracked
            .is_some_and(|(pos, flying)| self.range_check(here, pos) && self.other_check(flying));
        if !held {
            self.medli = None;
            self.action = ACT_SEARCH;
            return;
        }
        self.order(list, active, entity);
    }

    /// `daTag_Etc_action_event` (`:155-165`): `endCheck` releases the target and the event, else
    /// `demoProc`. `dComIfGp_event_reset()` (`:159`) is `dEvDtBase_c::reset()` (`d_event.h:131`)
    /// = `mEventFlag |= 8` on the running event, which `dEvt_control_c::check`
    /// (`d_event.cpp:546-554`) turns into `endProc`; the sim's `ActiveEvent::end` is that teardown.
    fn action_event(
        &mut self,
        flying: Option<bool>,
        list: &EventList,
        active: &mut ActiveEvent,
        flags: &mut EventFlags,
    ) {
        if self.event_slot.is_some_and(|slot| active.end_check(slot)) {
            self.medli = None;
            self.action = ACT_SEARCH;
            if let Some(slot) = self.event_slot {
                let _ = active.end(slot);
            }
            return;
        }
        self.demo_proc(flying, list, active, flags);
    }

    /// `fopAcM_orderOtherEventId(this, mEventIdx)` (`:178`) - guarded as in `vfan::Vfan::order`,
    /// because re-ordering an event that is already ORDER / PLAY / CLOSE rewrites its state
    /// (`d_event_manager.cpp:367`) and would reset `Stage::Order` on every tick.
    fn order(&self, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        let Some(slot) = self.event_slot else {
            return;
        };
        if active.stage_of(slot).is_none() {
            let _ = active.order(list, slot, entity);
        }
    }

    fn demo_running(&self, active: &ActiveEvent) -> bool {
        self.event_slot.is_some_and(|slot| {
            matches!(
                active.stage_of(slot),
                Some(Stage::Play) | Some(Stage::Close)
            )
        })
    }

    /// The `mEventIdx` lookup and the action choice of `create` (`:107-120`):
    /// `dComIfGp_evmng_getEventIdx(NULL, stageEVNTListIndex)` resolves through the stage event-info
    /// table at `d_event_manager.cpp:347-348` - which `super::event::StageEvents` is - and falls to
    /// `-1` when that table is absent or shorter (`d_event_manager.cpp:350-351`). `create` arms
    /// `ACT_SEARCH` only when the index resolved *and* `type2 == 0`.
    fn create_init(
        &mut self,
        parameters: u32,
        actor: &mut Actor,
        stage: Option<&StageEvents>,
        list: &EventList,
    ) {
        // `shape_angle.z = 0; shape_angle.x = 0; current.angle.z = 0; current.angle.x = 0;`
        // (`:122-125`). `Actor::angle` is `current.angle` and `Actor::shape_angle` is
        // `shape_angle`; both are transcribed. The reference's `shape_angle` starts as the
        // placement angle (`f_op_actor.cpp:274`) and this port starts it at zero, so on the first
        // tick the two writes below are idempotent here - they are the marker's own state all the
        // same, and this module is the only writer for it.
        actor.shape_angle[2] = 0;
        actor.shape_angle[0] = 0;
        actor.angle[2] = 0;
        actor.angle[0] = 0;
        let event_no = (parameters >> EVENT_NO_SHIFT) as u8;
        self.event_slot = stage
            .and_then(|events| events.name(event_no))
            .and_then(|name| list.event_index(name));
        if self.event_slot.is_some() && self.kind == 0 {
            self.action = ACT_SEARCH;
        }
        self.entered = true;
    }

    /// `daTag_Etc_c::execute` (`:206-216`). `l_action[]` has five entries (`:207-213`) and
    /// `mAction` is a `u8`, so `ACT_WAIT` is the `default` arm and every index is in range.
    ///
    /// `here` is the marker's `current.pos`, which `rangeCheck` needs; `found` is
    /// `fopAcM_searchFromName("Md1", 0, 0)`'s result; `tracked` is `fopAcM_SearchByID(mMedliPID)`'s
    /// `(current.pos, checkStatusFly())`. The return value is the camera-tag bit `action_hunt`
    /// wants installed on the `Md1`.
    fn execute(
        &mut self,
        here: Vec3,
        found: Option<Entity>,
        tracked: Option<(Vec3, bool)>,
        list: &EventList,
        active: &mut ActiveEvent,
        flags: &mut EventFlags,
        entity: Entity,
    ) -> Option<bool> {
        match self.action {
            ACT_SEARCH => {
                self.action_search(found);
                None
            }
            ACT_HUNT => self.action_hunt(here, tracked),
            ACT_READY => {
                self.action_ready(here, tracked, list, active, flags, entity);
                None
            }
            ACT_EVENT => {
                self.action_event(tracked.map(|(_, flying)| flying), list, active, flags);
                None
            }
            _ => {
                self.action_wait();
                None
            }
        }
    }
}

pub fn is_tagetc(name: &str) -> bool {
    TAGETC_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    stage: Option<Res<StageEvents>>,
    mut active: ResMut<ActiveEvent>,
    mut flags: ResMut<EventFlags>,
    // `With<Medli>` / `Without<Medli>` is the disjoint pair RULE 9 requires: this system writes
    // `Actor` for the markers and reads it for the companion, and only a `Md1` carries a `Medli`.
    mut medlis: Query<(Entity, &Actor, &mut Medli), With<Medli>>,
    mut tags: Query<(Entity, &mut Actor, &mut TagEtc), Without<Medli>>,
) {
    let stage_ref = stage.as_deref();
    // `fopAcM_searchFromName("Md1", 0, 0)` (`d_a_tag_etc.cpp:142`) is `fopAcM_Search` over the
    // process list (`f_op_actor_mng.cpp:1428-1434`), so it yields the first live `Md1`; the sim's
    // first query row is the same choice `agbsw0.rs` makes.
    let first = medlis
        .iter_mut()
        .next()
        .map(|(entity, actor, medli)| (entity, actor.pos, medli.has_stts(STTS_FLY)));
    let mut cam_tag: Option<bool> = None;
    for (entity, mut actor, mut tag) in &mut tags {
        let actor = &mut *actor;
        let tag = &mut *tag;
        if !tag.entered {
            let parameters = actor.parameters;
            tag.create_init(parameters, actor, stage_ref, &list);
        }
        // `fopAcM_SearchByID(mMedliPID)` (`:67`, `:169`, `:186`) resolves only the id
        // `action_search` latched, so an `Md1` that is no longer first is not a match.
        let tracked = tag
            .medli
            .zip(first)
            .filter(|(id, (found, _, _))| id == found)
            .map(|(_, (_, pos, flying))| (pos, flying));
        if let Some(want) = tag.execute(
            actor.pos,
            first.map(|(entity, _, _)| entity),
            tracked,
            &list,
            &mut active,
            &mut flags,
            entity,
        ) {
            cam_tag = Some(want);
        }
    }
    // `onBitCamTagIn` / `offBitCamTagIn` (`d_a_tag_etc.cpp:194`, `:196`) on the tracked `Md1`.
    if let Some(want) = cam_tag
        && let Some((_, _, mut medli)) = medlis.iter_mut().next()
    {
        if want {
            medli.set_stts(STTS_CAM_TAG_IN);
        } else {
            medli.clr_stts(STTS_CAM_TAG_IN);
        }
    }
}
