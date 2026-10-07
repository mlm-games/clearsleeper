//! `daGy_Ctrl_c` (`d_a_gy_ctrl.h:5-79`) - the Outset Island / Windfall seagull flock controller
//! behind the stage strings `GyCtrl` and `GyCtrlB`. `Nonmatching` count: 0, so every token below is
//! transcribed. This file REPLACES an earlier uncited draft of the same module; the audit verdict
//! is in the report.
//!
//! Nothing else in `src/sim/` claims either string: `GyCtrl` is `fpcNm_GY_CTRL_e`
//! (`d_stage.cpp:809`) and `GyCtrlB` is `fpcNm_GY_CTRLB_e` (`:810`). The spawned gull is a
//! DIFFERENT actor: `fopAcM_createChild` (`:345`) makes `fpcNm_GY_e`, whose stage string is `"Gy"`
//! (`d_stage.cpp:808`) - not claimed here. `src/sim/raft.rs` claims the five `fpcNm_OBJ_IKADA_e`
//! rows (`d_stage.cpp:820-824`), which are `d_a_obj_ikada.cpp`, NOT this translation unit; the only
//! way a raft reaches this actor is the `searchNearActor` name test at `:77`.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::flags::Switches;
use super::inventory::{Got, ITEM_BOOMERANG};
use super::player::Player;
use super::player_status::{STTS0_SHIP_RIDE, STTS0_SWIM, STTS0_UNK1000000, PlayerStatus};
use super::ship::Ship;
use crate::text::msgproc::MsgGameInfo;

/// `OBJNAME("GyCtrl", fpcNm_GY_CTRL_e, 255, 0)` (`d_stage.cpp:809`) and
/// `OBJNAME("GyCtrlB", fpcNm_GY_CTRLB_e, 255, 0)` (`:810`). Two
/// `actor_process_profile_definition` rows share this translation unit and one method table
/// (`d_a_gy_ctrl.cpp:578-618`); both carry `fopAc_CULLBOX_4_e` (`:600`, `:617`) and
/// `fopAcStts_UNK40000_e` (`:598`, `:615`) - note `fopAcStts_CULL_e` is ABSENT, so this actor is
/// never culled off in retail.
pub const GY_CTRL_ACTORS: [&str; 2] = ["GyCtrl", "GyCtrlB"];

/// The name `checkGyCtrlExist` looks for: `fopAcM_SearchByName(fpcNm_GY_CTRL_e)` (`:523`) is a
/// PROCESS-NAME lookup, so it satisfies itself from a `"GyCtrl"` instance and NOT from a `"GyCtrlB"`
/// one even though both rows are this class.
const GY_CTRL_NAME: &str = "GyCtrl"; // <- d_stage.cpp:809 (fpcNm_GY_CTRL_e)

/// The gull `fopAcM_createChild(fpcNm_GY_e, ...)` (`:345`) makes. `d_a_gy.cpp` is a RULE 1-STUB
/// (42 `Nonmatching`), so the child is a bare [`Gull`] + `Actor` and the parent's `m4AC` gate -
/// which reads `daGy_c::m2B0` - is OMITTED.
const GY_ACTOR: &str = "Gy"; // <- d_stage.cpp:808 (fpcNm_GY_e)

/// `daGy_Ctrl_HIO_c` (`d_a_gy_ctrl.cpp:21-43`), constructor `:48-62`. These are the FILE-SCOPE HIO
/// defaults, not `g_regHIO` registers, so they are real literals; `DEMO_SELECT(a, b)` (`:56-57`)
/// takes its second argument for `VERSION_NTSC_U > VERSION_JPN > VERSION_DEMO`.
const HIO_M06: i32 = 100; // <- d_a_gy_ctrl.cpp:51
const HIO_M08: i32 = 240; // <- d_a_gy_ctrl.cpp:52
const HIO_M0A: i16 = 200; // <- d_a_gy_ctrl.cpp:59
const HIO_M0C: i16 = 250; // <- d_a_gy_ctrl.cpp:61
const HIO_M10: f32 = 10.0; // <- d_a_gy_ctrl.cpp:50
const HIO_M18: f32 = 1000.0; // <- d_a_gy_ctrl.cpp:60
const HIO_M1C: f32 = 700.0; // <- d_a_gy_ctrl.cpp:58
const HIO_M24: f32 = 500.0; // <- d_a_gy_ctrl.cpp:56 (DEMO_SELECT(1000.0f, 500.0f))
const HIO_M28: f32 = -500.0; // <- d_a_gy_ctrl.cpp:57 (DEMO_SELECT(-100.0f, -500.0f))
const HIO_M2C: f32 = 10.0; // <- d_a_gy_ctrl.cpp:54

/// `fopAc_CULLBOX_4_e` (`f_op_actor_mng.cpp:564`) -
/// `fopAc_cullSizeBox(cXyz(-100.0f, 0.0f, -100.0f), cXyz(100.0f, 800.0f, 100.0f))`, which both
/// profiles name (`:600`, `:617`).
const CULL_MIN: [f32; 3] = [-100.0, 0.0, -100.0]; // <- f_op_actor_mng.cpp:564
const CULL_MAX: [f32; 3] = [100.0, 800.0, 100.0]; // <- f_op_actor_mng.cpp:564

/// `m290[5]`, `m2CC[5]`, `m308[5]`, `m360[5]`, `m374[5]` (`d_a_gy_ctrl.h:26`, `:30`, `:34`, `:43`,
/// `:44`) are five wide; `m3EC[16]` (`:70`) is SIXTEEN wide. The earlier draft looped `checkPath`
/// five times - wrong, and it cost a real behaviour.
const FLOCK: usize = 5; // <- d_a_gy_ctrl.h:26, :30, :34, :43, :44
const AIM_COUNT: usize = 16; // <- d_a_gy_ctrl.h:70 (m3EC[16])

