use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::chest::{
    Chests, FUNC_EXTRA_SAVE_INFO, FUNC_EXTRA_SAVE_INFO_SPAWN, FUNC_GRAVITY, FUNC_NORMAL,
    FUNC_SWITCH_VISIBLE, NO_ROOM, SEA2_STAGE, SHAPE_MAX, SWITCH_CEILING,
};
use super::collision::Collision;
use super::combat::HitPoints;
use super::door::StayRoom;
use super::event::{ActiveEvent, EventList};
use super::flags::Switches;
use super::ground;
use super::player::Player;
use super::stream;
use super::walk;

// Translation unit: tww/src/d/actor/d_a_tbox.cpp (fpcNm_TBOX_e, g_profile_TBOX).
// d_a_boko.cpp is fpcNm_BOKO_e (enemy weapons, owned by src/sim/boko.rs) and has
// nothing to do with treasure chests.

// d_stage.cpp:483-501 -- the 19 takara*/tkr* names mapped to fpcNm_TBOX_e.
// d_stage.cpp:502 maps tkrSlv to fpcNm_SBOX_e, which this actor does NOT own.
pub const TBOX_ACTORS: [&str; 19] = [
    "takara", "takara2", "takara3", "takara4", "takara5", "takara6", "takara7", "takara8",
    "takaraK", "takaraI", "takaraM", "tkrASw", "tkrAGc", "tkrAKd", "tkrAIk", "tkrBMs", "tkrCTf",
    "tkrAOc", "tkrAOs",
];

const FUNC_TYPE_SWITCH: u8 = 1; // <- d_a_tbox.cpp:24
const FUNC_TYPE_ENEMIES: u8 = 2; // <- d_a_tbox.cpp:25
const FUNC_TYPE_SWITCH_TRANSPARENT: u8 = 4; // <- d_a_tbox.cpp:27
const FUNC_TYPE_TACT: u8 = 6; // <- d_a_tbox.cpp:29
// FUNC_NORMAL/SWITCH_VISIBLE/GRAVITY/EXTRA_SAVE_INFO(_SPAWN) live in chest.rs;
// they are d_a_tbox.cpp:23,26,28,30,31 and are shared with daChest_c.

const FLG_UNK_01: u16 = 0x01; // <- d_a_tbox.h:31
const FLG_UNK_02: u16 = 0x02; // <- d_a_tbox.h:32
const FLG_UNK_04: u16 = 0x04; // <- d_a_tbox.h:33

const PARM_FUNC_MASK: u32 = 0x7F; // <- d_a_tbox.cpp:397 (getFuncType)
const PARM_TBOX_SHIFT: u32 = 0x07; // <- d_a_tbox.h:37 (getTboxNo)
const PARM_TBOX_MASK: u32 = 0x1F; // <- d_a_tbox.h:37 (getTboxNo)
const PARM_SWITCH_SHIFT: u32 = 0x0C; // <- d_a_tbox.h:38 (getSwNo)
const PARM_SWITCH_MASK: u32 = 0xFF; // <- d_a_tbox.h:38 (getSwNo)
const PARM_SHAPE_SHIFT: u32 = 0x14; // <- d_a_tbox.cpp:391 (getShapeType)
const PARM_SHAPE_MASK: u32 = 0x0F; // <- d_a_tbox.cpp:391 (getShapeType)

const ROOM_MASK: i8 = 0x3F; // <- d_a_tbox.cpp:1241 (home.angle.x & 0x3F)
const ROOM_FULL: i8 = 0x3F; // <- d_a_tbox.cpp:408
const SEEN_LIMIT: u16 = 0x2000; // <- d_a_tbox.cpp:558
const BOX_REACH: f32 = 100.0; // <- d_a_tbox.cpp:557 (SQUARE(100.0f))
const GENOCIDE_DELAY: u8 = 65; // <- d_a_tbox.cpp:489
const FALL_GRAVITY: f32 = -2.5; // <- d_a_tbox.cpp:546
const MAX_FALL: f32 = -100.0; // <- f_op_actor.cpp:286 (maxFallSpeed)

const EVENT_APPEAR: &str = "DEFAULT_TREASURE_APPEAR"; // <- d_a_tbox.cpp:1009

// d_a_tbox.cpp:908,953,999,1017,1027. actionDemo / actionDemo2 are OMITTED:
// they are gated on dComIfGp_evmng staff cuts and BCK end frames (getEndFrame),
// neither of which exists here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Wait,
    OpenWait,
    SwOnWait,
    SwOnWait2,
    Genocide,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Tbox {
    pub action: Action,
    pub shape: u8,
    pub func: u8,
    pub switch_no: u8,
    pub tbox_no: u8,
    pub room: i8,
    pub flags: u16,
    pub genocide_delay: u8,
    pub gravity: f32,
    pub speed: Vec3,
}

