//! `daComing2::Act_c` (`d_a_coming2.h:31-89`) - the Great Sea barrel-bomb course driver behind the
//! stage string `Com_B`. `Nonmatching` count: 0, so every token below is transcribed. `_draw` is
//! `return true` with no draw call (`:635-637`), so retail renders nothing for this actor.
//!
//! Nothing else in `src/sim/` claims `Com_B`: `OBJNAME("Com_B", fpcNm_Coming2_e, 255, 0)`
//! (`d_stage.cpp:1167`) is the only row. The siblings are DIFFERENT profiles: `coming3.rs` owns
//! `"Com_C"` (`fpcNm_Coming3_e`, `d_stage.cpp:1168`) and `coming.rs` owns `"Com_A"`
//! (`fpcNm_Obj_Coming_e`, `d_stage.cpp:940`).
//!
//! CULL: the profile says `fopAc_CULLBOX_CUSTOM_e` (`:690`) but the file calls neither
//! `fopAcM_setCullSizeBox` nor `fopAcM_SetMin` / `SetMax` - grep confirms neither appears anywhere in
//! `d_a_coming2.cpp`, and the `mDoMtx_stack_c` writes are not those functions. Under the crate's cull
//! rule a CUSTOM profile with no box call means there is NO box, so this module deliberately exposes
//! no `cull()` and no `actor_views` branch (the `coming3.rs` cull refusal).
//!
//! OMITTED - the barrel and buoy PLACEMENTS. `daComing2::Act_c::get_water_height` (`:154-190`) is
//! the gate on every one of them, and its TAIL is a per-poly-group ground query:
//!
//! ```c
//! sp08 = *pos; sp08.y += 20000.0f;
//! gnd_work.SetPos(&sp08);
//! if (dComIfG_Bgsp()->GroundCross(&gnd_work) > waveHeight - 100.0f) { return FALSE; }
//! return TRUE;
//! ```
//!
//! (`:180-188`) - a `dBgS_ObjGndChk` clearance test, the exact call `super::ground::surface` cannot
//! answer (merged-DZB plane only) and that `coming3.rs` already declines for the sibling actor.
//! So `make_coming_param` (`:226`), `coming_game_main`'s spawn (`:517`) and `coming_setF_main`'s two
//! buoy spawns (`:445`) all lose their gate; assuming it passes would be the invented equivalent
//! RULE 1 forbids. The three helpers that hang off those spawns - `no_ship_obstacle` (`:101-108`),
//! `position_is_none_obj` (`:134-143` with its callback `:116-131`) and `check_in_large_sea`
//! (`:146-151`) - are portable but have no reader once their callers go, so RULE 12(b) removes them
//! rather than leaving dead `pub` items. Their literals are cited in the report.
//!
//! Everything else IS transcribed: the five-mode machine, both timers, the `request_*_exit`
//! bookkeeping, `barrel_execute`, `checkLineCrossXZ`, `chase_ship`, `renew_scope_info`, `get_speed`
//! and the `EventReg` run counter.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::buoybarrel::BuoyBarrel;
use super::flags::EventReg;
use super::player::Player;
use super::ship::Ship;

/// `OBJNAME("Com_B", fpcNm_Coming2_e, 255, 0)` (`d_stage.cpp:1167`) - the only row for
/// `g_profile_Coming2` (`d_a_coming2.cpp:676-691`).
pub const COMING2_ACTORS: [&str; 1] = ["Com_B"];

/// ```
/// unk_2A6 = 600;
/// ```
/// (`:216` `init_coming_info`, `:366` `coming_wait_init`) - the room-change grace period, in ticks.
const ROOM_GRACE: i16 = 600; // <- d_a_coming2.cpp:216, :366
/// ```
/// unk_29C = (s32)cM_rndF(2.0f) & 1;
/// ```
/// (`:217`) - the lane offset, chosen at random. There is NO global `cM_rndF` in this sim (the brief
/// lists it), so `Coming2::spawn` uses the crate's established per-actor xorshift
/// (`buoybarrel.rs:349-354`, `agbsw0.rs:561-573`).
const SIDE_SPAN: f32 = 2.0; // <- d_a_coming2.cpp:217
/// ```
/// f32 fVar1 = unk_2A0;
/// if (fVar1 < 35.0f) { fVar1 = 35.0f; } else { fVar1 = unk_2A0; }
/// sp18.x = 0.0f; sp18.y = 100.0f; sp18.z = fVar1 * 135.0f;
/// ```
/// (`:71-79`) - `renew_scope_info`'s arm: a FLOOR (not a clamp), and the `else` re-reads the raw value.
const SPEED_FLOOR: f32 = 35.0; // <- d_a_coming2.cpp:73
const ARM_SCALE: f32 = 135.0; // <- d_a_coming2.cpp:79
const ARM_LIFT: f32 = 100.0; // <- d_a_coming2.cpp:78
/// ```
/// s16 arr[2] = {0x546, -0x546};
/// s16 sVar3 = arr[(unk_2A4 + unk_29C) & 1];
/// sVar3 = sVar3 > 0 ? sVar3 + ((unk_2A4 >> 2) & 0xF) * 0x168
///                   : sVar3 - ((unk_2A4 >> 2) & 0xF) * 0x168;
/// s16 rot = mStartStat.unk_00 + sVar3;
/// ```
/// (`:81-84`) - the barrel's yaw off the ship's heading, fanning wider with each barrel index.
const SIDE_YAW: [i16; 2] = [0x546, -0x546]; // <- d_a_coming2.cpp:81
const FAN_STEP: i32 = 0x168; // <- d_a_coming2.cpp:83
const FAN_FOLD: i32 = 0xF; // <- d_a_coming2.cpp:83
/// ```
/// if (unk_46C < 0xB4) { ... unk_46C++; } else { ... coming_process_init(0); }
/// ```
/// (`:480`, `:485`, `:486`) - how long the ship-crossing test runs before giving up.
const CROSS_FRAMES: i16 = 0xB4; // <- d_a_coming2.cpp:480
/// `for (u32 i = 0; i < 15; i++)` (`:260`, `:340`) - the barrel count. `mBarrelInfo[20]`
/// (`d_a_coming2.h:64`) is TWENTY wide but only fifteen are ever used, and `unk_2A4 >= 15u`
/// (`:327`, `:512`) treats fifteen as the end.
const BARRELS: usize = 15; // <- d_a_coming2.cpp:260, :340
/// `mStartBuoy[2]` (`d_a_coming2.h:38`, `:73`) and `for (s32 i = 0; i < 2; i++)` (`:289`) - the two
/// start buoys.
const BOUYS: usize = 2; // <- d_a_coming2.cpp:289
/// `get_coming_point()` is `dComIfGs_getEventReg(dSv_event_flag_c::UNK_7EFF)`
/// (`d_a_coming2.h:37-39`) and `minus_coming_point1` (`:42-48`) writes it.
/// `dSv_event_flag_c::UNK_7EFF = 0x7EFF` (`d_save_event_flag.inc:503`).
const COMING_POINT: u16 = 0x7EFF; // <- d_a_coming2.h:38 (d_save_event_flag.inc:503)
/// `if (i_this->mStartStat.unk_00 != ...)` does not exist; the yaw itself is `shape_angle.y` (`:599`).
/// `daComing2::Act_c::_create` (`:589-609`) returns `cPhs_COMPLEATE_e` unconditionally (`:608`) and
/// the profile `Status` is `fopAcStts_UNK4000_e | fopAcStts_UNK40000_e` (`:688`) with `Group`
/// `fopAc_ACTOR_e` (`:689`), so there is NO refusal to model and RULE 6b needs no `refused` flag.
/// `dComIfGp_event_runCheck()` (`:622`) is the sim's `MsgGameInfo::event_run_check`.

