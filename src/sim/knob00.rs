//! Regular doors with knobs - `fpcNm_KNOB00_e`, `d_a_knob00.cpp`.
//!
//! `d_stage.cpp:465-472` registers EIGHT names against this one profile, not three:
//! `KNOB00`, `KNOB01`, `KNOB02`, `KNOB03`, `KNOB00D`, `KNOB01D`, `KNOB02D`, `KNOB03D` (all
//! `sub = 0`, `fpcNm_KNOB00_e`). The archive is `M_arcname[] = "knob"` (`d_a_knob00.cpp:17`) and
//! the profile is `g_profile_KNOB00` (`:929-944`). This module is the `dDoor_info_c` subclass
//! that never had one.
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE. `daKnob00_c` derives from `dDoor_info_c`
//! -> `fopAc_ac_c` and its own body (`d_a_knob00.h:60-76`) holds no `dCcD` member; the profile
//! `Group` is `fopAc_ACTOR_e` (`:940`). A closed door blocks the player as a registered `dBgW`
//! (`d_a_knob00.cpp:695`, `:278`), and moving backgrounds are one of the inputs RULE 3b lists as
//! absent. Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
//!
//! What survives is the decision half of the actor: the parameter / home-angle decode, the
//! `mDoorType` rewrites of `CreateInit`, `dDoor_info_c::frontCheck`, `daKnob00_c::msgDoor`,
//! `daKnob00_c::chkException` and the `actionInit` door-type ladder. Everything that turns those
//! choices into an opening - the stage event-info table behind `eventInfo.checkCommandDoor()`,
//! `makeEventId`, the `SHUTTER_DOOR` staff cut, the BCK, the `dBgW` and the password prompt - is
//! listed under OMITTED in the report.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::daynight::{DayNight, TIME_NIGHT};
use super::door::StayRoom;
use super::flags::{Collect, EventFlags, SYMBOL_DIN, SYMBOL_NAYRU};
use super::inventory::{Equipped, check_get_item};
use super::player::Player;

/// `d_stage.cpp:465-472`: all eight names bind to `fpcNm_KNOB00_e`.
pub const KNOB00_ACTORS: [&str; 8] = [
    "KNOB00",  // <- d_stage.cpp:465
    "KNOB01",  // <- d_stage.cpp:466
    "KNOB02",  // <- d_stage.cpp:467
    "KNOB03",  // <- d_stage.cpp:468
    "KNOB00D", // <- d_stage.cpp:469
    "KNOB01D", // <- d_stage.cpp:470
    "KNOB02D", // <- d_stage.cpp:471
    "KNOB03D", // <- d_stage.cpp:472
];

/// `dDoor_info_c::getType` (`d_door.cpp:27`).
const TYPE_SHIFT: u32 = 0x08; // <- d_door.cpp:27
const TYPE_MASK: u32 = 0x0F; // <- d_door.cpp:27
/// `dDoor_info_c::getSwbit2` (`d_door.cpp:22`), which `getShapeType` returns unchanged
/// (`d_a_knob00.cpp:131-133`).
const SHAPE_SHIFT: u32 = 0x14; // <- d_door.cpp:22
const SHAPE_MASK: u32 = 0xFF; // <- d_door.cpp:22
/// `getFRoomNo` (`d_door.cpp:45`) / `getBRoomNo` (`d_door.cpp:50`).
const ROOM_MASK: u32 = 0x3F; // <- d_door.cpp:45, :50
const ROOM_SHIFT: u32 = 0x06; // <- d_door.cpp:50
/// `getShipId` (`d_door.cpp:55`) / `getArg1` (`d_door.cpp:60`) are consumed by `openInitCom`'s
/// `valShipId()` and `dDoor_stop_c::closeInit` / `openInit` (`d_door.cpp:563`, `:589`), all of
/// which belong to the omitted `dBgW` and stop-bar halves, so they are not decoded here.

/// `mAction` (`d_a_knob00.h:70`), the values `execute`'s `switch(mAction)` uses
/// (`d_a_knob00.cpp:816-864`) and `setAction` / `nextAction` write (`d_a_knob00.h:14-25`).
const ACT_INIT: u8 = 0; // <- d_a_knob00.cpp:812, :817
const ACT_WAIT: u8 = 1; // <- d_a_knob00.cpp:822
const ACT_TALK_WAIT: u8 = 4; // <- d_a_knob00.cpp:830, :725
const ACT_PASSWORD: u8 = 5; // <- d_a_knob00.cpp:838, :712
const ACT_VILLA: u8 = 8; // <- d_a_knob00.cpp:846, :706
const ACT_FIGURE: u8 = 10; // <- d_a_knob00.cpp:854, :700
const ACT_ITEM: u8 = 11; // <- d_a_knob00.cpp:719

