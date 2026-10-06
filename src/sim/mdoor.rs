use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::combat::HitPoints;
use super::event::{ActiveEvent, EventList, Stage};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, Switches};

/// `OBJNAME("Mori1", fpcNm_MDOOR_e, 255, 0)` (`d_stage.cpp:459`) is the only row bound to
/// `fpcNm_MDOOR_e` (`d_a_mdoor.cpp:463`). The arcname is `"Mdoor"` (`:13`), so the stage string and
/// the resource name differ by one letter.
///
/// COLLISION CHECK, run over every `pub const` of type `&str` / `[&str; N]` in `src/**` (not just
/// the `_ACTORS` arrays): `Mori1` is claimed by NOTHING. `src/sim/door.rs` holds 21 stage strings
/// but its four arrays are `fpcNm_DOOR10_e` (`d_stage.cpp:451-457`), `fpcNm_DOOR12_e` (`:473-480`),
/// `fpcNm_KDDOOR_e` (`:460-464`) and `fpcNm_ATDOOR_e` (`:458`, an acknowledged second copy of
/// `src/sim/atdoor.rs`); `src/sim/mbdoor.rs` holds `SMBdor` / `KGBdor`; `src/sim/swtdoor.rs` holds
/// `SWTdoor`. The only other `"Mori1"` in `src/` is a staff-name comparison at
/// `src/sim/event.rs:879`, which is not an actor claim. This file adds no thirteenth duplicate.
pub const MDOOR_ACTORS: [&str; 1] = ["Mori1"];

/// `l_cullSizeBox[6]` - `/* fopAc_CULLBOX_6_e */ fopAc_cullSizeBox(cXyz(-150.0f, 0.0f, -150.0f),
/// cXyz(150.0f, 300.0f, 150.0f))` (`f_op_actor_mng.cpp:564`), reached from the profile's
/// `fopAc_CULLBOX_6_e` (`d_a_mdoor.cpp:473`). This translation unit calls no `fopAcM_setCullSizeBox`,
/// `fopAcM_SetMin` or `fopAcM_SetMax`, so the table entry is the whole box.
const CULL_MIN: [f32; 3] = [-150.0, 0.0, -150.0]; // <- f_op_actor_mng.cpp:564
const CULL_MAX: [f32; 3] = [150.0, 300.0, 150.0]; // <- f_op_actor_mng.cpp:564

/// `u8 daMdoor_c::getSwbit() { return (fopAcM_GetParam(this) >> 0) & 0xFF; }` (`:16-18`).
pub const SWBIT_SHIFT: u32 = 0; // <- d_a_mdoor.cpp:17
/// `u8 daMdoor_c::getType() { return (fopAcM_GetParam(this) >> 8) & 0xFF; }` (`:21-23`).
pub const TYPE_SHIFT: u32 = 8; // <- d_a_mdoor.cpp:22
/// `u8 daMdoor_c::getToolId() { return (fopAcM_GetParam(this) >> 16) & 0xFF; }` (`:26-28`).
pub const TOOL_SHIFT: u32 = 16; // <- d_a_mdoor.cpp:27
/// `u8 daMdoor_c::getShapeType() { return (fopAcM_GetParam(this) >> 24) & 0xFF; }` (`:31-33`).
pub const SHAPE_SHIFT: u32 = 24; // <- d_a_mdoor.cpp:32
/// The `& 0xFF` shared by all four accessors (`:17`, `:22`, `:27`, `:32`).
pub const BYTE_MASK: u32 = 0xFF; // <- d_a_mdoor.cpp:17
/// The `0xFF` "no switch" sentinel, compared as `uVar1 == 0xFF` (`:149`) and `uVar2 != 0xFF`
/// (`:338`, `:358`). Byte-exact: it is NOT clamped down into the 0..128 `Switches` window.
pub const NO_SWITCH: u8 = 0xFF; // <- d_a_mdoor.cpp:149
/// `m2C8 = 0xFF;` (`:123`) - the initial tool id, and the value
/// `if (m2C8 != 0xFF) { setAction(ACT_SWITCH12_e); } else { setAction(ACT_SWITCH8_e); }`
/// (`:155-159`) branches on. Distinct from [`NO_SWITCH`]: one is a tool id, the other a switch.
pub const NO_TOOL: u8 = 0xFF; // <- d_a_mdoor.cpp:123

