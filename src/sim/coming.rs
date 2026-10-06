//! `daObjComing::Act_c` (`d_a_obj_coming.h:31-84`) - the Outset Island barrel-bomb spawner behind
//! the stage string `Com_A`. `Nonmatching` count: 0, so every token below is transcribed.
//!
//! `_create` calls `fopAcM_offDraw(this)` (`:214`) and `_draw` is `return true` with no draw call
//! (`:530-532`), so retail renders NOTHING for this actor - it is a pure spawner.
//!
//! Nothing else in `src/sim/` claims `Com_A`: `OBJNAME("Com_A", fpcNm_Obj_Coming_e, 255, 0)`
//! (`d_stage.cpp:940`) is the only row. The siblings are DIFFERENT profiles: `coming3.rs` owns
//! `"Com_C"` (`fpcNm_Coming3_e`, `d_stage.cpp:1168`) and `coming2.rs` owns `"Com_B"`.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::buoybarrel::BuoyBarrel;
use super::collision::Collision;
use super::door::StayRoom;
use super::enemy::Enemy;
use super::flags::Switches;
use super::lines::Lines;
use super::player::Player;
use super::sea::Sea;
use super::ship::Ship;

/// `OBJNAME("Com_A", fpcNm_Obj_Coming_e, 255, 0)` (`d_stage.cpp:940`) - the only row for
/// `g_profile_Obj_Coming` (`d_a_obj_coming.cpp:571-586`).
pub const COMING_ACTORS: [&str; 1] = ["Com_A"];