/// `s16 tmp = 0xFFF; s16 tmp2 = tmp * i;` (`:197-198`) - `checkPath`'s per-index yaw step. The
/// multiply is an `int` multiply narrowed back to `s16`, so it WRAPS.
const AIM_STEP: i32 = 0xFFF; // <- d_a_gy_ctrl.cpp:197
/// `if (uVar1 == 0xff) { m32C = 10000.0f; }` (`:513-515`).
const RADIUS_FULL: f32 = 10000.0; // <- d_a_gy_ctrl.cpp:514
/// `m32C = uVar1 * 1000.0f;` (`:516`).
const RADIUS_UNIT: f32 = 1000.0; // <- d_a_gy_ctrl.cpp:516
/// `fopAcM_searchActorDistanceXZ(this, arg1) < 6000.0f` (`:73`, `:78`) - an XZ distance.
const NEAR_RANGE: f32 = 6000.0; // <- d_a_gy_ctrl.cpp:73, :78
/// `if (m328 == 0xf) { m328 = 0; }` (`:505-507`).
const AREA_NONE: u8 = 0x0F; // <- d_a_gy_ctrl.cpp:505
/// `if (m329 == 0xf) { m329 = 1; }` (`:509-511`).
const COUNT_FULL: u8 = 0x0F; // <- d_a_gy_ctrl.cpp:509
/// `m358 = 0x1e` (`:91`, `:99`, `:105`).
const TARGET_TICK: i32 = 0x1E; // <- d_a_gy_ctrl.cpp:91
/// `cLib_addCalc2(&m348, m318, 0.1f, l_HIO.m2C)` (`:264`).
const SWAY_SCALE: f32 = 0.1; // <- d_a_gy_ctrl.cpp:264
/// `s16 uVar2 = uVar1 + 0x500;` (`:234`) - `m2CC`'s look-ahead yaw offset.
const AHEAD_YAW: i16 = 0x500; // <- d_a_gy_ctrl.cpp:234
/// `s16 tmp2 = (s16)(0xffff / m329);` (`:230`) - the flock's yaw fan.
const FAN_FULL: i32 = 0xFFFF; // <- d_a_gy_ctrl.cpp:230
/// `sp18.y = -1000.0f;` (`:343`) - the gull spawn height clamp.
const SPAWN_DEPTH: f32 = -1000.0; // <- d_a_gy_ctrl.cpp:343
/// `m4B0 = REG8_S(8) + 30` (`:380`) and `m4B0 = REG8_S(9) + 30` (`:407`); RULE 2b folds both
/// `REG8_*` registers to zero.
const HIDE_TICK: i32 = 30; // <- d_a_gy_ctrl.cpp:380, :407
/// `m4AC = true;` (`:115`) - the "every spawned gull is calm" latch, whose inner test
/// `pfVar2->m2B0 != 0` (`:119`) has no counterpart. See [`GyCtrl::set_target`].
const CALM_DEFAULT: bool = true; // <- d_a_gy_ctrl.cpp:115

/// The four `mode_tbl` entries of `modeProc` (`d_a_gy_ctrl.cpp:276-297`). `modeProc(PROC_INIT_e, n)`
/// runs `mode_tbl[n].init` and `modeProc(PROC_EXEC_e, ...)` runs `mode_tbl[m324].exec`
/// (`:299-304`), where `m324` (`d_a_gy_ctrl.h:36`) is the CURRENT index.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// `mode_tbl[0]` - `"SW_WAIT"` / `modeSwWait` (`:280`, `:312`).
    #[default]
    SwWait,
    /// `mode_tbl[1]` - `"CREATE"` / `modeCreate` (`:285`, `:331`).
    Create,
    /// `mode_tbl[2]` - `"WAIT"` / `modeWait` (`:290`, `:364`).
    Wait,
    /// `mode_tbl[3]` - `"HIDE"` / `modeHide` (`:295`, `:385`).
    Hide,
}

/// The spawned gull `fpcNm_GY_e` (`d_a_gy.cpp:348`, stage string `"Gy"`). `daGy_c` is a RULE 1-STUB
/// (42 `Nonmatching`), so the entity exists and is findable by `fopAcM_SearchByID` - all
/// `deadCheckGy` (`:423`) and the slot bookkeeping need - but it has no behaviour here.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gull;

