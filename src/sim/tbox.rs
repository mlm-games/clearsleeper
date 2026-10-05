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
use super::flags::Switches;
use super::ground;
use super::player::Player;
use super::stream;
use super::walk;

pub const TBOX_ACTORS: [&str; 19] = [
    "takara",
    "takara2",
    "takara3",
    "takara4",
    "takara5",
    "takara6",
    "takara7",
    "takara8",
    "takaraK",
    "takaraI",
    "takaraM",
    "tkrASw",
    "tkrAGc",
    "tkrAKd",
    "tkrAIk",
    "tkrBMs",
    "tkrCTf",
    "tkrAOc",
    "tkrAOs",
];

const FUNC_TYPE_SWITCH: u8 = 1;
const FUNC_TYPE_ENEMIES: u8 = 2;
const FUNC_TYPE_SWITCH_TRANSPARENT: u8 = 4;
const FUNC_TYPE_TACT: u8 = 6;

const FLG_UNK_01: u16 = 0x01;
const FLG_UNK_02: u16 = 0x02;
const FLG_UNK_04: u16 = 0x04;

const PARM_FUNC_MASK: u32 = 0x7F;
const PARM_TBOX_SHIFT: u32 = 0x07;
const PARM_TBOX_MASK: u32 = 0x1F;
const PARM_SWITCH_SHIFT: u32 = 0x0C;
const PARM_SWITCH_MASK: u32 = 0xFF;
const PARM_SHAPE_SHIFT: u32 = 0x14;
const PARM_SHAPE_MASK: u32 = 0x0F;

const ROOM_MASK: i16 = 0x3F;
const ROOM_FULL: i8 = 0x3F;
const SEEN_LIMIT: u16 = 0x2000;
const BOX_REACH: f32 = 100.0;
const GENOCIDE_DELAY: u8 = 65;
const FALL_GRAVITY: f32 = -2.5;
const MAX_FALL: f32 = -100.0;

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
            shape: if raw_shape >= SHAPE_MAX { 0 } else { raw_shape },
            func: (parameters & PARM_FUNC_MASK) as u8,
            switch_no: ((parameters >> PARM_SWITCH_SHIFT) & PARM_SWITCH_MASK) as u8,
            tbox_no: ((parameters >> PARM_TBOX_SHIFT) & PARM_TBOX_MASK) as u8,
            room: (angle_x & ROOM_MASK) as i8,
            flags: 0,
            genocide_delay: 0,
            gravity: 0.0,
            speed: Vec3::ZERO,
        };
        tbox.create_init(chests, switches);
        tbox
    }

    fn check_env(&self) -> bool {
        self.shape < 3
    }

    fn check_open(&self, chests: &Chests) -> bool {
        let stage = if self.func == FUNC_EXTRA_SAVE_INFO || self.func == FUNC_EXTRA_SAVE_INFO_SPAWN
        {
            SEA2_STAGE
        } else {
            0
        };
        chests.is_open(stage, self.tbox_no)
    }

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

    fn create_init(&mut self, chests: &Chests, switches: &Switches) {
        self.flags = 0;
        if self.check_open(chests) {
            self.action = Action::Wait;
        } else if !self.check_env() {
            self.action = Action::OpenWait;
        } else if self.check_normal(switches) {
            if self.func == FUNC_SWITCH_VISIBLE && !switches.test(self.switch_no) {
                self.action = Action::SwOnWait2;
            } else {
                self.action = Action::OpenWait;
            }
        } else {
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
        if self.func == FUNC_GRAVITY {
            self.gravity = FALL_GRAVITY;
        }
    }

    pub fn drawn(&self) -> bool {
        self.flags & FLG_UNK_01 == 0
    }

    fn armed(&self, switches: &Switches) -> bool {
        switches.test(self.switch_no)
    }

    fn box_check(&self, actor: &Actor, player: &Player) -> bool {
        let diff = player.pos - actor.pos;
        if diff.x * diff.x + diff.z * diff.z >= BOX_REACH * BOX_REACH {
            return false;
        }
        let to_player = walk::to_s16(diff.x.atan2(diff.z));
        let chest_sight = to_player.wrapping_sub(actor.angle[1]).unsigned_abs();
        let to_chest = walk::to_s16(-diff.x.atan2(-diff.z));
        let player_sight = to_chest.wrapping_sub(walk::to_s16(player.face)).unsigned_abs();
        chest_sight < SEEN_LIMIT && player_sight < SEEN_LIMIT
    }

    fn genocide(&mut self, stay: i8, foes: bool, switches: &mut Switches) {
        if self.room == NO_ROOM || self.room != stay || foes {
            return;
        }
        if self.genocide_delay != 0 {
            self.genocide_delay -= 1;
        } else {
            switches.set(self.switch_no);
        }
    }

    fn fall(&mut self, actor: &mut Actor, solid: Option<&Collision>) {
        if self.func != FUNC_GRAVITY {
            return;
        }
        let (next, _) = stream::calc_speed(0.0, actor.angle[1], self.speed.y, self.gravity, MAX_FALL);
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
}

pub fn is_tbox(name: &str) -> bool {
    TBOX_ACTORS.contains(&name)
}

pub fn control(
    stay: Res<StayRoom>,
    collision: Option<Res<Collision>>,
    players: Query<&Player>,
    mut switches: ResMut<Switches>,
    foes: Query<(&Actor, Option<&HitPoints>), Without<Tbox>>,
    mut boxes: Query<(&mut Actor, &mut Tbox)>,
) {
    let solid = collision.as_deref();
    let player = players.iter().next().copied();
    for (mut actor, mut tbox) in &mut boxes {
        let actor = &mut *actor;
        let tbox = &mut *tbox;
        match tbox.action {
            Action::Wait => {}
            Action::OpenWait => {
                if let Some(player) = player.as_ref() {
                    tbox.box_check(actor, player);
                }
            }
            Action::SwOnWait => {
                tbox.armed(&switches);
            }
            Action::SwOnWait2 => {
                if tbox.armed(&switches) {
                    tbox.action = Action::OpenWait;
                }
            }
            Action::Genocide => {
                let busy = foes
                    .iter()
                    .any(|(other, hp)| hp.is_some() && other.room == tbox.room);
                tbox.genocide(stay.0, busy, &mut switches);
            }
        }
        tbox.fall(actor, solid);
    }
}