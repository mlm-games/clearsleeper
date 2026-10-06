use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::collision::Collision;
use super::flags::{Collect, EventFlags, StoryBits};
use super::ground;
use super::intent::DoIntent;
use super::inventory::{Equipped, NO_ITEM, check_get_item};
use super::item::Held;
use super::pickup::Rupees;
use super::player::Player;
use super::sitem::SItem;
use super::walk::{from_s16, to_s16};
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgproc::{FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_BOX_CLOSING, STATUS_MSG_CONTINUES, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS,
};

pub const NPC_NZ_ACTORS: [&str; 1] = ["NpcNz"];

fn is_bomb(item: u8) -> bool {
    (super::item::FIRST_BOMB..=super::item::LAST_BOMB).contains(&item)
}

fn is_arrow(item: u8) -> bool {
    (super::item::FIRST_ARROW..=super::item::LAST_ARROW).contains(&item)
}

fn check_get_bottle(equipped: &Equipped) -> bool {
    (SLOT_BOTTLE0..SLOT_BOTTLE0 + SLOT_BOTTLE_COUNT)
        .any(|slot| equipped.item(slot) != NO_ITEM)
}

fn check_empty_bottle(equipped: &Equipped) -> u32 {
    (SLOT_BOTTLE0..SLOT_BOTTLE0 + SLOT_BOTTLE_COUNT)
        .filter(|slot| equipped.item(*slot) == ITEM_EMPTY_BOTTLE)
        .count() as u32
}

fn check_bait_item_empty(equipped: &Equipped) -> u32 {
    u32::from(equipped.item(SLOT_BAIT_BAG) != NO_ITEM)
}

fn joint_origin(joint: usize) -> Vec3 {
    if joint == ACTOR_HIP {
        JNT_HIP
    } else if joint == ACTOR_SHIPPO {
        JNT_SHIPPO
    } else if joint == ACTOR_UDEL3 {
        JNT_UDEL3
    } else if joint == ACTOR_UDER3 {
        JNT_UDER3
    } else if joint == ACTOR_HEAD {
        JNT_HEAD_EYE
    } else {
        Vec3::ZERO
    }
}

pub fn is_npc_nz(name: &str) -> bool {
    NPC_NZ_ACTORS.contains(&name)
}

const SLOT_BOW: usize = super::inventory::BOW_SLOT;
const SLOT_BOMB: usize = super::inventory::BOMB_SLOT;
const SLOT_BAIT_BAG: usize = 11;
const SLOT_BOTTLE0: usize = 14;
const SLOT_BOTTLE_COUNT: usize = 4;
const ARROW_CAP: i32 = 30;
const BOMB_CAP: i32 = 30;

const STAFF_NZ: &str = "NpcNz";
const EVENT_ESA: &str = "DEFAULT_NPC_NZ_ESA";
const EVENT_GETITEM: &str = "DEFAULT_NPC_NZ_GETITEM";
const ACTOR_ESA: &str = "Esa";
const ACTOR_NZ: &str = "Nz";
const ACTOR_BOMB: &str = "Bomb";
const ACTOR_TSUBO: &str = "Tsubo";
const ACTOR_BARREL: &str = "Barrel";
const SHOP_ITEM_ACTOR: &str = "ShopItem";

const HIO_NO: i8 = -1;
const HIO_MAX_HEAD_X: i16 = 0x1F40;
const HIO_MAX_HEAD_Y: i16 = 0x1F40;
const HIO_MAX_BACKBONE_X: i16 = 0x0000;
const HIO_MAX_BACKBONE_Y: i16 = 0x1F40;
const HIO_MIN_HEAD_X: i16 = -0x0BB8;
const HIO_MIN_HEAD_Y: i16 = -0x1F40;
const HIO_MIN_BACKBONE_X: i16 = 0x0000;
const HIO_MIN_BACKBONE_Y: i16 = -0x1F40;
const HIO_MAX_TURN_STEP: i16 = 0x07D0;
const HIO_MAX_HEAD_TURN_VEL: i16 = 0x0640;
const HIO_FIELD_1A: i8 = 0;
const HIO_FIELD_1C: f32 = 100.0;
const HIO_XY_OFFSET: f32 = 250.0;
const HIO_XY_RADIUS: f32 = 250.0;

const ANM_IDX_COUNT: usize = 15;
const ANM_IDX_TBL: [(i8, i8); ANM_IDX_COUNT] = [
    (0x23, 0x12),
    (0x15, 0x07),
    (0x16, -1),
    (0x17, 0x08),
    (0x18, 0x09),
    (0x19, 0x0A),
    (0x1A, 0x0B),
    (0x1B, 0x0C),
    (0x1C, 0x0D),
    (0x1D, 0x0E),
    (0x1F, 0x10),
    (0x20, 0x11),
    (0x1E, 0x11),
    (0x21, -1),
    (0x22, -1),
];

const BCK_ATTACK: f32 = 14.0;
const BCK_ATTACK2: f32 = 15.0;
const BCK_BUTUKARU: f32 = 39.0;
const BCK_DEAD: f32 = 23.0;
const BCK_DEAD2: f32 = 18.0;
const BCK_EAT: f32 = 15.0;
const BCK_EAT2: f32 = 19.0;
const BCK_KOKERU: f32 = 20.0;
const BCK_KOKERU2: f32 = 28.0;
const BCK_MOTU: f32 = 14.0;
const BCK_RUN: f32 = 11.0;
const BCK_RUN2: f32 = 17.0;
const BCK_TYAKKA1: f32 = 10.0;
const BCK_TYAKKA2: f32 = 23.0;
const BCK_WAIT: f32 = 20.0;

const ANM_PRMS: [(i8, i8, i8, f32, f32, u8); ANM_IDX_COUNT] = [
    (0x00, -1, 0x00, 8.0, 1.0, EMODE_LOOP),
    (0x01, -1, 0x00, 8.0, 1.0, EMODE_NONE),
    (0x02, -1, 0x00, 8.0, 1.0, EMODE_NONE),
    (0x03, -1, 0x00, 8.0, 1.0, EMODE_NONE),
    (0x04, -1, 0x00, 8.0, 1.0, EMODE_NONE),
    (0x05, -1, 0x00, 8.0, 1.0, EMODE_NONE),
    (0x06, -1, 0x00, 8.0, 1.0, EMODE_LOOP),
    (0x07, -1, 0x00, 8.0, 1.0, EMODE_LOOP),
    (0x08, -1, 0x00, 8.0, 1.0, EMODE_NONE),
    (0x09, -1, 0x00, 8.0, 1.0, EMODE_NONE),
    (0x0A, -1, 0x00, 8.0, 1.0, EMODE_LOOP),
    (0x0B, -1, 0x00, 8.0, 1.0, EMODE_LOOP),
    (0x0C, -1, 0x00, 8.0, 1.0, EMODE_NONE),
    (0x0D, -1, 0x00, 8.0, 1.0, EMODE_NONE),
    (0x0E, -1, 0x00, 8.0, 1.0, EMODE_NONE),
];

const BCK_INDEX_FIRST: usize = 0x15;
const BCK_FRAME_MAX: [f32; ANM_IDX_COUNT] = [
    BCK_ATTACK,
    BCK_ATTACK2,
    BCK_BUTUKARU,
    BCK_DEAD,
    BCK_DEAD2,
    BCK_EAT,
    BCK_EAT2,
    BCK_KOKERU,
    BCK_KOKERU2,
    BCK_MOTU,
    BCK_RUN,
    BCK_RUN2,
    BCK_WAIT,
    BCK_TYAKKA1,
    BCK_TYAKKA2,
];

const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 2;
const STATE_STOP: u8 = 2;

const PRM_WAIT: i8 = 0;
const PRM_EAT: i8 = 6;
const PRM_KOKERU: i8 = 8;
const PRM_KOKERU2: i8 = 9;
const PRM_RUN: i8 = 0x0A;
const PRM_RUN2: i8 = 0x0B;
const PRM_MOTU: i8 = 0x0C;
const PRM_KEEP: i8 = 0xF;

const MODE_WAIT: i32 = 0;
const MODE_EVENT_ESA: i32 = 1;

const PROC_INIT: i32 = 0;
const PROC_EXEC: i32 = 1;

const ORDER_NONE: u8 = 0;
const ORDER_TALK: u8 = 1;
const ORDER_ITEM: u8 = 2;
const ORDER_BOUGHT: u8 = 3;

const TAIL_COUNT: usize = 10;
const TAIL_BIND_COUNT: usize = 2;
const TAIL_FIRST: f32 = 5.0;
const TAIL_STEP: f32 = 20.0;
const TAIL_FALL: f32 = 2.0;
const TAIL_FADE: f32 = 0.1;
const TAIL_GROUND: f32 = 5.0;
const TAIL_CHASE: f32 = 0.8;

const BAIT_PULL_FRAME: f32 = 6.0;
const BAIT_PULL_SCALE: f32 = 0.1;
const BAIT_PULL_MAX: f32 = 2.5;
const BAIT_SPAWN_MAGIC: f32 = 358.0;
const BAIT_SPAWN_RANGE: f32 = 30.0;
const BAIT_SPAWN_DIST: f32 = 150.0;
const TUBO_RANGE: f32 = 300.0;
const EAT_TURN: f32 = 2048.0;
const HOME_BACK: f32 = 50.0;
const HOME_ENTER_DIST: f32 = 100.0;
const HOME_KILL_DIST: f32 = 250.0;
const HOME_HALF_TURN: i16 = 0x8000;
const CUT_TURN_SCALE: i16 = 8;
const CUT_TURN_MAX: i16 = 0x1000;
const GO_HOME_TURN_SCALE: i16 = 4;
const GO_HOME_TURN_MAX: i16 = 0x1000;
const GO_HOME_NEAR: i32 = 0x4000;

const ITEM_LINE_MAX: i16 = 20;
const ITEM_LINE_STEP_A: i16 = 12;
const ITEM_LINE_STEP_B: i16 = 16;
const ITEM_LINE_STEP_C: i16 = 18;
const LINE_MULT_X1: f32 = 0.85;
const LINE_MULT_Y1: f32 = 1.4;
const LINE_MULT_X2: f32 = 1.1;
const LINE_MULT_Y2: f32 = 0.8;
const LINE_MULT_X3: f32 = 1.0;
const LINE_MULT_Y3: f32 = 1.0;
const LINE_LIFT: f32 = -20.0;
const LINE_SCALE: f32 = 20.0;