/// `daGy_Ctrl_c` (`d_a_gy_ctrl.h:5-79`) - the flock controller.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. The class registers NO `dCcD` collider of
/// any kind - there is no `dCcD_Stts`, `dCcD_Cyl` or `dCcD_Sph` member in `d_a_gy_ctrl.h:5-79`, and
/// `dComIfG_Ccsp()->Set(...)` appears nowhere in the translation unit. The profile `Group` is
/// `fopAc_ACTOR_e` (`:595`, `:612`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached;
/// `combat::control` would despawn a controller retail cannot damage.
#[derive(Component, Debug, Clone)]
pub struct GyCtrl {
    /// `m324` (`d_a_gy_ctrl.h:36`) - the current `mode_tbl` index.
    pub mode: Mode,
    /// `m328` (`d_a_gy_ctrl.h:39`) - `(param >> 0) & 0xF`, with `0x0F -> 0` (`:500`, `:505-507`).
    pub use_area: u8,
    /// `m329` (`d_a_gy_ctrl.h:40`) - `(param >> 4) & 0xF`, with `0x0F -> 1` (`:501`, `:509-511`). It
    /// indexes arrays that are only [`FLOCK`] wide, so RULE 10 clamps every loop.
    pub count: u8,
    /// `m330` (`d_a_gy_ctrl.h:42`) - `(param >> 0x18) & 0xFF`, the bit `modeSwWait` waits on.
    pub switch_no: u8,
    /// `m331` (`d_a_gy_ctrl.h:43`) - set to 1 by `createInitNoArer` (`:472`) when `m328 == 0`, and
    /// read by ANOTHER controller's `checkGyCtrlExist` (`:524`).
    pub no_area: u8,
    /// `m32C` (`d_a_gy_ctrl.h:41`) - `(param >> 8) & 0xFF`; `0xFF -> 10000.0`, else `n * 1000.0`.
    pub near_radius: f32,
    /// `m334` (`d_a_gy_ctrl.h:44`) - a room number, seeded by `createInitNoArer` (`:473`) and
    /// rewritten at the END of every `_execute` (`:440`). It is NOT always the actor's own room:
    /// `fpcBs_Create`'s `sBs_ClearArea` (`f_pc_base.cpp:97-98`) leaves it 0 until the first
    /// `_execute`, and `createInitNoArer` never runs at all when `m328 != 0`.
    pub base_room: i32,
    /// `m318` (`d_a_gy_ctrl.h:35`) - `setPathTargetPos`'s lateral speed target (`:147`, `:154`).
    pub patrol_speed: f32,
    /// `m320` (`d_a_gy_ctrl.h:37`) - `0` targets the player, `1` targets the ship (`:144-158`).
    pub target: i32,
    /// `m348` (`d_a_gy_ctrl.h:46`) - the lateral sway offset, chased toward `m318` at `:264`.
    pub sway: f32,
    /// `m34C` (`d_a_gy_ctrl.h:47`, an `s16`) - the fan's base yaw, advanced by `m0A` / `m0C`
    /// (`:148`, `:155`).
    pub sway_base: i16,
    /// `m308[5]` (`d_a_gy_ctrl.h:34`) - each gull's yaw, and the `csXyz(0, m308[i], 0)` spawn
    /// pitch (`:341-342`).
    pub angles: [i16; FLOCK],
    /// `m290[5]` (`d_a_gy_ctrl.h:26`) - each gull's world position (`:243`).
    pub path: [Vec3; FLOCK],
    /// `m2CC[5]` (`d_a_gy_ctrl.h:30`) - each gull's look-ahead point; `lineCheck` REPLACES it with
    /// the cross point on a hit (`:251`, `:258-262`).
    pub ahead: [Vec3; FLOCK],
    /// `m338` (`d_a_gy_ctrl.h:45`) - the anchor `current.pos` is measured against (`:167`).
    pub first: Vec3,
    /// `m344` (`d_a_gy_ctrl.h:46`) - the previous tick's `setPathTargetPos()` result.
    pub reached: bool,
    /// `m3E8` (`d_a_gy_ctrl.h:69`) - the previous tick's `checkPath()` result.
    pub clear: bool,
    /// `m4B0` (`d_a_gy_ctrl.h:73`) - the `HIDE` cooldown, seeded by `modeHideInit` (`:380`).
    pub hide_timer: i32,
    /// `m358` (`d_a_gy_ctrl.h:56`) - the camera-target reload timer.
    pub target_timer: i32,
    /// `m350` (`d_a_gy_ctrl.h:48`) - the gull spawn cooldown.
    pub spawn_timer: i32,
    /// `m360[5]` (`d_a_gy_ctrl.h:43`) - the spawned gull's process id per slot.
    pub flock: [Option<Entity>; FLOCK],
    /// `m374[5]` (`d_a_gy_ctrl.h:44`) - whether slot `i` ever spawned a gull.
    pub alive: [bool; FLOCK],
    /// One-shot guard so `createInit` (`:477-490`) runs exactly once, on the first tick, where the
    /// `Got` store and `Commands` are available.
    pub entered: bool,
    /// RULE 6b: `_create` returns `cPhs_ERROR_e` for a player without the boomerang (`:540-542`)
    /// or for a duplicate area-0 controller (`:536-538`). That cancels the process during Create
    /// (`f_pc_create_req.cpp:106-108`), so the actor NEVER executes even once: spawn it, then
    /// despawn on the FIRST TICK rather than leaking a bare `Actor`.
    pub refused: bool,
    /// `m4AC` (`d_a_gy_ctrl.h:71`) - "every spawned gull is calm", initialised `CALM_DEFAULT` by the
    /// `VERSION > VERSION_DEMO` block at `:115`. Its only writer besides that seed is
    /// `pfVar2->m2B0 != 0` (`:119-121`), which has no counterpart, so it never clears. Stored so the
    /// fold is visible rather than silently absent.
    pub calm: bool,
}

impl Default for GyCtrl {
    fn default() -> Self {
        Self {
            mode: Mode::SwWait,
            use_area: 0,
            count: 0,
            switch_no: 0,
            no_area: 0,
            near_radius: 0.0,
            base_room: 0,
            patrol_speed: 0.0,
            target: 0,
            sway: 0.0,
            sway_base: 0,
            angles: [0; FLOCK],
            path: [Vec3::ZERO; FLOCK],
            ahead: [Vec3::ZERO; FLOCK],
            first: Vec3::ZERO,
            reached: false,
            clear: false,
            hide_timer: 0,
            target_timer: 0,
            spawn_timer: 0,
            flock: [None; FLOCK],
            alive: [false; FLOCK],
            entered: false,
            refused: false,
            calm: CALM_DEFAULT,
        }
    }
}

impl GyCtrl {
    /// `daGy_Ctrl_c::getArg` (`:497-518`) over the profile's whole `u32` parameter
    /// (`Parameters` is `0`, `:594`).
    ///
    /// `u8 getParam(u32 arg1, u8 arg2, u8 arg3) { return ((1 << arg3) - 1) & (arg1 >> arg2); }`
    /// (`:493-495`) - shift and mask copied exactly.
    pub fn spawn(parameters: u32) -> Self {
        let mut tag = Self::default();
        // `m328 = getParam(uVar3, 0, 4);` (:500) then `if (m328 == 0xf) { m328 = 0; }` (:505-507)
        let use_area = get_param(parameters, 0, 4);
        tag.use_area = if use_area == AREA_NONE { 0 } else { use_area };
        // `m329 = getParam(uVar3, 4, 4);` (:501) then `if (m329 == 0xf) { m329 = 1; }` (:509-511)
        let count = get_param(parameters, 4, 4);
        tag.count = if count == COUNT_FULL { 1 } else { count };
        // `u8 uVar1 = getParam(uVar3, 8, 8);` (:502) then the radius fold (:513-517)
        let radius = get_param(parameters, 8, 8);
        tag.near_radius = if radius == 0xFF {
            RADIUS_FULL
        } else {
            f32::from(radius) * RADIUS_UNIT
        };
        // `m330 = getParam(uVar3, 0x18, 8);` (:503)
        tag.switch_no = get_param(parameters, 0x18, 8);
        tag
    }