/// The five `coming_process_init` table entries (`d_a_coming2.cpp:555-561`), indexed by `arg1`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// index 0 - `coming_clear_init` / `coming_clear_main` (`:346-362`).
    #[default]
    Clear,
    /// index 1 - `coming_wait_init` / `coming_wait_main` (`:365-394`).
    Wait,
    /// index 2 - `coming_setF_init` / `coming_setF_main` (`:397-470`).
    SetFlag,
    /// index 3 - `coming_checkS_init` / `coming_checkS_main` (`:473-493`).
    CheckSail,
    /// index 4 - `coming_game_init` / `coming_game_main` (`:496-550`).
    Game,
}

impl Mode {
    /// ```
    /// if (arg1 >= 0 && arg1 < 5) { if ((this->*init_table[arg1])()) { unk_46E = arg1; return true; } }
    /// return false;
    /// ```
    /// (`:563-569`) - the guard is the table bound, transcribed here.
    fn of(index: i16) -> Option<Self> {
        match index {
            0 => Some(Mode::Clear),
            1 => Some(Mode::Wait),
            2 => Some(Mode::SetFlag),
            3 => Some(Mode::CheckSail),
            4 => Some(Mode::Game),
            _ => None,
        }
    }

    /// `coming_process_main` (`:573-586`) dispatching on `unk_46E` through `main_table[5]`.
    fn index(self) -> i16 {
        match self {
            Mode::Clear => 0,
            Mode::Wait => 1,
            Mode::SetFlag => 2,
            Mode::CheckSail => 3,
            Mode::Game => 4,
        }
    }
}

/// One `daComing2::_barrel_info_` (`d_a_coming2.h:16-22`): `unk_00` the process id, `unk_04` the exit
/// countdown, `unk_06` whether the item has been given, `unk_08` the placement position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarrelInfo {
    /// `unk_00` (`:18`) - `Some` while the barrel is alive. `init_barrelN_info` (`:194`) sets the
    /// error sentinel; `:529` writes the new id; `request_barrel_exitN` (`:240`) and
    /// `barrel_execute` (`:307`) read it.
    pub barrel: Option<Entity>,
    /// `unk_04` (`:19`) - the exit delay, set to `0x93 - unk_2A4` at placement (`:534`) and
    /// decremented in `barrel_execute` (`:319`).
    pub delay: i16,
    /// `unk_06` (`:20`) - `barrel->chk_item_give()` (`d_a_obj_barrel2.h:96`), sampled at `:313` and
    /// modelled by `BuoyBarrel::given`.
    pub given: bool,
    /// `unk_08` (`:21`) - the placement position, which `renew_scope_info` (`:90`) reads as the arm's
    /// origin for barrel `unk_2A4`.
    pub pos: Vec3,
}

/// One `daComing2::_start_buoy_` (`d_a_coming2.h:33-38`): `unk_00` the process id, `unk_04` the
/// position the crossing test reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StartBuoy {
    /// `unk_00` (`:35`) - `Some` once placed; `init_flag_info` (`:209-210`) clears it.
    pub buoy: Option<Entity>,
    /// `unk_04` (`:36`) - the world position, written at `:459` and read by `coming_checkS_main`
    /// (`:481`).
    pub pos: Vec3,
}

/// `daComing2::Act_c` (`d_a_coming2.h:41-89`).
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. The struct holds two counter arrays, a
/// `cXyz` and twelve scalars (`d_a_coming2.h:63-88`) - no collider member of any kind, and
/// `dComIfG_Ccsp()->Set(...)` never appears in the translation unit. The profile `Group` is
/// `fopAc_ACTOR_e` (`d_a_coming2.cpp:689`). Neither `combat::HitPoints` nor `enemy::Hurt` is
/// attached. The barrels it spawns are separate `daObjBarrel2::Act_c` (`Ktarur`) entities, which
/// carry their own damage state in `buoybarrel.rs`.
#[derive(Component, Debug, Clone)]
pub struct Coming2 {
    /// `unk_46E` (`d_a_coming2.h:79`) - the current mode index, stored by `coming_process_init`
    /// (`:566`) and read by `coming_process_main` (`:583`).
    pub mode: Mode,
    /// `unk_294` (`:65`) - the completed-run counter, incremented by `coming_game_init` (`:500`).
    pub runs: i32,
    /// `unk_298` (`:66`) - the player's room at the last `coming_setF_init` (`:400`).
    pub saved_room: i8,
    /// `unk_299` (`:67`) - the player's room at the last clear, refreshed by `coming_clear_init`
    /// (`:349`), `coming_wait_main` (`:381`, `:390`) and `coming_game_init` (`:499`).
    pub room: i8,
    /// `unk_29A` (`:68`) - the "a barrel just popped" flag: cleared by `coming_game_main` (`:531`),
    /// set by `barrel_execute` (`:332`), forced by `coming_game_init` (`:497`).
    pub popped: bool,
    /// `unk_29C` (`:69`) - the randomly chosen lane side, `(s32)cM_rndF(2.0f) & 1` (`:217`).
    pub side: i16,
    /// `unk_2A0` (`d_a_coming2.h:72`) - the XZ speed this tick, from `get_speed` (`:70`).
    pub speed: f32,
    /// `unk_2A4` (`:73`) - the barrel index, 0..14.
    pub placed: i16,
    /// `unk_2A6` (`:74`) - the room-change grace counter, `ROOM_GRACE` at init (`:216`, `:366`).
    pub grace: i16,
    /// `unk_2A8` (`:75`) - the placement retry counter, checked at `:544`. The only surviving
    /// reader is `coming_game_main`'s `else` arm, since the spawn arm is OMITTED (see the module
    /// docs); it is cleared by `coming_game_init` (`:498`) and here.
    pub budget: i16,
    /// `mBarrelInfo[15]` (`d_a_coming2.h:64`).
    pub barrels: [BarrelInfo; BARRELS],
    /// `mStartBuoy[2]` (`d_a_coming2.h:73`).
    pub buoys: [StartBuoy; BOUYS],
    /// `mStartStat.unk_00` (`d_a_coming2.h:69`) - the ship yaw captured by `coming_wait_init`
    /// (`:382`) and consumed by `renew_scope_info` (`:84`).
    pub start_yaw: i16,
    /// `unk_440` (`d_a_coming2.h:70`) - the position captured alongside `mStartStat` (`:383`, `:600-602`).
    pub start_pos: Vec3,
    /// `unk_46C` (`d_a_coming2.h:78`) - the crossing-test frame counter (`:474`, `:485`).
    pub cross: i16,
    /// `unk_4A8` (`d_a_coming2.h:87`) - the course scope's aim point, from `renew_scope_info` (`:97`).
    pub aim: Vec3,
    /// Per-actor RNG state standing in for the absent global `cM_rndF`; see the module note.
    pub seed: u32,
}

