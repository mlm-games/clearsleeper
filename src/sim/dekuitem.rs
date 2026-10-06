use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, Stage};
use super::events::{SimEvent, SimEvents};
use super::flags::EventFlags;
use super::player::Player;

/// The stage string is `itemDek` (`d_stage.cpp:649`, `fpcNm_DEKU_ITEM_e`). It differs from
/// `daDekuItem_c::m_arcname`, which is `Deku` (`d_a_deku_item.cpp:11`) - the OBJNAME row is the
/// stage name, so `itemDek` is what this array carries, not the archive name.
pub const DEKUITEM_ACTORS: [&str; 1] = ["itemDek"];

/// `enum Mode_e { Mode_WAIT_e, Mode_GETDEMO_INIT_e, Mode_GETDEMO_WAIT_e, Mode_GETDEMO_e }`
/// (`d_a_deku_item.h:22-27`) - the decomp's own order, which `mode_proc[]`
/// (`d_a_deku_item.cpp:160-165`) mirrors.
const MODE_WAIT: u8 = 0; // <- d_a_deku_item.cpp:161
const MODE_GETDEMO_INIT: u8 = 1; // <- d_a_deku_item.cpp:162
const MODE_GETDEMO_WAIT: u8 = 2; // <- d_a_deku_item.cpp:163
const MODE_GETDEMO: u8 = 3; // <- d_a_deku_item.cpp:164

/// `daDekuItem_prm::getItemBitNo(ac)` = `(fopAcM_GetParam(ac) >> 0) & 0xFF`
/// (`d_a_deku_item.h:34-36`).
const ITEM_BIT_SHIFT: u32 = 0; // <- d_a_deku_item.h:35
const ITEM_BIT_MASK: u32 = 0xFF; // <- d_a_deku_item.h:35

/// `if (!dComIfGs_isEventBit(dSv_event_flag_c::UNK_1801)) return cPhs_ERROR_e;`
/// (`d_a_deku_item.cpp:116-118`). `dSv_event_flag_c::UNK_1801 = 0x1801`
/// (`d_save_event_flag.inc:188`), so under RULE 11c this is byte index `0x18`, bit mask `0x01`.
const EVENT_BIT: u32 = 0x1801; // <- d_a_deku_item.cpp:116

/// `l_cyl_src`'s `cM3dGCylS` (`d_a_deku_item.cpp:36-40`) - `Center {0,0,0}`, `Radius 50.0f`,
/// `Height 100.0f`; `mCyl.SetC(current.pos)` (`:183`) and `dComIfG_Ccsp()->Set(&mCyl)` (`:184`)
/// place the axis at the leaf.
const CYL_RADIUS: f32 = 50.0; // <- d_a_deku_item.cpp:38
const CYL_HEIGHT: f32 = 100.0; // <- d_a_deku_item.cpp:39

/// `fopAcM_setCullSizeBox(this, -50.0f, 0.0f, -50.0f, 50.0f, 100.0f, 50.0f)`
/// (`d_a_deku_item.cpp:92`) - a real custom cull box, so the profile's `fopAc_CULLBOX_CUSTOM_e`
/// (`:297`) resolves.
const CULL_MIN: [f32; 3] = [-50.0, 0.0, -50.0]; // <- d_a_deku_item.cpp:92
const CULL_MAX: [f32; 3] = [50.0, 100.0, 50.0]; // <- d_a_deku_item.cpp:92

/// `fopAcM_createItemForTrBoxDemo(&current.pos, dItemNo_DEKU_LEAF_e, -1, fopAcM_GetRoomNo(this))`
/// (`d_a_deku_item.cpp:203`) with `dItemNo_DEKU_LEAF_e = 0x34` (`d_item_data.h:59`). That reaches
/// `fopAcM_createDemoItem` (`f_op_actor_mng.cpp:945-953`), whose
/// `params = (i_itemNo & 0xFF) << 0 | (i_itemBitNo & 0x7F) << 8 | (i_argFlag & 0xFF) << 16`
/// (`:950`) with `i_itemBitNo = -1` (the header's third argument, defaulted at
/// `f_op_actor_mng.h:568`) and `i_argFlag = 0x00` (`f_op_actor_mng.cpp:801`), and creates
/// `fpcNm_Demo_Item_e` - stage string `Ditem` (`d_stage.cpp:1257`).
pub const ITEM_NO: u8 = 0x34; // <- d_item_data.h:59 (dItemNo_DEKU_LEAF_e)
const ITEM_BIT_PARAM: u32 = 0x7F; // <- f_op_actor_mng.cpp:950 ((-1) & 0x7F)
const ITEM_PARAM: u32 = ITEM_NO as u32 | (ITEM_BIT_PARAM << 8); // <- f_op_actor_mng.cpp:950
const DEMO_ITEM_ACTOR: &str = "Ditem"; // <- d_stage.cpp:1257 (fpcNm_Demo_Item_e)