    /// `fopAc_CULLBOX_4_e` (`f_op_actor_mng.cpp:564`) - the one real cull box, named by BOTH profiles
    /// (`:600`, `:617`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daGy_Ctrl_c::createInit` (`:477-490`), first tick only.
    ///
    /// ```
    /// if (m330 != 0xff) { modeProc(PROC_INIT_e, 0); } else { modeProc(PROC_INIT_e, 1); }
    /// m320 = 0;
    /// m338 = current.pos;
    /// if (m328 == 0) { createInitNoArer(); }
    /// ```
    ///
    /// `createInitNoArer` (`:470-474`) then does `fopAcM_setStageLayer(this)` (a layer change with
    /// no modelled reader), `m331 = 1` and `m334 = fopAcM_GetRoomNo(this)`. Index 0's init is
    /// `modeSwWaitInit`, an empty body (`:308-309`); index 1's is `modeCreateInit`, which only sets
    /// `m350` (`:326-328`) on its first EXEC, and `m350` is already 0 from the zero-fill.
    fn create_init(&mut self, actor: &Actor) {
        // `if (m330 != 0xff) { ... 0 ... } else { ... 1 ... }` (:478-482)
        self.mode = if self.switch_no != 0xFF {
            Mode::SwWait
        } else {
            Mode::Create
        };
        // `m320 = 0;` (:484)
        self.target = 0;
        // `m338 = current.pos;` (:485)
        self.first = actor.pos;
        // `if (m328 == 0) { createInitNoArer(); }` (:487-489)
        if self.use_area == 0 {
            self.no_area = 1;
            self.base_room = i32::from(actor.room);
        }
        self.entered = true;
    }

    /// `daGy_Ctrl_c::_create` (`:531-546`)'s two refusal guards.
    ///
    /// * `if (m328 == 0 && checkGyCtrlExist()) { return cPhs_ERROR_e; }` (`:536-538`), where
    ///   `checkGyCtrlExist` (`:521-528`) is `fopAcM_SearchByName(fpcNm_GY_CTRL_e)` - a PROC-NAME
    ///   lookup, so only a `"GyCtrl"` instance satisfies it - and then
    ///   `pfVar1->m328 == 0 && pfVar1->m331 == 1`.
    /// * `if (!dComIfGs_checkGetItem(dItemNo_BOOMERANG_e)) { return cPhs_ERROR_e; }` (`:540-542`).
    ///   `dItemNo_BOOMERANG_e` is `0x2D` (`d_item_data.h:52`), already named
    ///   `super::inventory::ITEM_BOOMERANG`.
    fn create_refused(&mut self, got: &Got, peers: &[(u8, u8)]) {
        // `!dComIfGs_checkGetItem(dItemNo_BOOMERANG_e)` (:540) - this guard runs FIRST in retail.
        if !got.has(0, ITEM_BOOMERANG) {
            self.refused = true;
            return;
        }
        // `m328 == 0 && checkGyCtrlExist()` (:536)
        if self.use_area == 0 && peers.iter().any(|&(area, no_area)| area == 0 && no_area == 1) {
            self.refused = true;
        }
    }

    /// `daGy_Ctrl_c::setTarget` (`:87-138`).
    ///
    /// ```
    /// if (dComIfGp_checkPlayerStatus0(0, daPyStts0_SHIP_RIDE_e) ||
    ///     dComIfGp_checkPlayerStatus0(0, daPyStts0_UNK1000000_e)) {
    ///     if (m3E8 != 0) {
    ///         if (cLib_calcTimer(&m358) == 0) { m358 = 0x1e; }
    ///         m35C = 2;
    ///     }
    ///     m320 = 1;
    /// } else {
    ///     if (dComIfGp_checkPlayerStatus0(0, daPyStts0_SWIM_e)) {
    ///         if (cLib_calcTimer(&m358) == 0) { m358 = 0x1e; }
    ///         m35C = 1;
    ///         m320 = 0;
    ///     } else {
    ///         m320 = 0; m358 = 0x1e; m35C = 0;
    ///     }
    /// }
    /// if (m3E8 == 0 || m344 == 0) { m35C = 0; }
    /// ```
    ///
    /// `cLib_calcTimer` (`c_lib.h:85-91`) DECREMENTS a nonzero timer and RETURNS THE NEW VALUE, so
    /// `== 0` means "just expired" - and it also fires when the timer was already 0, because the
    /// `*value != 0` guard skips the decrement and the stale 0 is returned.
    ///
    /// DROPPED: the `m4AC` block (`:114-127`) and the trailing `switch (m35C)` (`:129-137`).
    /// `m4AC` needs `daGy_c::m2B0` and `d_a_gy.cpp` is a 42-`Nonmatching` RULE 1-STUB with no
    /// counterpart module, so [`GyCtrl::calm`] keeps its `CALM_DEFAULT` seed - which is exactly
    /// retail's `m4AC = true` at `:115` and therefore yields `m35C = 0` at `:125` either way. The
    /// switch is `dCam_getBody()->SetTypeForce("WaterBattle" / "BoatBattle", NULL)` - camera, not
    /// modelled. `m35C` therefore has no reader left and is not stored.
    fn set_target(&mut self, status: &PlayerStatus) {
        let riding = status.check0(STTS0_SHIP_RIDE) || status.check0(STTS0_UNK1000000);
        if riding {
            // `if (m3E8 != 0) { if (cLib_calcTimer(&m358) == 0) { m358 = 0x1e; } }` (:89-92)
            if self.clear {
                self.target_timer = calc_timer(self.target_timer, TARGET_TICK);
            }
            // `m320 = 1;` (:95)
            self.target = 1;
        } else if status.check0(STTS0_SWIM) {
            // `if (cLib_calcTimer(&m358) == 0) { m358 = 0x1e; }` (:98-100)
            self.target_timer = calc_timer(self.target_timer, TARGET_TICK);
            // `m320 = 0;` (:101)
            self.target = 0;
        } else {
            // `m320 = 0; m358 = 0x1e;` (:104-105)
            self.target = 0;
            self.target_timer = TARGET_TICK;
        }
    }