/// ```
/// static const Attr_c L_Attr = {
///     {2, 3, 4, 5, 20, 60, 40, 80, 80, 80, 100, 100, 30, 30, 300, 30},
///     {25.0f, 5000.0f, 6000.0f, 5000.0f, 0.05f, 0.05f},
/// };
/// ```
/// (`d_a_obj_coming.cpp:24-27`). `Attr_c::m00[16]` (`d_a_obj_coming.h:20-22`) is read at six distinct
/// indices and `L_Hio[6]` at five:
/// * `m00[0..3] = {2, 3, 4, 5}` - the barrel COUNT, indexed by `prm_get_makeNum()` in
///   `_execute` (`:507`). NOTE `PRM_MAKENUM_W = 2` so it decodes 0..=3, which lands exactly here.
/// * `m00[4] = 20`, `m00[5] = 60` - the first wait timer's min and span (`:43`, `:49-50`).
/// * `m00[6] = 40`, `m00[7] = 80` - the second (`:43`, `:49-50`).
/// * `m00[8] = 80`, `m00[9] = 80` - the third.
/// * `m00[10] = 100`, `m00[11] = 100` - the fourth.
/// * `m00[12] = 30`, `m00[13] = 30` - `set_retry_timer`: `m04 = m00[12] + (s32)cM_rndF(m00[13])`
///   (`:55`).
/// * `m00[14] = 300`, `m00[15] = 30` - `set_appear_timer`: `m04 = m00[14] + (s32)cM_rndF(m00[15])`
///   (`:60`).
const COUNT_TABLE: [s32; 4] = [2, 3, 4, 5]; // <- d_a_obj_coming.cpp:25 (m00[0..3])
const WAIT_MIN: [s32; 4] = [20, 40, 80, 100]; // <- d_a_obj_coming.cpp:25 (m00[4], :6, :8, :10)
const WAIT_SPAN: [s32; 4] = [60, 80, 80, 100]; // <- d_a_obj_coming.cpp:25 (m00[5], :7, :9, :11)
const RETRY_BASE: s32 = 30; // <- d_a_obj_coming.cpp:25 (m00[12])
const RETRY_SPAN: s32 = 30; // <- d_a_obj_coming.cpp:25 (m00[13])
const APPEAR_BASE: s32 = 300; // <- d_a_obj_coming.cpp:25 (m00[14])
const APPEAR_SPAN: s32 = 30; // <- d_a_obj_coming.cpp:25 (m00[15])
/// `L_Attr::L_Hio[0] = 25.0f` - `chk_ship_spd`: `if (ship->speedF > attr()->L_Hio[0])` (`:261`).
const SHIP_SPEED_MIN: f32 = 25.0; // <- d_a_obj_coming.cpp:26
/// ```
/// f32 tmp1 = cM_rndF(attr()->L_Hio[2]);   // 6000.0f
/// tmp1 += attr()->L_Hio[1];               // 5000.0f
/// f32 tmp2 = (cM_rnd() - 0.5f);
/// tmp2 *= attr()->L_Hio[3];               // 5000.0f
/// ```
/// (`:272-275`) - where a new barrel appears, in the SHIP's frame: forward 5000..11000, lateral
/// +/-2500.
const MAKE_FWD_BASE: f32 = 5000.0; // <- d_a_obj_coming.cpp:26 (L_Hio[1])
const MAKE_FWD_SPAN: f32 = 6000.0; // <- d_a_obj_coming.cpp:26 (L_Hio[2])
const MAKE_SIDE_SPAN: f32 = 5000.0; // <- d_a_obj_coming.cpp:26 (L_Hio[3])
/// ```
/// f32 dVar2 = cM_rnd();
/// f32 tmp = attr()->L_Hio[4] + attr()->L_Hio[5];
/// if (tmp > 0.0f) {
///     if (dVar2 < attr()->L_Hio[4]) { cVar2 = PTN_01_e; }
///     else if (dVar2 < tmp) { cVar2 = PTN_00_e; }
/// }
/// ```
/// (`:353-362`) - the SPECIAL-CASE branch, taken with probability 0.05 + 0.05.
const SPECIAL_LOW: f32 = 0.05; // <- d_a_obj_coming.cpp:26 (L_Hio[4])
const SPECIAL_SPAN: f32 = 0.05; // <- d_a_obj_coming.cpp:26 (L_Hio[5])
/// `f32 end = tmp->m18 + 100.0f;` (`:289`) with `sp08.m18 = 200.0f` (`:410`) - the ground-sweep
/// length in `chk_make_pos_gnd`, plus its margin.
const GND_SWEEP: f32 = 200.0; // <- d_a_obj_coming.cpp:410
const GND_MARGIN: f32 = 100.0; // <- d_a_obj_coming.cpp:289
/// `while (fVar2 < end) { ... fVar2 += 20.0f; sVar4 += 20000; }` (`:293-315`) - the sweep step and
/// the per-iteration yaw twist.
const GND_STEP: f32 = 20.0; // <- d_a_obj_coming.cpp:313
const GND_YAW_STEP: i16 = 20000; // <- d_a_obj_coming.cpp:314
/// ```
/// if (i_this->current.pos.y - tmp->m0C.y < tmp->m1C + 100.0f) { ... }
/// ```
/// (`:335`) with `tmp->m1C = 400.0f` (`:411`), plus `fabsf` - the actor-exclusion cylinder's
/// height half-window and its XZ radius.
const ACT_HEIGHT: f32 = 400.0; // <- d_a_obj_coming.cpp:411
const ACT_MARGIN: f32 = 100.0; // <- d_a_obj_coming.cpp:335-336
/// ```
/// return ship->current.pos.absXZ(current.pos) < scale.x * 10000.0f;
/// ```
/// (`:256`) - the range fallback, scaled by the actor's OWN `scale.x`. RULE 2: `dStage_actorInit`
/// never writes `fopAcM_prm_class::scale`, so `scale.x` is exactly 1.0.
const RANGE_SCALE: f32 = 10000.0; // <- d_a_obj_coming.cpp:256
/// `s16 rnd = cM_rndFX(0x2000); barrel2->mAngleY = angle + rnd;` (`:420-421`) - the barrel's yaw
/// jitter, +/- 0x2000.
const ANGLE_JITTER: f32 = 0x2000.0; // <- d_a_obj_coming.cpp:420
/// `cXyz sp08(current.pos.x, current.pos.y + 500.0f, current.pos.z); M_gnd_work.SetPos(&sp08);`
/// (`:220-221`) in `_create` - the height the room query is made from.
const ROOM_PROBE_LIFT: f32 = 500.0; // <- d_a_obj_coming.cpp:220
/// `f32 dVar3 = -G_CM3D_F_INF;` (`:235`) and `if (dVar5 > -G_CM3D_F_INF)` (`:302`) - the
/// no-water sentinel of `get_aqua_y`.
const NO_WATER: f32 = f32::NEG_INFINITY; // <- d_a_obj_coming.cpp:235
/// `cXyz sp08(pos->x, pos->y - 1000.0f, pos->z); wtr.Set(sp08, pos->y + 1000.0f);` (`:241-242`) -
/// the water probe's vertical window.
const WATER_DOWN: f32 = 1000.0; // <- d_a_obj_coming.cpp:241
const WATER_UP: f32 = 1000.0; // <- d_a_obj_coming.cpp:242
/// `if (dComIfG_Bgsp()->GroundCross(&M_gnd_work) > dVar5 - 100.0f) { ret = false; break; }` (`:305`)
/// and `sp08.y += 20000.0f;` (`:303`) - `chk_make_pos_gnd`'s clearance test.
const GND_CLEAR: f32 = 100.0; // <- d_a_obj_coming.cpp:305
const GND_LIFT: f32 = 20000.0; // <- d_a_obj_coming.cpp:303
/// `daObjComing::Act_c::M_item_tbl[7][16]` (`d_a_obj_coming.cpp:64-191`) - the pattern table
/// `barrel2_get_ptn` indexes with `cM_rndF(16.0f)` (`:365`) and `prm_get_tblNum()` (`:366`). The
/// values are `daObjComing::Act_c::Ptn_e` (`d_a_obj_coming.h:34-57`), spelled numerically.
const ITEM_TBL: [[u8; 16]; 7] = [
    [0x03, 0x03, 0x03, 0x0E, 0x0E, 0x0A, 0x0A, 0x04, 0x12, 0x12, 0x05, 0x06, 0x07, 0x07, 0x07, 0x07],
    [0x03, 0x03, 0x03, 0x03, 0x04, 0x04, 0x0A, 0x0A, 0x04, 0x04, 0x05, 0x05, 0x05, 0x06, 0x07, 0x07],
    [0x02, 0x02, 0x02, 0x03, 0x03, 0x03, 0x03, 0x03, 0x04, 0x04, 0x04, 0x04, 0x04, 0x05, 0x06, 0x07],
    [0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x01, 0x01, 0x01, 0x01, 0x01, 0x05, 0x07, 0x07],
    [0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0B, 0x0B, 0x0B, 0x0D, 0x07, 0x07],
    [0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0F, 0x0F, 0x0F, 0x0F, 0x10, 0x11, 0x11, 0x07, 0x07],
    [0x14, 0x14, 0x13, 0x13, 0x12, 0x12, 0x12, 0x12, 0x08, 0x08, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07],
];
/// ```
/// static const u8 a[] = { 1, 0, 2, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 0 };
/// static const s32 b[] = { 0, 1, 2 };
/// ```
/// (`d_a_obj_coming.cpp:380-382`) - `barrel2_get_type`'s two-step indirection, giving
/// `daObjBarrel2::Type_e`. Only `Type_01_e = 0x1` (`d_a_obj_barrel2.h:21`) is a legal enum value;
/// 0 and 2 fall outside it, and `make_prm` masks the result into two bits
/// (`PRM_TYPE_W = 2`, `d_a_obj_barrel2.h:81`), so they are stored as the barrel's `kind`.
const TYPE_INDIRECT: [u8; 21] = [
    1, 0, 2, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 0,
]; // <- d_a_obj_coming.cpp:380
const TYPE_TABLE: [u8; 3] = [0, 1, 2]; // <- d_a_obj_coming.cpp:382
/// ```
/// static const s8 item_data[] = {255, 255, 255, 1, 2, 3, 4, 0, 0, 11, 12, 13, 14, 14, 16, 17, 18, 18, 9,
///                                 10, 10};
/// ```
/// (`d_a_obj_coming.cpp:390`) - `barrel2_get_item`'s table; 255 is "no item" and the rest are
/// `dItemNo_*` from `d_item_data.h`.
const ITEM_DATA: [u8; 21] = [
    255, 255, 255, 1, 2, 3, 4, 0, 0, 11, 12, 13, 14, 14, 16, 17, 18, 18, 9, 10, 10,
]; // <- d_a_obj_coming.cpp:390
/// ```
/// static const bool buoy[] = {false, false, true, false, false, false, false, false, false, false,
///                             false, false, false, false, false, false, false, false, false, false};
/// ```
/// (`d_a_obj_coming.cpp:398-399`) - `barrel2_get_buoy`. Only pattern `PTN_02_e` floats a buoy, and
/// `make_coming` INVERTS it (`!hasFlag`, `d_a_obj_barrel2.h:78`), so only pattern 2 has no buoy.
const BUOY_DATA: [bool; 21] = [
    false, false, true, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false,
]; // <- d_a_obj_coming.cpp:398
/// `daObjBuoyflag::Texture_00_e = 0x0` (`d_a_obj_buoyflag.h:24`) - the texture
/// `mode_barrel2_wait` passes to `make_coming` (`:442`).
const TEXTURE_00: u8 = 0x00; // <- d_a_obj_coming.cpp:442 (d_a_obj_buoyflag.h:24)
/// ```
/// make_prm: (itemNo << PRM_ITEM_NO_S) | (0x7F << PRM_ITEM_SAVE_S) | (type << PRM_TYPE_S)
///          | (buoy << PRM_BUOY_S) | ((tex == Texture_00_e ? 0 : 1) << PRM_TEXTURE_S)
///          | (coming << PRM_COMING_S)
/// ```
/// (`d_a_obj_barrel2.h:83-91`) with `PRM_ITEM_NO_S = 0`, `PRM_ITEM_SAVE_S = 0x10`,
/// `PRM_ITEM_SAVE_W = 7`, `PRM_BUOY_S = 8`, `PRM_TYPE_S = 0x18`, `PRM_TEXTURE_S = 0xA`,
/// `PRM_COMING_S = 0x1C` (`:74-83`). Transcribed so the spawned `Ktarur`'s parameter word is exactly
/// the one retail builds.
const PRM_ITEM_NO_S: u32 = 0; // <- d_a_obj_barrel2.h:76
const PRM_ITEM_SAVE_S: u32 = 0x10; // <- d_a_obj_barrel2.h:74
const PRM_ITEM_SAVE: u32 = 0x7F; // <- d_a_obj_barrel2.h:88
const PRM_BUOY_S: u32 = 8; // <- d_a_obj_barrel2.h:79
const PRM_TYPE_S: u32 = 0x18; // <- d_a_obj_barrel2.h:81
const PRM_TEXTURE_S: u32 = 0xA; // <- d_a_obj_barrel2.h:83
const PRM_COMING_S: u32 = 0x1C; // <- d_a_obj_barrel2.h:85
/// `if (switchIndex == 0xFF) { return true; } else if (fopAcM_isSwitch(this, switchIndex))` -
/// `chk_switch` (`:199-207`). `prm_get_swSave()` is `PRM_SAVE_W = 8`, `PRM_SAVE_S = 0x10`
/// (`d_a_obj_coming.h:61-62`), i.e. the parameter's bits 16..23.
const SW_SAVE_SHIFT: u32 = 0x10; // <- d_a_obj_coming.h:62
const SW_SAVE_MASK: u32 = 0xFF; // <- d_a_obj_coming.h:61
const NO_SWITCH: u8 = 0xFF; // <- d_a_obj_coming.cpp:201
/// `s32 waitTime = actor->prm_get_waitTime() * 2;` (`:42`, `:48`) -
/// `prm_get_waitTime()` is `PRM_WAIT_TIME_W = 2`, `PRM_WAIT_TIME_S = 0x18`
/// (`d_a_obj_coming.h:59-60`), so it decodes 0..=3 and the doubled index is 0, 2, 4 or 6.
const WAIT_TIME_SHIFT: u32 = 0x18; // <- d_a_obj_coming.h:60
const WAIT_TIME_WIDTH: u32 = 2; // <- d_a_obj_coming.h:59
/// `if (m31C >= 7)` is not in this actor - the clamp is `if (iVar1 >= 7) { iVar1 = 0; }` (`:367`).
/// `prm_get_tblNum()` is `PRM_TBL_NUM_W = 3`, `PRM_TBL_NUM_S = 8` (`d_a_obj_coming.h:64-65`).
const TBL_NUM_SHIFT: u32 = 8; // <- d_a_obj_coming.h:65
const TBL_NUM_WIDTH: u32 = 3; // <- d_a_obj_coming.h:64
const TBL_NUM_COUNT: u8 = 7; // <- d_a_obj_coming.cpp:367
/// `prm_get_makeNum()` is `PRM_MAKENUM_W = 2`, `PRM_MAKENUM_S = 0x1C` (`d_a_obj_coming.h:67-68`).
const MAKE_NUM_SHIFT: u32 = 0x1C; // <- d_a_obj_coming.h:68
const MAKE_NUM_WIDTH: u32 = 2; // <- d_a_obj_coming.h:67
/// `prm_get_type()` is `PRM_TYPE_W = 4`, `PRM_TYPE_S = 0` (`d_a_obj_coming.h:70-71`) and lands in
/// `m290`, which indexes `static modeFunc mode_proc[][3]` (`:499-505`) - a table with exactly ONE
/// row. Any nonzero value is retail out-of-bounds; RULE 10 folds it to row 0.
const TYPE_PARAM_SHIFT: u32 = 0; // <- d_a_obj_coming.h:71
const TYPE_PARAM_WIDTH: u32 = 4; // <- d_a_obj_coming.h:70
const MODE_ROW_COUNT: u8 = 1; // <- d_a_obj_coming.cpp:499-505
/// `prm_get_range()` is `PRM_RANGE_W = 1`, `PRM_RANGE_S = 6` (`d_a_obj_coming.h:54-55`), and
/// `prm_get_always()` is `PRM_ALWAYS_W = 1`, `PRM_ALWAYS_S = 6` (`d_a_obj_coming.h:73-74`) - the
/// SAME bit. Transcribed twice, exactly as the decomp declares it; they are not interchangeable
/// names for one field in the struct, they are two accessors over one bit.
const RANGE_SHIFT: u32 = 6; // <- d_a_obj_coming.h:55, :74
const RANGE_WIDTH: u32 = 1; // <- d_a_obj_coming.h:54, :73
/// `if (bVar2) { fopAcM_delete(this); }` (`:523-525`) - the self-delete once every slot has left.
/// `cM_rndF(16.0f)` (`:365`) - the pattern-table pick.
const PATTERN_SPAN: f32 = 16.0; // <- d_a_obj_coming.cpp:365
/// `m294[5]` (`d_a_obj_coming.h:78`) - `Ctrl_c::init` is called five times in `_create` (`:216`).
const SLOTS: usize = 5; // <- d_a_obj_coming.cpp:216
/// `dComIfGs_isEventBit` never appears in this translation unit, so no event flag is read.