const ITEM_NONE: u8 = 0xFF;
const ITEM_EMPTY_BOTTLE: u8 = 0x50;
const ITEM_BIRD_BAIT_5: u8 = 0x82;
const ITEM_HYOI_PEAR: u8 = 0x83;
const ITEM_RED_POTION: u8 = 0x51;
const ITEM_BLUE_POTION: u8 = 0x52;
const ITEM_BOMB_10: u8 = 0x0C;
const ITEM_BOMB_30: u8 = 0x0E;
const ITEM_ARROW_10: u8 = 0x10;
const ITEM_ARROW_30: u8 = 0x12;
const ITEM_BOMB_BAG: u8 = 0x31;

const SHOP_ITEM_ROWS: usize = 4;
const SHOP_ITEM_COLS: usize = 2;
const SHOP_ITEMS: [[u8; SHOP_ITEM_COLS]; SHOP_ITEM_ROWS] = [
    [ITEM_BIRD_BAIT_5, ITEM_HYOI_PEAR],
    [ITEM_RED_POTION, ITEM_BLUE_POTION],
    [ITEM_BOMB_10, ITEM_BOMB_30],
    [ITEM_ARROW_10, ITEM_ARROW_30],
];

const SHOP_NEXT_MSG: [[u32; SHOP_ITEM_COLS]; SHOP_ITEM_ROWS] = [
    [0x33FB, 0x33FC],
    [0x33FD, 0x33FE],
    [0x33FF, 0x3400],
    [0x3401, 0x3402],
];

const MSG_OPEN_A: u32 = 0x33F5;
const MSG_OPEN_B: u32 = 0x33F6;
const MSG_PICK_FIRST: u32 = 0x33F7;
const MSG_PICK_LAST: u32 = 0x33FA;
const MSG_BUY_FIRST: u32 = 0x33FB;
const MSG_BUY_LAST: u32 = 0x3402;
const MSG_QUIT: u32 = 0x3404;
const MSG_NO_RUPEE: u32 = 0x3405;
const MSG_NO_BOTTLE: u32 = 0x3406;
const MSG_MAX_ITEM: u32 = 0x3407;
const MSG_BOUGHT: u32 = 0x3403;
const MSG_PICK_BASE: u32 = 0x33F7;

const SELECT_PICK: u8 = 0;
const SELECT_BUY: u8 = 1;
const SELECT_CANCEL: u8 = 2;

const ORDER_FLAG_TALK: u16 = 0x0001;
const ORDER_FLAG_TALK_ITEM: u16 = 0x0020;

const ATTN_TALK: u8 = 0xB1;
const ATTN_SPEAK: u8 = 0xB1;
const ATTN_ACTION_SPEAK: u32 = 0x0000_0200;
const ATTN_TALKFLAG_NOTALK: u32 = 0x0000_0002;

const PLAYER_HEAD_TOP: f32 = 40.0;
const TALK_RANGE: f32 = 200.0;
const JNT_SCALE: i16 = 4;
const JNT_MIN_STEP: i32 = 1;
const JNT_CHASE_STEP: i16 = 1;

const CULL_MIN: [f32; 3] = [-450.0, -50.0, -450.0];
const CULL_MAX: [f32; 3] = [450.0, 750.0, 450.0];

const SPEED_TARGET: f32 = 0.0;
const SPEED_SCALE: f32 = 0.3;
const SPEED_MAX: f32 = 4.0;

const SE_TALK_IN: u32 = 0x4923;
const SE_TALK_OUT: u32 = 0x4924;

const SMOKE_SCALE: f32 = 0.65;
const SMOKE_PUFF_X: f32 = 10.0;
const SMOKE_PUFF_Y: f32 = 0.0;
const SMOKE_PUFF_Z: f32 = 10.0;
const SMOKE_PUFF_ROT: i16 = -0x4000;
const SMOKE_COLOR: u8 = 0x80;
const SMOKE_FRAME_A: f32 = 8.0;
const SMOKE_FRAME_B: f32 = 13.0;

const ACTOR_HIP: usize = 0x01;
const ACTOR_SHIPPO: usize = 0x09;
const ACTOR_UDEL3: usize = 0x12;
const ACTOR_UDER3: usize = 0x15;
const ACTOR_HEAD: usize = 0x0B;

const JNT_HIP: Vec3 = Vec3::new(0.0, 22.0, -14.0);
const JNT_SHIPPO: Vec3 = Vec3::new(0.0, 24.0, -30.0);
const JNT_UDEL3: Vec3 = Vec3::new(-7.0, 21.0, 3.0);
const JNT_UDER3: Vec3 = Vec3::new(7.0, 21.0, 3.0);
const JNT_HEAD_EYE: Vec3 = Vec3::new(0.0, 33.0, -6.0);

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const FLOAT_MAX: f32 = f32::MAX;

const REG10_F0: f32 = 0.0;
const REG10_F1: f32 = 0.0;
const REG10_F2: f32 = 0.0;
const REG10_F3: f32 = 0.0;
const REG10_F4: f32 = 0.0;
const REG10_F5: f32 = 0.0;
const REG10_F9: f32 = 0.0;
const REG10_F10: f32 = 0.0;
const REG10_F11: f32 = 0.0;
const REG10_F12: f32 = 0.0;
const REG10_F13: f32 = 0.0;
const REG10_F25: f32 = 0.0;
const REG10_S0: i32 = 0;
const REG10_S1: i32 = 0;
const REG10_S2: i32 = 0;
const REG10_S4: i32 = 0;
const REG10_S5: i32 = 0;
const REG10_S6: i32 = 0;

const JNT_HEAD: usize = 0;
const JNT_BACKBONE: usize = 1;
const JNT_AXIS_X: usize = 0;
const JNT_AXIS_Y: usize = 1;

const CUT_KEY_ANM_NO: &str = "anm_no";
const CUT_KEY_LOOP: &str = "loop_count";

const CUT_EAT_ESA: usize = 0;
const CUT_EAT_ESA_FIRST: usize = 1;
const CUT_KILL_ALL: usize = 2;
const CUT_SHOW: usize = 3;
const CUT_HIDE: usize = 4;
const CUT_SET_ANM: usize = 5;
const CUT_GO_HOME: usize = 6;
const CUT_CHK_GET_ITEM: usize = 7;

const ACTIONS: [&str; 8] = [
    "EATESA",
    "EATESA1ST",
    "KILLALL",
    "SHOW",
    "HIDE",
    "SETANM",
    "GOHOME",
    "CHKGETITEM",
];

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Statics {
    pub no: i8,
    pub max_head_x: i16,
    pub max_head_y: i16,
    pub max_backbone_x: i16,
    pub max_backbone_y: i16,
    pub min_head_x: i16,
    pub min_head_y: i16,
    pub min_backbone_x: i16,
    pub min_backbone_y: i16,
    pub max_turn_step: i16,
    pub max_head_turn_vel: i16,
    pub field_1a: i8,
    pub field_1c: f32,
    pub xy_offset: f32,
    pub xy_radius: f32,
}