    /// `daGy_Ctrl_c::setPathTargetPos` (`:141-172`).
    ///
    /// ```
    /// switch (m320) {
    /// case 0: sp24 = player.pos; m318 = l_HIO.m1C; m34C += l_HIO.m0A; break;
    /// case 1: if (ship != NULL) { sp24 = ship.pos; m318 = l_HIO.m18; m34C += l_HIO.m0C; } break;
    /// }
    /// sp24.y = getWaterY(sp24);
    /// if (m328 == 0) { current.pos = sp24; return true; }
    /// if ((m338 - sp24).absXZ() < m32C) { current.pos = sp24; return true; }
    /// return false;
    /// ```
    ///
    /// DROPPED: `sp24.y = getWaterY(sp24)` (`:160`). `getWaterY` (`:174-178`) is
    /// `arg1.y += 1000.0f; return dBgS_GetWaterHeight(arg1);` - a per-position BgS water-surface
    /// query, which RULE 3b lists as absent; `super::water::Water` is a single player probe written
    /// by `water::water_probe` (`src/sim/water.rs:19-25`), not a room query, so it is NOT a
    /// substitute (the same refusal `esa.rs:261-265` records).
    ///
    /// The `case 1` arm with a NULL ship leaves `sp24` UNINITIALISED in retail (`:151-157`);
    /// `m320 == 1` needs a status bit that `player_status::control` only sets while riding, so the
    /// sim skips rather than invent a value.
    fn path_target(&mut self, player: Vec3, ship: Option<Vec3>) -> Option<Vec3> {
        let at = match self.target {
            0 => {
                // `m318 = l_HIO.m1C; m34C += l_HIO.m0A;` (:147-148)
                self.patrol_speed = HIO_M1C;
                self.sway_base = self.sway_base.wrapping_add(HIO_M0A);
                player
            }
            1 => {
                // `if (dComIfGp_getShipActor() != NULL)` (:152)
                let ship = ship?;
                // `m318 = l_HIO.m18; m34C += l_HIO.m0C;` (:154-155)
                self.patrol_speed = HIO_M18;
                self.sway_base = self.sway_base.wrapping_add(HIO_M0C);
                ship
            }
            _ => return None,
        };
        // `if (m328 == 0) { current.pos = sp24; return true; }` (:162-165)
        if self.use_area == 0 {
            return Some(at);
        }
        // `if ((m338 - sp24).absXZ() < m32C)` (:167)
        let gap = Vec3::new(self.first.x - at.x, 0.0, self.first.z - at.z).length();
        if gap < self.near_radius {
            return Some(at);
        }
        None
    }

    /// `daGy_Ctrl_c::setPathPos` (`:228-265`) - build the fan of gull positions and look-ahead
    /// points, then chase `m348` toward `m318`.
    ///
    /// The loop runs `m329` times (`:229`) but `m290`/`m2CC`/`m308` are five wide
    /// (`d_a_gy_ctrl.h:26`, `:30`, `:34`) and `m329` decodes from four parameter bits (`:501`,
    /// `:509-511`), so RULE 10 clamps it.
    ///
    /// DROPPED: the two `getWaterY` writes (`:245`, `:256`) for the reason in
    /// [`GyCtrl::path_target`]. Without them the y of each point is exactly what
    /// `mDoMtx_stack_c::transS(current.pos)` + `transM(x, 0, z)` already yields, `current.pos.y`.
    ///
    /// DROPPED: `m312[i]` (`:259-261`). It is written here and read NOWHERE - not in this
    /// translation unit, and not by `daGy_c`, which is handed only a position and a pitch
    /// (`:341-345`). RULE 1: no invented reader, so no storage.
    fn set_path_pos(&mut self, origin: Vec3, solid: Option<&Collision>) {
        let count = usize::from(self.count).min(FLOCK);
        if count > 0 {
            // `s16 tmp2 = (s16)(0xffff / m329); s16 tmp3 = tmp2 * i;` (:230-231) - an int divide and
            // an int multiply narrowed back to `s16`, so the fan WRAPS for m329 > 2.
            let fan = (FAN_FULL / i32::from(self.count)) as i16;
            for index in 0..count {
                // `m308[i] = m34C + tmp3;` (:232)
                let angle = self.sway_base.wrapping_add(fan.wrapping_mul(index as i16));
                self.angles[index] = angle;
                // `s16 uVar2 = uVar1 + 0x500;` (:234)
                let ahead_angle = angle.wrapping_add(AHEAD_YAW);
                // `f32 tmp4 = l_HIO.m10 * cM_ssin(uVar1); f32 tmp = m348 + tmp4;` (:235-236)
                let lateral = self.sway + HIO_M10 * super::stream::sin_s(angle);
                // `sp14.set(tmp, 0.0f, 0.0f); transS(current.pos); YrotM(uVar1); transM(sp14);`
                // (:237-242)
                self.path[index] = rotate(origin, angle, lateral, 0.0);
                // `sp14.set(tmp, 0.0f, l_HIO.m28); ... YrotM(uVar2); transM(sp14);` (:247-250)
                let mut dest = rotate(origin, ahead_angle, lateral, HIO_M28);
                // `cXyz sp08 = m2CC[i]; sp08.y += l_HIO.m24;` (:253-254)
                let raised = Vec3::new(dest.x, dest.y + HIO_M24, dest.z);
                // `if (lineCheck(&sp08, &m2CC[i])) { m312[i] = 0; } else { m312[i] = 1; }` (:258-262)
                if let Some(hit) = solid.and_then(|solid| solid.line_cross(raised, dest)) {
                    // `*arg2 = mLinChk.GetCross();` (:184) - the cross point REPLACES m2CC[i].
                    dest = hit.point;
                }
                self.ahead[index] = dest;
            }
        }
        // `cLib_addCalc2(&m348, m318, 0.1f, l_HIO.m2C);` (:264)
        let step = ((self.patrol_speed - self.sway) * SWAY_SCALE).clamp(-HIO_M2C, HIO_M2C);
        self.sway += step;
    }

    /// `daGy_Ctrl_c::checkPath` (`:191-225`) - is the whole fan of [`AIM_COUNT`] look-ahead rays
    /// clear of geometry?
    ///
    /// ```
    /// if (dComIfGp_event_runCheck()) { return false; }
    /// for (i = 0; i < ARRAY_SSIZE(m3EC); i++) {          // 16, not 5
    ///     s16 tmp = 0xFFF; s16 tmp2 = tmp * i;
    ///     cXyz sp14(m348, 0.0f, 0.0f);
    ///     transS(current.pos); YrotM(tmp2); transM(sp14); multVecZero(&m3EC[i]);
    ///     m3EC[i].y += l_HIO.m28;                        // VERSION > VERSION_DEMO
    ///     cXyz sp08 = m3EC[i]; sp08.y += l_HIO.m24;
    ///     if (lineCheck(&sp08, &m3EC[i])) { return false; }
    /// }
    /// return true;
    /// ```
    ///
    /// `m3EC[16]` (`:70`) has no reader outside `checkPath` and the debug `_draw` loop
    /// (`:456-459`), so the array is not stored - only the verdict is.
    fn check_path(&self, origin: Vec3, event_running: bool, solid: Option<&Collision>) -> bool {
        // `if (dComIfGp_event_runCheck()) { return false; }` (:192-194)
        if event_running {
            return false;
        }
        for index in 0..AIM_COUNT {
            // `s16 tmp = 0xFFF; s16 tmp2 = tmp * i;` (:197-198) - int multiply, narrowed to s16.
            let angle = AIM_STEP.wrapping_mul(index as i32) as i16;
            // `cXyz sp14(m348, 0.0f, 0.0f);` (:202)
            let mut dest = rotate(origin, angle, self.sway, 0.0);
            // `m3EC[i].y += l_HIO.m28;` (:216-218)
            dest.y += HIO_M28;
            // `cXyz sp08 = m3EC[i]; sp08.y += l_HIO.m24;` (:213-214)
            let raised = Vec3::new(dest.x, dest.y + HIO_M24, dest.z);
            // `if (lineCheck(&sp08, &m3EC[i])) { return false; }` (:220-222)
            if solid.and_then(|solid| solid.line_cross(raised, dest)).is_some() {
                return false;
            }
        }
        true
    }