/// `daObjComing::Ctrl_c` (`d_a_obj_coming.h:13-22`) - one spawn slot's state.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotMode {
    /// `mode_barrel2_wait` (`:429-451`).
    Wait,
    /// `mode_barrel2_appear` (`:456-476`).
    Appear,
    /// `mode_barrel2_leave` (`:481-494`).
    Leave,
}

impl SlotMode {
    /// `this->*mode_proc[m290][ctrl->m00]` (`:516`) - `m00` selects the column of the row.
    pub fn index(self) -> usize {
        match self {
            SlotMode::Wait => 0,
            SlotMode::Appear => 1,
            SlotMode::Leave => 2,
        }
    }
}

/// One `daObjComing::Ctrl_c` (`d_a_obj_coming.h:13-22`): `m00` the mode, `m04` the timer, `m08` the
/// spawned barrel's process id (`fpcM_ERROR_PROCESS_ID_e` when none).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ctrl {
    /// `m00` (`:20`) - the mode, and the `mode_proc` column index.
    pub mode: SlotMode,
    /// `m04` (`:21`) - the countdown, decremented once per `_execute` (`:512-514`).
    pub timer: s32,
    /// `m08` (`:22`) - `Some` while a barrel is alive. `Ctrl_c::init` sets it to the error sentinel
    /// (`:37`); `mode_barrel2_wait` writes the new id (`:435`), `mode_barrel2_appear` and
    /// `mode_barrel2_leave` clear it when `fopAcM_SearchByID` misses (`:467`, `:491`).
    pub barrel: Option<Entity>,
}

impl Ctrl {
    /// `daObjComing::Ctrl_c::init` (`:34-38`): `init_timer(actor); m00 = 0; m08 = error`.
    ///
    /// `init_timer` (`:41-44`):
    /// ```
    /// s32 waitTime = actor->prm_get_waitTime() * 2;
    /// m04 = (s32)cM_rndF(attr()->m00[4 + waitTime] + attr()->m00[5 + waitTime]);
    /// ```
    /// The two table entries are ADDED before the roll, so this is one roll over `min + span` -
    /// `getRndValue`'s shape, NOT `low + rnd * |high - low|`. The roll value is sampled by the
    /// caller because the RNG lives on [`Coming`] (there is no global `cM_rndF`; see the module
    /// note).
    fn init(&mut self, wait: i32, roll: s32) {
        self.timer = WAIT_MIN[slot_of(wait)] + roll;
        // `m00 = 0;` (:36) - `mode_barrel2_wait`.
        self.mode = SlotMode::Wait;
        // `m08 = fpcM_ERROR_PROCESS_ID_e;` (:37)
        self.barrel = None;
    }

    /// `set_wait_timer` (`:47-51`):
    /// ```
    /// s32 waitTime = actor->prm_get_waitTime() * 2;
    /// s32 attr2 = attr()->m00[4 + waitTime];
    /// m04 = (s32)cM_rndF(attr()->m00[5 + waitTime]) + attr2;
    /// ```
    /// The same value as `init_timer`; the two are separate functions in retail and stay separate.
    fn set_wait_timer(&mut self, wait: i32, roll: s32) {
        self.timer = WAIT_MIN[slot_of(wait)] + roll;
    }

    /// `set_retry_timer` (`:54-56`): `m04 = m00[12] + (s32)cM_rndF(m00[13])`.
    fn set_retry_timer(&mut self, roll: s32) {
        self.timer = RETRY_BASE + roll.min(RETRY_SPAN - 1);
    }

    /// `set_appear_timer` (`:59-61`): `m04 = m00[14] + (s32)cM_rndF(m00[15])`.
    fn set_appear_timer(&mut self, roll: s32) {
        self.timer = APPEAR_BASE + roll.min(APPEAR_SPAN - 1);
    }
}

/// `waitTime` index for `prm_get_waitTime() * 2` (`:42`, `:48`). The doubled value is 0, 2, 4 or 6,
/// and the two tables are read at `4 + waitTime` and `5 + waitTime`, so the four legal inputs map to
/// the four table pairs `(m00[4],m00[5]) .. (m00[10],m00[11])`.
fn slot_of(wait_time: i32) -> usize {
    (wait_time as usize) / 2
}