/// `mDoorType` (`d_a_knob00.h:75`) as `msgDoor` (`:195-206`) and `actionInit` (`:698-721`) read it.
const DOOR_MSG_ALWAYS: u8 = 1; // <- d_a_knob00.cpp:195
const DOOR_MSG_NIGHT: u8 = 2; // <- d_a_knob00.cpp:200
const DOOR_MSG_DAY: u8 = 3; // <- d_a_knob00.cpp:203
const DOOR_PASSWORD: u8 = 4; // <- d_a_knob00.cpp:710
const DOOR_VILLA: u8 = 5; // <- d_a_knob00.cpp:704
const DOOR_FIGURE: u8 = 6; // <- d_a_knob00.cpp:698

/// `getShapeType() == 4 && mDoorType == 1` -> `mDoorType = 5` (`d_a_knob00.cpp:355-357`).
const SHAPE_MARKET: u8 = 4; // <- d_a_knob00.cpp:355
/// `getShapeType() == 6` -> `mDoorType = 6; home.angle.z = 0x366a` (`:362-365`).
const SHAPE_FIGURE: u8 = 6; // <- d_a_knob00.cpp:362
/// The exception selector `CreateInit` stamps into `home.angle.z` for the figure door.
const FIGURE_EXCEPTION: i16 = 0x366A; // <- d_a_knob00.cpp:364

/// `chkException`'s `switch (home.angle.z)` cases (`d_a_knob00.cpp:287-329`).
const EXC_NAYRU: i16 = 0x6A6; // <- d_a_knob00.cpp:288
const EXC_UNK_0520: i16 = 0x6A7; // <- d_a_knob00.cpp:298
const EXC_ENDLESS_NIGHT: i16 = 0x6A8; // <- d_a_knob00.cpp:304
const EXC_UNK_1701: i16 = 0x6A9; // <- d_a_knob00.cpp:314
const EXC_DIN: i16 = 0x6AA; // <- d_a_knob00.cpp:320

/// `dSv_event_flag_c` ids (`d_save_event_flag.inc`), passed straight through per RULE 11c.
const EVENT_2110: u32 = 0x2110; // <- d_a_knob00.cpp:293, :309 (UNK_2110)
const EVENT_0520: u32 = 0x0520; // <- d_a_knob00.cpp:299 (UNK_0520)
const EVENT_ENDLESS_NIGHT: u32 = 0x0A02; // <- d_a_knob00.cpp:305 (ENDLESS_NIGHT)
const EVENT_1701: u32 = 0x1701; // <- d_a_knob00.cpp:315 (UNK_1701)
const EVENT_1A80: u32 = 0x1A80; // <- d_a_knob00.cpp:325 (UNK_1A80)
const EVENT_3401: u32 = 0x3401; // <- d_a_knob00.cpp:698, :747 (UNK_3401)
const EVENT_2D80: u32 = 0x2D80; // <- d_a_knob00.cpp:671, :704 (UNK_2D80)
const EVENT_1910: u32 = 0x1910; // <- d_a_knob00.cpp:633, :711 (UNK_1910)

/// `dItemNo_BOMB_BAG_e` (`d_com_inf_game.h` / `d_item_data.h:56`).
const ITEM_BOMB_BAG: u8 = 0x31; // <- d_a_knob00.cpp:717 (dItemNo_BOMB_BAG_e)

/// `fopAc_CULLBOX_6_e` (`d_a_knob00.cpp:943`) resolves through `l_cullSizeBox[6]`
/// (`f_op_actor_mng.cpp:564`).
const CULL_MIN: [f32; 3] = [-150.0, 0.0, -150.0]; // <- f_op_actor_mng.cpp:564 (fopAc_CULLBOX_6_e)
const CULL_MAX: [f32; 3] = [150.0, 300.0, 150.0]; // <- f_op_actor_mng.cpp:564 (fopAc_CULLBOX_6_e)

/// `frontCheck`'s sentinel for "the player is in neither of this door's rooms"
/// (`d_door.cpp:109-110`), and the value `checkExecute` compares against at `:167`.
const FRONT_NONE: u8 = 2; // <- d_door.cpp:110
/// `frontCheckOld`'s `angle < 0x4000` gate (`d_door.cpp:96`).
const FRONT_HALF_TURN: i16 = 0x4000; // <- d_door.cpp:96

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Knob00 {
    /// `mDoorType` (`d_a_knob00.h:75`), `getType()` with the two `CreateInit` rewrites applied.
    door_type: u8,
    /// `home.angle.z` (`d_a_knob00.h`), the `chkException` selector.
    exception: i16,
    /// `getFRoomNo()` = `home.angle.x & 0x3F` (`d_door.cpp:45`).
    front_room: u8,
    /// `getBRoomNo()` = `(home.angle.x >> 6) & 0x3F` (`d_door.cpp:50`).
    back_room: u8,
    /// `mFrontCheck` (`d_door.h:49`), written by `checkExecute` (`d_door.cpp:157`).
    front_check: u8,
    /// `mAction` (`d_a_knob00.h:70`).
    action: u8,
}