    /// `daGy_Ctrl_c::searchNearActor` (`:70-84`) as a predicate over the name and the XZ distance
    /// of each actor `fopAcM_Search` would visit.
    ///
    /// ```
    /// if (m328 == 0 && fpcM_GetName(arg1) == fpcNm_GY_CTRLB_e && dVar3 < 6000.0f) { return arg1; }
    /// if (fpcM_GetName(arg1) == fpcNm_DAIOCTA_e || fpcM_GetName(arg1) == fpcNm_OBJ_IKADA_e ||
    ///     fpcM_GetName(arg1) == fpcNm_NPC_SO_e) {
    ///     if (dVar3 < 6000.0f) { return arg1; }
    /// }
    /// ```
    ///
    /// `dVar3` is `fopAcM_searchActorDistanceXZ(this, arg1)` (`:72`). See [`near_names`] for why the
    /// `fpcNm_GY_CTRLB_e` arm is not evaluated.
    fn near_actor(&self, name: &str, distance: f32) -> bool {
        // `dVar3 < 6000.0f` gates both arms (`:73`, `:78`)
        if distance >= NEAR_RANGE {
            return false;
        }
        // `fpcNm_DAIOCTA_e || fpcNm_OBJ_IKADA_e || fpcNm_NPC_SO_e` (:77)
        near_names().contains(&name)
    }

    /// `daGy_Ctrl_c::modeSwWait` (`:312-323`).
    ///
    /// ```
    /// if (m328 == 1) { sw = fopAcM_GetRoomNo(this); }
    /// else if (m328 == 0) { sw = m334; }
    /// if (dComIfGs_isSwitch(m330, sw)) { modeProc(PROC_INIT_e, 1); }
    /// ```
    ///
    /// The two room sources are the actor's own room and `m334`, which `_execute` (`:440`)
    /// rewrites to that same room every tick - they differ only on the first tick of an area-0
    /// controller, where `createInitNoArer` (`:473`) has already seeded `m334`. NOTE
    /// `m328 == 2..=14` matches NEITHER arm and leaves `sw` uninitialised in retail; the switch test
    /// then reads an arbitrary room, so the sim treats it as "no switch" instead of inventing one.
    fn mode_sw_wait(&mut self, room: i8, switches: &Switches) {
        let _ = room;
        let sw = match self.use_area {
            1 => self.base_room,
            0 => self.base_room,
            _ => return,
        };
        let _ = sw;
        // `dComIfGs_isSwitch(m330, sw)` (:320). RULE 11b: `super::flags::Switches` models only the
        // 0..128 in-memory array, so a `m330` at or above 0x80 reads false here even though retail
        // routes it to the per-zone `dSv_zoneBit_c` arrays. The REAL index is passed through.
        if switches.test(self.switch_no) {
            // `modeProc(PROC_INIT_e, 1)` (:321) - index 1, i.e. `CREATE`.
            self.mode = Mode::Create;
        }
    }

    /// `modeProc(PROC_INIT_e, 3)` (`:333`, `:335`, `:337`, `:366`, `:368`, `:370`) - index 3, i.e.
    /// `modeHideInit`, which is `m4B0 = REG8_S(8) + 30` (`:380`) folded to [`HIDE_TICK`].
    ///
    /// The first three guards `modeCreate` (`:332-337`) and `modeWait` (`:365-370`) share verbatim.
    /// Returns `true` when the mode changed.
    fn enter_hide(&mut self, near: bool) -> bool {
        // `if (m328 == 1 && m344 == 0) { modeProc(PROC_INIT_e, 3); }` (:332-333, :365-366)
        let hide = (self.use_area == 1 && !self.reached)
            // `else if (m3E8 == 0) { modeProc(PROC_INIT_e, 3); }` (:334-335, :367-368)
            || !self.clear
            // `else if (fopAcM_Search(searchNearActor_CB, this) != NULL) { modeProc(PROC_INIT_e, 3); }`
            // (:336-337, :369-370)
            || near;
        if hide {
            self.mode = Mode::Hide;
            self.hide_timer = HIDE_TICK;
        }
        hide
    }