/// `daMdoor_c::Action_e` (`d_a_mdoor.h:12-30`), one constant per enum label, in declaration order.
/// The sixteen labels are also the sixteen entries of `daMdoor_c::execute`'s `l_action` table
/// (`d_a_mdoor.cpp:393-410`), so an index and its label are interchangeable.
pub const ACT_WAIT0: u8 = 0; // <- d_a_mdoor.h:13
pub const ACT_DEMO_WAIT: u8 = 1; // <- d_a_mdoor.h:14 (d_a_mdoor.cpp:130)
pub const ACT_DEMO: u8 = 2; // <- d_a_mdoor.h:15
pub const ACT_WAIT3: u8 = 3; // <- d_a_mdoor.h:16
pub const ACT_GENOCIDE: u8 = 4; // <- d_a_mdoor.h:17 (d_a_mdoor.cpp:142)
pub const ACT_READY5: u8 = 5; // <- d_a_mdoor.h:18
pub const ACT_EVENT6: u8 = 6; // <- d_a_mdoor.h:19
pub const ACT_WAIT7: u8 = 7; // <- d_a_mdoor.h:20 (d_a_mdoor.cpp:138)
pub const ACT_SWITCH8: u8 = 8; // <- d_a_mdoor.h:21 (d_a_mdoor.cpp:158)
pub const ACT_READY9: u8 = 9; // <- d_a_mdoor.h:22
pub const ACT_EVENT10: u8 = 10; // <- d_a_mdoor.h:23
pub const ACT_WAIT11: u8 = 11; // <- d_a_mdoor.h:24 (d_a_mdoor.cpp:150)
pub const ACT_SWITCH12: u8 = 12; // <- d_a_mdoor.h:25 (d_a_mdoor.cpp:156)
pub const ACT_READY_OPEN: u8 = 13; // <- d_a_mdoor.h:26 (d_a_mdoor.cpp:361)
pub const ACT_OPEN: u8 = 14; // <- d_a_mdoor.h:27
pub const ACT_WAIT15: u8 = 15; // <- d_a_mdoor.h:28

/// `void nextAction() { mAction++; }` (`d_a_mdoor.h:34`) - the successor step. It is a bare
/// increment with no bounds check, so the table's wrap-around is a `u8` wrap in the sim too.
const ACT_STEP: u8 = 1; // <- d_a_mdoor.h:34

/// `m2CC = 280.0f;` (`:131`, `:139`, `:151`) and the open targets
/// `cLib_chaseF(&i_this->m2CC, 280.0f, i_this->speedF)` (`:237`, `:314`) - the fully-raised cage.
pub const LIFT_OPEN: f32 = 280.0; // <- d_a_mdoor.cpp:131
/// `m2CC = 0.0f;` (`:122`) and `cLib_chaseF(&i_this->m2CC, 0.0f, speedF)` (`:221`) - fully closed.
pub const LIFT_CLOSED: f32 = 0.0; // <- d_a_mdoor.cpp:122
/// `cLib_chaseF(&speedF, 30.0f, 3.0f);` (`:236`, `:313`) - the opening chase target and step.
pub const OPEN_SPEED: f32 = 30.0; // <- d_a_mdoor.cpp:236
pub const OPEN_STEP: f32 = 3.0; // <- d_a_mdoor.cpp:236
/// `cLib_chaseF(&speedF, 60.0f, 6.0f);` (`:220`) - the closing chase target and step. It is NOT
/// interchangeable with the opening pair.
pub const CLOSE_SPEED: f32 = 60.0; // <- d_a_mdoor.cpp:220
pub const CLOSE_STEP: f32 = 6.0; // <- d_a_mdoor.cpp:220
/// `i_this->m2C4 = 0x14;` (`:361`) - the open-delay counter `ACT_READY_OPEN` arms.
pub const DELAY_FULL: i32 = 0x14; // <- d_a_mdoor.cpp:361
/// `i_this->m2C4 = 1;` (`:144`) - the genocide arm's single retry tick.
pub const DELAY_ONE: i32 = 1; // <- d_a_mdoor.cpp:144
/// `bool chkFlag(unsigned short bit) { return (m2CA & bit) == bit; }` (`d_a_mdoor.h:31`) with
/// `setFlag(1)` (`:204`) / `clrFlag(1)` (`:224`) - the "the close began at a demo advance point"
/// latch. Its only effect at `:224-228` is `smokeInit()`, a particle emitter, so `m2CA` is omitted
/// rather than stored; see the report.
/// `speedF = 0.0f;` (`:203`, `:210`, `:304`) - the reset-to-rest that begins either chase.
pub const SPEED_REST: f32 = 0.0; // <- d_a_mdoor.cpp:203