impl Knob00 {
    /// `daKnob00_c::CreateInit` (`:342-367`) plus the `dDoor_info_c` decodes it leans on.
    ///
    /// `create` (`:370-388`) adds nothing else: the res load, `fopAcM_ct` pair,
    /// `fopAcM_GetRoomNo(this) == -1 -> fopAcM_SetRoomNo(this, getFRoomNo())` (`:378-380`) and
    /// `fopAcM_entrySolidHeap` (`:382`) are all allocation or room plumbing.
    pub fn spawn(parameters: u32, angle: [i16; 3]) -> Self {
        let shape = ((parameters >> SHAPE_SHIFT) & SHAPE_MASK) as u8;
        let mut door_type = ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8;
        let mut exception = angle[2];
        // `if (getShapeType() == 4 && mDoorType == 1) mDoorType = 5;` (`:355-356`). The
        // `fopAcM_create(fpcNm_TAG_MK_e, 0xffffff03, ...)` at `:357` is a different actor and the
        // two `+= 60.0f` attention raises at `:358-359` are the attention/catch state RULE 3b
        // lists as absent.
        if shape == SHAPE_MARKET && door_type == DOOR_MSG_ALWAYS {
            door_type = DOOR_VILLA;
        }
        // `if (getShapeType() == 6) { mDoorType = 6; home.angle.z = 0x366a; }` (`:362-365`).
        if shape == SHAPE_FIGURE {
            door_type = DOOR_FIGURE;
            exception = FIGURE_EXCEPTION;
        }
        Self {
            door_type,
            exception,
            front_room: (u32::from(angle[0] as u16) & ROOM_MASK) as u8,
            back_room: ((u32::from(angle[0] as u16) >> ROOM_SHIFT) & ROOM_MASK) as u8,
            front_check: FRONT_NONE,
            action: ACT_INIT,
        }
    }

    /// `fopAc_CULLBOX_6_e` (`d_a_knob00.cpp:943`) -> `l_cullSizeBox[6]`
    /// (`f_op_actor_mng.cpp:564`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `dDoor_info_c::frontCheckOld` (`d_door.cpp:87-101`). `cSGlobe globe(player->current.pos -
    /// current.pos)` then `angle1 = globe.U() - current.angle.y`, and `cSGlobe::U()` is
    /// `mInclination` (`c_angle.h:177`), which `cSPolar::Globe` (`c_angle.cpp:246-248`) fills from
    /// `mAngle2` = `cAngle::Radian_to_SAngle(cM_atan2f(x, z))` (`c_angle.cpp:230`) - the XZ
    /// bearing in the usual `0x10000 == one turn` units. The `cSAngle::Abs()` is a plain `abs` on
    /// the `s16`, so per RULE 10 it is `unsigned_abs` here.
    fn front_check_old(&self, here: Vec3, yaw: i16, player: Vec3) -> u8 {
        let delta = player - here;
        let bearing = super::stream::angle_s(delta.x.atan2(delta.z));
        let angle = bearing.wrapping_sub(yaw).unsigned_abs();
        if angle < FRONT_HALF_TURN as u16 {
            0
        } else {
            1
        }
    }

    /// `dDoor_info_c::frontCheck` (`d_door.cpp:104-111`).
    fn front_check(&self, here: Vec3, yaw: i16, stay: i8, player: Vec3) -> u8 {
        if self.front_room == self.back_room {
            return self.front_check_old(here, yaw, player);
        }
        match stay {
            room if room == self.front_room as i8 => 0,
            room if room == self.back_room as i8 => 1,
            _ => FRONT_NONE,
        }
    }

    /// `daKnob00_c::msgDoor` (`:194-207`). `dKy_daynight_check()` is `d_kankyo.h:423`;
    /// `dKy_TIME_NIGHT_e` is `1` (`d_kankyo.h:388`).
    fn msg_door(&self, night: bool) -> bool {
        if self.door_type == DOOR_MSG_ALWAYS {
            return true;
        }
        if night {
            if self.door_type == DOOR_MSG_NIGHT {
                return true;
            }
        } else if self.door_type == DOOR_MSG_DAY {
            return true;
        }
        false
    }