impl Coming2 {
    /// `daComing2::Act_c::_create` (`:589-609`).
    ///
    /// ```
    /// fopAcM_ct(this, daComing2::Act_c);
    /// unk_299 = -1;
    /// init_barrel_info();          // 15x init_barrelN_info
    /// init_coming_info();          // unk_2A4 = 0; unk_2A6 = 600; unk_29C = rndF(2) & 1;
    ///                              // unk_29A = 0; unk_2A8 = 0
    /// init_flag_info();            // mStartBuoy[0..1].unk_00 = -1
    /// coming_process_init(0);
    /// daPy_py_c* player = daPy_getPlayerActorClass();
    /// mStartStat.unk_00 = shape_angle.y;
    /// unk_440.x = current.pos.x; unk_440.y = current.pos.y; unk_440.z = current.pos.z;
    /// if (player != NULL) { unk_299 = player->current.roomNo; }
    /// return cPhs_COMPLEATE_e;
    /// ```
    ///
    /// The profile's `Parameters` field is `0` (`d_a_coming2.cpp:684`), so this actor reads NO
    /// parameter at all - there is nothing to decode, and RULE 7 is satisfied trivially.
    ///
    /// DROPPED: `fopAcM_ct` (`:590`) and `coming_process_init(0)` (`:595`) - the latter needs the
    /// player's room, which `control` supplies on the first tick; `coming_clear_init` (`:346-352`)
    /// otherwise always returns TRUE, so index 0 is the correct seed either way.
    pub fn spawn(yaw: i16, pos: Vec3) -> Self {
        // ```
        // unk_29C = (s32)cM_rndF(2.0f) & 1;
        // ```
        // (:217) - the crate's per-actor xorshift stands in for the absent global `cM_rndF`.
        let seed = 0x2540_F491_u32;
        let side = (((seed >> 8) as f32 / 16777216.0) * SIDE_SPAN) as i16 & 1;
        Self {
            // `unk_46E = arg1` from `coming_process_init(0)` (:566).
            mode: Mode::Clear,
            // `unk_294` takes `sBs_ClearArea`'s zero fill (`f_pc_base.cpp:97-98`).
            runs: 0,
            // `unk_298` is written only by `coming_setF_init` (:400).
            saved_room: 0,
            // `unk_299 = -1;` (:591)
            room: -1,
            // `unk_29A = 0;` (:218)
            popped: false,
            side,
            // `unk_2A0` is assigned by `renew_scope_info` (:70) on the first tick.
            speed: 0.0,
            // `unk_2A4 = 0;` (:215)
            placed: 0,
            // `unk_2A6 = 600;` (:216)
            grace: ROOM_GRACE,
            // `unk_2A8 = 0;` (:219)
            budget: 0,
            // ```
            // for (u32 i = 0; i < 15; i++) { init_barrelN_info(i); }
            // ```
            // (:202-204) - `init_barrelN_info` (:193-198): `unk_00 = fpcM_ERROR_PROCESS_ID_e;
            // unk_04 = 0; unk_06 = 0; unk_08 = current.pos;`
            barrels: [BarrelInfo {
                barrel: None,
                delay: 0,
                given: false,
                pos,
            }; BARRELS],
            // ```
            // mStartBuoy[0].unk_00 = -1;
            // mStartBuoy[1].unk_00 = -1;
            // ```
            // (:209-210) - `unk_04` is NOT reset, so it keeps its last placement.
            buoys: [StartBuoy {
                buoy: None,
                pos: Vec3::ZERO,
            }; BOUYS],
            // `mStartStat.unk_00 = shape_angle.y;` (:599)
            start_yaw: yaw,
            // `unk_440 = current.pos;` (:600-602)
            start_pos: pos,
            // `unk_46C` is written by `coming_checkS_init` (:474) on entry.
            cross: 0,
            // `unk_4A8` is assigned by `renew_scope_info` (:97) on the first tick.
            aim: Vec3::ZERO,
            seed,
        }
    }

    /// `daComing2::Act_c::chase_ship` (`:48-55`):
    /// ```
    /// fpc_ProcID proc = fopAcM_GetLinkId(this);
    /// fopAc_ac_c* actor = fopAcM_SearchByID(proc);
    /// if (actor != NULL) { current.pos = actor->current.pos; shape_angle = actor->shape_angle; }
    /// ```
    ///
    /// `fopAcM_GetLinkId` is the parent link `fopAcM_createChild` sets
    /// (`f_op_actor_mng.cpp:191`), so the ship is this actor's PARENT. The sim's `Actor` has no link
    /// field, so the ship is resolved as the one `Ship` in the stage - the same reduction the
    /// sibling `coming2`/`coming` modules use, documented rather than invented.
    fn chase_ship(actor: &mut Actor, ship: Option<(&Vec3, [i16; 3])>) {
        if let Some((pos, shape)) = ship {
            actor.pos = *pos;
            actor.shape_angle = shape;
        }
    }

    /// `daComing2::get_speed` (`:58-62`):
    /// ```
    /// f32 x = actor->current.pos.x - actor->old.pos.x;
    /// f32 z = actor->current.pos.z - actor->old.pos.z;
    /// return std::sqrtf(x * x + z * z);
    /// ```
    ///
    /// `old` is the start-of-tick transform `fopAc_Execute` snapshots (`f_op_actor.cpp:196`), which
    /// `Actor::snapshot` already maintains crate-wide.
    fn get_speed(actor: &Actor) -> f32 {
        let x = actor.pos.x - actor.old.pos.x;
        let z = actor.pos.z - actor.old.pos.z;
        (x * x + z * z).sqrt()
    }