/// `daObjComing::Act_c` (`d_a_obj_coming.h:57-84`).
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. The struct holds `m290`, five `Ctrl_c` and
/// `mRoomId` (`d_a_obj_coming.h:75-84`) - no collider member of any kind, and
/// `dComIfG_Ccsp()->Set(...)` never appears. The profile `Group` is `fopAc_ACTOR_e`
/// (`d_a_obj_coming.cpp:585`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached. The barrels
/// this actor SPAWNS are `daObjBarrel2::Act_c` (`Ktarur`), which carries its own damage state in
/// `buoybarrel.rs` - that is a different entity, not this one.
#[derive(Component, Debug, Clone)]
pub struct Coming {
    /// `m290` (`d_a_obj_coming.h:75`) - `prm_get_type()`, the `mode_proc` ROW index. The table has
    /// one row (`:499-505`), so a nonzero value is retail out-of-bounds; folded to 0 under RULE 10.
    pub mode_row: u8,
    /// `prm_get_waitTime()` (`d_a_obj_coming.h:82`) - the four-way timer table selector.
    pub wait_time: i32,
    /// `prm_get_tblNum()` (`d_a_obj_coming.h:79`) - the `M_item_tbl` row, clamped by `:367`.
    pub tbl_num: u8,
    /// `prm_get_swSave()` (`d_a_obj_coming.h:76`) - the switch bit `chk_switch` tests (`:200`).
    pub sw_save: u8,
    /// `prm_get_range()` (`d_a_obj_coming.h:74`) - bit 6; `chk_ship_range` returns true outright
    /// when set (`:252-254`).
    pub range: bool,
    /// `prm_get_always()` (`d_a_obj_coming.h:70`) - the SAME bit as `range` in the decomp
    /// (`PRM_ALWAYS_S = 6` too), read by `mode_barrel2_wait` (`:431`). Stored separately so both
    /// call sites stay visible.
    pub always: bool,
    /// `prm_get_makeNum()` (`d_a_obj_coming.h:68`) - the barrel COUNT, indexing `L_Attr::m00`
    /// (`:507`).
    pub make_num: u8,
    /// `m294[5]` (`d_a_obj_coming.h:78`) - the five spawn slots.
    pub slots: [Ctrl; SLOTS],
    /// `mRoomId` (`d_a_obj_coming.h:84`) - `dComIfG_Bgsp()->GetRoomId(M_gnd_work)` from the
    /// `ROOM_PROBE_LIFT` ground query in `_create` (`:220-224`). That is a per-poly-group ground
    /// query plus a room-id lookup, which RULE 3b lists as absent, so it is NOT modelled and the
    /// room test at `:431` loses one of its three alternatives.
    pub room_id: i8,
    /// The actor's own room, `fopAcM_GetHomeRoomNo(this)` - the OTHER half of that test (`:431`),
    /// which IS modelled.
    pub home_room: i8,
    /// Per-actor RNG state. See the RNG note in the module docs.
    pub seed: u32,
}

/// The xorshift generator `cM_rnd` / `cM_rndF` stand in for, matching `buoybarrel.rs:349-354`.
struct Rng {
    seed: u32,
}

impl Rng {
    fn roll(&mut self, span: s32) -> s32 {
        if span <= 0 {
            return 0;
        }
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        ((self.seed >> 8) as f32 / 16777216.0 * span as f32) as s32
    }
}

impl Coming {
    /// `daObjComing::Act_c::_create` (`:210-226`), with the parameter decode from
    /// `daObj::PrmAbstract` (`d/d_a_obj.h:14`, `((1 << W) - 1) & (param >> S)`) copied exactly.
    /// The profile's `Parameters` field is `0` (`d_a_obj_coming.cpp:578`), so the whole `u32` is
    /// available.
    ///
    /// DROPPED: `fopAcM_ct` (`:211`), `fopAcM_offDraw(this)` (`:214` - a render flag; retail draws
    /// nothing for this actor anyway), and the `mRoomId` ground query (`:220-224`) - see
    /// [`Coming::room_id`].
    pub fn spawn(parameters: u32) -> Self {
        // ```
        // s32 waitTime = actor->prm_get_waitTime() * 2;
        // ```
        // (:42) - `PRM_WAIT_TIME_W = 2`, `PRM_WAIT_TIME_S = 0x18`.
        let wait_time = abstract_param(parameters, WAIT_TIME_WIDTH, WAIT_TIME_SHIFT) as i32;
        let mut rng = Self::rng_of(parameters);
        let mut slots = [Ctrl {
            mode: SlotMode::Wait,
            timer: 0,
            barrel: None,
        }; SLOTS];
        for slot in &mut slots {
            // `for (s32 i = 0; i < 5; i++) { m294[i].init(this); }` (:216-218) - five independent
            // rolls of `(s32)cM_rndF(min + span)`.
            let roll = rng.roll((WAIT_MIN[slot_of(wait_time)] + WAIT_SPAN[slot_of(wait_time)]));
            slot.init(wait_time, roll);
        }
        Self {
            // `m290 = prm_get_type();` (:213) - `PRM_TYPE_W = 4`, `PRM_TYPE_S = 0`.
            mode_row: abstract_param(parameters, TYPE_PARAM_WIDTH, TYPE_PARAM_SHIFT) as u8,
            wait_time,
            // `prm_get_tblNum()` - `PRM_TBL_NUM_W = 3`, `PRM_TBL_NUM_S = 8`.
            tbl_num: abstract_param(parameters, TBL_NUM_WIDTH, TBL_NUM_SHIFT) as u8,
            // `prm_get_swSave()` - `PRM_SAVE_W = 8`, `PRM_SAVE_S = 0x10`.
            sw_save: abstract_param(parameters, SW_SAVE_WIDTH, SW_SAVE_SHIFT) as u8,
            // `prm_get_range()` - `PRM_RANGE_W = 1`, `PRM_RANGE_S = 6`.
            range: abstract_param(parameters, RANGE_WIDTH, RANGE_SHIFT) != 0,
            // `prm_get_always()` - `PRM_ALWAYS_W = 1`, `PRM_ALWAYS_S = 6`, the same bit.
            always: abstract_param(parameters, RANGE_WIDTH, RANGE_SHIFT) != 0,
            // `prm_get_makeNum()` - `PRM_MAKENUM_W = 2`, `PRM_MAKENUM_S = 0x1C`.
            make_num: abstract_param(parameters, MAKE_NUM_WIDTH, MAKE_NUM_SHIFT) as u8,
            slots,
            // `mRoomId` - see the note above; `sBs_ClearArea` would leave it 0
            // (`f_pc_base.cpp:97-98`), which is the honest stand-in for "not queried".
            room_id: 0,
            // `fopAcM_GetHomeRoomNo(this)` - supplied by `control` from `Actor::room`.
            home_room: 0,
            seed: Self::seed_of(parameters),
        }
    }

    /// The per-actor xorshift state. There is NO global `cM_rndF` / `cM_rnd` / `cM_rndFX` in this sim
    /// - the brief lists that explicitly, and `ajav.rs:432-438`, `agbsw0.rs:561-573` and
    /// `buoybarrel.rs:349-354` each carry a private one for their own actor. This module follows that
    /// established precedent rather than substituting another subsystem's generator.
    fn next(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        // `cM_rnd()` is a float in [0, 1) (`:274`, `:353`).
        (self.seed >> 8) as f32 / 16777216.0
    }