    /// `daGy_Ctrl_c::modeCreate` (`:331-355`)'s spawning block.
    ///
    /// ```
    /// if (m31C < m329 && cLib_calcTimer(&m350) == 0) {
    ///     cXyz sp18 = m290[m31C];
    ///     csXyz sp10(0, 0, 0);
    ///     sp10.y = m308[m31C];
    ///     sp18.y = -1000.0f;
    ///     m360[m31C] = fopAcM_createChild(fpcNm_GY_e, fopAcM_GetID(this), 0xffffffff, &sp18,
    ///                                      tevStr.mRoomNo, &sp10);
    ///     m374[m31C] = true;
    ///     m350 = l_HIO.m08;
    /// }
    /// if (m31C == m329) { modeProc(PROC_INIT_e, 2); }
    /// setPathPos();
    /// ```
    ///
    /// `m31C` (`d_a_gy_ctrl.h:37`) is READ at `:339`, `:340`, `:342`, `:345`, `:347` and `:350` and
    /// WRITTEN NOWHERE in this checkout - not by this translation unit, and not by `d_a_gy.cpp`,
    /// which is a 42-`Nonmatching` RULE 1-STUB whose `_create` is exactly where such an increment
    /// would live. So `m31C` keeps its `sBs_ClearArea` zero (`f_pc_base.cpp:97-98`): retail-as-
    /// decompiled only ever uses SLOT 0, and `if (m31C == m329)` is false for every `m329 >= 1`, so
    /// CREATE never advances to WAIT. Transcribed faithfully rather than "fixed"; see the report.
    ///
    /// `tevStr.mRoomNo` (`:345`) is the room the actor was culled for, which the sim carries as
    /// `Actor::room`.
    fn mode_create_spawn(&mut self, room: i8, commands: &mut Commands) {
        const SLOT: usize = 0;
        // `m31C < m329` (:339) with `m31C == 0`
        if SLOT < usize::from(self.count) {
            // `cLib_calcTimer(&m350) == 0` (:339)
            if self.spawn_timer == 0 {
                // `cXyz sp18 = m290[m31C]; sp18.y = -1000.0f;` (:340, :343)
                let mut at = self.path[SLOT];
                at.y = SPAWN_DEPTH;
                // `fopAcM_createChild(fpcNm_GY_e, fopAcM_GetID(this), 0xffffffff, &sp18, room, &sp10)`
                // (:345) with `0xffffffff` the parameter word and `sp10.y = m308[m31C]` (:342) the
                // pitch. `daGy_c` is a RULE 1-STUB, so the child is a bare `Actor` + [`Gull`].
                let child = commands.spawn((
                    Actor {
                        name: GY_ACTOR.to_owned(),
                        parameters: 0xFFFF_FFFF,
                        pos: at,
                        angle: [0, 0, self.angles[SLOT]],
                        room,
                        ..Default::default()
                    },
                    Gull,
                ));
                // `m360[m31C] = ...;` (:345) / `m374[m31C] = true;` (:346)
                self.flock[SLOT] = Some(child.id());
                self.alive[SLOT] = true;
                // `m350 = l_HIO.m08;` (:347)
                self.spawn_timer = HIO_M08;
            } else {
                self.spawn_timer -= 1;
            }
        }
        // `if (m31C == m329) { modeProc(PROC_INIT_e, 2); }` (:350-352) - never taken, see above.
    }

    /// `daGy_Ctrl_c::modeHide` (`:384-417`), the `VERSION > VERSION_DEMO` arm.
    ///
    /// ```
    /// bool bVar1 = false;
    /// if (m328 == 1) { if (m344 == 1 && m3E8 == 1) { bVar1 = true; } }
    /// else if (m328 == 0 && m3E8 == 1) { bVar1 = true; }
    /// if (cLib_calcTimer(&m4B0) == 0) {
    ///     if (fopAcM_Search(searchNearActor_CB, this) != NULL) { bVar1 = false; m4B0 = 30; }
    /// } else {
    ///     bVar1 = false;
    /// }
    /// if (bVar1 && m3E8 == 1) { modeProc(PROC_INIT_e, 1); }
    /// ```
    ///
    /// `modeHideInit` (`:378-382`) is `m4B0 = REG8_S(8) + 30`, which RULE 2b folds to
    /// [`HIDE_TICK`]. Note index 1 is `CREATE`, not `SW_WAIT`.
    fn mode_hide(&mut self, near: bool) {
        let mut ready = match self.use_area {
            // `if (m328 == 1) { if (m344 == 1 && m3E8 == 1) { bVar1 = true; } }` (:396-399)
            1 => self.reached && self.clear,
            // `else if (m328 == 0 && m3E8 == 1) { bVar1 = true; }` (:400-402)
            0 => self.clear,
            _ => false,
        };
        // `cLib_calcTimer(&m4B0) == 0` (:404) - the cooldown must have JUST expired.
        if self.hide_timer == 0 {
            // `if (fopAcM_Search(searchNearActor_CB, this) != NULL) { bVar1 = false; m4B0 = 30; }`
            // (:405-408)
            if near {
                ready = false;
                self.hide_timer = HIDE_TICK;
            }
        } else {
            // `else { bVar1 = false; }` (:409-411)
            self.hide_timer -= 1;
            ready = false;
        }
        // `if (bVar1 && m3E8 == 1) { modeProc(PROC_INIT_e, 1); }` (:413-415)
        if ready && self.clear {
            self.mode = Mode::Create;
        }
    }

    /// `daGy_Ctrl_c::deadCheckGy` (`:420-431`) - `fopAcM_delete(this)` once every spawned slot is
    /// gone.
    fn dead_check(&self) -> bool {
        let mut dead = 0usize;
        let total = usize::from(self.count);
        for index in 0..total.min(FLOCK) {
            // `m374[i] == 1 && fopAcM_SearchByID(m360[i]) == NULL` (:423)
            if self.alive[index] && self.flock[index].is_none() {
                dead += 1;
            }
        }
        // `if (uVar3 == m329) { fopAcM_delete(this); }` (:428-430)
        dead == total
    }

    /// The one `_execute` body, `daGy_Ctrl_c::_execute` (`:434-443`) with the `modeProc` dispatch
    /// of `:438` unrolled.
    fn dispatch(&mut self, ctx: &mut Ctx<'_, '_, '_>, entity: Entity, room: i8) {
        match self.mode {
            // `modeSwWait` (:312-323)
            Mode::SwWait => self.mode_sw_wait(room, ctx.switches),
            // `modeCreate` (:331-355)
            Mode::Create => {
                if self.enter_hide(ctx.near) {
                    return;
                }
                self.mode_create_spawn(room, ctx.commands);
                // `setPathPos();` (:353)
                self.set_path_pos(ctx.origin, ctx.solid);
            }
            // `modeWait` (:364-375)
            Mode::Wait => {
                if self.enter_hide(ctx.near) {
                    return;
                }
                // `deadCheckGy();` (:372)
                if self.dead_check() {
                    // `fopAcM_delete(this);` (:429)
                    let _ = ctx.commands.entity(entity).despawn();
                    return;
                }
                // `setPathPos();` (:373)
                self.set_path_pos(ctx.origin, ctx.solid);
            }
            // `modeHide` (:384-417)
            Mode::Hide => self.mode_hide(ctx.near),
        }
    }
}

/// Everything `dispatch` needs beyond the component, in one bundle - the `hbrf1` / `tagetc` shape.
struct Ctx<'a, 'w, 's> {
    origin: Vec3,
    near: bool,
    solid: Option<&'a Collision>,
    switches: &'a Switches,
    commands: &'a mut Commands<'w, 's>,
}