    /// `daComing2::Act_c::renew_scope_info` (`:65-98`).
    ///
    /// ```
    /// unk_2A0 = get_speed(this);
    /// f32 fVar1 = unk_2A0;
    /// if (fVar1 < 35.0f) { fVar1 = 35.0f; } else { fVar1 = unk_2A0; }
    /// sp18.x = 0.0f; sp18.y = 100.0f; sp18.z = fVar1 * 135.0f;
    /// s16 arr[2] = {0x546, -0x546};
    /// s16 sVar3 = arr[(unk_2A4 + unk_29C) & 1];
    /// sVar3 = sVar3 > 0 ? sVar3 + ((unk_2A4 >> 2) & 0xF) * 0x168
    ///                   : sVar3 - ((unk_2A4 >> 2) & 0xF) * 0x168;
    /// s16 rot = mStartStat.unk_00 + sVar3;
    /// if (unk_2A4 == 0) { paVar5 = &current.pos; }
    /// else { paVar5 = &mBarrelInfo[unk_2A4 - 1].unk_08; }
    /// sp0C = *paVar5;
    /// mDoMtx_stack_c::transS(sp0C); YrotM(rot); MTXCopy(...);
    /// cMtx_multVec(sp24, &sp18, &unk_4A8);
    /// ```
    fn renew_scope_info(&mut self, origin: Vec3, speed: f32) {
        // `unk_2A0 = get_speed(this);` (:70)
        self.speed = speed;
        // `if (fVar1 < 35.0f) { fVar1 = 35.0f; } else { fVar1 = unk_2A0; }` (:71-76)
        let reach = if speed < SPEED_FLOOR { SPEED_FLOOR } else { speed };
        // `sp18.set(0.0f, 100.0f, fVar1 * 135.0f);` (:77-79)
        let arm = Vec3::new(0.0, ARM_LIFT, reach * ARM_SCALE);
        // ```
        // s16 sVar3 = arr[(unk_2A4 + unk_29C) & 1];
        // sVar3 = sVar3 > 0 ? sVar3 + ((unk_2A4 >> 2) & 0xF) * 0x168
        //                   : sVar3 - ((unk_2A4 >> 2) & 0xF) * 0x168;
        // ```
        // (:81-83) - RULE 10: the multiply is an `int` multiply narrowed back to `s16`.
        let lane = usize::from((self.placed.wrapping_add(self.side) & 1) as u8);
        let base = SIDE_YAW[lane];
        let fan = (i32::from(self.placed >> 2) & FAN_FOLD) * FAN_STEP;
        let spread = if base > 0 {
            base.wrapping_add(fan as i16)
        } else {
            base.wrapping_sub(fan as i16)
        };
        // `s16 rot = mStartStat.unk_00 + sVar3;` (:84)
        let rot = self.start_yaw.wrapping_add(spread);
        // ```
        // if (unk_2A4 == 0) { paVar5 = &current.pos; }
        // else { paVar5 = &mBarrelInfo[unk_2A4 - 1].unk_08; }
        // ```
        // (:87-91) - RULE 10 guards the `unk_2A4 - 1` index.
        let anchor = if self.placed == 0 {
            origin
        } else {
            let slot = (usize::from(self.placed as u16) - 1).min(BARRELS - 1);
            self.barrels[slot].pos
        };
        // `transS(sp0C); YrotM(rot); cMtx_multVec(sp24, &sp18, &unk_4A8);` (:94-97) - RULE 5b:
        // `super::stream::sin_cos` delegates to `super::walk::from_s16`, the crate's only path.
        let (sin_y, cos_y) = super::stream::sin_cos(rot);
        self.aim = Vec3::new(
            anchor.x + arm.x * cos_y + arm.z * sin_y,
            anchor.y + arm.y,
            anchor.z - arm.x * sin_y + arm.z * cos_y,
        );
    }