    /// `cM_rndF(range)` as an integer - `(s32)cM_rndF(range)` (`:43`, `:50`, `:55`, `:60`, `:365`).
    /// NOTE retail's `cM_rndF(x)` is `[0, x)` with NO `min +` term, which is why `set_retry_timer`
    /// writes `m00[12] + (s32)cM_rndF(m00[13])` and `set_appear_timer` writes `m00[14] + ...`
    /// explicitly.
    fn roll(&mut self, span: s32) -> s32 {
        if span <= 0 {
            return 0;
        }
        (self.next() * span as f32) as i32
    }

    /// The stand-alone generator used only by `spawn`, so the five `Ctrl_c::init` rolls are the
    /// first draws from the same stream the actor uses afterwards.
    fn rng_of(parameters: u32) -> Rng {
        Rng {
            seed: Self::seed_of(parameters),
        }
    }

    /// `chk_switch` (`:199-207`).
    ///
    /// ```
    /// s32 switchIndex = prm_get_swSave();
    /// if (switchIndex == 0xFF) { return true; }
    /// else if (fopAcM_isSwitch(this, switchIndex)) { return true; }
    /// return false;
    /// ```
    ///
    /// RULE 11b: `fopAcM_isSwitch(this, i_no)` is `dComIfGs_isSwitch(i_no, homeRoomNo)`, and
    /// `Switches` models only the 0..128 in-memory array - a `swSave` at or above 0x80 reads false
    /// here even though retail routes it to the per-zone `dSv_zoneBit_c` arrays. The REAL index is
    /// passed through; nothing is down-shifted.
    fn chk_switch(&self, switches: &Switches) -> bool {
        if self.sw_save == NO_SWITCH {
            return true;
        }
        switches.test(self.sw_save)
    }

    /// `chk_ship_range` (`:251-257`).
    ///
    /// ```
    /// if (prm_get_range()) { return true; }
    /// return ship->current.pos.absXZ(current.pos) < scale.x * 10000.0f;
    /// ```
    ///
    /// RULE 2: `dStage_actorInit` never writes `fopAcM_prm_class::scale`, so `scale.x` is 1.0.
    fn chk_ship_range(&self, ship: Vec3, origin: Vec3) -> bool {
        if self.range {
            return true;
        }
        let gap = Vec3::new(ship.x - origin.x, 0.0, ship.z - origin.z).length();
        gap < RANGE_SCALE
    }

    /// `chk_ship_spd` (`:260-265`): `if (ship->speedF > attr()->L_Hio[0]) { return true; }`.
    fn chk_ship_spd(ship_speed: f32) -> bool {
        ship_speed > SHIP_SPEED_MIN
    }

    /// `get_make_pos` (`:268-284`), in the SHIP's frame.
    ///
    /// ```
    /// mDoMtx_stack_c::transS(ship->current.pos.x, ship->current.pos.y, ship->current.pos.z);
    /// mDoMtx_stack_c::YrotM(ship->shape_angle.y);
    /// f32 tmp1 = cM_rndF(attr()->L_Hio[2]);
    /// tmp1 += attr()->L_Hio[1];
    /// f32 tmp2 = (cM_rnd() - 0.5f);
    /// tmp2 *= attr()->L_Hio[3];
    /// mDoMtx_stack_c::transM(tmp2, 0.0f, tmp1);
    /// cXyz sp08;
    /// mDoMtx_stack_c::multVec(&cXyz::Zero, &sp08);
    /// outPos->x = sp08.x;
    /// outPos->y = daSea_calcWave(sp08.x, sp08.z);
    /// outPos->z = sp08.z;
    /// ```
    ///
    /// RULE 5b: `YrotM` puts local Z on `(sin a, 0, cos a)`, so `transM(lateral, 0, forward)` is a
    /// rotation by the ship's yaw. `daSea_calcWave` is `Sea::height_at`.
    fn get_make_pos(&mut self, ship: Vec3, ship_yaw: i16, sea: Option<&Sea>) -> Vec3 {
        let forward = MAKE_FWD_BASE + self.roll(MAKE_FWD_SPAN as s32) as f32;
        let lateral = (self.next() - 0.5) * MAKE_SIDE_SPAN;
        let (sin_y, cos_y) = super::stream::sin_cos(ship_yaw);
        let x = ship.x + lateral * cos_y + forward * sin_y;
        let z = ship.z - lateral * sin_y + forward * cos_y;
        // `outPos->y = daSea_calcWave(sp08.x, sp08.z);` (:282)
        let y = sea.map_or(0.0, |sea| sea.height_at(x, z));
        Vec3::new(x, y, z)
    }

    /// `get_aqua_y` (`:234-248`) as far as it can be evaluated:
    /// `daSea_ChkArea` (`:236`) is `Sea::draw_contains` and `daSea_calcWave` (`:237`) is
    /// `Sea::height_at`; the `else` arm's `dBgS_WtrChk` (`:239-246`) is a per-position DZB water
    /// column query that RULE 3b lists as absent, and `super::water::Water` is a single player probe
    /// (`src/sim/water.rs:19-25`), not a substitute. Returns `NO_WATER` outside the sea, which is
    /// the sentinel `chk_make_pos_gnd` tests at `:302`.
    fn get_aqua_y(&self, pos: Vec3, sea: Option<&Sea>) -> f32 {
        match sea {
            Some(sea) if sea.draw_contains(pos.x, pos.z) => sea.height_at(pos.x, pos.z),
            _ => NO_WATER,
        }
    }

    /// `barrel2_get_ptn` (`:350-373`).
    ///
    /// ```
    /// daObjComing::Act_c::Ptn_e cVar2 = PTN_15_e;
    /// f32 dVar2 = cM_rnd();
    /// f32 tmp = attr()->L_Hio[4] + attr()->L_Hio[5];
    /// if (tmp > 0.0f) {
    ///     if (dVar2 < attr()->L_Hio[4]) { cVar2 = PTN_01_e; }
    ///     else if (dVar2 < tmp) { cVar2 = PTN_00_e; }
    /// }
    /// if (cVar2 == PTN_15_e) {
    ///     s32 fVar3 = cM_rndF(16.0f);
    ///     s32 iVar1 = prm_get_tblNum();
    ///     if (iVar1 >= 7) { iVar1 = 0; }
    ///     cVar2 = daObjComing::Act_c::Ptn_e(M_item_tbl[iVar1][fVar3]);
    /// }
    /// ```
    ///
    /// `PTN_15_e = 0x15` (`d_a_obj_coming.h:56`) is the "no special case" sentinel.
    fn barrel2_get_ptn(&mut self) -> u8 {
        let draw = self.next();
        let total = SPECIAL_LOW + SPECIAL_SPAN;
        let mut pattern = 0x15_u8;
        // `if (tmp > 0.0f)` (:356)
        if total > 0.0 {
            // `if (dVar2 < attr()->L_Hio[4]) { cVar2 = PTN_01_e; }` (:357-358)
            if draw < SPECIAL_LOW {
                pattern = 0x01;
            } else if draw < total {
                // `else if (dVar2 < tmp) { cVar2 = PTN_00_e; }` (:359-360)
                pattern = 0x00;
            }
        }
        // `if (cVar2 == PTN_15_e)` (:364)
        if pattern == 0x15 {
            // `s32 fVar3 = cM_rndF(16.0f);` (:365)
            let column = self.roll(PATTERN_SPAN as s32).clamp(0, 15) as usize;
            // ```
            // s32 iVar1 = prm_get_tblNum();
            // if (iVar1 >= 7) { iVar1 = 0; }
            // ```
            // (:366-368) - RULE 10, the clamp is retail's own.
            let row = if self.tbl_num >= TBL_NUM_COUNT {
                0
            } else {
                usize::from(self.tbl_num)
            };
            pattern = ITEM_TBL[row][column];
        }
        pattern
    }