impl Default for Statics {
    fn default() -> Self {
        Self {
            no: HIO_NO,
            max_head_x: HIO_MAX_HEAD_X,
            max_head_y: HIO_MAX_HEAD_Y,
            max_backbone_x: HIO_MAX_BACKBONE_X,
            max_backbone_y: HIO_MAX_BACKBONE_Y,
            min_head_x: HIO_MIN_HEAD_X,
            min_head_y: HIO_MIN_HEAD_Y,
            min_backbone_x: HIO_MIN_BACKBONE_X,
            min_backbone_y: HIO_MIN_BACKBONE_Y,
            max_turn_step: HIO_MAX_TURN_STEP,
            max_head_turn_vel: HIO_MAX_HEAD_TURN_VEL,
            field_1a: HIO_FIELD_1A,
            field_1c: HIO_FIELD_1C,
            xy_offset: HIO_XY_OFFSET,
            xy_radius: HIO_XY_RADIUS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub attribute: u8,
    pub state: u8,
    pub start: f32,
    pub end: f32,
    pub loop_point: f32,
    pub rate: f32,
    pub frame: f32,
    pub cur_morf: f32,
    pub prev_morf: f32,
    pub morf_step: f32,
}

impl Frame {
    pub const fn idle() -> Self {
        Self {
            attribute: EMODE_LOOP,
            state: 0,
            start: 0.0,
            end: 0.0,
            loop_point: 0.0,
            rate: 1.0,
            frame: 0.0,
            cur_morf: 1.0,
            prev_morf: 1.0,
            morf_step: 0.0,
        }
    }

    fn set_morf(&mut self, morf: f32) {
        if self.prev_morf < 0.0 || morf <= 0.0 {
            self.cur_morf = 1.0;
        } else {
            self.cur_morf = 0.0;
            self.morf_step = 1.0 / morf;
        }
        self.prev_morf = self.cur_morf;
    }

    fn set_anm(&mut self, end: f32, attribute: u8, morf: f32, speed: f32) {
        self.start = 0.0;
        self.end = end;
        self.loop_point = 0.0;
        self.rate = speed;
        self.frame = 0.0;
        self.attribute = attribute;
        self.state = 0;
        self.set_morf(morf);
    }

    fn update(&mut self) {
        self.state = 0;
        self.frame += self.rate;
        let start = self.start;
        let end = self.end;
        if self.attribute == EMODE_NONE {
            if self.frame < start {
                self.frame = start;
                self.rate = 0.0;
                self.state |= STATE_STOP;
            }
            if self.frame >= end {
                self.frame = end - 0.001;
                self.rate = 0.0;
                self.state |= STATE_STOP;
            }
        } else if self.attribute == EMODE_LOOP {
            let low = self.loop_point - start;
            let high = end - self.loop_point;
            while self.frame < start {
                self.state |= EMODE_LOOP;
                if low <= 0.0 {
                    break;
                }
                self.frame += low;
            }
            while self.frame >= end {
                self.state |= EMODE_LOOP;
                if high <= 0.0 {
                    break;
                }
                self.frame -= high;
            }
        }
    }

    fn play(&mut self) {
        if self.cur_morf < 1.0 {
            self.prev_morf = self.cur_morf;
            super::walk::chase_f(&mut self.cur_morf, 1.0, self.morf_step);
        }
        self.update();
    }

    pub fn get_frame(&self) -> f32 {
        self.frame
    }

    pub fn get_end_frame(&self) -> f32 {
        self.end
    }

    pub fn is_stop(&self) -> bool {
        self.state & STATE_STOP != 0
    }

    pub fn check_frame(&self, frame: f32) -> bool {
        (self.frame - frame).abs() < 1.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Mat {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
    pub t: Vec3,
}

impl Mat {
    const fn none() -> Self {
        Self {
            x: Vec3::X,
            y: Vec3::Y,
            z: Vec3::Z,
            t: Vec3::ZERO,
        }
    }

    fn pos(&self, src: Vec3) -> Vec3 {
        self.t + self.x * src.x + self.y * src.y + self.z * src.z
    }

    fn concat(&mut self, a: Vec3, b: Vec3, c: Vec3) {
        let x = a.x * self.x + a.y * self.y + a.z * self.z;
        let y = b.x * self.x + b.y * self.y + b.z * self.z;
        let z = c.x * self.x + c.y * self.y + c.z * self.z;
        self.x = x;
        self.y = y;
        self.z = z;
    }

    fn trans_s(&mut self, at: Vec3) {
        self.x = Vec3::X;
        self.y = Vec3::Y;
        self.z = Vec3::Z;
        self.t = at;
    }

    fn scale(&mut self, at: f32) {
        self.x *= at;
        self.y *= at;
        self.z *= at;
    }

    fn xrot_s(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.x = Vec3::X;
        self.y = Vec3::new(0.0, cos, sin);
        self.z = Vec3::new(0.0, -sin, cos);
        self.t = Vec3::ZERO;
    }

    fn xrot_m(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.concat(
            Vec3::X,
            Vec3::new(0.0, cos, sin),
            Vec3::new(0.0, -sin, cos),
        );
    }

    fn yrot_s(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.x = Vec3::new(cos, 0.0, -sin);
        self.y = Vec3::Y;
        self.z = Vec3::new(sin, 0.0, cos);
        self.t = Vec3::ZERO;
    }

    fn zxyrot_m(&mut self, angle: [i16; 3]) {
        if angle[1] != 0 {
            self.yrot_s(angle[1]);
        }
        if angle[0] != 0 {
            self.xrot_s(angle[0]);
        }
        if angle[2] != 0 {
            self.zrot_s(angle[2]);
        }
    }

    fn zrot_s(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.x = Vec3::new(cos, sin, 0.0);
        self.y = Vec3::new(-sin, cos, 0.0);
        self.z = Vec3::Z;
        self.t = Vec3::ZERO;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JntCtrl {
    pub angles: [[i16; 2]; 2],
    pub min_angles: [[i16; 2]; 2],
    pub max_angles: [[i16; 2]; 2],
    pub max_turn_step: [[i16; 2]; 2],
    pub trn: bool,
    pub head_lock: bool,
    pub back_bone_lock: bool,
}

impl JntCtrl {
    pub const fn new() -> Self {
        Self {
            angles: [[0; 2]; 2],
            min_angles: [[0; 2]; 2],
            max_angles: [[0; 2]; 2],
            max_turn_step: [[0; 2]; 2],
            trn: false,
            head_lock: false,
            back_bone_lock: false,
        }
    }

    pub fn trn_chk(&self) -> bool {
        self.trn
    }

    pub fn on_head_lock(&mut self) {
        self.head_lock = true;
    }

    pub fn off_head_lock(&mut self) {
        self.head_lock = false;
    }

    pub fn on_back_bone_lock(&mut self) {
        self.back_bone_lock = true;
    }

    pub fn off_back_bone_lock(&mut self) {
        self.back_bone_lock = false;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_param(
        &mut self,
        max_backbone_x: i16,
        max_backbone_y: i16,
        min_backbone_x: i16,
        min_backbone_y: i16,
        max_head_x: i16,
        max_head_y: i16,
        min_head_x: i16,
        min_head_y: i16,
        max_turn_step: i16,
    ) {
        self.max_angles[JNT_BACKBONE][JNT_AXIS_X] = max_backbone_x;
        self.max_angles[JNT_BACKBONE][JNT_AXIS_Y] = max_backbone_y;
        self.min_angles[JNT_BACKBONE][JNT_AXIS_X] = min_backbone_x;
        self.min_angles[JNT_BACKBONE][JNT_AXIS_Y] = min_backbone_y;
        self.max_angles[JNT_HEAD][JNT_AXIS_X] = max_head_x;
        self.max_angles[JNT_HEAD][JNT_AXIS_Y] = max_head_y;
        self.min_angles[JNT_HEAD][JNT_AXIS_X] = min_head_x;
        self.min_angles[JNT_HEAD][JNT_AXIS_Y] = min_head_y;
        self.max_turn_step[JNT_BACKBONE][JNT_AXIS_X] = max_turn_step;
        self.max_turn_step[JNT_BACKBONE][JNT_AXIS_Y] = max_turn_step;
        self.max_turn_step[JNT_HEAD][JNT_AXIS_X] = max_turn_step;
        self.max_turn_step[JNT_HEAD][JNT_AXIS_Y] = max_turn_step;
    }

    fn limitter(&self, value: &mut i16, joint: usize, axis: usize) {
        if *value > self.max_angles[joint][axis] {
            *value = self.max_angles[joint][axis];
        }
        if *value < self.min_angles[joint][axis] {
            *value = self.min_angles[joint][axis];
        }
    }

    fn ang_calc_s(&self, out: &mut i16, target: i16, speed: i16, max_vel: i16) -> bool {
        let orig_diff = out.wrapping_sub(target);
        let diff = super::walk::add_calc_angle_s(orig_diff, 0, speed, max_vel, JNT_CHASE_STEP);
        if i32::from(diff).abs() > i32::from(speed) {
            *out = out.wrapping_add(diff.wrapping_sub(orig_diff));
        } else {
            *out = target;
        }
        target.wrapping_sub(*out) == 0
    }

    fn follow(&mut self, out: &mut i16, target: i16, max_vel: i16, axis: usize) -> bool {
        let orig = *out;
        self.ang_calc_s(out, target, JNT_SCALE, max_vel);
        let delta = out.wrapping_sub(orig);
        let left = target.wrapping_sub(*out);
        let mut sum_prev = 0_i16;
        let mut sum = 0_i16;
        for joint in (0..2).rev() {
            sum = sum.wrapping_add(self.angles[joint][axis]);
            if (delta > 0 && sum > left) || (delta < 0 && sum < left) {
                let mut value = left.wrapping_sub(sum_prev);
                self.limitter(&mut value, joint, axis);
                self.angles[joint][axis] = value;
            }
            sum_prev = sum_prev.wrapping_add(self.angles[joint][axis]);
        }
        target != *out
    }

    fn jnt_move(&mut self, mut delta: i16, axis: usize) -> bool {
        let mut assigned = [0_i16; 2];
        for joint in 0..2 {
            let mut angle = if (joint == JNT_HEAD && self.head_lock)
                || (joint == JNT_BACKBONE && self.back_bone_lock)
            {
                0
            } else {
                delta
            };
            self.limitter(&mut angle, joint, axis);
            delta = delta.wrapping_sub(angle);
            assigned[joint] = angle;
            let mut current = i32::from(self.angles[joint][axis]);
            let max_step = i32::from(self.max_turn_step[joint][axis]);
            add_calc_angle_l(&mut current, i32::from(angle), JNT_SCALE as i32, max_step, JNT_MIN_STEP);
            self.angles[joint][axis] = current as i16;
        }
        let mut settled = 0_i32;
        for joint in 0..2 {
            if i32::from(assigned[joint].wrapping_sub(self.angles[joint][axis])).abs()
                <= i32::from(self.max_turn_step[joint][axis]) / 2
            {
                settled += 1;
            }
        }
        settled == 2
    }

    fn look_at_target(
        &mut self,
        out_y: &mut i16,
        dst: Option<Vec3>,
        src: Vec3,
        default_y: i16,
        max_vel: i16,
        hold_body: bool,
    ) {
        let (target_y, target_x) = match dst {
            Some(point) => {
                let delta = point - src;
                let dist_xz = abs_xz(delta);
                (
                    to_s16(delta.x.atan2(delta.z)),
                    to_s16(delta.y.atan2(dist_xz)),
                )
            }
            None => (default_y, 0),
        };
        let delta_y = target_y.wrapping_sub(*out_y);
        let max_y = i32::from(self.max_angles[JNT_BACKBONE][JNT_AXIS_Y])
            .wrapping_add(i32::from(self.max_angles[JNT_HEAD][JNT_AXIS_Y]));
        let min_y = i32::from(self.min_angles[JNT_BACKBONE][JNT_AXIS_Y])
            .wrapping_add(i32::from(self.min_angles[JNT_HEAD][JNT_AXIS_Y]));
        let wide = i32::from(delta_y);
        let settled = self.jnt_move(delta_y, JNT_AXIS_Y);
        if (self.trn || (wide >= max_y || wide <= min_y) && settled) && !hold_body {
            self.trn = self.follow(out_y, target_y, max_vel, JNT_AXIS_Y);
        }
        self.jnt_move(target_x, JNT_AXIS_X);
    }
}

impl Default for JntCtrl {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct NpcNz {
    pub entered: bool,
    pub health: i32,
    pub max_health: i32,
    pub cur_mode: i32,
    pub anm_idx: i8,
    pub prm_idx: i8,
    pub old_prm_idx: i8,
    pub speed_f: f32,
    pub staff_id: i32,
    pub order: u8,
    pub order_flags: u16,
    pub event_esa: i16,
    pub shop_visible: bool,
    pub msg_base: u32,
    pub msg_pick: i16,
    pub timer: i16,
    pub item_no: u8,
    pub shop_slot: u8,
    pub bait: Option<Entity>,
    pub bait_dist: f32,
    pub smoke: bool,
    pub xy_ok: bool,
    pub base_pos: Vec3,
    pub home_pos: Vec3,
    pub home_angle: [i16; 3],
    pub shape_angle: [i16; 3],
    pub old_pos: Vec3,
    pub speed: Vec3,
    pub scale: Vec3,
    pub eye_pos: Vec3,
    pub model_t: Vec3,
    pub model_basis: [Vec3; 3],
    pub attn_pos: Vec3,
    pub attn_dist: [u8; 2],
    pub attn_flags: u32,
    pub morph: Frame,
    pub jnt: JntCtrl,
    pub tail_pos: [Vec3; TAIL_COUNT],
    pub tail_prev: [Vec3; TAIL_COUNT],
    pub tail_bind: [Vec3; TAIL_BIND_COUNT],
    pub line_len: i16,
    pub shop: [Slot; 2],
    pub speak_ordered: bool,
    pub ordered_player: Option<Entity>,
    pub msg_no: u32,
    pub msg_status: u8,
    pub item_rupee_count: i16,
    pub given_item: u8,
    pub smoke_state: SmokeOp,
    pub smoke_origin: Vec3,
    pub smoke_parent: [i16; 3],
    pub smoke_scale: Vec3,
    pub smoke_color: [u8; 3],
    pub smoke_roll: i16,
    pub seed: [i32; 3],
    pub do_held: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slot {
    pub entity: Option<Entity>,
    pub pos: Vec3,
    pub scale: Vec3,
    pub hidden: bool,
}

impl Slot {
    const fn none() -> Self {
        Self {
            entity: None,
            pos: Vec3::ZERO,
            scale: Vec3::ZERO,
            hidden: true,
        }
    }
}

impl NpcNz {
    pub fn spawn(_parameters: u32) -> Self {
        Self {
            entered: false,
            health: 0,
            max_health: 0,
            cur_mode: MODE_WAIT,
            anm_idx: 0,
            prm_idx: 0,
            old_prm_idx: 0,
            speed_f: 1.0,
            staff_id: -1,
            order: ORDER_NONE,
            order_flags: 0,
            event_esa: -1,
            shop_visible: false,
            msg_base: 0,
            msg_pick: 0,
            timer: 0,
            item_no: 0,
            shop_slot: 0,
            bait: None,
            bait_dist: FLOAT_MAX,
            smoke: false,
            xy_ok: false,
            base_pos: Vec3::ZERO,
            home_pos: Vec3::ZERO,
            home_angle: [0; 3],
            shape_angle: [0; 3],
            old_pos: Vec3::ZERO,
            speed: Vec3::ZERO,
            scale: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            model_t: Vec3::ZERO,
            model_basis: [Vec3::X, Vec3::Y, Vec3::Z],
            attn_pos: Vec3::ZERO,
            attn_dist: [0; 2],
            attn_flags: 0,
            morph: Frame::idle(),
            jnt: JntCtrl::new(),
            tail_pos: [Vec3::ZERO; TAIL_COUNT],
            tail_prev: [Vec3::ZERO; TAIL_COUNT],
            tail_bind: [Vec3::ZERO; TAIL_BIND_COUNT],
            line_len: 0,
            shop: [Slot::none(); 2],
            speak_ordered: false,
            ordered_player: None,
            msg_no: 0,
            msg_status: 0,
            item_rupee_count: 0,
            given_item: 0,
            smoke_state: SmokeOp::Stop,
            smoke_origin: Vec3::ZERO,
            smoke_parent: [0; 3],
            smoke_scale: Vec3::ZERO,
            smoke_color: [0; 3],
            smoke_roll: 0,
            seed: [RND_SEED; 3],
            do_held: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.shop_visible
    }
}

fn add_calc_angle_l(value: &mut i32, target: i32, scale: i32, max_step: i32, min_step: i32) {
    let diff = target.wrapping_sub(*value);
    if *value != target {
        let mut step = diff / scale;
        if step > min_step || step < -min_step {
            step = step.clamp(-max_step, max_step);
            *value = value.wrapping_add(step);
        } else if diff >= 0 {
            *value = value.wrapping_add(min_step);
            if target.wrapping_sub(*value) <= 0 {
                *value = target;
            }
        } else {
            *value = value.wrapping_sub(min_step);
            if target.wrapping_sub(*value) >= 0 {
                *value = target;
            }
        }
    }
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let mut step = scale * (target - *value);
        if step > max_step {
            step = max_step;
        } else if step < -max_step {
            step = -max_step;
        }
        *value += step;
    }
}

fn add_calc_pos_xz(
    value: &mut Vec3,
    target: Vec3,
    scale: f32,
    max_step: f32,
    min_step: f32,
) -> f32 {
    if value.x != target.x || value.z != target.z {
        let mut diff = *value - target;
        let mut step = abs_xz(diff);
        if step < min_step {
            value.x = target.x;
            value.z = target.z;
        } else {
            step *= scale;
            diff *= scale;
            if step != 0.0 {
                if step > max_step {
                    diff *= max_step / step;
                } else if step < min_step {
                    diff *= min_step / step;
                }
                value.x -= diff.x;
                value.z -= diff.z;
            } else {
                value.x = target.x;
                value.z = target.z;
            }
        }
    }
    abs_xz(*value - target)
}

fn add_calc_pos2(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32) {
    if *value != target {
        let mut diff = (*value - target) * scale;
        if diff.length() > max_step {
            diff = diff.normalize_or_zero() * max_step;
        }
        *value -= diff;
    }
}

fn calc_timer(value: &mut i16) -> i16 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

fn abs_xz(v: Vec3) -> f32 {
    (v.x * v.x + v.z * v.z).sqrt()
}

fn atan2s(x: f32, y: f32) -> i16 {
    to_s16(x.atan2(y))
}

fn in_talk_range(range: f32, from: Vec3, to: Vec3) -> bool {
    let gap = to - from;
    gap.x * gap.x + gap.z * gap.z < range * range
}

fn target_angle_y(lhs: Vec3, rhs: Vec3) -> i16 {
    atan2s(rhs.x - lhs.x, rhs.z - lhs.z)
}

fn distance_angle_s(x: i16, y: i16) -> i32 {
    i32::from(x.wrapping_sub(y)).abs()
}

fn add_calc_angle_s2(value: &mut i16, target: i16, scale: i16, max_step: i16) {
    let diff = target.wrapping_sub(*value);
    let step = diff / scale;
    if step > max_step {
        *value = value.wrapping_add(max_step);
    } else if step < -max_step {
        *value = value.wrapping_sub(max_step);
    } else {
        *value = value.wrapping_add(step);
    }
}

fn ssin(angle: i16) -> f32 {
    from_s16(angle).sin()
}

fn scos(angle: i16) -> f32 {
    from_s16(angle).cos()
}

fn deg2s(deg: f32) -> i16 {
    to_s16(deg * (std::f32::consts::PI / 180.0))
}

fn max_limit_i(value: i32, max: i32) -> i32 {
    if value > max { max } else { value }
}

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0]) as f32 / RND_SCALE[0]
        + (seed[1]) as f32 / RND_SCALE[1]
        + (seed[2]) as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

impl NpcNz {
    fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (rnd(&mut self.seed) - 0.5) * 2.0
    }

    fn anm_prm(&self) -> Option<(i8, i8, i8, f32, f32, u8)> {
        let index = usize::try_from(self.prm_idx).unwrap_or(ANM_IDX_COUNT);
        ANM_PRMS.get(index).copied()
    }

    fn set_anm(&mut self, param_1: i8, force: bool) {
        if param_1 != PRM_KEEP {
            self.prm_idx = param_1;
        }
        if self.prm_idx == PRM_WAIT {
            self.jnt.off_head_lock();
            self.jnt.off_back_bone_lock();
        } else {
            self.jnt.on_head_lock();
            self.jnt.on_back_bone_lock();
        }
        let Some((anm_idx, _next, _old, morf, speed, loop_mode)) = self.anm_prm() else {
            return;
        };
        if (self.old_prm_idx != self.prm_idx && anm_idx != -1) || force {
            self.anm_idx = anm_idx;
            let index = usize::try_from(anm_idx).unwrap_or(ANM_IDX_COUNT);
            let bck = ANM_IDX_TBL
                .get(index)
                .map(|entry| usize::try_from(entry.0).unwrap_or(BCK_INDEX_FIRST))
                .unwrap_or(BCK_INDEX_FIRST);
            let end = BCK_FRAME_MAX
                .get(bck.saturating_sub(BCK_INDEX_FIRST))
                .copied()
                .unwrap_or(0.0);
            self.morph.set_anm(end, loop_mode, morf, speed);
        }
        self.old_prm_idx = self.prm_idx;
        if self.morph.is_stop() {
            let Some((_, next, _, _, _, loop_mode)) = self.anm_prm() else {
                return;
            };
            if next != -1 && loop_mode == EMODE_NONE {
                self.prm_idx = next;
            }
        }
    }

    fn tail_control(&mut self) {
        let sp58 = Vec3::new(0.0, 0.0, TAIL_FIRST);
        self.tail_pos[0] = self.tail_bind[0];
        let sp64 = self.tail_bind[1] - self.tail_bind[0];
        let angle = atan2s(sp64.x, sp64.z);
        let dist = abs_xz(sp64);
        let angle2 = atan2s(sp64.y, dist).wrapping_neg();
        let mut base = Mat::none();
        base.yrot_s(angle);
        base.xrot_m(angle2);
        let sp40 = base.pos(sp58);
        for i in 1..TAIL_COUNT {
            let temp3 = 1.0 - (i as f32 - 1.0) * TAIL_FADE;
            let mut sp34 = self.tail_prev[i];
            sp34.x += sp40.x * temp3;
            sp34.y += sp40.y * temp3;
            sp34.z += sp40.z * temp3;
            let mut temp5 = self.tail_pos[i].y + sp34.y - TAIL_FALL;
            if temp5 < self.home_pos.y + TAIL_GROUND {
                temp5 = self.home_pos.y + TAIL_GROUND;
            }
            let delta = Vec3::new(
                sp34.x + (self.tail_pos[i].x - self.tail_pos[i - 1].x),
                temp5 - self.tail_pos[i - 1].y,
                sp34.z + (self.tail_pos[i].z - self.tail_pos[i - 1].z),
            );
            let temp7 = atan2s(delta.x, delta.z);
            let dist = abs_xz(delta);
            let temp9 = atan2s(delta.y, dist).wrapping_neg();
            let sp58 = Vec3::new(0.0, 0.0, TAIL_STEP);
            let mut link = Mat::none();
            link.yrot_s(temp7);
            link.xrot_m(temp9);
            let sp4c = link.pos(sp58);
            self.tail_prev[i] = self.tail_pos[i];
            self.tail_pos[i] = self.tail_pos[i - 1] + sp4c;
            self.tail_prev[i].x = (self.tail_pos[i].x - self.tail_prev[i].x) * TAIL_CHASE;
            self.tail_prev[i].y = (self.tail_pos[i].y - self.tail_prev[i].y) * TAIL_CHASE;
            self.tail_prev[i].z = (self.tail_pos[i].z - self.tail_prev[i].z) * TAIL_CHASE;
        }
    }

    fn set_attention(&mut self, actor_pos: Vec3) {
        self.attn_pos = Vec3::new(actor_pos.x, self.eye_pos.y + REG10_F9, actor_pos.z);
    }

    fn look_back(&mut self, angle: &mut [i16; 3], player_eye: Option<Vec3>, statics: &Statics) {
        let target_y = angle[1];
        let dst_pos = player_eye;
        let max_follow_rot_vel = if self.jnt.trn_chk() {
            statics.max_head_turn_vel
        } else {
            0
        };
        self.jnt.set_param(
            statics.max_backbone_x,
            statics.max_backbone_y,
            statics.min_backbone_x,
            statics.min_backbone_y,
            statics.max_head_x,
            statics.max_head_y,
            statics.min_head_x,
            statics.min_head_y,
            statics.max_turn_step,
        );
        self.jnt.look_at_target(
            &mut angle[1],
            dst_pos,
            self.eye_pos,
            target_y,
            max_follow_rot_vel,
            true,
        );
    }

    fn event_order(&mut self) {
        if self.order == ORDER_TALK || self.order == ORDER_ITEM {
            self.order_flags |= ORDER_FLAG_TALK;
            self.order_flags |= ORDER_FLAG_TALK_ITEM;
            if self.order == ORDER_TALK {
                self.speak_ordered = true;
            }
        }
    }

    fn check_order(&mut self, talk_command: bool, player: Entity) {
        if talk_command && (self.order == ORDER_TALK || self.order == ORDER_ITEM) {
            if self.order == ORDER_ITEM {
                self.mode_proc_init(MODE_EVENT_ESA);
            }
            self.order = ORDER_NONE;
            self.ordered_player = Some(player);
        }
    }

    fn mode_wait_init(&mut self) {}

    fn mode_wait(&mut self) {
        self.order = ORDER_ITEM;
    }

    fn mode_event_esa_init(&mut self) {}

    fn mode_proc(&mut self, proc: i32, new_mode: i32, ctx: &mut Ctx<'_>) {
        if proc == PROC_INIT {
            self.mode_proc_init(new_mode);
        } else if proc == PROC_EXEC {
            match self.cur_mode {
                MODE_WAIT => self.mode_wait(),
                _ => self.mode_event_esa(ctx),
            }
        }
    }

    fn mode_proc_init(&mut self, new_mode: i32) {
        self.cur_mode = new_mode;
        match self.cur_mode {
            MODE_WAIT => self.mode_wait_init(),
            _ => self.mode_event_esa_init(),
        }
    }

    fn set_mtx(&mut self, base_pos: Vec3, shape_angle: [i16; 3], scale: Vec3) {
        let mut base = Mat::none();
        base.trans_s(base_pos);
        base.zxyrot_m(shape_angle);
        base.scale(scale.x);
        self.model_t = base.t;
        self.model_basis = [base.x, base.y, base.z];
        if self.prm_idx != PRM_MOTU && self.prm_idx != PRM_RUN2 {
            return;
        }
        let index = usize::from(self.shop_slot);
        if self.shop.get(index).is_none() {
            return;
        }
        self.line_len = max_limit_i(
            i32::from(self.line_len) + 1,
            i32::from(ITEM_LINE_MAX) + REG10_S2,
        ) as i16;
        let r7 = ITEM_LINE_STEP_A + REG10_S0 as i16;
        let r8 = ITEM_LINE_STEP_B + REG10_S1 as i16;
        let r6 = ITEM_LINE_STEP_C + REG10_S2 as i16;
        let mult_x1 = LINE_MULT_X1 + REG10_F0;
        let mult_y1 = LINE_MULT_Y1 + REG10_F1;
        let mult_x2 = LINE_MULT_X2 + REG10_F2;
        let mult_y2 = LINE_MULT_Y2 + REG10_F3;
        let mult_x3 = LINE_MULT_X3 + REG10_F4;
        let mult_y3 = LINE_MULT_Y3 + REG10_F5;
        let mut scale_x;
        let mut scale_y;
        let line = f32::from(self.line_len);
        if self.line_len < r7 {
            scale_x = line / f32::from(r7);
            scale_y = line / f32::from(r7);
            scale_x *= mult_x1;
            scale_y *= mult_y1;
        } else if self.line_len < r8 {
            scale_x = (line - f32::from(r7)) / f32::from(r8 - r7);
            scale_y = (line - f32::from(r7)) / f32::from(r8 - r7);
            scale_x = (scale_x * mult_x2) + (1.0 - scale_x) * mult_x1;
            scale_y = (scale_y * mult_y2) + (1.0 - scale_y) * mult_y1;
        } else {
            scale_x = (line - f32::from(r8)) / f32::from(r6 - r8);
            scale_y = (line - f32::from(r8)) / f32::from(r6 - r8);
            scale_x = (scale_x * mult_x3) + (1.0 - scale_x) * mult_x2;
            scale_y = (scale_y * mult_y3) + (1.0 - scale_y) * mult_y2;
        }
        let mut temp3 = joint_origin(ACTOR_UDEL3) + joint_origin(ACTOR_UDER3);
        let temp4 = match self.item_no {
            ITEM_BIRD_BAIT_5 => -15.0,
            ITEM_HYOI_PEAR => -5.0,
            ITEM_RED_POTION | ITEM_BLUE_POTION => -5.0,
            ITEM_BOMB_10 | ITEM_BOMB_30 => -15.0,
            ITEM_ARROW_10 | ITEM_ARROW_30 => -10.0,
            _ => 0.0,
        };
        temp3 *= 0.5;
        temp3.x += REG10_F10;
        temp3.y += (LINE_LIFT + REG10_F11) + scale_y * (LINE_SCALE + REG10_F13);
        temp3.z += REG10_F12;
        temp3.y += temp4;
        if let Some(slot) = self.shop.get_mut(index) {
            slot.scale = Vec3::new(scale_x, scale_y, scale_x);
            slot.pos = temp3;
        }
    }

    fn slot_mut(&mut self) -> Option<&mut Slot> {
        self.shop.get_mut(usize::from(self.shop_slot))
    }

    fn mode_event_esa(&mut self, ctx: &mut Ctx<'_>) {
        if !ctx.m_event_cut {
            self.cut_proc(ctx);
        }
        if self.event_esa >= 0
            && let Some(index) = usize::try_from(self.event_esa).ok()
            && ctx.end_check(index)
        {
            ctx.event_reset = true;
            self.event_esa = -1;
            self.delete_shop_item(ctx);
            self.shop_visible = false;
            self.old_pos = self.home_pos;
            ctx.actor_pos = self.old_pos;
            self.shape_angle = self.home_angle;
            ctx.actor_angle = self.shape_angle;
            self.mode_proc_init(MODE_WAIT);
        }
    }

    fn shop_bought_msg(&mut self, item: u8, ctx: &mut Ctx<'_>) -> u32 {
        let message_rupee = ctx.text.message_rupee();
        if i32::from(message_rupee) > i32::try_from(ctx.rupees.count).unwrap_or(i32::MAX) {
            return MSG_NO_RUPEE;
        }
        if (item == ITEM_RED_POTION || item == ITEM_BLUE_POTION)
            && check_empty_bottle(ctx.equipped) == 0
        {
            return MSG_NO_BOTTLE;
        }
        if is_bomb(item) && ctx.bomb_num == ctx.bomb_max {
            return MSG_MAX_ITEM;
        }
        if is_arrow(item) && ctx.arrow_num == ctx.arrow_max {
            return MSG_MAX_ITEM;
        }
        if (item == ITEM_BIRD_BAIT_5 || item == ITEM_HYOI_PEAR)
            && check_bait_item_empty(ctx.equipped) == 0
        {
            return MSG_MAX_ITEM;
        }
        if check_get_item(ctx.equipped, ctx.collect, item) {
            let spend = i32::from(message_rupee);
            ctx.rupees.count = ctx
                .rupees
                .count
                .saturating_sub(u32::try_from(spend).unwrap_or(0));
            ctx.item_rupee_count = -message_rupee;
            ctx.give_item = Some(item);
            return MSG_BOUGHT;
        }
        0
    }

    fn create_shop_item(
        &mut self,
        item_1: u8,
        item_2: u8,
        pos: Vec3,
        angle: [i16; 3],
        room: i8,
        ctx: &mut Ctx<'_>,
    ) {
        if self.shop[0].entity.is_none() {
            ctx.spawn_items.push((item_1, pos, angle, room));
        }
        if self.shop[1].entity.is_none() {
            ctx.spawn_items.push((item_2, pos, angle, room));
        }
    }

    fn delete_shop_item(&mut self, ctx: &mut Ctx<'_>) {
        for index in 0..self.shop.len() {
            let entity = self.shop.get(index).and_then(|slot| slot.entity);
            if let Some(entity) = entity {
                ctx.despawns.push(entity);
                if let Some(slot) = self.shop.get_mut(index) {
                    *slot = Slot::none();
                }
            }
        }
    }

    fn next_msg_status(&mut self, msg_no: &mut u32, ctx: &mut Ctx<'_>) -> u8 {
        let mut msg_status = STATUS_MSG_CONTINUES;
        if *msg_no == MSG_OPEN_A {
            *msg_no = MSG_OPEN_B;
        } else if *msg_no == MSG_OPEN_B {
            let mut temp = 1usize;
            if check_get_bottle(ctx.equipped) {
                temp += 1;
            }
            if check_get_item(ctx.equipped, ctx.collect, ITEM_BOMB_BAG) {
                temp += 1;
            }
            if ctx.equipped.item(SLOT_BOW) != NO_ITEM {
                temp += 1;
            }
            self.msg_pick = self.rnd_f(temp as f32) as i16;
            let pick = usize::try_from(self.msg_pick).unwrap_or(0);
            *msg_no = MSG_PICK_BASE + self.msg_pick as u32;
            let row = SHOP_ITEMS.get(pick).copied().unwrap_or([ITEM_NONE; SHOP_ITEM_COLS]);
            let pos = ctx.actor_pos;
            let angle = ctx.actor_angle;
            let room = ctx.actor_room;
            self.create_shop_item(row[0], row[1], pos, angle, room, ctx);
            self.msg_base = *msg_no;
        } else if (MSG_PICK_FIRST..=MSG_PICK_LAST).contains(msg_no) {
            let select = ctx.text.select_num();
            if select == SELECT_PICK {
                let pick = usize::try_from(self.msg_pick).unwrap_or(0);
                let row = SHOP_ITEMS.get(pick).copied().unwrap_or([ITEM_NONE; SHOP_ITEM_COLS]);
                let next = SHOP_NEXT_MSG.get(pick).copied().unwrap_or([0; SHOP_ITEM_COLS]);
                self.item_no = row.get(usize::from(select)).copied().unwrap_or(ITEM_NONE);
                *msg_no = next.get(usize::from(select)).copied().unwrap_or(0);
                self.shop_slot = select;
            } else if select == SELECT_CANCEL {
                *msg_no = MSG_QUIT;
            }
        } else if (MSG_BUY_FIRST..=MSG_BUY_LAST).contains(msg_no) {
            let select = ctx.text.select_num();
            if select == SELECT_BUY {
                let bought = self.shop_bought_msg(self.item_no, ctx);
                if bought != 0 {
                    *msg_no = bought;
                } else {
                    if let Some(slot) = self.slot_mut() {
                        slot.hidden = true;
                    }
                    self.set_anm(PRM_WAIT, false);
                    self.order = ORDER_BOUGHT;
                    self.event_esa = i16::try_from(ctx.event_index(EVENT_GETITEM)).unwrap_or(-1);
                    ctx.gt_item = Some(self.item_no);
                    msg_status = STATUS_MSG_ENDS;
                }
            } else if select == SELECT_PICK {
                *msg_no = self.msg_base;
            }
        } else if *msg_no == MSG_QUIT
            || *msg_no == MSG_NO_RUPEE
            || *msg_no == MSG_NO_BOTTLE
            || *msg_no == MSG_MAX_ITEM
        {
            *msg_no = MSG_BOUGHT;
        } else {
            msg_status = STATUS_MSG_ENDS;
        }
        msg_status
    }

    fn anm_atr(&mut self, ctx: &mut Ctx<'_>) {
        let end = self.morph.get_end_frame();
        if self.prm_idx == PRM_KOKERU2 && self.morph.check_frame(end - 1.0) {
            self.set_anm(PRM_WAIT, false);
        } else if self.prm_idx == PRM_MOTU && self.morph.check_frame(end - 1.0) {
            self.set_anm(PRM_RUN2, false);
        }
        if !ctx.text.send_button() {
            return;
        }
        let live = ctx.text.status();
        if live == Some(STATUS_BOX_CLOSING)
            || live == Some(STATUS_BOX_CLOSED)
            || live == Some(STATUS_MSG_ENDS)
        {
            return;
        }
        let msg_no = ctx.text.msg_no();
        if msg_no == MSG_OPEN_A || msg_no == MSG_OPEN_B {
        } else if (MSG_PICK_FIRST..=MSG_PICK_LAST).contains(&msg_no) {
            if let Some(slot) = self.slot_mut() {
                slot.hidden = true;
            }
            self.set_anm(PRM_WAIT, false);
        } else if (MSG_BUY_FIRST..=MSG_BUY_LAST).contains(&msg_no) {
            if let Some(slot) = self.slot_mut() {
                slot.hidden = false;
            }
            self.line_len = 0;
            ctx.se(SE_TALK_IN);
            self.set_anm(PRM_MOTU, false);
        } else if msg_no == MSG_NO_RUPEE || msg_no == MSG_NO_BOTTLE || msg_no == MSG_MAX_ITEM {
            if let Some(slot) = self.slot_mut() {
                slot.hidden = true;
            }
            self.set_anm(PRM_WAIT, false);
        } else if msg_no == MSG_QUIT {
            if let Some(slot) = self.slot_mut() {
                slot.hidden = true;
            }
            self.set_anm(PRM_WAIT, false);
        } else if msg_no == MSG_BOUGHT {
            if let Some(slot) = self.slot_mut() {
                slot.hidden = true;
            }
            self.set_anm(PRM_WAIT, false);
        }
    }

    fn xy_check(&self, item: u8, player_pos: Option<Vec3>, player_yaw: i16, statics: &Statics) -> bool {
        if item != ITEM_BIRD_BAIT_5 {
            return false;
        }
        let Some(player) = player_pos else {
            return false;
        };
        let mut temp = player;
        temp.x += statics.xy_offset * ssin(player_yaw);
        temp.z += statics.xy_offset * scos(player_yaw);
        let temp3 = self.base_pos - temp;
        abs_xz(temp3) < statics.xy_radius
    }

    pub fn xy_event(&self, list: &EventList) -> i16 {
        i16::try_from(list.event_index(EVENT_ESA).unwrap_or(usize::MAX)).unwrap_or(-1)
    }

    fn set_smoke_particle(&mut self, ctx: &mut Ctx<'_>) {
        if self.smoke {
            ctx.smoke.push(SmokeOp::Stop);
            self.smoke = false;
        }
        if !self.smoke {
            self.smoke = true;
            self.smoke_origin = self.base_pos;
            self.smoke_parent = ctx.actor_angle;
            ctx.smoke.push(SmokeOp::Start);
        }
    }

    fn smoke_puff(&mut self, shape_angle: [i16; 3], actor_angle: [i16; 3], ctx: &mut Ctx<'_>) {
        let f0 = SMOKE_SCALE + REG10_F25;
        let scale = Vec3::new(f0, f0, f0);
        let mut temp2 = actor_angle;
        temp2[1] = temp2[1].wrapping_sub(SMOKE_PUFF_ROT);
        self.smoke_roll = temp2[1];
        let temp4 = Vec3::new(
            SMOKE_PUFF_X + REG10_F0,
            SMOKE_PUFF_Y + REG10_F1,
            SMOKE_PUFF_Z + REG10_F2,
        );
        let color = [
            (i32::from(SMOKE_COLOR) + REG10_S4) as u8,
            (i32::from(SMOKE_COLOR) + REG10_S5) as u8,
            (i32::from(SMOKE_COLOR) + REG10_S6) as u8,
        ];
        self.smoke_origin = JNT_HEAD_EYE + temp4;
        self.smoke_parent = shape_angle;
        self.smoke_scale = scale;
        self.smoke_color = color;
        ctx.smoke.push(SmokeOp::Puff);
    }

    fn create_init(&mut self, actor: &mut Actor) {
        self.set_anm(PRM_WAIT, false);
        self.mode_proc_init(MODE_WAIT);
        self.entered = true;
        self.base_pos = actor.pos;
        self.home_pos = actor.pos;
        self.home_angle = actor.angle;
        self.shape_angle = actor.angle;
        self.old_pos = actor.pos;
        self.attn_pos = actor.pos;
        self.attn_dist = [ATTN_TALK, ATTN_SPEAK];
        self.attn_flags = ATTN_ACTION_SPEAK | ATTN_TALKFLAG_NOTALK;
        self.order = ORDER_NONE;
        self.shop_visible = false;
        self.timer = 0;
        self.home_pos.x -= ssin(self.home_angle[1]) * HOME_BACK;
        self.home_pos.z -= scos(self.home_angle[1]) * HOME_BACK;
        actor.pos = self.home_pos;
        self.shop = [Slot::none(); 2];
    }

    fn search_esa(
        &mut self,
        hits: &[Hit],
        world_pos: Vec3,
        frame: f32,
        pulls: &mut Vec<(Entity, Vec3)>,
    ) -> Option<Hit> {
        let mut found = None;
        let mut best = self.bait_dist;
        for hit in hits {
            if !hit.esa_free {
                continue;
            }
            let delta = hit.pos - world_pos;
            if frame < BAIT_PULL_FRAME {
                let mut pos = hit.pos;
                add_calc_pos2(&mut pos, world_pos, BAIT_PULL_SCALE, BAIT_PULL_MAX);
                pulls.push((hit.entity, pos));
            }
            let gap = abs_xz(delta);
            if gap < best {
                best = gap;
                found = Some(hit.clone());
            }
        }
        if found.is_some() {
            self.bait_dist = best;
        }
        found
    }

    fn search_kill_nz_bomb(hits: &[Hit], kills: &mut Vec<Entity>) {
        for hit in hits {
            if hit.name == ACTOR_NZ || hit.name == ACTOR_BOMB {
                kills.push(hit.entity);
            }
        }
    }

    fn search_tsubo_barrel(&self, hits: &[Hit], home: Vec3, kills: &mut Vec<Entity>) {
        for hit in hits {
            if hit.name != ACTOR_TSUBO && hit.name != ACTOR_BARREL {
                continue;
            }
            let delta = home - hit.pos;
            if abs_xz(delta) < TUBO_RANGE {
                let success = if hit.name == ACTOR_TSUBO {
                    hit.pos_init
                } else if hit.name == ACTOR_TSUBO {
                    hit.pos_init
                } else {
                    false
                };
                if !success {
                    kills.push(hit.entity);
                }
            }
        }
    }

    fn search_esa_init(&self, hits: &[Hit], mut pos: Vec3, home_angle: i16, seed: &mut [i32; 3]) -> Vec<(Entity, Vec3)> {
        let angle = home_angle;
        pos.x += ssin(angle) * BAIT_SPAWN_DIST;
        pos.z += scos(angle) * BAIT_SPAWN_DIST;
        let deg = rnd(seed) * BAIT_SPAWN_MAGIC;
        let spread_angle = deg2s(deg);
        let mag = rnd(seed) * BAIT_SPAWN_RANGE;
        pos.x += mag * ssin(spread_angle);
        pos.z += mag * scos(spread_angle);
        let mut out = Vec::new();
        for hit in hits {
            if hit.name == ACTOR_ESA {
                out.push((hit.entity, pos));
            }
        }
        out
    }

    fn cut_proc(&mut self, ctx: &mut Ctx<'_>) {
        let Some((cut, advance)) = ctx.active.staff_cut(ctx.list, STAFF_NZ) else {
            self.staff_id = -1;
            return;
        };
        self.staff_id = i32::try_from(ctx.staff_of(STAFF_NZ)).unwrap_or(-1);
        let name = ctx.list.cut_name(cut).unwrap_or("");
        let index = ACTIONS.iter().position(|action| *action == name);
        let Some(index) = index else {
            ctx.active.end_staff_cut(ctx.list, ctx.ev_flags, STAFF_NZ);
            return;
        };
        if advance {
            match index {
                CUT_EAT_ESA => self.cut_eatesa_start(),
                CUT_EAT_ESA_FIRST => self.cut_eatesa_first_start(ctx),
                CUT_KILL_ALL => self.cut_kill_all_start(ctx),
                CUT_SHOW => {
                    self.shop_visible = true;
                }
                CUT_HIDE => {
                    self.shop_visible = false;
                }
                CUT_SET_ANM => self.cut_set_anm_start(cut, ctx),
                CUT_GO_HOME => self.cut_go_home_start(),
                CUT_CHK_GET_ITEM => self.cut_chk_get_item_start(ctx),
                _ => {}
            }
        }
        match index {
            CUT_EAT_ESA => self.cut_eatesa_proc(ctx),
            CUT_EAT_ESA_FIRST => self.cut_eatesa_first_proc(ctx),
            CUT_KILL_ALL => self.cut_kill_all_proc(ctx),
            CUT_SHOW => self.cut_show_proc(ctx),
            CUT_HIDE => self.cut_hide_proc(ctx),
            CUT_SET_ANM => self.cut_set_anm_proc(ctx),
            CUT_GO_HOME => self.cut_go_home_proc(ctx),
            CUT_CHK_GET_ITEM => self.cut_chk_get_item_proc(ctx),
            _ => {}
        }
    }

    fn cut_eatesa_start(&mut self) {
        self.set_anm(PRM_EAT, false);
    }

    fn cut_eatesa_proc(&mut self, ctx: &mut Ctx<'_>) {
        let mut pulls = Vec::new();
        let bait = self.search_esa(&ctx.hits, ctx.actor_pos, self.morph.get_frame(), &mut pulls);
        ctx.pulls.append(&mut pulls);
        let caught = bait.map(|hit| hit.entity);
        self.bait = caught;
        let end = self.morph.get_end_frame();
        if self.morph.check_frame(end - 1.0) {
            let turn = (self.rnd_fx(4.0) * EAT_TURN) as i16;
            ctx.actor_angle[1] = self.home_angle[1].wrapping_add(turn);
            self.shape_angle[1] = ctx.actor_angle[1];
            ctx.actor_angle = self.shape_angle;
        }
        if let Some(bait) = caught {
            ctx.despawns.push(bait);
            self.bait = None;
            self.bait_dist = FLOAT_MAX;
        } else if self.morph.check_frame(end - 1.0) {
            ctx.actor_angle[1] = self.home_angle[1];
            self.shape_angle = ctx.actor_angle;
            ctx.actor_angle = self.shape_angle;
            ctx.cut_end();
        }
    }

    fn cut_eatesa_first_start(&mut self, ctx: &mut Ctx<'_>) {
        let mut pulls = Vec::new();
        let bait = self.search_esa(&ctx.hits, ctx.actor_pos, self.morph.get_frame(), &mut pulls);
        ctx.pulls.append(&mut pulls);
        let caught = bait.map(|hit| hit.entity);
        self.bait = caught;
        ctx.actor_angle[1] = self.home_angle[1];
        self.shape_angle = ctx.actor_angle;
        self.set_anm(PRM_RUN, false);
    }

    fn cut_eatesa_first_proc(&mut self, ctx: &mut Ctx<'_>) {
        let mut target = self.base_pos;
        target.x += ssin(self.home_angle[1]) * HOME_ENTER_DIST;
        target.z += scos(self.home_angle[1]) * HOME_ENTER_DIST;
        if self.prm_idx == PRM_RUN {
            let temp2 = add_calc_pos_xz(&mut ctx.actor_pos, target, 0.25, 10.0, 2.0);
            let temp3 = target_angle_y(ctx.actor_pos, target);
            add_calc_angle_s2(&mut self.shape_angle[1], temp3, CUT_TURN_SCALE, CUT_TURN_MAX);
            if temp2 == 0.0 && self.morph.check_frame(self.morph.get_end_frame() - 1.0) {
                self.set_anm(PRM_EAT, false);
                self.timer = 2;
            } else {
                return;
            }
        }
        let mut pulls = Vec::new();
        let bait = self.search_esa(&ctx.hits, ctx.actor_pos, self.morph.get_frame(), &mut pulls);
        ctx.pulls.append(&mut pulls);
        let caught = bait.map(|hit| hit.entity);
        self.bait = caught;
        if let Some(bait) = caught {
            if self.morph.check_frame(self.morph.get_end_frame() - 1.0)
                && calc_timer(&mut self.timer) == 0
            {
                ctx.despawns.push(bait);
                self.bait = None;
                ctx.cut_end();
            }
        } else {
            ctx.cut_end();
        }
    }

    fn cut_kill_all_start(&mut self, ctx: &mut Ctx<'_>) {
        let mut kills = Vec::new();
        Self::search_kill_nz_bomb(&ctx.hits, &mut kills);
        self.search_tsubo_barrel(&ctx.hits, self.base_pos, &mut kills);
        let mut seed = self.seed;
        let moves = self.search_esa_init(&ctx.hits, self.base_pos, self.home_angle[1], &mut seed);
        self.seed = seed;
        ctx.kills.append(&mut kills);
        ctx.moves.extend(moves);
        let mut pos = self.base_pos;
        let r5 = self.home_angle[1].wrapping_add(HOME_HALF_TURN);
        pos.x += ssin(self.home_angle[1]) * HOME_KILL_DIST;
        pos.z += scos(self.home_angle[1]) * HOME_KILL_DIST;
        ctx.player_move = Some((pos, r5));
    }

    fn cut_kill_all_proc(&mut self, ctx: &mut Ctx<'_>) {
        ctx.cut_end();
    }

    fn cut_show_proc(&mut self, ctx: &mut Ctx<'_>) {
        ctx.cut_end();
    }

    fn cut_hide_proc(&mut self, ctx: &mut Ctx<'_>) {
        ctx.cut_end();
    }

    fn cut_set_anm_start(&mut self, cut: usize, ctx: &mut Ctx<'_>) {
        let anm_no = ctx.list.cut_number(cut, CUT_KEY_ANM_NO).unwrap_or(0);
        self.timer = ctx.list.cut_number(cut, CUT_KEY_LOOP).unwrap_or(0) as i16;
        self.set_anm(anm_no as i8, false);
        if anm_no == 8 {
            ctx.se(SE_TALK_IN);
        }
    }

    fn cut_set_anm_proc(&mut self, ctx: &mut Ctx<'_>) {
        if self.morph.check_frame(self.morph.get_end_frame() - 1.0) {
            self.timer = self.timer.wrapping_sub(1);
        }
        if self.timer <= 0 {
            ctx.cut_end();
        }
        if self.prm_idx == PRM_KOKERU {
            if self.morph.check_frame(SMOKE_FRAME_A) {
                self.set_smoke_particle(ctx);
            } else if self.morph.check_frame(SMOKE_FRAME_B) {
                self.smoke_puff(self.shape_angle, ctx.actor_angle, ctx);
            }
        }
    }

    fn cut_go_home_start(&mut self) {}

    fn cut_go_home_proc(&mut self, ctx: &mut Ctx<'_>) {
        let target = target_angle_y(ctx.actor_pos, self.home_pos);
        add_calc_angle_s2(&mut ctx.actor_angle[1], target, GO_HOME_TURN_SCALE, GO_HOME_TURN_MAX);
        self.shape_angle[1] = ctx.actor_angle[1];
        let dist = distance_angle_s(target, ctx.actor_angle[1]);
        if dist < GO_HOME_NEAR {
            if self.prm_idx != PRM_RUN {
                ctx.se(SE_TALK_OUT);
            }
            self.set_anm(PRM_RUN, false);
            if add_calc_pos_xz(&mut ctx.actor_pos, self.home_pos, 0.5, 10.0, 1.0) < 1.0 {
                ctx.cut_end();
            }
        }
    }

    fn cut_chk_get_item_start(&mut self, ctx: &mut Ctx<'_>) {
        if self.order >= ORDER_BOUGHT {
            ctx.order_change = Some((self.ordered_player, self.event_esa));
        }
    }

    fn cut_chk_get_item_proc(&mut self, ctx: &mut Ctx<'_>) {
        ctx.cut_end();
    }

    pub fn execute(&mut self, actor: &mut Actor, statics: &Statics, ctx: &mut Ctx<'_>) {
        add_calc2(&mut self.speed_f, SPEED_TARGET, SPEED_SCALE, SPEED_MAX);
        self.check_order(ctx.talk_command, ctx.player);
        self.mode_proc(PROC_EXEC, MODE_EVENT_ESA, ctx);
        self.look_back(&mut actor.angle, ctx.player_eye, statics);
        self.event_order();
        let mut base = Mat::none();
        base.trans_s(actor.pos);
        base.zxyrot_m(self.shape_angle);
        base.scale(self.scale.x);
        self.eye_pos = base.pos(joint_origin(ACTOR_HEAD));
        self.tail_bind = [
            base.pos(joint_origin(ACTOR_HIP)),
            base.pos(joint_origin(ACTOR_SHIPPO)),
        ];
        self.set_attention(actor.pos);
        self.morph.play();
        self.tail_control();
        self.bait = None;
        self.bait_dist = FLOAT_MAX;
        self.set_mtx(actor.pos, self.shape_angle, self.scale);
    }
}
#[derive(Clone)]
pub struct Hit {
    pub entity: Entity,
    pub name: String,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub esa_free: bool,
    pub pos_init: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmokeOp {
    Start,
    Stop,
    Puff,
}

pub struct Text<'a> {
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub one: &'a mut Msg1Box,
    pub two: &'a mut Msg2Box,
    pub send: u8,
}

impl Text<'_> {
    fn proc_name(&self) -> Option<u16> {
        if self.procs.search_by_name(FPC_NM_MSG).is_some() {
            Some(FPC_NM_MSG)
        } else if self.procs.search_by_name(FPC_NM_MSG2).is_some() {
            Some(FPC_NM_MSG2)
        } else {
            None
        }
    }

    pub fn status(&self) -> Option<u8> {
        match self.proc_name() {
            Some(FPC_NM_MSG) => Some(self.one.status),
            Some(FPC_NM_MSG2) => Some(self.two.status),
            _ => None,
        }
    }

    pub fn set_status(&mut self, status: u8) {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.status = status,
            Some(FPC_NM_MSG2) => self.two.status = status,
            _ => {}
        }
    }

    pub fn msg_no(&self) -> u32 {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.msg_no,
            Some(FPC_NM_MSG2) => self.two.msg_no,
            _ => 0,
        }
    }

    pub fn set_msg_no(&mut self, msg_no: u32) {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.msg_no = msg_no,
            Some(FPC_NM_MSG2) => self.two.msg_no = msg_no,
            _ => {}
        }
    }

    pub fn select_num(&self) -> u8 {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.select_num,
            _ => 0,
        }
    }

    pub fn send_button(&self) -> bool {
        self.send != 0
    }

    pub fn message_rupee(&self) -> i16 {
        self.info.message_rupee
    }

}

pub struct Ctx<'a> {
    pub player: Entity,
    pub player_pos: Option<Vec3>,
    pub player_yaw: i16,
    pub player_eye: Option<Vec3>,
    pub actor_pos: Vec3,
    pub actor_angle: [i16; 3],
    pub actor_room: i8,
    pub talk_command: bool,
    pub select_item: u8,
    pub held_item: u8,
    pub held_count: u8,
    pub rupees: &'a mut Rupees,
    pub equipped: &'a mut Equipped,
    pub collect: &'a Collect,
    pub stories: &'a StoryBits,
    pub bomb_num: i32,
    pub bomb_max: i32,
    pub arrow_num: i32,
    pub arrow_max: i32,
    pub active: &'a mut ActiveEvent,
    pub list: &'a EventList,
    pub ev_flags: &'a mut EventFlags,
    pub events: &'a mut super::events::SimEvents,
    pub text: &'a mut Text<'a>,
    pub hits: &'a [Hit],
    pub m_event_cut: bool,
    pub event_reset: bool,
    pub event_esa: usize,
    pub staff_nz: usize,
    pub spawn_items: Vec<(u8, Vec3, [i16; 3], i8)>,
    pub despawns: Vec<Entity>,
    pub kills: Vec<Entity>,
    pub moves: Vec<(Entity, Vec3)>,
    pub pulls: Vec<(Entity, Vec3)>,
    pub player_move: Option<(Vec3, i16)>,
    pub smoke: Vec<SmokeOp>,
    pub give_item: Option<u8>,
    pub item_rupee_count: i16,
    pub gt_item: Option<u8>,
    pub order_change: Option<(Option<Entity>, i16)>,
}