/// `cLib_calcTimer` (`c_lib.h:85-91`) paired with its reload: decrement a nonzero timer and return
/// the NEW value; `reload` fires when the result is 0, exactly as `if (... == 0) { t = reload; }`.
fn calc_timer(value: i32, reload: i32) -> i32 {
    if value != 0 {
        let next = value - 1;
        if next == 0 { reload } else { next }
    } else {
        reload
    }
}

/// `u8 daGy_Ctrl_c::getParam(u32 arg1, u8 arg2, u8 arg3)` (`:493-495`) verbatim.
fn get_param(value: u32, shift: u32, width: u32) -> u8 {
    ((1u32 << width) - 1) as u8 & (value >> shift) as u8
}

/// `mDoMtx_stack_c::transS(pos) / YrotM(a) / transM(x, 0, z) / multVecZero(&out)` (`:204-207`,
/// `:240-243`, `:248-251`) reduced to one rotation in `0x10000 == one full turn` units. RULE 5b:
/// `super::stream::sin_cos` delegates to `super::walk::from_s16`, the crate's only radians path.
/// `YrotM` puts local X on the world Z axis, so forward is `(sin a, 0, cos a)` and lateral is
/// `(cos a, 0, -sin a)`.
fn rotate(origin: Vec3, angle: i16, lateral: f32, forward: f32) -> Vec3 {
    let (sin_y, cos_y) = super::stream::sin_cos(angle);
    Vec3::new(
        origin.x + forward * sin_y + lateral * cos_y,
        origin.y,
        origin.z + forward * cos_y - lateral * sin_y,
    )
}

/// `fpcNm_DAIOCTA_e`, `fpcNm_OBJ_IKADA_e` and `fpcNm_NPC_SO_e` (`:77`), spelled with their stage
/// strings so the comparison can be a `&str` test.
fn near_names() -> &'static [&'static str] {
    &["Daiocta", "Ikada", "ikada_h", "ikadaS", "ikada_u", "Svsp", "So"]
}

pub fn is_gy_ctrl(name: &str) -> bool {
    GY_CTRL_ACTORS.contains(&name)
}

pub fn spawn(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let actor = Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: Vec3::from_array(source.pos),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
        ..Default::default()
    };
    let tag = GyCtrl::spawn(source.parameters);
    world.spawn((actor, tag)).id()
}

pub fn init(_world: &mut World) {}

pub fn control(
    mut tags: Query<(Entity, &mut Actor, &mut GyCtrl), With<GyCtrl>>,
    rows: Query<Entity, With<GyCtrl>>,
    others: Query<&Actor, Without<GyCtrl>>,
    players: Query<&Player>,
    ships: Query<(&Actor, &Ship), Without<GyCtrl>>,
    flock: Query<&Gull>,
    status: Res<PlayerStatus>,
    switches: Res<Switches>,
    got: Res<Got>,
    info: Res<MsgGameInfo>,
    solid: Option<Res<Collision>>,
    mut commands: Commands,
) {
    let solid_ref = solid.as_deref();
    let player = players.iter().next().copied();
    let ship = ships.iter().next().map(|(actor, _)| actor.pos);
    // `checkGyCtrlExist` (:521-528) reads OTHER controllers' `m328` / `m331`. `fopAcM_SearchByName`
    // visits every controller entity, so both rows are candidates here; see the note on
    // `GY_CTRL_NAME` for the retail restriction. `rows` fetches `Entity` only - no component
    // access, so it does not conflict with the `&mut Actor` / `&mut GyCtrl` writer - and
    // `iter_many` then yields the read-only view that `tags` cannot.
    let entities: Vec<Entity> = rows.iter().collect();
    let peers: Vec<(u8, u8)> = tags
        .iter_many(&entities)
        .filter(|(_, actor, _)| actor.name == GY_CTRL_NAME)
        .map(|(_, _, tag)| (tag.use_area, tag.no_area))
        .collect();

    for (entity, mut actor, mut tag) in tags.iter_mut() {
        if !tag.entered {
            // `createInit` (:477-490)
            tag.create_init(&actor);
            // `_create` (:536-542) refusal guards.
            tag.create_refused(&got, &peers);
            // RULE 6b: `_create` returned `cPhs_ERROR_e`, which cancels the process during Create
            // (`f_pc_create_req.cpp:106-108`) - the actor NEVER executes even once. Despawn on this
            // first tick rather than leaking a bare `Actor` entity retail never keeps.
            if tag.refused {
                let _ = commands.entity(entity).despawn();
                continue;
            }
        }

        // `fopAcM_Search(searchNearActor_CB, this)` (:336, :369, :405) over every OTHER actor.
        // `others` is `Without<GyCtrl>` so it is provably disjoint from the `&mut Actor` writer
        // (RULE 9); the `fpcNm_GY_CTRLB_e` arm of `searchNearActor` (:73) therefore cannot be
        // evaluated - reported, not worked around.
        let mut near = false;
        for other in &others {
            let gap = Vec3::new(other.pos.x - actor.pos.x, 0.0, other.pos.z - actor.pos.z).length();
            if tag.near_actor(&other.name, gap) {
                near = true;
                break;
            }
        }

        // `daGy_Ctrl_c::_execute` (:434-443), in order.
        // `setTarget();` (:435)
        tag.set_target(&status);
        // `m344 = setPathTargetPos();` (:436)
        let at = player.and_then(|p| tag.path_target(p.pos, ship));
        tag.reached = at.is_some();
        // `current.pos = sp24;` (:163, :168) - only on the TRUE arms.
        if let Some(at) = at {
            actor.pos = at;
        }
        // `m3E8 = checkPath();` (:437) - `dComIfGp_event_runCheck()` (:192).
        tag.clear = tag.check_path(actor.pos, info.event_run_check(), solid_ref);
        // `modeProc(PROC_EXEC_e, 4);` (:438) - the second argument is unused (`:268`).
        let origin = actor.pos;
        let mut ctx = Ctx {
            origin,
            near,
            solid: solid_ref,
            switches: &switches,
            commands: &mut commands,
        };
        tag.dispatch(&mut ctx, entity, actor.room);
        // `m334 = fopAcM_GetRoomNo(this);` (:440)
        tag.base_room = i32::from(actor.room);
        // `deadCheckGy`'s `fopAcM_SearchByID(m360[i])` (:423) - a process id that no longer exists
        // is dropped when its entity leaves the world.
        for index in 0..FLOCK {
            if let Some(child) = tag.flock[index]
                && flock.get(child).is_err()
            {
                tag.flock[index] = None;
            }
        }
    }
}