    /// `daObjBarrel2::Act_c::make_prm` (`d_a_obj_barrel2.h:83-91`) verbatim, for
    /// `make_coming(pos, roomNo, type, droppedItem, hasFlag, angleY, tex)` (`:77-82`).
    ///
    /// `buoy = hasFlag ? 1 : 0` is the INVERTED flag `make_coming` passes as `!hasFlag`, so a pattern
    /// with a buoy produces `buoy == 0` and vice versa.
    pub fn make_prm(
        kind: u8,
        dropped_item: u8,
        has_flag: bool,
        texture: u8,
    ) -> u32 {
        let item_no = u32::from(dropped_item & 0x3F);
        let buoy = u32::from(has_flag);
        // `coming << PRM_COMING_S` with `coming = true` (:89).
        let coming = 1_u32;
        // `(tex == daObjBuoyflag::Texture_00_e ? 0 : 1) << PRM_TEXTURE_S` (:87).
        let tex_bit = u32::from(texture != TEXTURE_00);
        (item_no << PRM_ITEM_NO_S)
            | (PRM_ITEM_SAVE << PRM_ITEM_SAVE_S)
            | (u32::from(kind) << PRM_TYPE_S)
            | (buoy << PRM_BUOY_S)
            | (tex_bit << PRM_TEXTURE_S)
            | (coming << PRM_COMING_S)
    }

    /// `barrel2_get_type` (`:378-384`), `barrel2_get_item` (`:389-392`) and `barrel2_get_buoy`
    /// (`:397-401`) for one pattern, as the triple `barrel2_get_param` fills into `PrmBarrel2_c`
    /// (`:414-417`).
    ///
    /// RULE 10: all three tables are 21 wide and the pattern is masked to `& 0x3F` by
    /// `make_prm`, so an out-of-range pattern would read past them in retail; every index is
    /// clamped here.
    pub fn barrel2_params(&mut self) -> (u8, u8, bool) {
        let pattern = self.barrel2_get_ptn() as usize;
        let slot = pattern.min(TYPE_INDIRECT.len() - 1);
        // `return b[a[(s32)ptn]];` (:383)
        let kind = TYPE_TABLE[TYPE_INDIRECT[slot] as usize];
        // `return item_data[(s32)ptn] & 0x3F;` (:391) - 255 & 0x3F == 63.
        let item = ITEM_DATA[slot] & 0x3F;
        // `return buoy[(s32)ptn];` (:400)
        let flag = BUOY_DATA[slot];
        (kind, item, flag)
    }

    /// `chk_make_pos_wall` (`:323-326`):
    /// `M_wall_work.Set((cXyz*)&tmp->m00, (cXyz*)&tmp->m0C, NULL); return
    /// !dComIfG_Bgsp()->LineCross(&M_wall_work);` - the house reduction `Lines::cross`, which
    /// returns `None` when no `Collision` resource exists, so the test passes then. Documented, not
    /// worked around.
    fn chk_make_pos_wall(from: Vec3, to: Vec3, solid: Option<&Collision>) -> bool {
        Lines::cross(from, to, solid).is_none()
    }

    /// `chk_make_pos_act` (`:329-342`), the actor-clearance half of `chk_make_pos`:
    ///
    /// ```
    /// if (fopAcM_IsActor(i_this)) {
    ///     s16 proc = fopAcM_GetName(i_this);
    ///     if (proc != fpcNm_SHIP_e && proc != fpcNm_PLAYER_e &&
    ///         (std::fabsf(i_this->current.pos.y - tmp->m0C.y) < tmp->m1C + 100.0f)) {
    ///         if (i_this->current.pos.absXZ(tmp->m0C) < tmp->m18 + 100.0f) { return i_this; }
    ///     }
    /// }
    /// return NULL;
    /// ```
    ///
    /// `fopAcM_Search(cb, data)` returns the first match, and `chk_make_pos` only tests for NULL -
    /// so a boolean is the exact reduction. The ship and the player are EXCLUDED by name.
    fn chk_make_pos_act(others: &[(Vec3, &str)], centre: Vec3, radius: f32, height: f32) -> bool {
        others.iter().any(|(pos, name)| {
            // `proc != fpcNm_SHIP_e && proc != fpcNm_PLAYER_e` (:335)
            if *name == "Ship" || *name == "Player" {
                return false;
            }
            // `std::fabsf(i_this->current.pos.y - tmp->m0C.y) < tmp->m1C + 100.0f` (:335)
            if (pos.y - centre.y).abs() >= height + ACT_MARGIN {
                return false;
            }
            // `i_this->current.pos.absXZ(tmp->m0C) < tmp->m18 + 100.0f` (:336)
            let gap = Vec3::new(pos.x - centre.x, 0.0, pos.z - centre.z).length();
            gap < radius + ACT_MARGIN
        })
    }

    /// `barrel2_get_param` (`:404-426`) - the whole placement decision for one barrel.
    ///
    /// ```
    /// daShip_c* ship = dComIfGp_getShipActor();
    /// if (ship != NULL && chk_ship_spd(ship) && chk_ship_range(ship)) {
    ///     MakeTmp_c sp08;
    ///     sp08.m00 = ship->current.pos;
    ///     get_make_pos(ship, (cXyz*)&sp08.m0C);
    ///     sp08.m18 = 200.0f;
    ///     sp08.m1C = 400.0f;
    ///     if (chk_make_pos(&sp08)) { ... }
    /// }
    /// return false;
    /// ```
    ///
    /// DROPPED: `chk_make_pos_gnd` (`:287-318`) - the whole body is a per-poly-group
    /// `dComIfG_Bgsp()->GroundCross(&M_gnd_work)` sweep, which RULE 3b lists as absent;
    /// `super::ground::surface` is the merged-DZB plane query and is NOT a substitute (see the
    /// `coming3.rs:26-36` refusal, which declines the identical call). Without it `chk_make_pos`
    /// cannot be evaluated as a whole, so the ground half is reported rather than assumed to pass.
    fn barrel2_get_param(&mut self, ctx: &Ctx<'_>) -> Option<Barrel2> {
        // ```
        // if (ship != NULL && chk_ship_spd(ship) && chk_ship_range(ship)) { ... }
        // ```
        // (:406)
        let ship = ctx.ship?;
        if !Self::chk_ship_spd(ctx.ship_speed) {
            return None;
        }
        if !self.chk_ship_range(ship, ctx.origin) {
            return None;
        }
        // `get_make_pos(ship, (cXyz*)&sp08.m0C); sp08.m18 = 200.0f; sp08.m1C = 400.0f;` (:409-411)
        let pos = self.get_make_pos(ship, ctx.ship_yaw, ctx.sea);
        let radius = GND_SWEEP;
        let height = ACT_HEIGHT;
        // ```
        // if (chk_make_pos(&sp08)) { ... }
        // ```
        // (:412) - `chk_make_pos` (`:345-347`) is
        // `chk_make_pos_gnd(tmp) && chk_make_pos_wall(tmp) && fopAcM_Search(...) == NULL`.
        // The wall and actor arms are modelled; the ground arm is NOT (see above), so it is
        // reported and NOT assumed.
        if !Self::chk_make_pos_wall(ship, pos, ctx.solid) {
            return None;
        }
        if Self::chk_make_pos_act(ctx.others, pos, radius, height) {
            return None;
        }
        let (kind, item, flag) = self.barrel2_params();
        // ```
        // s16 angle = cLib_targetAngleY(fopAcM_GetPosition_p(ship), (cXyz*)&sp08.m0C);
        // s16 rnd = cM_rndFX(0x2000);
        // barrel2->mAngleY = angle + rnd;
        // ```
        // (:419-421) - the yaw FROM the ship TOWARD the barrel, plus a +/- 0x2000 jitter.
        let to_barrel = Vec3::new(pos.x - ship.x, 0.0, pos.z - ship.z);
        let angle = super::walk::to_s16(to_barrel.x.atan2(to_barrel.z));
        let jitter = self.roll(ANGLE_JITTER as s32) as i16 - (ANGLE_JITTER as i16) / 2;
        Some(Barrel2 {
            pos,
            kind,
            item,
            flag,
            angle: angle.wrapping_add(jitter),
        })
    }