/// `fopAcM_orderOtherEvent(this, "DEFAULT_GETITEM")` (`d_a_deku_item.cpp:219`),
/// `dComIfGp_evmng_startCheck("DEFAULT_GETITEM")` (`:227`) and
/// `dComIfGp_evmng_endCheck("DEFAULT_GETITEM")` (`:231`).
const EVENT_NAME: &str = "DEFAULT_GETITEM"; // <- d_a_deku_item.cpp:219, :227, :231

/// `fopAcM_seStart(this, JA_SE_OBJ_DEKU_HA_LIGHT, 0)` (`d_a_deku_item.cpp:182`), played on every
/// tick `mode_wait` runs. `JA_SE_OBJ_DEKU_HA_LIGHT = 0x7052` (`JAZelAudio_SE.h:2164`).
const SE_OBJ_DEKU_HA_LIGHT: u32 = 0x7052; // <- JAZelAudio_SE.h:2164

/// The Deku Leaf pickup - `daDekuItem_c` (`d_a_deku_item.h:11-42`), the actor behind the stage
/// string `itemDek`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `l_cyl_src` (`d_a_deku_item.cpp:13-41`)
/// sets `SrcObjAt Type = 0` (`:16`), `SrcObjAt Atp = 0` (`:17`) and `SrcObjAt SPrm = 0` (`:18`), so
/// the leaf owns no AT collider and cannot hurt anything; its `SrcObjTg Type = AT_TYPE_ALL` (`:20`)
/// is a pure trigger - it *senses* every attack and routes nothing through it - and its
/// `SrcObjCo SPrm` (`:22`) is only the solid the player's body bumps. The profile `Group` is
/// `fopAc_ACTOR_e` (`:296`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `_create` can refuse creation twice: `fopAcM_isItem(this, mItemBitNo)` (`:112-114`) and
/// `!dComIfGs_isEventBit(dSv_event_flag_c::UNK_1801)` (`:116-118`). The second is ported; the
/// first is omitted because the item-bit save array has no resource in this sim. Both return
/// `cPhs_ERROR_e`, which cancels the process inside Create (`f_pc_create_req.cpp:106-108`), so the
/// ported arm is spawned and dropped on its first tick. `daDekuItem_IsDelete` (`:271-273`) is the
/// bare "accepts a delete request" form of RULE 6b.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DekuItem {
    /// `mMode` (`d_a_deku_item.h:28`), the `mode_proc[]` index `mode_proc_call` dispatches on
    /// (`d_a_deku_item.cpp:160-167`).
    pub mode: u8,
    /// `unk630` (`d_a_deku_item.h:29`), the `BOOL` `mode_getdemo_init` raises (`:197`) and
    /// `checkOrder` lowers once the event starts (`:227-229`).
    pub accepted: bool,
    /// `mItemBitNo` (`d_a_deku_item.h:31`), `daDekuItem_prm::getItemBitNo` (`:110`).
    pub item_bit_no: u8,
    /// `mItemPID` (`d_a_deku_item.h:32`), the `fpc_ProcID` `mode_getdemo_wait` stores (`:203`).
    pub item: Option<Entity>,
    /// `fopAcM_offDraw(this)` (`d_a_deku_item.cpp:189`) - the leaf draws until the pickup starts.
    pub drawn: bool,
    /// The `DEFAULT_GETITEM` slot; `None` is `getEventIdx`'s `-1`.
    pub event_slot: Option<usize>,
    /// `_create`'s `cPhs_ERROR_e` arm, which cancels the actor inside Create.
    pub refused: bool,
    /// One-shot guard so the `DEFAULT_GETITEM` lookup happens once.
    pub entered: bool,
}