/// `dSv_event_flag_c::UNK_1101 = 0x1101` (`d_save_event_flag.inc:133`), read at `:127` and `:137`.
pub const EVENT_UNK_1101: u32 = 0x1101; // <- d_a_mdoor.cpp:127
/// `dSv_event_flag_c::UNK_1140 = 0x1140` (`d_save_event_flag.inc:127`), cleared at `:141` and set
/// at `:344`.
pub const EVENT_UNK_1140: u32 = 0x1140; // <- d_a_mdoor.cpp:141
/// `JA_SE_OBJ_CAGE_CLOSE = 0x697C` (`JAZelAudio_SE.h:1905`), at `:205`.
pub const SE_CLOSE: u32 = 0x697C; // <- d_a_mdoor.cpp:205
/// `JA_SE_OBJ_CAGE_OPEN = 0x697D` (`JAZelAudio_SE.h:1906`), at `:213` and `:308`.
pub const SE_OPEN: u32 = 0x697D; // <- d_a_mdoor.cpp:213
/// `dComIfGp_evmng_getEventIdx("MORI1_EVENT", 0xFF)` (`:133`) - type 0's event.
pub const EVENT_MORI1: &str = "MORI1_EVENT"; // <- d_a_mdoor.cpp:133
/// `dComIfGp_evmng_getEventIdx("DEFAULT_STOP_OPEN", ...)` (`:143`, `:154`). This is a DIFFERENT
/// name from `src/sim/mbdoor.rs`'s `MBDOOR_STOP_OPEN` (`:27`), so no string is duplicated here.
pub const EVENT_STOP_OPEN: &str = "DEFAULT_STOP_OPEN"; // <- d_a_mdoor.cpp:143
/// `i_this->m2C0 = dComIfGp_evmng_getMyStaffId("Mori1", NULL, 0);` (`:259`).
pub const STAFF_MORI1: &str = "Mori1"; // <- d_a_mdoor.cpp:259
/// `i_this->m2C0 = dComIfGp_evmng_getMyStaffId("SHUTTER_DOOR", NULL, 0);` (`:334`). The same
/// staff name is already held by `src/sim/knob00.rs:44`; a staff name is not an actor claim, so
/// this adds no `spawn_actor` branch.
pub const STAFF_SHUTTER_DOOR: &str = "SHUTTER_DOOR"; // <- d_a_mdoor.cpp:334
/// `static char* action_table[] = { "WAIT", "CLOSE", "STOP_OPEN", };` (`demoProc`,
/// `d_a_mdoor.cpp:194-196`) - the three names `dComIfGp_evmng_getMyActIdx` matches the running cut
/// against. Index 0 is `WAIT` (the fall-through / cut-end arm) and indices 1 and 2 drive the two
/// chases at `:220` and `:236`.
pub const DEMO_CLOSE: usize = 1; // <- d_a_mdoor.cpp:194
pub const DEMO_STOP_OPEN: usize = 2; // <- d_a_mdoor.cpp:195

/// The Forbidden Woods / Cabana Maze / Dragon Roost Cavern / Earth Temple wooden-and-metal cage.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daMdoor_c` registers no `dCcD` collider
/// anywhere in the translation unit - no `dCcD_Stts`, no `dCcD_Sph`, no `fopAcM_entrySolidHeap`
/// collider - and its profile `Group` is `fopAc_ACTOR_e` (`:471`). The cage blocks the player in
/// retail through its `dBgW` moving background (`mpBgW`, `:65`, `:77`, `:117`), and RULE 3b lists
/// `dBgW` / `cBgW` moving backgrounds as confirmed-absent. `HitPoints` is imported here ONLY for
/// the `fopAcM_myRoomSearchEnemy` probe below, which needs to know whether any hittable actor shares
/// the room; it is not attached to the cage.
///
/// `daMdoor_IsDelete` (`:422-425`) is the bare "accepts a delete request" form of RULE 6b, and this
/// translation unit contains no `fopAcM_delete` at all. `daMdoor_c::create` returns
/// `cPhs_ERROR_e` only from the `fopAcM_entrySolidHeap` else arm (`:185`), which is heap work, so
/// `spawn` never refuses and needs no `refused` flag.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Mdoor {
    /// `mAction` (`d_a_mdoor.h:44`, a `u8`) - the `l_action` dispatch index (`:413`).
    pub action: u8,
    /// `m2CC` (`d_a_mdoor.h:46`) - the cage's vertical offset from `current.pos`, chased between
    /// [`LIFT_CLOSED`] and [`LIFT_OPEN`]. Retail offsets the MODEL matrix (`calcMtx`, `:85`) and
    /// drives the moving background from the same value, so this is the cage's own travel and NOT
    /// `Actor::pos`.
    pub lift: f32,
    /// `fopAc_ac_c::speedF` - the rate `lift` is chased at. Retail keeps it on the actor; the sim
    /// keeps it here because the cage's `speedF` is a private channel (no other reader).
    pub speed_f: f32,
    /// `m2C4` (`d_a_mdoor.h:43`) - the genocide retry count (`:278-279`) and the open delay
    /// (`:301-311`), both read as `> 0` / `== 0`.
    pub delay: i32,
    /// `m2C6` (`d_a_mdoor.h:42`, an `s16`) - `dComIfGp_evmng_getEventIdx(...)` resolved into an
    /// `EventList` index. `None` stands for retail's `0xFF` fallback (`:133`, `:143`).
    pub event_idx: Option<usize>,
    /// `getSwbit()` (`:16-18`).
    pub swbit: u8,
    /// `getType()` (`:21-23`) - the `CreateInit` switch subject (`:125`) and the type-1 arms at
    /// `:282` and `:342`.
    pub kind: u8,
}