    /// `daComing2::checkLineCrossXZ` (`:269-285`) - the pure segment-crossing test
    /// `coming_checkS_main` calls.
    ///
    /// ```
    /// cXyz sp44(arg1->x - arg0->x, 0.0f, arg1->z - arg0->z);
    /// cXyz sp38(arg2->x - arg0->x, 0.0f, arg2->z - arg0->z);
    /// cXyz sp2C(arg3->x - arg0->x, 0.0f, arg3->z - arg0->z);
    /// if ((sp44.x * sp38.z - sp44.z * sp38.x) * (sp44.x * sp2C.z - sp44.z * sp2C.x) < 0.0f) {
    ///     cXyz sp20(arg2->x - arg3->x, 0.0f, arg2->z - arg3->z);
    ///     cXyz sp14(arg0->x - arg3->x, 0.0f, arg0->z - arg3->z);
    ///     cXyz sp08(arg1->x - arg3->x, 0.0f, arg1->z - arg3->z);
    ///     if ((sp20.x * sp14.z - sp20.z * sp14.x) * (sp20.x * sp08.z - sp20.z * sp08.x) < 0.0f) {
    ///         return TRUE;
    ///     }
    /// }
    /// return FALSE;
    /// ```
    fn line_cross_xz(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> bool {
        let p = Vec3::new(b.x - a.x, 0.0, b.z - a.z);
        let q = Vec3::new(c.x - a.x, 0.0, c.z - a.z);
        let r = Vec3::new(d.x - a.x, 0.0, d.z - a.z);
        // `(sp44.x * sp38.z - sp44.z * sp38.x) * (sp44.x * sp2C.z - sp44.z * sp2C.x) < 0.0f` (:274)
        if cross_xz(p, q) * cross_xz(p, r) < 0.0 {
            let s = Vec3::new(c.x - d.x, 0.0, c.z - d.z);
            let t = Vec3::new(a.x - d.x, 0.0, a.z - d.z);
            let u = Vec3::new(b.x - d.x, 0.0, b.z - d.z);
            // `(sp20.x * sp14.z - sp20.z * sp14.x) * (sp20.x * sp08.z - sp20.z * sp08.x) < 0.0f` (:279)
            if cross_xz(s, t) * cross_xz(s, u) < 0.0 {
                return true;
            }
        }
        false
    }

    /// `daComing2::Act_c::coming_process_init` (`:553-570`) - the `init_table` dispatch.
    fn process_init(&mut self, index: i16, ctx: &mut Ctx<'_>) -> bool {
        // ```
        // if (arg1 >= 0 && arg1 < 5) { if ((this->*init_table[arg1])()) { unk_46E = arg1;
        //                                                            return true; } }
        // return false;
        // ```
        // (:563-569)
        let Some(mode) = Mode::of(index) else {
            return false;
        };
        let ok = match mode {
            // ```
            // BOOL coming_clear_init() {
            //     daPy_py_c* player = daPy_getPlayerActorClass();
            //     if (player != NULL) { unk_299 = player->current.roomNo; }
            //     return TRUE;
            // }
            // ```
            // (:346-352)
            Mode::Clear => {
                if let Some(room) = ctx.player_room {
                    self.room = room;
                }
                true
            }
            // ```
            // BOOL coming_wait_init() { unk_2A6 = 600; unk_2A4 = 0; return TRUE; }
            // ```
            // (:365-369)
            Mode::Wait => {
                self.grace = ROOM_GRACE;
                self.placed = 0;
                true
            }
            // ```
            // BOOL coming_setF_init() {
            //     daPy_py_c* player = daPy_getPlayerActorClass();
            //     if (player != NULL) { unk_298 = player->current.roomNo; minus_coming_point2();
            //                             return TRUE; }
            //     return FALSE;
            // }
            // ```
            // (:397-405)
            Mode::SetFlag => {
                let Some(room) = ctx.player_room else {
                    return false;
                };
                self.saved_room = room;
                // `minus_coming_point2()` (:401) is two `minus_coming_point1` calls (`d_a_coming2.h:49-51`).
                self.minus_coming_point(ctx);
                self.minus_coming_point(ctx);
                true
            }
            // ```
            // BOOL coming_checkS_init() { unk_46C = 0; return TRUE; }
            // ```
            // (:473-477)
            Mode::CheckSail => {
                self.cross = 0;
                true
            }
            // ```
            // BOOL coming_game_init() { unk_29A = 1; unk_2A8 = 0; unk_299 = unk_298; unk_294++;
            //                          return TRUE; }
            // ```
            // (:496-502)
            Mode::Game => {
                self.popped = true;
                self.budget = 0;
                self.room = self.saved_room;
                self.runs += 1;
                true
            }
        };
        // `unk_46E = arg1;` (:566)
        if ok {
            self.mode = mode;
        }
        ok
    }

    /// `minus_coming_point1` (`d_a_coming2.h:42-48`):
    /// ```
    /// s32 evnt = dComIfGs_getEventReg(dSv_event_flag_c::UNK_7EFF);
    /// u8 evnt2 = (evnt - 1 < 0) ? 0 : evnt - 1;
    /// dComIfGs_setEventReg(dSv_event_flag_c::UNK_7EFF, evnt2);
    /// ```
    ///
    /// RULE 11c: `dSv_event_c` is byte-indexed by `hi_byte << 8 | mask` (`d_save.h:722`), so
    /// `UNK_7EFF` goes straight into `EventReg` (a `StoryBits`) with no shift or renumber. RULE 10:
    /// the `evnt - 1 < 0` test is a SIGNED compare on an `s32`, so the subtraction is done in `i32`.
    fn minus_coming_point(&mut self, ctx: &mut Ctx<'_>) {
        let evnt = i32::from(ctx.regs.reg(COMING_POINT));
        let next = if evnt - 1 < 0 {
            0
        } else {
            (evnt - 1) as u8
        };
        ctx.regs.set_reg(COMING_POINT, next);
    }

    /// `daComing2::Act_c::coming_process_main` (`:573-586`).
    fn process_main(&mut self, ctx: &mut Ctx<'_>, barrels: &mut Query<&mut BuoyBarrel>) {
        // `if (unk_46E >= 0 && unk_46E < 5) { (this->*main_table[unk_46E])(); }` (:583-584)
        match self.mode {
            // `coming_clear_main` (:355-362)
            Mode::Clear => self.clear_main(ctx, barrels),
            // `coming_wait_main` (:372-394)
            Mode::Wait => self.wait_main(ctx),
            // ```
            // coming_setF_main (:408-470)
            // ```
            // OMITTED with the placement gate (see the module docs): its whole body is the two buoy
            // spawns, each behind `get_water_height(&sp28.y, &sp28)` at `:445`. The mode therefore
            // never leaves `coming_setF_main`, exactly as the unavailable gate would leave it.
            Mode::SetFlag => {}
            // `coming_checkS_main` (:479-493)
            Mode::CheckSail => self.check_sail_main(ctx, barrels),
            // `coming_game_main` (:505-550)
            Mode::Game => self.game_main(),
        }
        // `Mode::index()` is the `main_table` index (`:575-581`); it is read so the table shape has
        // an observable use, mirroring the `arr[(unk_2A4 + unk_29C) & 1]` guard above.
        let _ = self.mode.index();
    }

    /// `coming_clear_main` (`:355-362`):
    /// ```
    /// if (request_all_barrel_exit() && request_all_flag_exit()) {
    ///     init_barrel_info();
    ///     init_coming_info();
    ///     init_flag_info();
    ///     coming_process_init(1);
    /// }
    /// ```
    fn clear_main(&mut self, ctx: &mut Ctx<'_>, barrels: &mut Query<&mut BuoyBarrel>) {
        // `request_all_barrel_exit()` (:356)
        if !self.request_all_barrel_exit(barrels) {
            return;
        }
        // `request_all_flag_exit()` (:356)
        if !self.request_all_flag_exit(barrels) {
            return;
        }
        let pos = self.start_pos;
        // `init_barrel_info();` (:357) -> `init_barrelN_info` (:193-198) for all fifteen.
        self.init_barrel_info(pos);
        // `init_coming_info();` (:358) -> `init_coming_info` (:214-220):
        // `unk_2A4 = 0; unk_2A6 = 600; unk_29C = (s32)cM_rndF(2.0f) & 1; unk_29A = 0; unk_2A8 = 0;`
        self.placed = 0;
        self.grace = ROOM_GRACE;
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        self.side = ((self.seed >> 8) as f32 / 16777216.0 * SIDE_SPAN) as i16 & 1;
        self.popped = false;
        self.budget = 0;
        // `init_flag_info();` (:359) -> `mStartBuoy[0..1].unk_00 = -1;` (:208-211)
        self.init_flag_info();
        // `coming_process_init(1);` (:360)
        self.process_init(1, ctx);
    }

    /// `coming_wait_main` (`:372-394`):
    ///
    /// ```
    /// daPy_py_c* player = daPy_getPlayerActorClass();
    /// if (player != NULL) {
    ///     if (dComIfGp_checkPlayerStatus0(0, daPyStts0_SHIP_RIDE_e)) {
    ///         if (unk_2A0 > 35.0f && get_coming_point() >= 2) {
    ///             if (unk_2A6 > 0) { unk_2A6--; }
    ///             if (unk_299 != fopAcM_GetRoomNo(player) && unk_2A6 == 0) {
    ///                 mStartStat.unk_00 = shape_angle.y;
    ///                 unk_440 = current.pos;
    ///                 coming_process_init(2);
    ///             }
    ///         } else { coming_process_init(1); }
    ///     } else {
    ///         unk_299 = player->current.roomNo;
    ///         coming_process_init(1);
    ///     }
    /// }
    /// ```
    fn wait_main(&mut self, ctx: &mut Ctx<'_>) {
        // `if (player != NULL)` (:373) - `daPy_getPlayerActorClass()`.
        let Some(room) = ctx.player_room else {
            return;
        };
        // `if (dComIfGp_checkPlayerStatus0(0, daPyStts0_SHIP_RIDE_e))` (:375)
        if !ctx.riding {
            // ```
            // unk_299 = player->current.roomNo;
            // coming_process_init(1);
            // ```
            // (:390-391)
            self.room = room;
            self.process_init(1, ctx);
            return;
        }
        // ```
        // if (unk_2A0 > 35.0f && get_coming_point() >= 2) { ... } else { coming_process_init(1); }
        // ```
        // (:376-388) - `get_coming_point()` is `dComIfGs_getEventReg(UNK_7EFF)`
        // (`d_a_coming2.h:38`).
        if self.speed > SPEED_FLOOR && ctx.regs.reg(COMING_POINT) >= 2 {
            // `if (unk_2A6 > 0) { unk_2A6--; }` (:377-379)
            if self.grace > 0 {
                self.grace -= 1;
            }
            // ```
            // if (unk_299 != fopAcM_GetRoomNo(player) && unk_2A6 == 0) {
            //     mStartStat.unk_00 = shape_angle.y;
            //     unk_440 = current.pos;
            //     coming_process_init(2);
            // }
            // ```
            // (:381-385)
            if self.room != room && self.grace == 0 {
                self.start_yaw = ctx.shape_yaw;
                self.start_pos = ctx.origin;
                self.process_init(2, ctx);
            }
        } else {
            // `} else { coming_process_init(1); }` (:387)
            self.process_init(1, ctx);
        }
    }

    /// `coming_checkS_main` (`:479-493`):
    ///
    /// ```
    /// if (unk_46C < 0xB4) {
    ///     if (checkLineCrossXZ(&old.pos, &current.pos, &mStartBuoy[0].unk_04, &mStartBuoy[1].unk_04)) {
    ///         request_all_flag_exit();
    ///         coming_process_init(4);
    ///     }
    ///     unk_46C++;
    /// } else {
    ///     daPy_py_c* player = daPy_getPlayerActorClass();
    ///     if (player != NULL) { unk_299 = player->current.roomNo; coming_process_init(0); }
    /// }
    /// ```
    fn check_sail_main(&mut self, ctx: &mut Ctx<'_>, barrels: &mut Query<&mut BuoyBarrel>) {
        if self.cross < CROSS_FRAMES {
            // ```
            // if (checkLineCrossXZ(&old.pos, &current.pos, &mStartBuoy[0].unk_04,
            //                      &mStartBuoy[1].unk_04)) { request_all_flag_exit();
            //                                               coming_process_init(4); }
            // ```
            // (:481-484)
            if Self::line_cross_xz(
                ctx.old,
                ctx.origin,
                self.buoys[0].pos,
                self.buoys[1].pos,
            ) {
                // `request_all_flag_exit();` (:482)
                self.request_all_flag_exit(barrels);
                // `coming_process_init(4);` (:483)
                self.process_init(4, ctx);
            }
            // `unk_46C++;` (:485)
            self.cross += 1;
        } else if let Some(room) = ctx.player_room {
            // ```
            // unk_299 = player->current.roomNo;
            // coming_process_init(0);
            // ```
            // (:489-490)
            self.room = room;
            self.process_init(0, ctx);
        }
    }

    /// `coming_game_main` (`:505-550`):
    ///
    /// ```
    /// u32 unk294 = unk_2A4;
    /// if (unk294 >= 15) { return; }
    /// if (unk_29A == 1) { ...make_coming_param + spawn...; if (unk_2A8++ >= 60) {
    ////            coming_process_init(0); } } else { unk_2A8 = 0; }
    /// ```
    ///
    /// (`:510-549`). The `unk_29A == 1` arm is OMITTED with the placement gate (module docs); the
    /// `else` arm survives.
    fn game_main(&mut self) {
        // `u32 unk294 = unk_2A4;` (:510) - NOTE the cast to `u32`, so a negative `unk_2A4` wraps
        // and the `>= 15` test passes.
        let index = self.placed as u16 as u32;
        // `if (unk294 >= 15) { return; }` (:512-514)
        if index >= BARRELS as u32 {
            return;
        }
        // ```
        // if (unk_29A == 1) { ... } else { unk_2A8 = 0; }
        // ```
        // (:516, :547-548)
        if !self.popped {
            // `unk_2A8 = 0;` (:548)
            self.budget = 0;
        }
    }

    /// `daComing2::Act_c::barrel_execute` (`:306-336`) for one slot.
    ///
    /// ```
    /// fpc_ProcID barrelProc = mBarrelInfo[idx].unk_00;
    /// if (barrelProc != fpcM_ERROR_PROCESS_ID_e) {
    ///     if ((BOOL)fopAcM_SearchByID(barrelProc, (fopAc_ac_c**)&barrel) == TRUE) {
    ///         if (barrel != NULL) { mBarrelInfo[idx].unk_06 = barrel->chk_item_give(); }
    ///         if (mBarrelInfo[idx].unk_04 == 0) { request_barrel_exitN(idx); }
    ///         else if (mBarrelInfo[idx].unk_04 > 0) { mBarrelInfo[idx].unk_04--; }
    ///     } else if (mBarrelInfo[idx].unk_06 == 0) {
    ///         init_barrelN_info(idx);
    ///         coming_process_init(0);
    ///     }
    ///     if (mBarrelInfo[idx].unk_06 == 1) {
    ///         if (unk_2A4 >= 15u) { coming_process_init(0); }
    ///         else { request_barrel_exitN(idx); init_barrelN_info(idx); unk_29A = 1; }
    ///     }
    /// }
    /// ```
    fn barrel_execute(&mut self, index: usize, ctx: &mut Ctx<'_>, barrels: &mut Query<&mut BuoyBarrel>) {
        // `if (barrelProc != fpcM_ERROR_PROCESS_ID_e)` (:310)
        let Some(barrel) = self.barrels[index].barrel else {
            return;
        };
        // ```
        // if ((BOOL)fopAcM_SearchByID(barrelProc, ...) == TRUE) { ... }
        // else if (mBarrelInfo[idx].unk_06 == 0) { init_barrelN_info(idx); coming_process_init(0); }
        // ```
        // (:311-324)
        let mut alive = false;
        if let Ok(found) = barrels.get_mut(barrel) {
            alive = true;
            // `mBarrelInfo[idx].unk_06 = barrel->chk_item_give();` (:313) - `chk_item_give()` is
            // `m476` (`d_a_obj_barrel2.h:96`), which `buoybarrel.rs` models as `BuoyBarrel::given`.
            self.barrels[index].given = found.given;
        } else if !self.barrels[index].given {
            // ```
            // init_barrelN_info(idx);
            // coming_process_init(0);
            // ```
            // (:322-323) - the barrel vanished WITHOUT handing over its item, so the run resets.
            self.init_barrelN(index, self.start_pos);
            self.process_init(0, ctx);
            return;
        }
        if alive {
            if self.barrels[index].delay == 0 {
                // `request_barrel_exitN(idx);` (:317)
                self.request_barrel_exitN(index, barrels);
            } else if self.barrels[index].delay > 0 {
                // `mBarrelInfo[idx].unk_04--;` (:319)
                self.barrels[index].delay -= 1;
            }
        }
        // ```
        // if (mBarrelInfo[idx].unk_06 == 1) {
        //     if (unk_2A4 >= 15u) { coming_process_init(0); }
        //     else { request_barrel_exitN(idx); init_barrelN_info(idx); unk_29A = 1; }
        // }
        // ```
        // (:326-334)
        if self.barrels[index].given {
            if (self.placed as u16 as u32) >= BARRELS as u32 {
                // `if (unk_2A4 >= 15u) { coming_process_init(0); }` (:327-328)
                self.process_init(0, ctx);
            } else {
                // ```
                // request_barrel_exitN(idx);
                // init_barrelN_info(idx);
                // unk_29A = 1;
                // ```
                // (:329-333)
                self.request_barrel_exitN(index, barrels);
                self.init_barrelN(index, self.start_pos);
                self.popped = true;
            }
        }
    }

    /// `daComing2::Act_c::request_barrel_exitN` (`:239-254`):
    ///
    /// ```
    /// fpc_ProcID barrelProc = mBarrelInfo[idx].unk_00;
    /// if (barrelProc != fpcM_ERROR_PROCESS_ID_e) {
    ///     if (fopAcM_SearchByID(barrelProc, ...) != NULL && barrel != NULL) {
    ///         barrel->exit_req(); return TRUE;
    ///     } else { return FALSE; }
    /// } else { return TRUE; }
    /// ```
    ///
    /// `exit_req()` is `m474 = 1` (`d_a_obj_barrel2.h:94`), modelled by `BuoyBarrel::request_exit`.
    fn request_barrel_exitN(&mut self, index: usize, barrels: &mut Query<&mut BuoyBarrel>) -> bool {
        let Some(barrel) = self.barrels[index].barrel else {
            return true;
        };
        match barrels.get_mut(barrel) {
            Ok(mut found) => {
                found.request_exit();
                true
            }
            Err(_) => false,
        }
    }

    /// `daComing2::Act_c::request_all_barrel_exit` (`:257-266`):
    /// ```
    /// for (i = 0; i < 15; i++) { if (request_barrel_exitN(i)) { count++; } }
    /// return count == 15 ? TRUE : FALSE;
    /// ```
    fn request_all_barrel_exit(&mut self, barrels: &mut Query<&mut BuoyBarrel>) -> bool {
        let mut count = 0usize;
        for index in 0..BARRELS {
            if self.request_barrel_exitN(index, barrels) {
                count += 1;
            }
        }
        count == BARRELS
    }

    /// `daComing2::Act_c::request_all_flag_exit` (`:288-303`):
    ///
    /// ```
    /// for (s32 i = 0; i < 2; i++) {
    ///     fpc_ProcID buoyProc = mStartBuoy[i].unk_00;
    ///     if (buoyProc != fpcM_ERROR_PROCESS_ID_e && fopAcM_SearchByID(buoyProc, ...) != NULL
    ///         && buoy != NULL) {
    ///         buoy->exit_req();
    ///         mStartBuoy[i].unk_00 = fpcM_ERROR_PROCESS_ID_e;
    ///     }
    /// }
    /// if ((mStartBuoy[0].unk_00 == fpcM_ERROR_PROCESS_ID_e)
    ///     && (mStartBuoy[1].unk_00 == fpcM_ERROR_PROCESS_ID_e)) { return TRUE; }
    /// return FALSE;
    /// ```
    fn request_all_flag_exit(&mut self, barrels: &mut Query<&mut BuoyBarrel>) -> bool {
        for index in 0..BOUYS {
            let Some(buoy) = self.buoys[index].buoy else {
                continue;
            };
            if barrels.get_mut(buoy).is_ok() {
                if let Ok(mut found) = barrels.get_mut(buoy) {
                    // `buoy->exit_req();` (:294)
                    found.request_exit();
                }
                // `mStartBuoy[i].unk_00 = fpcM_ERROR_PROCESS_ID_e;` (:295)
                self.buoys[index].buoy = None;
            }
        }
        // ```
        // if ((mStartBuoy[0].unk_00 == error) && (mStartBuoy[1].unk_00 == error)) { return TRUE; }
        // ```
        // (:299-301)
        self.buoys.iter().all(|buoy| buoy.buoy.is_none())
    }

    /// `daComing2::Act_c::init_barrelN_info` (`:193-198`) for one slot.
    fn init_barrelN(&mut self, index: usize, pos: Vec3) {
        let entry = &mut self.barrels[index];
        // `mBarrelInfo[idx].unk_00 = fpcM_ERROR_PROCESS_ID_e;` (:194)
        entry.barrel = None;
        // `mBarrelInfo[idx].unk_04 = 0;` (:195)
        entry.delay = 0;
        // `mBarrelInfo[idx].unk_06 = 0;` (:196)
        entry.given = false;
        // `mBarrelInfo[idx].unk_08 = current.pos;` (:197)
        entry.pos = pos;
    }

    /// `daComing2::Act_c::init_barrel_info` (`:201-205`): `for (u32 i = 0; i < 15; i++)
    /// { init_barrelN_info(i); }`.
    fn init_barrel_info(&mut self, pos: Vec3) {
        for index in 0..BARRELS {
            self.init_barrelN(index, pos);
        }
    }

    /// `daComing2::Act_c::init_flag_info` (`:208-211`): `mStartBuoy[0].unk_00 = -1;
    /// mStartBuoy[1].unk_00 = -1;`. `unk_04` is NOT reset, so it keeps its last placement.
    fn init_flag_info(&mut self) {
        for index in 0..BOUYS {
            self.buoys[index].buoy = None;
        }
    }

    /// `daComing2::Act_c::_execute` (`:617-632`).
    ///
    /// ```
    /// daShip_c* ship = (daShip_c*)fopAcM_SearchByID(fopAcM_GetLinkId(this));
    /// if (ship != NULL) {
    ///     chase_ship();
    ///     renew_scope_info();
    ///     if (dComIfGp_event_runCheck() || ship->getTornadoActor() != NULL
    ///         || ship->getWhirlActor() != NULL) {
    ///         daPy_py_c* player = daPy_getPlayerActorClass();
    ///         unk_299 = player->current.roomNo;
    ///         coming_process_init(0);
    ///     } else { coming_process_main(); }
    ///     barrel_main();
    /// }
    /// ```
    ///
    /// DROPPED: `ship->getTornadoActor()` and `ship->getWhirlActor()` (`:622`). The sim's
    /// `super::ship::Ship` carries only `speed`, `tiller`, `riding` and `warp_y`
    /// (`src/sim/ship.rs:43-51`) and has no tornado or whirl actor, so two of the three storm gates
    /// on this branch are unavailable. They are reported, not assumed false.
    fn execute(
        &mut self,
        ctx: &mut Ctx<'_>,
        actor: &mut Actor,
        barrels: &mut Query<&mut BuoyBarrel>,
    ) {
        // ```
        // daShip_c* ship = (daShip_c*)fopAcM_SearchByID(fopAcM_GetLinkId(this));
        // if (ship != NULL) { ... }
        // ```
        // (:618-619) - a NULL parent stops the actor cold; `fopAcM_SearchByID` failing also leaves
        // `actor` NULL, and retail dereferences it at `:624` in the event branch, so the sim guards
        // the same way rather than inventing a player.
        let Some(_) = ctx.ship else {
            return;
        };
        // `chase_ship();` (:620)
        Self::chase_ship(actor, ctx.ship);
        // `renew_scope_info();` (:621) - `get_speed` reads `current.pos` against `old.pos`, and
        // `chase_ship` has just overwritten `current.pos`, so the delta is this tick's ship motion.
        let speed = Self::get_speed(actor);
        self.renew_scope_info(actor.pos, speed);
        // ```
        // if (dComIfGp_event_runCheck() || ship->getTornadoActor() != NULL
        //     || ship->getWhirlActor() != NULL) { ... }
        // ```
        // (:622)
        if ctx.event_running {
            if let Some(room) = ctx.player_room {
                // `unk_299 = player->current.roomNo;` (:624)
                self.room = room;
            }
            // `coming_process_init(0);` (:625)
            self.process_init(0, ctx);
        } else {
            // `} else { coming_process_main(); }` (:627)
            self.process_main(ctx, barrels);
        }
        // ```
        // barrel_main();   ->  for (u32 i = 0; i < 15; i++) { barrel_execute(i); }
        // ```
        // (:629`, `:340-342`)
        for index in 0..BARRELS {
            self.barrel_execute(index, ctx, barrels);
        }
    }
}

/// `sp44.x * sp38.z - sp44.z * sp38.x` (`d_a_coming2.cpp:274`, `:279`) - the 2D cross product.
fn cross_xz(a: Vec3, b: Vec3) -> f32 {
    a.x * b.z - a.z * b.x
}

/// Everything `execute` needs beyond the component, in one bundle - the `hbrf1` / `tagetc` shape.
struct Ctx<'a> {
    /// `current.pos` after `chase_ship` (`:481`, `:384`).
    origin: Vec3,
    /// `old.pos` (`:481`) - the start-of-tick transform `fopAc_Execute` snapshots
    /// (`f_op_actor.cpp:196`), which `Actor::snapshot` maintains crate-wide.
    old: Vec3,
    /// `shape_angle.y` (`:382`) - the course driver's BODY yaw, copied from the ship by
    /// `chase_ship` (`:53`).
    shape_yaw: i16,
    /// The ship, resolved from the stage's single `Ship` (see [`Coming2::chase_ship`]).
    ship: Option<(&'a Vec3, [i16; 3])>,
    /// `dComIfGp_checkPlayerStatus0(0, daPyStts0_SHIP_RIDE_e)` (`:375`).
    riding: bool,
    /// `daPy_getPlayerActorClass()`'s `current.roomNo` (`:349`, `:381`, `:400`).
    player_room: Option<i8>,
    /// `dComIfGp_event_runCheck()` (`:622`).
    event_running: bool,
    /// `dSv_event_c`'s event registers, mutable for `minus_coming_point1`.
    regs: &'a mut EventReg,
}

/// The barrels `daObjBarrel2::Act_c::make_coming` created, queried for `chk_item_give` and
/// `exit_req`. An alias so the chain of functions that needs it reads as one word.
type Barrels<'w, 's, 'a> = Query<'w, 's, &'a mut BuoyBarrel>;

pub fn is_coming2(name: &str) -> bool {
    COMING2_ACTORS.contains(&name)
}

pub fn control(
    mut courses: Query<(Entity, &mut Actor, &mut Coming2), With<Coming2>>,
    mut barrels: Barrels<'_, '_, '_>,
    ships: Query<(&Actor, &Ship), Without<Coming2>>,
    players: Query<&Actor, With<Player>>,
    mut regs: ResMut<EventReg>,
    info: Res<crate::text::msgproc::MsgGameInfo>,
) {
    // RULE 9: the `&mut Actor` writer carries `With<Coming2>` and this reader `Without<Coming2>`,
    // which is the disjointness pair the brief asks for - `Without` on ONE side alone would not
    // prove it.
    let ship = ships.iter().next();
    // `dComIfGp_checkPlayerStatus0(0, daPyStts0_SHIP_RIDE_e)` (:375) - `Ship::riding`.
    let riding = ship.is_some_and(|(_, ship)| ship.riding);
    // `daPy_getPlayerActorClass()` (`:347`, `:373`, `:398`, `:487`, `:597`) - the room number is
    // `current.roomNo`, which `Actor::room` carries.
    let player_room = players.iter().next().map(|actor| actor.room);
    // `dComIfGp_event_runCheck()` (:622).
    let event_running = info.event_run_check();
    // `dComIfGs_getEventReg` / `dComIfGs_setEventReg` over `dSv_event_c` - `ResMut` so
    // `minus_coming_point1`'s write (`d_a_coming2.h:47`) reaches the real store.
    let store: &mut EventReg = &mut regs;
    for (_entity, mut actor, mut course) in &mut courses {
        let mut ctx = Ctx {
            // `current.pos` (`:481`, `:384`) - seeded from the stage placement by `fopAc_Init`.
            origin: actor.pos,
            // `old.pos` (`:481`).
            old: actor.old.pos,
            // `shape_angle.y` (`:382`).
            shape_yaw: actor.shape_angle[1],
            // The ship, resolved from the stage's single `Ship` - see [`Coming2::chase_ship`].
            ship: ship.map(|(actor, _)| (&actor.pos, actor.shape_angle)),
            riding,
            player_room,
            event_running,
            regs: store,
        };
        course.execute(&mut ctx, &mut actor, &mut barrels);
    }
}