impl DekuItem {
    /// `_create` (`:107-131`), minus `fopAcM_ct` (`:108`), the omitted `fopAcM_isItem` refusal
    /// (`:112-114`), `dComIfG_resLoad` (`:120`), `fopAcM_entrySolidHeap` / `CheckCreateHeap` /
    /// `CreateHeap` (`:123-125`, `:56-86`) and `CreateInit` (`:127`) - archive and heap work. What
    /// survives is the parameter decode and the `UNK_1801` refusal.
    pub fn spawn(parameters: u32, flags: &EventFlags) -> Self {
        Self {
            // `mMode = Mode_WAIT_e;` (`:102`)
            mode: MODE_WAIT,
            // `unk630` has no explicit initialiser: it is the `sBs_ClearArea` zero-fill from
            // `fpcBs_Create` (`:96`, `f_pc_base.cpp:98`), i.e. FALSE.
            accepted: false,
            item_bit_no: ((parameters >> ITEM_BIT_SHIFT) & ITEM_BIT_MASK) as u8,
            // `mItemPID` is likewise zero-filled, but `mode_getdemo_wait` only ever compares it
            // against `fpcM_ERROR_PROCESS_ID_e` (`:205`) and never dereferences it, so the sim
            // carries the created entity itself.
            item: None,
            drawn: true,
            event_slot: None,
            refused: !flags.test(EVENT_BIT),
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -50.0f, 0.0f, -50.0f, 50.0f, 100.0f, 50.0f)`
    /// (`d_a_deku_item.cpp:92`) - the actor's one real custom cull box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// The `DEFAULT_GETITEM` slot. `fopAcM_orderOtherEvent(this, "DEFAULT_GETITEM")` (`:219`)
    /// resolves by name (`d_event_manager.cpp:350-356`), so no stage event-info index is involved.
    fn create_init(&mut self, list: &EventList) {
        self.event_slot = list.event_index(EVENT_NAME);
        self.entered = true;
    }

    /// `mode_wait` (`:171-185`): a Co overlap arms the pickup, the sparkle emitter spawns or
    /// follows the leaf, and the light sound plays. Only the overlap and the sound survive - the
    /// emitter is particles (RULE 3b) and `mCyl.SetC` / `dComIfG_Ccsp()->Set` (`:183-184`) only
    /// register the very collider whose `ChkCoHit` this already evaluates.
    fn mode_wait(&mut self, ctx: &mut Context<'_>) {
        if ctx.touched {
            self.mode = MODE_GETDEMO_INIT;
        }
        ctx.events.list.push(SimEvent::Se(SE_OBJ_DEKU_HA_LIGHT));
    }

    /// `mode_getdemo_init` (`:188-198`): `fopAcM_offDraw(this)` (`:189`), the emitter (`:191-194`)
    /// and `mMode = Mode_GETDEMO_WAIT_e; unk630 = TRUE;` (`:196-197`).
    fn mode_getdemo_init(&mut self) {
        self.drawn = false;
        self.mode = MODE_GETDEMO_WAIT;
        self.accepted = true;
    }

    /// `mode_getdemo_wait` (`:201-211`): `if (!unk630) { mItemPID =
    /// fopAcM_createItemForTrBoxDemo(&current.pos, dItemNo_DEKU_LEAF_e, -1,
    /// fopAcM_GetRoomNo(this)); if (mItemPID != fpcM_ERROR_PROCESS_ID_e)
    /// dComIfGp_event_setItemPartnerId(mItemPID); mMode = Mode_GETDEMO_e; }`.
    ///
    /// The item's position and room are the leaf's and its angle and scale are NULL
    /// (`f_op_actor_mng.h:568-569` defaults), so the created actor gets a zero angle.
    /// `dComIfGp_event_setItemPartnerId` (`:206`) writes the running event's `mPtItem`
    /// (`d_event.h:112`) and `getPtI()` has no reader anywhere in the decomp, so it is a dead store
    /// and is dropped.
    fn mode_getdemo_wait(&mut self, ctx: &Context<'_>, commands: &mut Commands) {
        if self.accepted {
            return;
        }
        self.item = Some(
            commands
                .spawn(Actor {
                    name: DEMO_ITEM_ACTOR.to_owned(),
                    parameters: ITEM_PARAM,
                    pos: ctx.leaf,
                    angle: [0; 3],
                    set_id: 0,
                    room: ctx.room,
                    ..Default::default()
                })
                .id(),
        );
        self.mode = MODE_GETDEMO;
    }

    /// `mode_getdemo` (`:214`) has an empty body: the leaf rides `Mode_GETDEMO_e` until
    /// `checkOrder` deletes it.
    fn mode_getdemo(&mut self) {}

    /// `mode_proc_call` (`:159-168`). `mode_proc` has four entries (`:160-165`) and `mMode` is an
    /// enum only ever loaded from a `Mode_*_e`, so every index is in range.
    fn mode_proc_call(&mut self, ctx: &mut Context<'_>, commands: &mut Commands) {
        match self.mode {
            MODE_WAIT => self.mode_wait(ctx),
            MODE_GETDEMO_INIT => self.mode_getdemo_init(),
            MODE_GETDEMO_WAIT => self.mode_getdemo_wait(ctx, commands),
            _ => self.mode_getdemo(),
        }
    }

    /// `eventOrder` (`:217-222`): while `unk630 == TRUE`, order `DEFAULT_GETITEM` and raise
    /// `dEvtCnd_UNK2_e`. The condition is dropped for `htobi3.rs`'s reason: nothing in the decomp
    /// ever calls `chkCondition(dEvtCnd_UNK2_e)`, and `beforeProc` clears `mCondition` on every
    /// `fopAc_Execute` (`f_op_actor.cpp:176`).
    fn event_order(&self, ctx: &mut Context<'_>) {
        if !self.accepted {
            return;
        }
        let Some(slot) = self.event_slot else {
            return;
        };
        if ctx.active.stage_of(slot).is_none() {
            let _ = ctx.active.order(ctx.list, slot, ctx.entity);
        }
    }

    /// `checkOrder` (`:225-237`). `eventInfo.checkCommandDemoAccrpt()` (`:226`) is
    /// `mCommand == dEvtCmd_INDEMO_e` (`f_op_actor.h:177`), a latch `dEvt_control_c::demoCheck`
    /// sets on both ordered actors when the event starts (`d_event.cpp:314-316`) and `demoEnd`
    /// clears (`:331-337`) - so, as `htobi3.rs` argues, "the event this actor ordered is in
    /// `Stage::Play` or `Stage::Close`". `startCheck` is `ActiveEvent::start_check`, `endCheck` is
    /// `ActiveEvent::end_check`, and the guard is `vfan::Vfan::order`'s. Returns true when
    /// `fopAcM_delete(this)` (`:233`) fires.
    ///
    /// `fopAcM_onItem(this, mItemBitNo)` (`:234`) is `dComIfGs_onItem(mItemBitNo,
    /// fopAcM_GetHomeRoomNo(item))` (`f_op_actor_mng.h:395-397`), i.e. `dSv_info_c::onItem`
    /// (`d_save.cpp:1629-1647`) writing `dSv_memBit_c::mItem` - the item-bit save array, which this
    /// sim has no resource for. `dComIfGp_event_reset()` (`:232`) is `mEventFlag |= 8` on the
    /// running event (`d_event.h:131`), which `dEvt_control_c::check` (`d_event.cpp:546-554`)
    /// turns into `endProc`; the actor is deleted on this same tick, which is the sim's own
    /// teardown, so the running event is dropped with it.
    fn check_order(&mut self, ctx: &mut Context<'_>) -> bool {
        let Some(slot) = self.event_slot else {
            return false;
        };
        if !matches!(
            ctx.active.stage_of(slot),
            Some(Stage::Play) | Some(Stage::Close)
        ) {
            return false;
        }
        if ctx.active.start_check(slot) && self.accepted {
            self.accepted = false;
        }
        if ctx.active.end_check(slot) {
            let _ = ctx.active.end(slot);
            return true;
        }
        false
    }

    /// `_execute` (`:143-156`) - `eyePos = current.pos` (`:144`), `fopAcM_posMoveF(this,
    /// mStts.GetCCMoveP())` (`:145`), `mAcch.CrrPos(*dComIfG_Bgsp())` (`:146`), `checkOrder()`,
    /// `mode_proc_call()`, `eventOrder()` (`:148-150`), `set_mtx()` (`:152`) and
    /// `mBck1.setFrame(mBck1.getEndFrame())` (`:153`).
    ///
    /// The two motion lines are a proven no-op and are deliberately not re-implemented:
    /// `CreateInit` sets only `fopAcM_SetGravity(this, -3.0f)` (`:103`) and never touches `speedF`
    /// or `maxFallSpeed`, so both keep the `sBs_ClearArea` zero-fill (`f_pc_base.cpp:96-98`);
    /// `fopAcM_calcSpeed` then computes `xSpeed = 0`, `zSpeed = 0` and
    /// `ySpeed = 0 + -3.0 = -3.0`, and its
    /// `if (ySpeed < fopAcM_GetMaxFallSpeed(i_this)) ySpeed = fopAcM_GetMaxFallSpeed(i_this);`
    /// (`f_op_actor_mng.cpp:462-463`) clamps `-3.0` back up to the zero `maxFallSpeed`, so
    /// `fopAcM_posMove` (`:467-478`) adds nothing on every axis. `eyePos` has no sim reader, and
    /// `set_mtx` and the BCK frame set are renderer and archive work. Returns true when the actor
    /// asked to be deleted.
    fn execute(&mut self, ctx: &mut Context<'_>, commands: &mut Commands) -> bool {
        let deleted = self.check_order(ctx);
        self.mode_proc_call(ctx, commands);
        self.event_order(ctx);
        deleted
    }
}

/// Everything `_execute` needs beyond the component, in one bundle so the state machine has a
/// single reader. `leaf` and `room` are the actor's `current.pos` and `fopAcM_GetRoomNo(this)`.
struct Context<'a> {
    /// `mCyl.ChkCoHit()` (`d_a_deku_item.cpp:172`) on the radius-50 / height-100 cylinder at
    /// `current.pos` (`:183`).
    touched: bool,
    leaf: Vec3,
    room: i8,
    events: &'a mut SimEvents,
    list: &'a EventList,
    active: &'a mut ActiveEvent,
    entity: Entity,
}

pub fn is_dekuitem(name: &str) -> bool {
    DEKUITEM_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut events: ResMut<SimEvents>,
    players: Query<&Player>,
    mut leaves: Query<(Entity, &Actor, &mut DekuItem)>,
    mut commands: Commands,
) {
    let player = players.iter().next().copied();
    let mut gone: Vec<Entity> = Vec::new();
    for (entity, actor, mut leaf) in &mut leaves {
        let leaf = &mut *leaf;
        if leaf.refused {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if !leaf.entered {
            leaf.create_init(&list);
        }
        // `mCyl.ChkCoHit()` (`d_a_deku_item.cpp:172`). `cM3d_Cross_CpsCyl`
        // (`SSystem/SComponent/c_m3d.cpp:1963-2064`) asks whether the two solids overlap and its
        // general case is `VECDistance(&pos1, &pos2) < cps.GetR() + cyl.GetR()` (`:2058`), i.e.
        // `dist(seg(capsule), seg(cylinder axis)) < cps.R + cyl.R`; the cylinder
        // axis is its `Center` raised by `cyl.h`. The leaf's `SrcObjCo SPrm` is
        // `cCcD_CoSPrm_Set_e | cCcD_CoSPrm_IsOther_e | cCcD_CoSPrm_VsEnemy_e` (`:22`), i.e. the
        // player body, which the sim carries as a point at `Player::pos` - the same reduction
        // `ritem.rs` makes for an item's own collider.
        let touched = player.is_some_and(|player| cyl_touches(actor.pos, player.pos));
        let mut ctx = Context {
            touched,
            leaf: actor.pos,
            room: actor.room,
            events: &mut events,
            list: &list,
            active: &mut active,
            entity,
        };
        if leaf.execute(&mut ctx, &mut commands) {
            gone.push(entity);
        }
    }
    for entity in gone {
        let _ = commands.entity(entity).despawn();
    }
}

/// The `l_cyl_src` cylinder against the player's body point. `cM3d_CylAttr::CalcAabBox`
/// (`SSystem/SComponent/c_cc_d.cpp:438-447`) spans the cylinder from its `Center` to `Center.y + h`, and `scale.x` is 1.0
/// for a stage-placed actor (RULE 2), so neither is scaled.
fn cyl_touches(base: Vec3, at: Vec3) -> bool {
    let dx = at.x - base.x;
    let dz = at.z - base.z;
    if CYL_RADIUS * CYL_RADIUS < dx * dx + dz * dz {
        return false;
    }
    at.y >= base.y && at.y <= base.y + CYL_HEIGHT
}