impl Tbox {
    pub fn spawn(parameters: u32, angle_x: i16, chests: &Chests, switches: &Switches) -> Self {
        let raw_shape = ((parameters >> PARM_SHAPE_SHIFT) & PARM_SHAPE_MASK) as u8;
        let mut tbox = Self {
            action: Action::Wait,
            // d_a_tbox.cpp:392 `shapeType >= 4 ? 0 : shapeType`
            shape: if raw_shape >= SHAPE_MAX { 0 } else { raw_shape },
            func: (parameters & PARM_FUNC_MASK) as u8,
            switch_no: ((parameters >> PARM_SWITCH_SHIFT) & PARM_SWITCH_MASK) as u8,
            tbox_no: ((parameters >> PARM_TBOX_SHIFT) & PARM_TBOX_MASK) as u8,
            // d_a_tbox.cpp:1241 `tbox->mRoomNo = tbox->home.angle.x & 0x3F`
            room: (angle_x & i16::from(ROOM_MASK)) as i8,
            flags: 0,
            genocide_delay: 0,
            gravity: 0.0,
            speed: Vec3::ZERO,
        };
        tbox.create_init(chests, switches);
        tbox
    }

    // d_a_tbox.cpp:310 `l_modelInfo[getShapeType()].brkId <= 0 ? FALSE : TRUE`.
    // l_modelInfo (d_a_tbox.cpp:35-68) carries a positive brkId for shapes 0..2 and
    // -1 for shape 3, so checkEnv is TRUE exactly for shapes 0,1,2.
    fn check_env(&self) -> bool {
        self.shape < 3
    }

    // d_a_tbox.cpp:314-321
    fn check_open(&self, chests: &Chests) -> bool {
        let stage = if self.func == FUNC_EXTRA_SAVE_INFO || self.func == FUNC_EXTRA_SAVE_INFO_SPAWN
        {
            SEA2_STAGE // <- d_a_tbox.cpp:316 (dSv_save_c::STAGE_SEA2, d_save.h:890)
        } else {
            0
        };
        chests.is_open(stage, self.tbox_no)
    }

    // d_a_tbox.cpp:401-422
    fn check_normal(&self, switches: &Switches) -> bool {
        if matches!(
            self.func,
            FUNC_NORMAL | FUNC_EXTRA_SAVE_INFO | FUNC_SWITCH_VISIBLE | FUNC_GRAVITY
        ) {
            return true;
        }
        if self.room == NO_ROOM || self.room == ROOM_FULL {
            return false;
        }
        if self.switch_no >= SWITCH_CEILING {
            return false;
        }
        switches.test(self.switch_no)
    }

    // d_a_tbox.cpp:447-519
    fn create_init(&mut self, chests: &Chests, switches: &Switches) {
        self.flags = 0;
        if self.check_open(chests) {
            // d_a_tbox.cpp:454-464
            self.action = Action::Wait;
        } else if !self.check_env() {
            // d_a_tbox.cpp:466-467
            self.action = Action::OpenWait;
        } else if self.check_normal(switches) {
            // d_a_tbox.cpp:470-482
            if self.func == FUNC_SWITCH_VISIBLE && !switches.test(self.switch_no) {
                self.action = Action::SwOnWait2;
            } else {
                self.action = Action::OpenWait;
            }
        } else {
            // d_a_tbox.cpp:484-516
            self.flags |= FLG_UNK_04;
            match self.func {
                FUNC_TYPE_ENEMIES => {
                    self.action = Action::Genocide;
                    self.genocide_delay = GENOCIDE_DELAY;
                    self.flags |= FLG_UNK_01 | FLG_UNK_02;
                }
                FUNC_TYPE_SWITCH | FUNC_EXTRA_SAVE_INFO_SPAWN => {
                    self.action = Action::SwOnWait;
                    self.flags |= FLG_UNK_01 | FLG_UNK_02;
                }
                FUNC_TYPE_TACT => {
                    self.action = Action::SwOnWait;
                    self.flags |= FLG_UNK_01 | FLG_UNK_02;
                }
                FUNC_TYPE_SWITCH_TRANSPARENT => {
                    self.action = Action::SwOnWait;
                    self.flags |= FLG_UNK_02;
                }
                _ => {}
            }
        }
        // d_a_tbox.cpp:542-547
        if self.func == FUNC_GRAVITY {
            self.gravity = FALL_GRAVITY;
        }
    }

    // d_a_tbox.cpp:1081 `if (flagCheck(daTboxFlg_UNK_01)) return TRUE;`
    pub fn drawn(&self) -> bool {
        self.flags & FLG_UNK_01 == 0
    }

    // d_a_tbox.cpp:1008/1019 `dComIfGs_isSwitch(getSwNo(), mRoomNo)`. Switches only
    // models the flat 0..128 memory+dungeon words (d_save.cpp:1586), which is exactly
    // the room-independent range checkNormal can reach.
    fn armed(&self, switches: &Switches) -> bool {
        switches.test(self.switch_no)
    }