    /// `mode_barrel2_wait` (`:429-451`).
    ///
    /// ```
    /// s32 bVar1 = dComIfGp_roomControl_getStayNo();
    /// if (chk_switch() && (fopAcM_GetHomeRoomNo(this) == bVar1 || mRoomId == bVar1 ||
    ///                       prm_get_always())) {
    ///     if (ctrl->m04 <= 0) {
    ///         PrmBarrel2_c barrel2;
    ///         if (barrel2_get_param(&barrel2)) {
    ///             ctrl->m08 = daObjBarrel2::Act_c::make_coming(barrel2.mPos,
    ///                 fopAcM_GetHomeRoomNo(this), Type_e(barrel2.mType), barrel2.mItem,
    ///                 barrel2.m14, barrel2.mAngleY, daObjBuoyflag::Texture_00_e);
    ///             ctrl->set_appear_timer(this);
    ///             ctrl->m00 = 1;
    ///         } else { ctrl->set_retry_timer(this); }
    ///     }
    /// }
    /// ```
    ///
    /// The room test has three alternatives and only two survive: `mRoomId` is the un-queryable
    /// per-poly ground result (see [`Coming::room_id`]), while `prm_get_always()` - bit 6, the same
    /// bit as `prm_get_range()` - short-circuits it anyway whenever the flag is set.
    fn mode_barrel2_wait(&mut self, index: usize, ctx: &mut Ctx<'_>) {
        // `if (ctrl->m04 <= 0)` (:432)
        if self.slots[index].timer > 0 {
            return;
        }
        // ```
        // if (chk_switch() && (fopAcM_GetHomeRoomNo(this) == bVar1 || mRoomId == bVar1 ||
        //                       prm_get_always()))
        // ```
        // (:431)
        if !self.chk_switch(ctx.switches) {
            return;
        }
        let in_room = self.home_room == ctx.stay
            // `mRoomId == bVar1` - the un-queryable arm, see above.
            || self.room_id == ctx.stay
            // `prm_get_always()` (:431)
            || self.always;
        if !in_room {
            return;
        }
        match self.barrel2_get_param(ctx) {
            // ```
            // ctrl->m08 = daObjBarrel2::Act_c::make_coming(...);
            // ctrl->set_appear_timer(this);
            // ctrl->m00 = 1;
            // ```
            // (:435-445)
            Some(barrel) => {
                let parameters = Self::make_prm(barrel.kind, barrel.item, barrel.flag, TEXTURE_00);
                let child = ctx.commands.spawn((
                    Actor {
                        name: super::buoybarrel::BARREL2_ACTOR.to_owned(),
                        parameters,
                        // `make_coming(..., fopAcM_GetHomeRoomNo(this), ...)` (:437) - the barrel
                        // is parented to this actor's HOME room, not its current one.
                        room: self.home_room,
                        pos: barrel.pos,
                        angle: [0, barrel.angle, 0],
                        ..Default::default()
                    },
                    // `!hasFlag` is what `make_coming` passes as `hasFlag` (d_a_obj_barrel2.h:78),
                    // so `barrel2.m14` (the pattern's own flag) is inverted here.
                    BuoyBarrel::spawn(parameters, barrel.pos),
                ));
                self.slots[index].barrel = Some(child.id());
                let roll = self.roll(APPEAR_SPAN);
                self.slots[index].set_appear_timer(roll);
                // `ctrl->m00 = 1;` (:445) - `mode_barrel2_appear`.
                self.slots[index].mode = SlotMode::Appear;
            }
            // `} else { ctrl->set_retry_timer(this); }` (:446-448)
            None => {
                let roll = self.roll(RETRY_SPAN);
                self.slots[index].set_retry_timer(roll);
            }
        }
    }

    /// `mode_barrel2_appear` (`:456-476`).
    ///
    /// ```
    /// if (ctrl->m04 <= 0) {
    ///     if (ctrl->m08 != fpcM_ERROR_PROCESS_ID_e) {
    ///         fopAc_ac_c* ac;
    ///         if (fopAcM_SearchByID(ctrl->m08, &ac)) {
    ///             if (ac != NULL) { barrel2_act->exit_req(); }
    ///         } else { ctrl->m08 = fpcM_ERROR_PROCESS_ID_e; }
    ///     }
    ///     if (ctrl->m08 != fpcM_ERROR_PROCESS_ID_e) {
    ///         ctrl->set_wait_timer(this);
    ///         ctrl->m00 = 0;
    ///     }
    /// }
    /// ```
    ///
    /// `barrel2_act->exit_req()` is `m474 = 1` (`d_a_obj_barrel2.h:94`), which `buoybarrel.rs` models
    /// as `BuoyBarrel::request_exit`.
    fn mode_barrel2_appear(
        &mut self,
        index: usize,
        ctx: &Ctx<'_>,
        barrels: &mut Query<&mut BuoyBarrel>,
    ) {
        // `if (ctrl->m04 <= 0)` (:457)
        if self.slots[index].timer > 0 {
            return;
        }
        let alive = self.slots[index].barrel;
        let Some(barrel) = alive else {
            // `ctrl->m08` is already the error sentinel, so both `if` bodies fall through.
            return;
        };
        // ```
        // if (fopAcM_SearchByID(ctrl->m08, &ac)) { if (ac != NULL) { barrel2_act->exit_req(); } }
        // else { ctrl->m08 = fpcM_ERROR_PROCESS_ID_e; }
        // ```
        // (:460-468)
        if barrels.get_mut(barrel).is_ok() {
            if let Ok(mut found) = barrels.get_mut(barrel) {
                // `barrel2_act->exit_req();` (:464)
                found.request_exit();
            }
        } else {
            // `ctrl->m08 = fpcM_ERROR_PROCESS_ID_e;` (:467)
            self.slots[index].barrel = None;
            return;
        }
        // ```
        // if (ctrl->m08 != fpcM_ERROR_PROCESS_ID_e) { ctrl->set_wait_timer(this); ctrl->m00 = 0; }
        // ```
        // (:471-474)
        let roll = self.roll(WAIT_SPAN[slot_of(self.wait_time)]);
        self.slots[index].set_wait_timer(self.wait_time, roll);
        // `ctrl->m00 = 0;` (:473) - back to `mode_barrel2_wait`.
        self.slots[index].mode = SlotMode::Wait;
    }

    /// `mode_barrel2_leave` (`:481-494`): the same exit request, with no follow-up.
    fn mode_barrel2_leave(
        &mut self,
        index: usize,
        barrels: &mut Query<&mut BuoyBarrel>,
    ) {
        let Some(barrel) = self.slots[index].barrel else {
            return;
        };
        if barrels.get_mut(barrel).is_ok() {
            if let Ok(mut found) = barrels.get_mut(barrel) {
                // `barrel2_act->exit_req();` (:488)
                found.request_exit();
            }
        } else {
            // `ctrl->m08 = fpcM_ERROR_PROCESS_ID_e;` (:491)
            self.slots[index].barrel = None;
        }
    }