    /// `daKnob00_c::chkException` (`:286-332`).
    fn chk_exception(&self, collect: &Collect, flags: &EventFlags) -> bool {
        match self.exception {
            EXC_NAYRU => {
                if collect.is_symbol(SYMBOL_NAYRU) {
                    return true;
                }
                !flags.test(EVENT_2110)
            }
            EXC_UNK_0520 => !flags.test(EVENT_0520),
            EXC_ENDLESS_NIGHT => {
                !flags.test(EVENT_ENDLESS_NIGHT) || flags.test(EVENT_2110)
            }
            EXC_UNK_1701 => flags.test(EVENT_1701),
            EXC_DIN => !collect.is_symbol(SYMBOL_DIN) || flags.test(EVENT_1A80),
            _ => false,
        }
    }

    /// `daKnob00_c::actionInit` (`:693-732`) reduced to the action each arm selects. Every arm's
    /// body is `actionFigure` / `actionVilla` / `actionPassward` / `actionTalkWait` /
    /// `actionWait`, and all five of those reach nothing but `eventInfo.checkCommandDoor()`
    /// (`:551`, `:593`, `:667`, `:681`, `:736`) and `setEventPrm` (`:556`, `:597`, `:674`, `:687`,
    /// `:750`) - the stage event-info table. What is left is the ladder, and the ladder is the
    /// part that decides which event the door offers.
    ///
    /// Note the `mDoorType == 4` arm (`:710-721`) has no `else`: with the Bomb Bag in hand a
    /// type-4 door still falls through to the `msgDoor` test.
    fn action_init(&self, collect: &Collect, flags: &EventFlags, equipped: &Equipped, night: bool) -> u8 {
        if self.door_type == DOOR_FIGURE && !flags.test(EVENT_3401) {
            return ACT_FIGURE;
        }
        if self.door_type == DOOR_VILLA && !flags.test(EVENT_2D80) {
            return ACT_VILLA;
        }
        if self.door_type == DOOR_PASSWORD {
            if !flags.test(EVENT_1910) {
                return ACT_PASSWORD;
            }
            if check_get_item(equipped, collect, ITEM_BOMB_BAG) {
                return ACT_ITEM;
            }
        }
        if self.msg_door(night) && !self.chk_exception(collect, flags) {
            return ACT_TALK_WAIT;
        }
        ACT_WAIT
    }

    /// `daKnob00_c::execute` (`:809-886`) and `dDoor_info_c::checkExecute` (`d_door.cpp:156-171`).
    /// `checkExecute`'s first statement, `mFrontCheck = frontCheck()` (`:157`), and `execute`'s
    /// last statement, `mRoomNo2 = dComIfGp_roomControl_getStayNo()` (`:878`), both run
    /// unconditionally in retail and are the whole of the per-tick bookkeeping that survives.
    fn execute(
        &mut self,
        collect: &Collect,
        flags: &EventFlags,
        equipped: &Equipped,
        stay: i8,
        night: bool,
        here: Vec3,
        yaw: i16,
        player: Vec3,
    ) {
        self.front_check = self.front_check(here, yaw, stay, player);
        // `checkExecute` (d_door.cpp:158-170) returns 2 when `mFrontCheck != 2 && adjoinPlayer()`
        // or when either `eventInfo` test fires, and `execute`'s `case 2:` with `mAction == 0`
        // is the only path to `actionInit` (d_a_knob00.cpp:816-819). The `eventInfo` tests read
        // the per-actor `mCommand` of the stage event-info table and `adjoinPlayer`
        // (d_door.cpp:64-75) needs `dComIfGp_roomControl_checkRoomDisp`; both are unavailable,
        // so the gate is reduced to its one available conjunct.
        if self.action != ACT_INIT || self.front_check == FRONT_NONE {
            return;
        }
        self.action = self.action_init(collect, flags, equipped, night);
    }
}

pub fn is_knob00(name: &str) -> bool {
    KNOB00_ACTORS.contains(&name)
}

pub fn control(
    stay: Res<StayRoom>,
    clock: Res<DayNight>,
    collect: Res<Collect>,
    flags: Res<EventFlags>,
    equipped: Res<Equipped>,
    players: Query<&Player>,
    mut doors: Query<(&mut Actor, &mut Knob00)>,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    let night = clock.daynight() == TIME_NIGHT;
    for (actor, mut door) in &mut doors {
        let here = actor.pos;
        let yaw = actor.angle[1];
        door.execute(
            &collect,
            &flags,
            &equipped,
            stay.0,
            night,
            here,
            yaw,
            player.pos,
        );
    }
}