    // d_a_tbox.cpp:553-564. Retail measures from home.pos; Actor carries no home
    // transform, and for every funcType other than FUNC_TYPE_GRAVITY the actor never
    // moves so current.pos == home.pos holds.
    fn box_check(&self, actor: &Actor, player: &Player) -> bool {
        let diff = player.pos - actor.pos;
        if diff.x * diff.x + diff.z * diff.z >= BOX_REACH * BOX_REACH {
            return false;
        }
        // fopAcM_seenPlayerAngleY (f_op_actor_mng.cpp) = |targetAngleY - shape_angle.y|
        let to_player = walk::to_s16(diff.x.atan2(diff.z));
        let chest_sight = to_player.wrapping_sub(actor.angle[1]).unsigned_abs();
        // fopAcM_seenActorAngleY(player, this)
        let to_chest = walk::to_s16(-diff.x.atan2(-diff.z));
        let player_sight = to_chest
            .wrapping_sub(walk::to_s16(player.face))
            .unsigned_abs();
        chest_sight < SEEN_LIMIT && player_sight < SEEN_LIMIT
    }

    // d_a_tbox.cpp:1036-1046
    fn genocide(&mut self, stay: i8, foes: bool, switches: &mut Switches) -> bool {
        if self.room == NO_ROOM || self.room != stay || foes {
            return false;
        }
        if self.genocide_delay != 0 {
            self.genocide_delay -= 1;
            return false;
        }
        switches.set(self.switch_no);
        true
    }

    // d_a_tbox.cpp:1170-1185 for FUNC_TYPE_GRAVITY. fopAcM_posMoveF is
    // calcSpeed (f_op_actor_mng.cpp:454) then posMove; dBgS_ObjAcch wall collision is
    // the merged-DZB Collision here.
    fn fall(&mut self, actor: &mut Actor, solid: Option<&Collision>) {
        if self.func != FUNC_GRAVITY {
            return;
        }
        let (next, _) =
            stream::calc_speed(0.0, actor.angle[1], self.speed.y, self.gravity, MAX_FALL);
        self.speed = next;
        let from = actor.pos;
        actor.pos += self.speed;
        if let Some(solid) = solid {
            actor.pos = solid.slide(from, actor.pos);
        }
        if let Some(floor) = ground::surface(solid, actor.pos)
            && actor.pos.y < floor
        {
            actor.pos.y = floor;
            self.speed.y = 0.0;
        }
    }

    // fopAcM_orderOtherEvent(this, "DEFAULT_TREASURE_APPEAR") at d_a_tbox.cpp:1009
    // and :1041. Retail fires it whenever the switch reads true, every frame; guard
    // on stage_of so a running event is never re-ordered back to Stage::Order.
    fn order_appear(list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        let Some(slot) = list.event_index(EVENT_APPEAR) else {
            return;
        };
        if active.stage_of(slot).is_none() {
            active.order(list, slot, entity);
        }
    }
}

pub fn is_tbox(name: &str) -> bool {
    TBOX_ACTORS.contains(&name)
}

pub fn control(
    stay: Res<StayRoom>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    collision: Option<Res<Collision>>,
    players: Query<&Player>,
    mut switches: ResMut<Switches>,
    foes: Query<(&Actor, Option<&HitPoints>), Without<Tbox>>,
    mut boxes: Query<(Entity, &mut Actor, &mut Tbox)>,
) {
    let solid = collision.as_deref();
    let player = players.iter().next().copied();
    for (entity, mut actor, mut tbox) in &mut boxes {
        let actor = &mut *actor;
        let tbox = &mut *tbox;
        match tbox.action {
            // d_a_tbox.cpp:909 `return TRUE;`
            Action::Wait => {}
            // d_a_tbox.cpp:954-996. checkCommandDoor() is absent (attention requests
            // are a confirmed-absent input), so only the boxCheck half can run and its
            // setEventName consumer is gated on that same command.
            Action::OpenWait => {
                if let Some(player) = player.as_ref() {
                    tbox.box_check(actor, player);
                }
            }
            // d_a_tbox.cpp:1000-1014
            Action::SwOnWait => {
                if tbox.armed(&switches) {
                    Tbox::order_appear(&list, &mut active, entity);
                }
            }
            // d_a_tbox.cpp:1018-1024 (setDzb is a dBgW collision swap, absent)
            Action::SwOnWait2 => {
                if tbox.armed(&switches) {
                    tbox.action = Action::OpenWait;
                }
            }
            // d_a_tbox.cpp:1028-1049
            Action::Genocide => {
                // fopAcM_myRoomSearchEnemy(mRoomNo) (f_op_actor_mng.cpp:1282) returns a
                // grabbed fopAc_ENEMY_e first, else any enemy in the room's layer.
                let busy = foes
                    .iter()
                    .any(|(other, hp)| hp.is_some() && other.room == tbox.room);
                if tbox.genocide(stay.0, busy, &mut switches) {
                    Tbox::order_appear(&list, &mut active, entity);
                }
            }
        }
        tbox.fall(actor, solid);
    }
}