    /// `daObjComing::Act_c::_execute` (`:497-527`).
    ///
    /// ```
    /// s32 end = attr()->m00[prm_get_makeNum()];
    /// bool bVar2 = true;
    /// for (s32 i = 0; i < end; i++) {
    ///     Ctrl_c* ctrl = &m294[i];
    ///     if (ctrl->m04 > 0) { ctrl->m04--; }
    ///     (this->*mode_proc[m290][ctrl->m00])(ctrl);
    ///     if ((ctrl->m00 != 2) || (ctrl->m08 != fpcM_ERROR_PROCESS_ID_e)) { bVar2 = false; }
    /// }
    /// if (bVar2) { fopAcM_delete(this); }
    /// ```
    ///
    /// Returns TRUE when `fopAcM_delete(this)` fired, so `control` despawns. The `mode_proc` ROW
    /// index is `m290`, whose table has one row - see [`Coming::mode_row`].
    fn execute(&mut self, ctx: &mut Ctx<'_>, barrels: &mut Query<&mut BuoyBarrel>) -> bool {
        // `s32 end = attr()->m00[prm_get_makeNum()];` (:507) - `L_Attr::m00[0..3]`.
        let end = COUNT_TABLE[usize::from(self.make_num).min(COUNT_TABLE.len() - 1)];
        let mut all_done = true;
        for index in 0..(end as usize).min(SLOTS) {
            // ```
            // if (ctrl->m04 > 0) { ctrl->m04--; }
            // ```
            // (:512-514)
            if self.slots[index].timer > 0 {
                self.slots[index].timer -= 1;
            }
            // `(this->*mode_proc[m290][ctrl->m00])(ctrl);` (:516)
            match self.slots[index].mode {
                SlotMode::Wait => self.mode_barrel2_wait(index, ctx),
                SlotMode::Appear => self.mode_barrel2_appear(index, ctx, barrels),
                SlotMode::Leave => self.mode_barrel2_leave(index, barrels),
            }
            // ```
            // if ((ctrl->m00 != 2) || (ctrl->m08 != fpcM_ERROR_PROCESS_ID_e)) { bVar2 = false; }
            // ```
            // (:518-520) - `m00 != 2` is `mode != SlotMode::Leave`.
            if self.slots[index].mode != SlotMode::Leave || self.slots[index].barrel.is_some() {
                all_done = false;
            }
        }
        // `if (bVar2) { fopAcM_delete(this); }` (:523-525)
        all_done
    }
}

/// `daObjComing::PrmBarrel2_c` (`d_a_obj_coming.h:9-16`) filled by `barrel2_get_param` (`:414-421`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Barrel2 {
    /// `mPos` (`:11`) - the world position `get_make_pos` produced.
    pub pos: Vec3,
    /// `mType` (`:13`) - `barrel2_get_type`'s `b[a[ptn]]` (`:383`).
    pub kind: u8,
    /// `mItem` (`:14`) - `barrel2_get_item`'s `item_data[ptn] & 0x3F` (`:391`).
    pub item: u8,
    /// `m14` (`:15`) - `barrel2_get_buoy`'s table entry (`:400`), which `make_coming` INVERTS.
    pub flag: bool,
    /// `mAngleY` (`:16`) - `cLib_targetAngleY` plus a `cM_rndFX(0x2000)` jitter (`:419-421`).
    pub angle: i16,
}

/// `daObj::PrmAbstract(this, W, S)` (`d/d_a_obj.h:14`), `((1 << W) - 1) & (param >> S)`, copied
/// exactly. `fopAcM_GetParam(this)` returns the WHOLE `u32` (`f_op_actor_mng.h:127-129`).
fn abstract_param(param: u32, width: u32, shift: u32) -> u32 {
    ((1 << width) - 1) & (param >> shift)
}

impl Coming {
    /// A deterministic per-actor seed from the parameter word. Retail's global `cM_rndF` is a
    /// single shared generator with no counterpart here, so the same module family carries a private
    /// one - see the RNG note in the module docs.
    fn seed_of(parameters: u32) -> u32 {
        let mut seed = parameters ^ 0x2540_F491;
        if seed == 0 {
            seed = 0x2540_F491;
        }
        seed
    }
}

/// Everything `execute` needs beyond the component, in one bundle - the `hbrf1` / `tagetc` shape.
struct Ctx<'a> {
    origin: Vec3,
    ship: Option<Vec3>,
    ship_yaw: i16,
    ship_speed: f32,
    sea: Option<&'a Sea>,
    solid: Option<&'a Collision>,
    stay: i8,
    switches: &'a Switches,
    others: &'a [(Vec3, &'static str)],
    commands: &'a mut Commands,
}

pub fn is_coming(name: &str) -> bool {
    COMING_ACTORS.contains(&name)
}

pub fn control(
    mut actors: Query<(Entity, &mut Actor, &mut Coming), With<Coming>>,
    mut barrels: Query<&mut BuoyBarrel>,
    ships: Query<(&Actor, &Ship), Without<Coming>>,
    players: Query<&Player>,
    foes: Query<&Actor, With<Enemy>>,
    sea: Option<Res<Sea>>,
    solid: Option<Res<Collision>>,
    switches: Res<Switches>,
    stay: Res<StayRoom>,
    mut commands: Commands,
) {
    let sea_ref = sea.as_deref();
    let solid_ref = solid.as_deref();
    // ```
    // daShip_c* ship = dComIfGp_getShipActor();
    // f32 abs = (player->current.pos - current.pos).absXZ();     // not this actor
    // f32 max = 2000.0f;                                          // not this actor
    // ```
    // (:405) - `dComIfGp_getShipActor()` is a process-id lookup; `Query<(&Actor, &Ship)>` is the
    // house reduction. The `&mut Actor` writer carries `With<Coming>` and this reader
    // `Without<Coming>`, which is the disjointness pair RULE 9 asks for.
    let ship = ships.iter().next();
    let ship_pos = ship.map(|(actor, _)| actor.pos);
    let ship_speed = ship.map_or(0.0, |(_, ship)| ship.speed);
    let ship_yaw = ship.map_or(0, |(actor, _)| actor.angle[1]);
    // ```
    // if (proc != fpcNm_SHIP_e && proc != fpcNm_PLAYER_e) { ... }
    // ```
    // (:335) - `fopAcM_Search(chk_make_pos_act, ...)` visits EVERY actor and the decomp does NOT
    // filter by group, so the candidate list is every actor's position under a name that is neither
    // `fpcNm_SHIP_e` nor `fpcNm_PLAYER_e`. The two excluded ones are added with their real names so
    // `chk_make_pos_act`'s own exclusion still runs on them, which keeps the predicate honest about
    // which entities it skips.
    let mut others: Vec<(Vec3, &'static str)> = Vec::new();
    for foe in &foes {
        others.push((foe.pos, "Enemy"));
    }
    for player in &players {
        others.push((player.pos, "Player"));
    }
    for (actor, _) in &ships {
        others.push((actor.pos, "Ship"));
    }

    for (entity, mut actor, mut coming) in &mut actors {
        // `fopAcM_GetHomeRoomNo(this)` (`:431`, `:437`) - the actor's home room, which
        // `dStage_actorInit` seeded from the OBJNAME room column.
        coming.home_room = actor.room;
        let mut ctx = Ctx {
            // `current.pos` (`:256`) - the spawner's own position, which `fopAc_Init` seeded from
            // the stage placement and nothing ever moves.
            origin: actor.pos,
            ship: ship_pos,
            ship_yaw,
            ship_speed,
            sea: sea_ref,
            solid: solid_ref,
            stay: stay.0,
            switches: &switches,
            others: &others,
            commands: &mut commands,
        };
        // `daObjComing::Act_c::_execute` (`:497-527`)
        if coming.execute(&mut ctx, &mut barrels) {
            // `fopAcM_delete(this);` (:524) - a REAL delete request in retail.
            let _ = commands.entity(entity).despawn();
        }
    }
}