/// The per-tick inputs `daMdoor_c::execute` reads, bundled so the sixteen action bodies stay
/// readable. It is private: `control` is the only entry point.
struct Ctx<'a> {
    entity: Entity,
    list: &'a EventList,
    active: &'a mut ActiveEvent,
    switches: &'a Switches,
    flags: &'a mut EventFlags,
    events: &'a mut SimEvents,
    room_cleared: bool,
}

impl Mdoor {
    /// `daMdoor_c::CreateInit` (`:109-173`), minus the `dBgW` registration (`:117`), the tevStr
    /// room stamp (`:121`), the two `calcMtx()` / `mpBgW->Move()` calls (`:170-171`) and the
    /// attention / eye lifts (`:168-169`).
    ///
    /// `m2C0` is deliberately absent: it is only ever assigned by
    /// `dComIfGp_evmng_getMyStaffId` inside two action bodies (`:259`, `:334`), so it is a local of
    /// those bodies rather than persistent state, and modelling it as a field would give it a
    /// lifetime retail does not have.
    ///
    /// `getToolId()` (`:26-28`) is decoded here and NOT stored, because its only surviving effect
    /// is the `ACT_SWITCH12_e` / `ACT_SWITCH8_e` choice below (`:155-159`); its other two uses are
    /// the second argument of `dComIfGp_evmng_getEventIdx` (`:154`) and of
    /// `fopAcM_orderOtherEventId` (`:286`, `:325`, `:348`, `:363`), and `EventList::event_index`
    /// and `ActiveEvent::order` take no such argument. `m2C8` is therefore an omitted field.
    ///
    /// `getShapeType()` (`:31-33`) is likewise not stored: it selects the `MORI2` model and DZB in
    /// `CreateHeap` (`:45-55`) and gates three `AK_SN_WOODCAGE00` bursts (`:210`, `:226`, `:305`),
    /// all of which are resource load or particle work.
    ///
    /// The type-1 else arm's `dComIfGs_offEventBit(UNK_1140)` (`:141`) MUTATES the save at create
    /// time, so it belongs in the spawn tuple rather than in `spawn`; see the wiring report.
    pub fn spawn(parameters: u32, list: &EventList, flags: &EventFlags, switches: &Switches) -> Self {
        let swbit = ((parameters >> SWBIT_SHIFT) & BYTE_MASK) as u8;
        let kind = ((parameters >> TYPE_SHIFT) & BYTE_MASK) as u8;
        // ```
        // switch (bVar5) {
        // case 0: if (isEventBit(UNK_1101)) { setAction(ACT_WAIT3_e); }
        //         else { setAction(ACT_DEMO_WAIT_e); m2CC = 280.0f; }
        //         m2C6 = getEventIdx("MORI1_EVENT", 0xFF); break;
        // case 1: if (isEventBit(UNK_1101)) { setAction(ACT_WAIT7_e); m2CC = 280.0f; }
        //         else { offEventBit(UNK_1140); setAction(ACT_GENOCIDE_e);
        //                m2C6 = getEventIdx("DEFAULT_STOP_OPEN", 0xFF); m2C4 = 1; }
        //         break;
        // case 2: if (uVar1 == 0xFF || isSwitch(uVar1, roomNo)) { setAction(ACT_WAIT11_e); m2CC = 280.0f; }
        //         else { m2C8 = getToolId(); m2C6 = getEventIdx("DEFAULT_STOP_OPEN", getToolId());
        //                if (m2C8 != 0xFF) { setAction(ACT_SWITCH12_e); } else { setAction(ACT_SWITCH8_e); } }
        //         break;
        // default: setAction(ACT_WAIT0_e); break;
        // }
        // ```
        // (:125-166). The type-1 else arm's `dComIfGs_offEventBit(UNK_1140)` (`:141`) MUTATES the
        // save at create time, so it belongs in the spawn tuple rather than in `spawn`; see the
        // wiring report.
        let (action, lift, delay, event_idx) = match kind {
            0 => {
                let event_idx = list.event_index(EVENT_MORI1);
                if flags.test(EVENT_UNK_1101) {
                    (ACT_WAIT3, LIFT_CLOSED, 0, event_idx)
                } else {
                    (ACT_DEMO_WAIT, LIFT_OPEN, 0, event_idx)
                }
            }
            1 => {
                if flags.test(EVENT_UNK_1101) {
                    (ACT_WAIT7, LIFT_OPEN, 0, None)
                } else {
                    (
                        ACT_GENOCIDE,
                        LIFT_CLOSED,
                        DELAY_ONE,
                        list.event_index(EVENT_STOP_OPEN),
                    )
                }
            }
            2 => {
                if swbit == NO_SWITCH || switches.test(swbit) {
                    (ACT_WAIT11, LIFT_OPEN, 0, None)
                } else {
                    let event_idx = list.event_index(EVENT_STOP_OPEN);
                    let tool_id = ((parameters >> TOOL_SHIFT) & BYTE_MASK) as u8;
                    let armed = if tool_id != NO_TOOL {
                        ACT_SWITCH12
                    } else {
                        ACT_SWITCH8
                    };
                    (armed, LIFT_CLOSED, 0, event_idx)
                }
            }
            _ => (ACT_WAIT0, LIFT_CLOSED, 0, None),
        };
        Self {
            action,
            lift,
            // `speedF` is never assigned in `CreateInit`, so it takes the ctor zero-fill
            // (`f_pc_base.cpp:98`); the first `cLib_chaseF` of any action therefore starts from rest.
            speed_f: SPEED_REST,
            delay,
            event_idx,
            swbit,
            kind,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// The world point `calcMtx` puts the cage at:
    /// `mDoMtx_stack_c::transS(current.pos.x, current.pos.y + m2CC, current.pos.z)` (`:85`).
    pub fn leaf(&self, actor: &Actor) -> Vec3 {
        Vec3::new(actor.pos.x, actor.pos.y + self.lift, actor.pos.z)
    }

    /// `daMdoor_c::execute` (`:391-415`): `smokeEnd(); l_action[mAction](this);`.
    fn execute(&mut self, ctx: &mut Ctx<'_>) {
        // `smokeEnd();` (`:412`) - `m290.remove()` on the `dPa_smokeEcallBack`, run unconditionally
        // at the head of every tick. The sim has no particle system (RULE 3b), so it is a no-op
        // here and carries nothing across ticks.
        match self.action {
            // ```
            // if (dComIfGp_evmng_startCheck(m2C6)) {
            //     nextAction(); m2C0 = getMyStaffId("Mori1", NULL, 0); demoProc();
            // }
            // ```
            // `daMdoor_actionDemoWait` (`:257-261`). Slot 1's successor is slot 2, which is
            // `daMdoor_actionDemo`.
            ACT_DEMO_WAIT => {
                if ctx.started(self.event_idx) {
                    self.action = self.action.wrapping_add(ACT_STEP);
                    self.demo_proc(ctx, STAFF_MORI1);
                }
            }
            // ```
            // if (dComIfGp_evmng_endCheck(m2C6)) { nextAction(); return TRUE; }   // actionDemo, :267-270
            // if (dComIfGp_evmng_endCheck(m2C6)) { nextAction(); event_reset(); }  // actionEvent, :370-373
            // demoProc();
            // ```
            // The two bodies differ only in `dComIfGp_event_reset()` (`:297`, `:372`), which resets
            // the global event-manager state - a channel with no reader in this sim - so slots 2,
            // 6 and 10 share one transcription.
            ACT_DEMO | ACT_EVENT6 | ACT_EVENT10 => {
                if ctx.ended(self.event_idx) {
                    self.action = self.action.wrapping_add(ACT_STEP);
                    return;
                }
                self.demo_proc(ctx, STAFF_MORI1);
            }
            // ```
            // if (fopAcM_myRoomSearchEnemy(fopAcM_GetRoomNo(i_this)) == NULL) {
            //     if (m2C4 > 0) { m2C4--; }
            //     else { switch (getType()) { case 1: mDoAud_subBgmStop(); break; }
            //            fopAcM_orderOtherEventId(i_this, m2C6, m2C8, 0xFFFF, 0, 1); nextAction(); }
            // }
            // ```
            // `daMdoor_actionGenocide` (`:277-289`). `mDoAud_subBgmStop()` is audio with no
            // "sub-BGM off" spelling on `SimEvent`, so it is omitted.
            ACT_GENOCIDE => {
                if !ctx.room_cleared {
                    return;
                }
                if self.delay > 0 {
                    self.delay -= 1;
                    return;
                }
                self.order(ctx);
                self.action = self.action.wrapping_add(ACT_STEP);
            }
            // ```
            // s32 uVar1 = getSwbit();
            // if (dComIfGs_isSwitch(uVar1, roomNo)) { fopAcM_orderOtherEventId(...); nextAction(); }
            // ```
            // `daMdoor_actionSwitch` (`:323-328`) serves both switch slots (`:402`, `:406`). The
            // `0xFF` sentinel is NOT special-cased: `dComIfGs_isSwitch(0xFF, room)` has no flag to
            // read, so it reads false - the same answer `Switches::test` gives.
            ACT_SWITCH8 | ACT_SWITCH12 => {
                if ctx.switches.test(self.swbit) {
                    self.order(ctx);
                    self.action = self.action.wrapping_add(ACT_STEP);
                }
            }
            // ```
            // if (eventInfo.checkCommandDemoAccrpt()) {
            //     m2C0 = getMyStaffId("SHUTTER_DOOR", NULL, 0); demoProc(); nextAction();
            //     s32 uVar2 = getSwbit(); if (uVar2 != 0xFF) { dComIfGs_isSwitch(uVar2, roomNo); }
            //     switch (getType()) { case 1: dComIfGs_onEventBit(UNK_1140); break; }
            // } else { fopAcM_orderOtherEventId(i_this, m2C6, m2C8, 0xFFFF, 0, 1); }
            // ```
            // `daMdoor_actionReady` (`:333-349`), serving slots 5 and 9 (`:399`, `:403`). The
            // `dComIfGs_isSwitch` result at `:339` is DISCARDED by the decomp, so only its
            // `!= 0xFF` guard is transcribed and no switch test is performed.
            ACT_READY5 | ACT_READY9 => {
                if ctx.demo_accepted(self.event_idx) {
                    self.demo_proc(ctx, STAFF_SHUTTER_DOOR);
                    self.action = self.action.wrapping_add(ACT_STEP);
                    if self.kind == 1 {
                        ctx.flags.set(EVENT_UNK_1140);
                    }
                } else {
                    self.order(ctx);
                }
            }
            // ```
            // if (eventInfo.checkCommandDemoAccrpt()) { nextAction();
            //     s32 uVar1 = getSwbit(); if (uVar1 != 0xFF) { dComIfGs_isSwitch(uVar1, roomNo); }
            //     m2C4 = 0x14;
            // } else { fopAcM_orderOtherEventId(...); }
            // ```
            // `daMdoor_actionReadyOpen` (`:355-364`).
            ACT_READY_OPEN => {
                if ctx.demo_accepted(self.event_idx) {
                    self.action = self.action.wrapping_add(ACT_STEP);
                    self.delay = DELAY_FULL;
                } else {
                    self.order(ctx);
                }
            }
            // ```
            // if (dComIfGp_evmng_endCheck(m2C6)) { nextAction(); event_reset(); return TRUE; }
            // if (m2C4 > 0) {
            //     m2C4--;
            //     if (m2C4 == 0) { speedF = 0.0f; [particles]; seStart(JA_SE_OBJ_CAGE_OPEN, 0); }
            // }
            // if (m2C4 == 0) { chaseF(speedF, 30, 3); chaseF(m2CC, 280, speedF); calcMtx(); Move(); }
            // ```
            // `daMdoor_actionOpen` (`:295-317`). The `AK_SN_WOODCAGE00` burst is omitted (no
            // particle system); the reset to rest on the last delay tick is kept because it is what
            // makes the cage accelerate out of the stop.
            ACT_OPEN => {
                if ctx.ended(self.event_idx) {
                    self.action = self.action.wrapping_add(ACT_STEP);
                    return;
                }
                if self.delay > 0 {
                    self.delay -= 1;
                    if self.delay == 0 {
                        self.speed_f = SPEED_REST;
                        ctx.events.list.push(SimEvent::Se(SE_OPEN));
                    }
                }
                if self.delay == 0 {
                    self.chase(OPEN_SPEED, OPEN_STEP);
                    self.chase(LIFT_OPEN, self.speed_f);
                }
            }
            // `daMdoor_actionWait` (`:251-253`) is `return TRUE;`, and slots 0, 3, 7, 11 and 15 all
            // resolve to it in the `l_action` table (`:394`, `:397`, `:401`, `:405`, `:409`).
            ACT_WAIT0 | ACT_WAIT3 | ACT_WAIT7 | ACT_WAIT11 | ACT_WAIT15 => {}
            _ => {}
        }
    }

    /// `daMdoor_c::demoProc` (`:193-248`) with its `m2C0` staff argument made explicit.
    fn demo_proc(&mut self, ctx: &mut Ctx<'_>, staff: &str) {
        // ```
        // s32 actIdx = dComIfGp_evmng_getMyActIdx(m2C0, action_table, 3, FALSE, 0);
        // ```
        // (`:198`). `ActiveEvent::staff_cut` is the sim's reduction of
        // `dEvent_manager_c::getMyStaffId` plus the running cursor, and `EventList::cut_name` is the
        // cut's own name, so the index is the position of that name in the three-entry
        // `action_table` (`:194-196`). A staff with no running cut has no cut name either, which is
        // modelled as "no action" - the decomp would report `0`, and index `0` ("WAIT") takes the
        // `default:` cut-end arm anyway (`:244-246`).
        let (index, advanced) = match ctx.active.staff_cut(ctx.list, staff) {
            Some((cut, advanced)) => (
                ctx.list.cut_name(cut).and_then(|name| match name {
                    "CLOSE" => Some(DEMO_CLOSE),
                    "STOP_OPEN" => Some(DEMO_STOP_OPEN),
                    _ => None,
                }),
                advanced,
            ),
            None => (None, false),
        };
        // ```
        // if (dComIfGp_evmng_getIsAddvance(m2C0)) {
        //     switch (actIdx) {
        //     case 1: speedF = 0.0f; setFlag(1); seStart(JA_SE_OBJ_CAGE_CLOSE, 0); break;
        //     case 2: speedF = 0.0f; if (getShapeType() != 1) { particle_set(AK_SN_WOODCAGE00); }
        //              seStart(JA_SE_OBJ_CAGE_OPEN, 0); break;
        //     }
        // }
        // ```
        // (`:200-216`). `staff_cut`'s second element IS `dComIfGp_evmng_getIsAddvance`. `setFlag(1)`
        // (`:204`) is omitted with the rest of `m2CA`: its only reader is `chkFlag(1)` at `:223`,
        // whose body is a particle burst and a smoke emitter.
        if advanced {
            match index {
                Some(DEMO_CLOSE) => {
                    self.speed_f = SPEED_REST;
                    ctx.events.list.push(SimEvent::Se(SE_CLOSE));
                }
                Some(DEMO_STOP_OPEN) => {
                    self.speed_f = SPEED_REST;
                    ctx.events.list.push(SimEvent::Se(SE_OPEN));
                }
                _ => {}
            }
        }
        match index {
            Some(DEMO_CLOSE) => {
                // ```
                // cLib_chaseF(&speedF, 60.0f, 6.0f);
                // if (cLib_chaseF(&m2CC, 0.0f, speedF)) { dComIfGp_evmng_cutEnd(m2C0);
                //     if (chkFlag(1)) { clrFlag(1); [particles]; smokeInit(); } }
                // ```
                // (`:220-230`).
                self.chase(CLOSE_SPEED, CLOSE_STEP);
                if self.chase(LIFT_CLOSED, self.speed_f) {
                    ctx.cut_end(staff);
                }
            }
            Some(DEMO_STOP_OPEN) => {
                // ```
                // cLib_chaseF(&speedF, 30.0f, 3.0f);
                // if (cLib_chaseF(&m2CC, 280.0f, speedF)) { dComIfGp_evmng_cutEnd(m2C0); }
                // ```
                // (`:236-239`).
                self.chase(OPEN_SPEED, OPEN_STEP);
                if self.chase(LIFT_OPEN, self.speed_f) {
                    ctx.cut_end(staff);
                }
            }
            // `default: dComIfGp_evmng_cutEnd(m2C0); break;` (`:244-246`) - the "WAIT" arm.
            _ => ctx.cut_end(staff),
        }
    }

    /// `cLib_chaseF(f32* pValue, f32 target, f32 step)` (`c_lib.cpp:276-290`) applied to `m2CC`,
    /// returning whether the value landed on the target - which is how the decomp decides to end the
    /// cut. `step * (*pValue - target) >= 0` (`:282`) is retail's `int` multiply (`c_lib.cpp:282`
    /// promotes through the `int` return type); in `f32` the sign test is identical and RULE 10
    /// rules out an `i16` multiply.
    fn chase(&mut self, target: f32, step: f32) -> bool {
        if step == 0.0 {
            return self.lift == target;
        }
        let mut step = step;
        if self.lift > target {
            step = -step;
        }
        self.lift += step;
        if step * (self.lift - target) >= 0.0 {
            self.lift = target;
            return true;
        }
        false
    }

    /// `fopAcM_orderOtherEventId(i_this, m2C6, m2C8, 0xFFFF, 0, 1)` (`:286`, `:325`, `:348`,
    /// `:363`) - ask the event manager to start `m2C6`. RULE 5b's guard is mandatory: re-ordering
    /// every tick resets `Stage::Order` and livelocks the demo, so the call is made only when the
    /// event has no stage yet.
    fn order(&mut self, ctx: &mut Ctx<'_>) {
        if let Some(index) = self.event_idx
            && ctx.active.stage_of(index).is_none()
        {
            ctx.active.order(ctx.list, index, ctx.entity);
        }
    }
}

impl Ctx<'_> {
    /// `dComIfGp_evmng_startCheck(m2C6)` (`:257`).
    fn started(&self, index: Option<usize>) -> bool {
        index.is_some_and(|index| self.active.start_check(index))
    }

    /// `dComIfGp_evmng_endCheck(m2C6)` (`:267`, `:295`, `:370`).
    fn ended(&self, index: Option<usize>) -> bool {
        index.is_some_and(|index| self.active.end_check(index))
    }

    /// `eventInfo.checkCommandDemoAccrpt()` (`:333`, `:355`) is `mCommand == dEvtCmd_INDEMO_e`
    /// (`f_op_actor.h:177`), a latch set on both ordered actors when the event starts
    /// (`d_event.cpp:314-316`) and cleared at `demoEnd` (`:331-337`), so it stays true through the
    /// CLOSE stage. `src/sim/htobi3.rs:160-166` establishes the sim's reduction of exactly that
    /// latch as "one of the events this actor ordered is in `Play` or `Close`", and the ordered
    /// event here is `m2C6`.
    fn demo_accepted(&self, index: Option<usize>) -> bool {
        index.is_some_and(|index| {
            matches!(
                self.active.stage_of(index),
                Some(Stage::Play) | Some(Stage::Close)
            )
        })
    }

    /// `dComIfGp_evmng_cutEnd(m2C0)` (`:222`, `:238`, `:245`).
    fn cut_end(&mut self, staff: &str) {
        self.active.end_staff_cut(self.list, self.flags, staff);
    }
}

pub fn is_mdoor(name: &str) -> bool {
    MDOOR_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    switches: Res<Switches>,
    mut flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    enemies: Query<(&Actor, Option<&HitPoints>)>,
    mut doors: Query<(Entity, &Actor, &mut Mdoor), With<Mdoor>>,
) {
    for (entity, actor, mut door) in &mut doors {
        let room = actor.room;
        // `fopAcM_myRoomSearchEnemy(fopAcM_GetRoomNo(i_this)) == NULL` (`:277`) - the same
        // "no hittable actor shares this room" reduction `src/sim/door.rs:475` and
        // `src/sim/mbdoor.rs:106-109` already use.
        let cleared = room >= 0
            && !enemies
                .iter()
                .any(|(other, hp)| hp.is_some() && other.room == room);
        let mut ctx = Ctx {
            entity,
            list: &list,
            active: &mut active,
            switches: &switches,
            flags: &mut flags,
            events: &mut events,
            room_cleared: cleared,
        };
        door.execute(&mut ctx);
    }
}