impl Ctx<'_> {
    pub fn staff_of(&self, name: &str) -> usize {
        self.list
            .staff_of(name)
            .unwrap_or(usize::MAX)
    }

    pub fn end_check(&self, index: usize) -> bool {
        self.active.end_check(index)
    }

    pub fn event_index(&self, name: &str) -> usize {
        self.list.event_index(name).unwrap_or(usize::MAX)
    }

    pub fn cut_end(&mut self) {
        self.active.end_staff_cut(self.list, self.ev_flags, STAFF_NZ);
    }

    pub fn se(&mut self, id: u32) {
        self.events.list.push(super::events::SimEvent::Se(id));
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    pub actors: Query<'w, 's, (Entity, &'static Actor, &'static SItem), (Without<NpcNz>, Without<Player>)>,
}

#[derive(SystemParam)]
pub struct World<'w, 's> {
    pub statics: Res<'w, Statics>,
    pub rupees: ResMut<'w, Rupees>,
    pub equipped: ResMut<'w, Equipped>,
    pub collect: Res<'w, Collect>,
    pub stories: Res<'w, StoryBits>,
    pub held: Res<'w, Held>,
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub ev_flags: ResMut<'w, EventFlags>,
    pub events: ResMut<'w, super::events::SimEvents>,
    pub players: Query<'w, 's, (Entity, &'static Player)>,
    pub collision: Option<Res<'w, Collision>>,
}

#[derive(SystemParam)]
pub struct Words<'w> {
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
}

#[derive(SystemParam)]
pub struct Moves<'w, 's> {
    pub actors: Query<'w, 's, (Entity, &'static mut Actor), (Without<NpcNz>, Without<Player>)>,
    pub player: Query<
        'w,
        's,
        (Entity, &'static mut Actor, &'static mut Player),
        (With<Player>, Without<NpcNz>),
    >,
}

pub fn control(
    mut world: World<'_, '_>,
    watch: Watch<'_, '_>,
    mut words: Words<'_>,
    mut moves: Moves<'_, '_>,
    do_intent: Res<DoIntent>,
    mut commands: Commands,
    mut rats: Query<(Entity, &mut Actor, &mut NpcNz), (With<NpcNz>, Without<Player>)>,
) {
    let mut granted = None;
    let mut ordered = None;
    let mut reset_event = false;
    let mut reset_index = usize::MAX;
    let mut hits: Vec<Hit> = Vec::new();
    for (entity, actor, item) in watch.actors.iter() {
        if item.item_no != ITEM_NONE {
            continue;
        }
        let name = actor.name.clone();
        if name != ACTOR_ESA
            && name != ACTOR_NZ
            && name != ACTOR_BOMB
            && name != ACTOR_TSUBO
            && name != ACTOR_BARREL
        {
            continue;
        }
        hits.push(Hit {
            entity,
            name,
            pos: actor.pos,
            angle: actor.angle,
            esa_free: true,
            pos_init: false,
        });
    }
    let Some((player_entity, player)) = world.players.iter().next().map(|(e, p)| (e, *p)) else {
        return;
    };
    let player_yaw = to_s16(player.yaw);
    let player_eye = Vec3::new(player.pos.x, player.pos.y + PLAYER_HEAD_TOP, player.pos.z);
    let held = *world.held;
    let arrow_num = i32::from(held.count);
    let bomb_num = i32::from(held.count);
    let arrow_max = if world.equipped.has(SLOT_BOW) {
        ARROW_CAP
    } else {
        0
    };
    let bomb_max = if world.equipped.has(SLOT_BOMB) {
        BOMB_CAP
    } else {
        0
    };
    let statics = *world.statics;
    let ctx_range = TALK_RANGE;
    let send_flag = words.one.m116a;
    let do_now = do_intent.0;
    let esa_event = world.list.event_index(EVENT_ESA).unwrap_or(usize::MAX);
    let staff_nz = world.list.staff_of(STAFF_NZ).unwrap_or(usize::MAX);
    for (_entity, mut actor, mut rat) in &mut rats {
        let tap = do_now && !rat.do_held;
        rat.do_held = do_now;
        let mut text = Text {
            procs: &mut words.procs,
            info: &mut words.info,
            one: &mut words.one,
            two: &mut words.two,
            send: send_flag,
        };
        let mut ctx = Ctx {
            player: player_entity,
            player_pos: Some(player.pos),
            player_yaw,
            player_eye: Some(player_eye),
            actor_pos: actor.pos,
            actor_angle: actor.angle,
            actor_room: actor.room,
            talk_command: tap && in_talk_range(ctx_range, actor.pos, player.pos),
            select_item: held.item,
            held_item: held.item,
            held_count: held.count,
            rupees: &mut world.rupees,
            equipped: &mut world.equipped,
            collect: &world.collect,
            stories: &world.stories,
            bomb_num,
            bomb_max,
            arrow_num,
            arrow_max,
            active: &mut world.active,
            list: &world.list,
            ev_flags: &mut world.ev_flags,
            events: &mut world.events,
            text: &mut text,
            hits: &hits,
            m_event_cut: false,
            event_reset: false,
            event_esa: esa_event,
            staff_nz,
            spawn_items: Vec::new(),
            despawns: Vec::new(),
            kills: Vec::new(),
            moves: Vec::new(),
            pulls: Vec::new(),
            player_move: None,
            smoke: Vec::new(),
            give_item: None,
            item_rupee_count: 0,
            gt_item: None,
            order_change: None,
        };
        if !rat.entered {
            rat.create_init(&mut actor);
        }
        rat.event_esa = rat.xy_event(&world.list);
        rat.staff_id = i32::try_from(ctx.staff_of(STAFF_NZ)).unwrap_or(-1);
        rat.ordered_player = Some(player_entity);
        rat.xy_ok = rat.xy_check(held.item, Some(player.pos), player_yaw, &statics);
        rat.execute(&mut actor, &statics, &mut ctx);
        actor.pos = ctx.actor_pos;
        actor.angle = ctx.actor_angle;
        if let Some(y) = ground::surface(world.collision.as_deref(), actor.pos) {
            actor.pos.y = y;
        }
        if ctx.text.status().is_some() {
            if ctx.text.status() == Some(STATUS_MSG_DISPLAYED) {
                let mut msg_no = ctx.text.msg_no();
                let status = rat.next_msg_status(&mut msg_no, &mut ctx);
                rat.msg_status = status;
                rat.msg_no = msg_no;
                ctx.text.set_msg_no(msg_no);
                ctx.text.set_status(status);
            }
            rat.anm_atr(&mut ctx);
        }
        for (item, pos, angle, room) in ctx.spawn_items {
            let id = commands
                .spawn((
                    Actor {
                        name: SHOP_ITEM_ACTOR.to_owned(),
                        parameters: u32::from(item),
                        pos,
                        angle,
                        set_id: u16::MAX,
                        room,
                    },
                    SItem::spawn(u32::from(item)),
                ))
                .id();
            if rat.shop[0].entity.is_none() {
                rat.shop[0].entity = Some(id);
            } else {
                rat.shop[1].entity = Some(id);
            }
        }
        for target in ctx.despawns.into_iter().chain(ctx.kills) {
            commands.entity(target).despawn();
        }
        for (target, pos) in ctx.moves.into_iter().chain(ctx.pulls) {
            if let Ok((_, mut other)) = moves.actors.get_mut(target) {
                other.pos = pos;
            }
        }
        if let Some((pos, yaw)) = ctx.player_move
            && let Ok((_, mut link, mut player)) = moves.player.get_mut(player_entity)
        {
            link.pos = pos;
            player.pos = pos;
            player.yaw = from_s16(yaw);
        }
        rat.item_rupee_count = ctx.item_rupee_count;
        if let Some(item) = ctx.gt_item {
            rat.given_item = item;
        }
        for op in ctx.smoke {
            rat.smoke_state = op;
        }
        granted = ctx.give_item;
        ordered = ctx.order_change;
        reset_event = ctx.event_reset;
        reset_index = ctx.event_esa;
    }
    if let Some(item) = granted
        && let Some(slot) = super::inventory::slot_of(item)
    {
        world.equipped.set(slot, item);
    }
    if granted.is_some() {
        world
            .events
            .list
            .push(super::events::SimEvent::Collect);
    }
    if let Some((player, event)) = ordered
        && let Ok(index) = usize::try_from(event)
    {
        world
            .active
            .order(&world.list, index, player.unwrap_or(player_entity));
    }
    if reset_event {
        world.active.end(reset_index);
    }
}
