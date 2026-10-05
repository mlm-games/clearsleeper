use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};

use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use crate::text::msgproc::MsgGameInfo;
use super::actor::Actor;
use super::bomb;
use super::chest::Chests;
use super::clock::Clock;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{Charts, Collect, MemBits, StoryBits, Switches};
use super::health::Health;
use super::inventory::{self, Equipped};
use super::item::{self, Held};
use super::makar::Makar;
use super::medli::Medli;
use super::pickup::Rupees;
use super::player::Player;
use super::stream::{cos_s, sin_s};
use super::swim::Swim;
use super::walk::to_s16;

pub const AGBSW0_ACTORS: [&str; 13] = [
    "agbA",
    "agbAT",
    "agbA2",
    "agbF",
    "agbF2",
    "agbMARK",
    "agbMW",
    "agbCSW",
    "agbTBOX",
    "agbR",
    "agbB",
    "agbD",
    "agbFA",
];

pub fn is_agbsw0(name: &str) -> bool {
    AGBSW0_ACTORS.contains(&name)
}

const PRM_MSG_SHIFT: u32 = 0;
const PRM_MSG_WIDTH: u32 = 16;
const PRM_MSG_MASK: u32 = (1 << PRM_MSG_WIDTH) - 1;
const PRM_SW0_SHIFT: u32 = 16;
const PRM_SW0_MASK: u32 = 0xFF;
const PRM_SW1_SHIFT: u32 = 24;
const PRM_SW1_MASK: u32 = 0xFF;

const NO_SWITCH: u8 = 0xFF;
const NO_ROOM: i8 = -1;
const NO_MSG: u16 = 0xFFFF;
const ITEM_HEART: i32 = 0;
const ITEM_BOMB_BAG: u8 = 0x31;
const ITEM_RECOVER_FAIRY: u8 = 0x16;
const ITEM_SMALL_KEY: u8 = 0x15;
const ITEM_HEART_PIECE: u8 = 0x07;
const ITEM_HEART_CONTAINER: u8 = 0x08;
const ITEM_TRIPLE_HEART: u8 = 0x1E;
const ITEM_HOOKSHOT: u8 = 0x2F;
const ITEM_IRON_BOOTS: u8 = 0x29;
const ITEM_MAGIC_ARMOR: u8 = 0x2A;
const ITEM_BAIT_BAG: u8 = 0x2C;
const ITEM_BOOMERANG: u8 = 0x2D;
const ITEM_BOW: u8 = 0x27;
const ITEM_GRAPPLE_HOOK: u8 = 0x25;
const ITEM_SKULL_HAMMER: u8 = 0x33;
const ITEM_DEKU_LEAF: u8 = 0x34;
const ITEM_MAGIC_ARROW: u8 = 0x35;
const ITEM_MASTER_SWORD_2: u8 = 0x3A;
const ITEM_MIRROR_SHIELD: u8 = 0x3C;
const ITEM_MASTER_SWORD_3: u8 = 0x3E;
const ITEM_HYOI_PEAR: u8 = 0x83;
const ITEM_JOY_PENDANT: u8 = 0x1F;
const ITEM_CABANA_DEED: u8 = 0x9C;
const ITEM_COUNT: i32 = 0x1F;

const TYPE_A: u8 = 0x00;
const TYPE_AT: u8 = 0x01;
const TYPE_M: u8 = 0x02;
const TYPE_A2: u8 = 0x03;
const TYPE_F2: u8 = 0x04;
const TYPE_F: u8 = 0x05;
const TYPE_T: u8 = 0x06;
const TYPE_MW: u8 = 0x07;
const TYPE_S: u8 = 0x08;
const TYPE_R: u8 = 0x09;
const TYPE_B: u8 = 0x0A;
const TYPE_D: u8 = 0x0B;
const TYPE_FA: u8 = 0x0C;
const TYPE_UNK_0D: u8 = 0x0D;
const TYPE_UNK_0E: u8 = 0x0E;

const AT_TYPE_BOMB: u32 = 1 << 5;

const BIT_MET_KORL: u16 = 0x0F80;
const BIT_UNK_1708: u16 = 0x1708;
const BIT_UNK_1820: u16 = 0x1820;
const BIT_UNLOCK_TING_DISCOUNT: u16 = 0x1A08;
const BIT_UNLOCK_TINGLE_BALLOON_DISCOUNT: u16 = 0x1A10;
const BIT_UNK_1E40: u16 = 0x1E40;
const BIT_UNK_2D01: u16 = 0x2D01;
const BIT_UNK_2E08: u16 = 0x2E08;
const BIT_UNK_3E02: u16 = 0x3E02;
const BIT_UNK_3E80: u16 = 0x3E80;

const SYMBOL_NAYRU: u8 = 0;
const SYMBOL_DIN: u8 = 1;

const TACT_NONE: u8 = 0;
const TACT_KORL: u8 = 1;

const COLLECT_TING: u8 = 2;
const COLLECT_TING_ITEM: u8 = 0;

const SAVE_STAGE_CURRENT: u8 = 0;
const SAVE_STAGE_DRC: u8 = 3;
const SAVE_STAGE_FW: u8 = 4;
const SAVE_STAGE_TOTG: u8 = 5;
const SAVE_STAGE_ET: u8 = 6;
const SAVE_STAGE_WT: u8 = 7;
const SAVE_TBOX: u8 = 0x0F;

const FPC_PLAYER: u16 = 0x00A9;
const FPC_NPC_KAM: u16 = 0x00C4;
const FPC_NPC_OS: u16 = 0x013C;
const FPC_NPC_CB1: u16 = 0x0150;
const FPC_NPC_MD: u16 = 0x0171;

const SCALE_S: f32 = 8000.0;
const SCALE_OTHER: f32 = 200.0;
const CYL_RADIUS_SRC: f32 = 100.0;
const CYL_HEIGHT_SRC: f32 = 100.0;

const HIT_Y_MIN: f32 = -10.0;
const HIT_PLAYER_RADIUS: f32 = 60.0;
const HIT_AGB_MOVE_GAP: f32 = 6.0;
const HIT_AGB_MOVE_LIFT: f32 = 5.0;
const HIT_AGB_INSIDE_MARGIN: f32 = 100.0;

const PLAYER_LIFT: f32 = 50.0;
const BAIT_SLOTS: usize = 4;
const NO_BAIT: u8 = 0xFF;

const TIMER_SCALE: u32 = 30;
const TIMER_MASK: u32 = 0xFFFF;
const TIMER_TRIGGER: u32 = 30;

const RESTRICTION_TBOX: i16 = 2;
const RESTRICTION_BOMB_DENY: i16 = 3;
const RESTRICTION_FIGURE: i16 = 4;
const RESTRICTION_DENY_MSG: i16 = 5;
const RESTRICTION_CURSOR: i16 = 6;

const PARAM_NO_M_START: i16 = 0x1A;
const PARAM_NO_M_END: i16 = 0x22;
const PARAM_NO_M_OFFSET: i16 = 0x19;
const PARAM_NO_B_DEFAULT_MSG: u32 = 0xE;
const PARAM_NO_B_LOW: i16 = -3;
const PARAM_NO_ZERO: i16 = 0;
const PARAM_NO_RD_MAX: i16 = 0x1E;
const PARAM_NO_RD_SEVEN: i16 = 7;
const PARAM_NO_RD_EIGHT: i16 = 8;
const PARAM_NO_RD_HEART_PIECE: i16 = 0x15;
const TBOX_RANGE: u8 = 0x20;

const DISPOSE_KEYS: usize = 2;
const BEAT_KEYS: usize = 6;
const BEAT_MSG: [u16; BEAT_KEYS] = [0x0005, 0x0011, 0x0012, 0x0031, 0x004F, 0x0058];
const DISPOSED_MSG: [u16; DISPOSE_KEYS] = [0x0059, 0x005C];

const ROOM_M_NEW_D2: i8 = 0xC;
const TBOX_M_NEW_D2: u8 = 0xC;
const STAGE_M_NEW_D2: &str = "M_NewD2";

const ISLE_ROOM_OVERLOOK: i32 = 0x07;
const ISLE_ROOM_NEEDLE_ROCK: i32 = 0x1D;
const ISLE_ROOM_STONE_WATCHER: i32 = 0x1F;
const ISLE_ROOM_PRIVATE_OASIS: i32 = 0x21;
const ISLE_ROOM_BIRDS_PEAK: i32 = 0x23;
const ISLE_ROOM_OUTSET: i32 = 0x2C;

const MOON_DEFAULT_ROOM: i32 = 0x05;
const MOON_SPECTACLE_ROOM: i32 = 0x0A;
const MOON_STAR_BELT_ROOM: i32 = 0x15;
const MOON_DIAMOND_ROOM: i32 = 0x24;
const MOON_THREE_EYE_ROOM: i32 = 0x22;
const MOON_NEEDLE_ROOM: i32 = 0x31;
const MOON_BIRDS_PEAK_ROOM: i32 = 0x17;

const MOON_CRESCENT: u32 = 1;
const MOON_STAR_ISLES: u32 = 2;
const MOON_SPECTACLE_ISLE: u32 = 3;
const MOON_NEEDLE_ROCK: u32 = 4;
const MOON_BIRDS_PEAK: u32 = 5;
const MOON_GREAT_FISH: u32 = 6;
const MOON_DEFAULT: u32 = 0;

const SWITCH_7C: u8 = 0x7C;
const RUPEE_BUY_MIN: u32 = 398;
const RUPEE_DONATE_MIN: u32 = 10;
const RUPEE_KEY_MIN: u32 = 500;

const TRIFORCE_ALL: u8 = 8;
const TRIFORCE_SLOTS: i32 = 8;
const CHART_TRIFORCE_SLOT: i32 = 0x24;
const CHART_FIRST: i32 = 1;
const CHART_LAST: i32 = 8;

const HOUR_DAWN: i32 = 6;
const HOUR_NIGHT_END: i32 = 0x13;

const MAGIC_LIMIT: i32 = 0;
const LIFE_CONTAINER: i32 = 8;

const OCEAN_SV_BIT: u16 = 0xF;
const OCEAN_GRIDS: usize = 50;

const TRIFORCE_RND_SPAN: f32 = 3.0;
const TRIFORCE_RND_LOW: f32 = 1.0;
const TRIFORCE_RND_MID: f32 = 2.0;

const JOY_PENDANT_FULL: u8 = 0x14;

const EVT_FLAG_TALK: u32 = 0x0000_0004;
const EVENT_ATTENTION: &str = "DEFAULT_AGB_LOOK_ATTENTION";
const EVENT_STAFF: &str = "AGB_SW0";
const ITEM_BUTTON_COUNT: usize = 3;
const ARROW_CAP: i32 = 30;
const BOMB_CAP: i32 = 30;
const EVENT_ATTN_DIST_EPS: f32 = 0.001;
const EVENT_ATTN_NUDGE: f32 = 10.0;

const SE_CHI_MEGAHORN: u32 = 0x484C;
const STATUS_CHANNEL_MAIL: u8 = 0x5;
const STATUS_CHANNEL_BOMB: u8 = 0xF;
const MAIL_SE_BOMB: u32 = 0x12;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const CULL_FLOOR: f32 = -10.0;

const MAIL_REACT_NONE: u8 = 0;
const MAIL_SFX_NONE: u8 = 0;
const MAIL_NONE: u16 = NO_MSG;
const GBA_LINK: bool = false;
const AGB_PRESENT: bool = false;
const NO_SET_ID: u16 = 0xFFFF;

const SPAWN_ITEM_ACTOR: &str = "item";

static MAIL_WORD: AtomicU64 = AtomicU64::new(0);
static MAIL_SE: AtomicU32 = AtomicU32::new(0);
static FIGURE_DISPOSE: AtomicU8 = AtomicU8::new(0);
static FIGURE_BEAT: AtomicU8 = AtomicU8::new(0);
static AGB_FLAG_3_5: AtomicU8 = AtomicU8::new(0);
static MW_SE_FLAG: AtomicU8 = AtomicU8::new(0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Agb {
    pub present: bool,
    pub active: bool,
    pub free: bool,
    pub held: bool,
    pub follow_target: u8,
    pub room: i32,
    pub pos: Vec3,
    pub home: Vec3,
    pub shape_angle_x: i16,
    pub deny_message: u32,
    pub cursor_message: u32,
    pub bomb_deny: bool,
    pub flag_0x67b: bool,
    pub flag_0x67d: bool,
    pub flag_0x67f: bool,
    pub flag_0x675: bool,
    pub flag_0x676: bool,
    pub reset_cursor: bool,
    pub on_free: bool,
    pub on_hold: bool,
}

impl Agb {
    pub fn new() -> Self {
        Self {
            present: true,
            active: false,
            free: false,
            held: false,
            follow_target: 0,
            room: 0,
            pos: Vec3::ZERO,
            home: Vec3::ZERO,
            shape_angle_x: 0,
            deny_message: 0,
            cursor_message: 0,
            bomb_deny: false,
            flag_0x67b: false,
            flag_0x67d: false,
            flag_0x67f: false,
            flag_0x675: false,
            flag_0x676: false,
            reset_cursor: false,
            on_free: false,
            on_hold: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_free(&self) -> bool {
        self.free
    }

    pub fn get_follow_target(&self) -> u8 {
        self.follow_target
    }

    pub fn on_free(&mut self) {
        self.on_free = true;
        self.held = false;
    }

    pub fn on_hold(&mut self) {
        self.on_hold = true;
        self.held = true;
    }

    pub fn reset_cursor(&mut self, deny: bool) {
        self.reset_cursor = deny;
    }

    pub fn on_bomb_deny(&mut self) {
        self.bomb_deny = true;
    }

    pub fn off_bomb_deny(&mut self) {
        self.bomb_deny = false;
    }

    pub fn set_deny_message(&mut self, message: u32) {
        self.deny_message = message;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spawn {
    pub pos: Vec3,
    pub room: i8,
    pub item: u8,
}

#[derive(Debug, Clone)]
pub struct Facts {
    pub stories: StoryBits,
    pub collect: Collect,
    pub equipped: Equipped,
    pub charts: Charts,
    pub chests: Chests,
    pub switches: Switches,
    pub mem: MemBits,
    pub rupees: u32,
    pub life: i32,
    pub max_life: i32,
    pub max_magic: i32,
    pub select_item: [u8; 3],
    pub arrow_num: i32,
    pub arrow_max: i32,
    pub bomb_num: i32,
    pub bomb_max: i32,
    pub key_num: i32,
    pub hour: i32,
    pub moon: u32,
    pub swim: bool,
    pub ship_ride: bool,
    pub winds_dir: i32,
    pub machete: bool,
    pub curse: bool,
    pub confuse: bool,
    pub bait: [u8; 4],
    pub ghostship_aboard: bool,
    pub player: u16,
    pub os_room: [bool; 3],
    pub md_room: bool,
    pub md_flying: bool,
    pub cb1_room: bool,
    pub cb1_flying: bool,
    pub ocean_sv: [u16; OCEAN_GRIDS],
    pub stage_m_new_d2: bool,
    pub agb_room: i32,
    pub demo_accept: bool,
    pub event_running: bool,
    pub look_attention: Option<usize>,
    pub agb_staff: Option<usize>,
    pub bombs: Vec<BombView>,
    pub player_pos: Option<Vec3>,
    pub player_yaw: i16,
}

impl Default for Facts {
    fn default() -> Self {
        Self {
            stories: StoryBits::default(),
            collect: Collect::default(),
            equipped: Equipped::default(),
            charts: Charts::default(),
            chests: Chests::default(),
            switches: Switches::default(),
            mem: MemBits::default(),
            rupees: 0,
            life: 0,
            max_life: 0,
            max_magic: 0,
            select_item: [inventory::NO_ITEM; 3],
            arrow_num: 0,
            arrow_max: 0,
            bomb_num: 0,
            bomb_max: 0,
            key_num: 0,
            hour: 0,
            moon: 0,
            swim: false,
            ship_ride: false,
            winds_dir: 0,
            machete: false,
            curse: false,
            confuse: false,
            bait: [NO_BAIT; BAIT_SLOTS],
            ghostship_aboard: false,
            player: FPC_PLAYER,
            os_room: [false; 3],
            md_room: false,
            md_flying: false,
            cb1_room: false,
            cb1_flying: false,
            ocean_sv: [0; OCEAN_GRIDS],
            stage_m_new_d2: false,
            agb_room: 0,
            demo_accept: false,
            event_running: false,
            look_attention: None,
            agb_staff: None,
            bombs: Vec::new(),
            player_pos: None,
            player_yaw: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BombView {
    pub pos: Vec3,
    pub mode: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cyl {
    pub centre: Vec3,
    pub radius: f32,
    pub height: f32,
    pub at_set: bool,
    pub at_atp: u8,
    pub tg_type: u32,
    pub tg_set: bool,
    pub tg_hit: bool,
    pub co_set: bool,
}

impl Cyl {
    const SRC: Cyl = Cyl {
        centre: Vec3::ZERO,
        radius: CYL_RADIUS_SRC,
        height: CYL_HEIGHT_SRC,
        at_set: false,
        at_atp: 0,
        tg_type: AT_TYPE_BOMB,
        tg_set: true,
        tg_hit: false,
        co_set: false,
    };

    fn touches(&self, at: Vec3) -> bool {
        let d = at - self.centre;
        let r = (d.x * d.x + d.z * d.z).sqrt();
        r < self.radius && d.y >= 0.0 && d.y <= self.height
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Agbsw0 {
    pub kind: u8,
    pub param_no: i16,
    pub msg_no: u16,
    pub sw0: u8,
    pub sw1: u8,
    pub entered: bool,
    pub orig_scale_x: f32,
    pub orig_scale_z: f32,
    pub field_0x298: u8,
    pub field_0x299: u8,
    pub non_circular: bool,
    pub field_0x29b: u8,
    pub timer: u32,
    pub scale: Vec3,
    pub pos: Vec3,
    pub room: i8,
    pub stage_no: u8,
    pub shape_angle_y: i16,
    pub eye_pos: Vec3,
    pub cyl: Cyl,
    pub seed: [i32; 3],
}

impl Agbsw0 {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: TYPE_A,
            param_no: 0,
            msg_no: ((parameters >> PRM_MSG_SHIFT) & PRM_MSG_MASK) as u16,
            sw0: ((parameters >> PRM_SW0_SHIFT) & PRM_SW0_MASK) as u8,
            sw1: ((parameters >> PRM_SW1_SHIFT) & PRM_SW1_MASK) as u8,
            entered: false,
            orig_scale_x: 0.0,
            orig_scale_z: 0.0,
            field_0x298: 0,
            field_0x299: 0,
            non_circular: false,
            field_0x29b: 0,
            timer: 0,
            scale: Vec3::ONE,
            pos: Vec3::ZERO,
            room: NO_ROOM,
            stage_no: 0,
            shape_angle_y: 0,
            eye_pos: Vec3::ZERO,
            cyl: Cyl::SRC,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (
            [-self.cyl.radius, CULL_FLOOR, -self.cyl.radius],
            [self.cyl.radius, self.scale.y, self.cyl.radius],
        )
    }

    pub fn drawn(&self) -> bool {
        true
    }

    fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane].wrapping_mul(RND_MUL[lane])) % RND_MOD[lane];
        }
        let sum = self.seed[0] as f32 / RND_SCALE[0]
            + self.seed[1] as f32 / RND_SCALE[1]
            + self.seed[2] as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    fn rnd_span(&mut self, range: f32) -> f32 {
        range * self.rnd()
    }

    fn gba_link(&self) -> bool {
        GBA_LINK
    }

    fn mail_send(&mut self, msg_no: u16, react_type: u8, to_check: u8, to_set: u8, sfx: u8) {
        let packed = u64::from(msg_no)
            | (u64::from(to_set) << 16)
            | (u64::from(self.stage_no) << 24)
            | (u64::from(self.room as u8) << 32)
            | (u64::from(react_type) << 40)
            | (u64::from(to_check) << 48)
            | (u64::from(sfx) << 56);
        MAIL_WORD.store(packed, Ordering::Relaxed);
    }

    fn mail_cancel(&mut self) {
        self.mail_send(MAIL_NONE, MAIL_REACT_NONE, NO_SWITCH, NO_SWITCH, MAIL_SFX_NONE);
    }

    fn relative_pos(&self, abs: Vec3) -> Vec3 {
        let angle = self.shape_angle_y.wrapping_neg();
        let offset = abs - self.pos;
        Vec3::new(
            offset.z * sin_s(angle) + offset.x * cos_s(angle),
            offset.y,
            offset.z * cos_s(angle) - offset.x * sin_s(angle),
        )
    }

    pub fn hit_check(&self, abs: Vec3, radius: f32) -> bool {
        if !self.non_circular {
            let y_diff = abs.y - self.pos.y;
            if -radius <= y_diff && y_diff <= self.scale.y {
                let x_diff = (abs.x - self.pos.x).abs();
                if x_diff < self.scale.x {
                    let z_diff = (abs.z - self.pos.z).abs();
                    if z_diff < self.scale.x && x_diff * x_diff + z_diff * z_diff < self.scale.x * self.scale.x
                    {
                        return true;
                    }
                }
            }
        } else {
            let pos = self.relative_pos(abs);
            if -radius <= pos.y && pos.y <= self.scale.y && pos.x.abs() < self.scale.x && pos.z.abs() < self.scale.z {
                return true;
            }
        }
        false
    }

    pub fn hit_check_actor(&self, abs: Vec3) -> bool {
        self.hit_check(abs, HIT_Y_MIN.abs())
    }

    pub fn ctor(&mut self, actor: &mut Actor, mut scale: Vec3, f: &Facts) -> bool {
        let mut kind = actor.angle[2] as u32 as u8;
        let sw0 = self.sw0;
        let mut param_no = actor.angle[0];

        if kind == TYPE_R {
            if param_no < PARAM_NO_ZERO && param_no >= PARAM_NO_B_LOW {
                kind = TYPE_B;
                actor.angle[2] = i16::from(kind);
                param_no = -param_no;
                actor.angle[0] = param_no;
            }
        } else if kind == TYPE_M {
            if (PARAM_NO_M_START..PARAM_NO_M_END).contains(&param_no) {
                kind = TYPE_UNK_0D;
                actor.angle[2] = i16::from(kind);
                param_no -= PARAM_NO_M_OFFSET;
                actor.angle[0] = param_no;
            } else if param_no == PARAM_NO_M_END {
                kind = TYPE_UNK_0E;
                actor.angle[2] = i16::from(kind);
            }
        }

        self.kind = kind;
        self.param_no = param_no;

        if kind == TYPE_AT {
            if sw0 < TBOX_RANGE && actor.angle[0] != PARAM_NO_ZERO && f.chests.is_open(SAVE_STAGE_CURRENT, sw0) {
                return false;
            }
        } else if kind == TYPE_B {
            if ((param_no == 1 || param_no == 3) && sw0 != NO_SWITCH && f.switches.test(sw0))
                || (param_no == RESTRICTION_TBOX
                    && sw0 < TBOX_RANGE
                    && f.chests.is_open(SAVE_STAGE_CURRENT, sw0))
            {
                return false;
            }
        } else if kind == TYPE_D {
            let sw1 = self.sw1;
            if (sw1 == NO_SWITCH || (sw0 != NO_SWITCH && f.switches.test(sw0)))
                || (sw1 != NO_SWITCH && f.switches.test(sw1))
            {
                return false;
            }
        } else if kind == TYPE_T || kind == TYPE_S || kind == TYPE_R {
            if sw0 == NO_SWITCH || (sw0 != NO_SWITCH && f.switches.test(sw0)) {
                return false;
            }
        } else if kind == TYPE_UNK_0D {
            if f.stories.test(BIT_UNK_2D01) {
                return false;
            }
        } else if kind == TYPE_UNK_0E {
            if f.triforce_num() == TRIFORCE_ALL {
                return false;
            }
        } else if kind != TYPE_M && kind != TYPE_MW {
            if sw0 != NO_SWITCH && actor.angle[0] != PARAM_NO_ZERO && f.switches.test(sw0) {
                return false;
            }
        }

        if kind == TYPE_B && self.msg_no == NO_MSG {
            actor.parameters = (actor.parameters & !PRM_MSG_MASK) | PARAM_NO_B_DEFAULT_MSG;
            self.msg_no = PARAM_NO_B_DEFAULT_MSG as u16;
        }

        if kind != TYPE_MW
            && kind != TYPE_T
            && kind != TYPE_S
            && kind != TYPE_UNK_0E
            && self.msg_no == NO_MSG
        {
            return false;
        }

        if (kind == TYPE_R || kind == TYPE_D)
            && (param_no < PARAM_NO_ZERO
                || param_no > PARAM_NO_RD_MAX
                || param_no == PARAM_NO_RD_SEVEN
                || param_no == PARAM_NO_RD_EIGHT
                || param_no == PARAM_NO_RD_HEART_PIECE)
        {
            return false;
        }

        self.non_circular = scale.x != scale.z;

        if kind == TYPE_S {
            scale *= SCALE_S;
        } else {
            scale *= SCALE_OTHER;
        }
        self.scale = scale;

        self.orig_scale_x = scale.x;
        self.orig_scale_z = scale.z;
        self.shape_angle_y = actor.angle[1];
        self.pos = actor.pos;
        self.field_0x298 = 0;
        self.field_0x299 = 0;
        self.timer = 0;

        self.cyl.centre = actor.pos;
        self.cyl.radius = scale.x;
        self.cyl.height = scale.y;

        if kind == TYPE_S {
            if param_no > 0 {
                self.timer = (param_no as u32).wrapping_mul(TIMER_SCALE) & TIMER_MASK;
            }
        } else if kind == TYPE_B && param_no == RESTRICTION_FIGURE {
            self.field_0x299 = 1;
            self.timer = 1;
        }

        true
    }

    fn execute(&mut self, ctx: &mut Ctx<'_>) -> bool {
        self.cc_calc(ctx);
        match self.kind {
            TYPE_A => self.exe_sub_a(ctx),
            TYPE_AT => self.exe_sub_at(ctx),
            TYPE_M => self.exe_sub_m(ctx),
            TYPE_A2 => self.exe_sub_a2(ctx),
            TYPE_F2 => self.exe_sub_f2(ctx),
            TYPE_F => self.exe_sub_f(ctx),
            TYPE_T => self.exe_sub_t(ctx),
            TYPE_MW => self.exe_sub_mw(ctx),
            TYPE_S => self.exe_sub_s(ctx),
            TYPE_R => self.exe_sub_r(ctx),
            TYPE_B => self.exe_sub_b(ctx),
            TYPE_D => self.exe_sub_d(ctx),
            TYPE_FA => self.exe_sub_fa(ctx),
            TYPE_UNK_0D => self.exe_sub_m2(ctx),
            TYPE_UNK_0E => self.exe_sub_m3(ctx),
            _ => true,
        }
    }

    fn cc_calc(&mut self, ctx: &Ctx<'_>) {
        self.cyl.tg_hit = self.cyl.tg_set
            && ctx.f.bombs.iter().any(|shell| {
                shell.mode == bomb::MODE_EXPLODE && self.cyl.touches(shell.pos)
            });
    }

    fn release(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let mut agb = ctx.agb;
        let kind = self.kind;
        if let Some(agb) = agb.as_mut() {
            if kind == TYPE_B {
                if self.param_no == RESTRICTION_BOMB_DENY {
                    agb.off_bomb_deny();
                } else if self.param_no == RESTRICTION_FIGURE {
                    AGB_FLAG_3_5.store(0, Ordering::Relaxed);
                    FIGURE_DISPOSE.store(0, Ordering::Relaxed);
                    FIGURE_BEAT.store(0, Ordering::Relaxed);
                } else if self.param_no == RESTRICTION_DENY_MSG {
                    agb.flag_0x67d = false;
                } else {
                    agb.flag_0x67b = false;
                }
            } else if kind == TYPE_F2 {
                if self.field_0x299 == 1 {
                    agb.reset_cursor(false);
                }
            } else if kind == TYPE_D {
                agb.reset_cursor(true);
            }
        }

        if matches!(
            kind,
            TYPE_A | TYPE_AT | TYPE_M | TYPE_A2 | TYPE_F2 | TYPE_R
        ) || kind >= TYPE_FA
        {
            if self.gba_link() && self.field_0x298 == 1 {
                let packed = u64::from(NO_MSG)
                    | (u64::from(NO_SWITCH) << 16)
                    | (u64::from(self.stage_no) << 24)
                    | (u64::from(self.room as u8) << 32)
                    | (u64::from(MAIL_REACT_NONE) << 40)
                    | (u64::from(NO_SWITCH) << 48)
                    | (u64::from(MAIL_SFX_NONE) << 56);
                MAIL_WORD.store(packed, Ordering::Relaxed);
            }
        }
        true
    }

    fn exe_sub_a(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let sw0 = self.sw0;
        let condition_no = self.param_no;

        if sw0 != NO_SWITCH {
            if condition_no == 0 {
                if !ctx.f.switches.test(sw0) {
                    if self.field_0x298 == 1 {
                        if self.gba_link() {
                            if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                                return true;
                            }
                            self.mail_cancel();
                        }
                        self.field_0x298 = 0;
                    }
                    return true;
                }
            } else if ctx.f.switches.test(sw0) {
                if self.field_0x298 == 1 {
                    if self.gba_link() {
                        if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                            return true;
                        }
                        self.mail_cancel();
                    }
                    self.field_0x298 = 0;
                }
                self.kill(ctx);
                return true;
            }
        }

        if self.move_check(ctx.f) {
            if self.field_0x298 == 1 {
                if self.gba_link() {
                    if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                        return true;
                    }
                    self.mail_cancel();
                }
                self.field_0x298 = 0;
            }
            return true;
        }

        if self.gba_link() && self.gba_status_check(STATUS_CHANNEL_MAIL) {
            let hit = match ctx.agb.as_ref() {
                Some(agb) => {
                    (agb.is_free() || agb.get_follow_target() != 1) && self.hit_check_actor(agb.pos)
                }
                None => false,
            };
            if hit {
                let react = big_little_change(u32::from(self.msg_no)) >> 16;
                self.mail_send(react as u16, 1, self.sw1, NO_SWITCH, MAIL_SFX_NONE);
                self.field_0x298 = 1;
            } else if self.field_0x298 == 1 {
                self.mail_cancel();
                self.field_0x298 = 0;
            }
        }
        true
    }

    fn exe_sub_at(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let flag = self.sw0;
        let condition_no = self.param_no;

        if flag < TBOX_RANGE {
            if condition_no == 0 {
                if !ctx.f.chests.is_open(SAVE_STAGE_CURRENT, flag) {
                    if self.field_0x298 == 1 {
                        if self.gba_link() {
                            if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                                return true;
                            }
                            self.mail_cancel();
                        }
                        self.field_0x298 = 0;
                    }
                    return true;
                }
            } else if ctx.f.chests.is_open(SAVE_STAGE_CURRENT, flag) {
                if self.field_0x298 == 1 {
                    if self.gba_link() {
                        if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                            return true;
                        }
                        self.mail_cancel();
                    }
                    self.field_0x298 = 0;
                }
                self.kill(ctx);
                return true;
            }

            if self.move_check(ctx.f) {
                return true;
            }
        } else if ctx
            .f
            .ocean_sv(usize::from(self.room_no() as u8), OCEAN_SV_BIT)
        {
            if self.field_0x298 == 1 {
                if self.gba_link() {
                    if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                        return true;
                    }
                    self.mail_cancel();
                }
            }
            self.kill(ctx);
            return true;
        }

        if self.gba_link() && self.gba_status_check(STATUS_CHANNEL_MAIL) {
            let hit = match ctx.agb.as_ref() {
                Some(agb) => {
                    (agb.is_free() || agb.get_follow_target() != 1) && self.hit_check_actor(agb.pos)
                }
                None => false,
            };
            if hit {
                let react = big_little_change(u32::from(self.msg_no)) >> 16;
                if flag < TBOX_RANGE {
                    self.mail_send(react as u16, 2, self.sw1, NO_SWITCH, MAIL_SFX_NONE);
                } else {
                    self.mail_send(react as u16, 2, NO_SWITCH, NO_SWITCH, MAIL_SFX_NONE);
                }
                self.field_0x298 = 1;
            } else if self.field_0x298 == 1 {
                self.mail_cancel();
                self.field_0x298 = 0;
            }
        }
        true
    }

    fn exe_sub_a2(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let sw0 = self.sw0;
        let condition_no = self.param_no;

        if sw0 != NO_SWITCH {
            if condition_no == 0 {
                if !ctx.f.switches.test(sw0) {
                    if self.field_0x298 == 1 {
                        if self.gba_link() {
                            if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                                return true;
                            }
                            self.mail_cancel();
                        }
                        self.field_0x298 = 0;
                    }
                    return true;
                }
            } else if ctx.f.switches.test(sw0) {
                if self.field_0x298 == 1 {
                    if self.gba_link() {
                        if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                            return true;
                        }
                        self.mail_cancel();
                    }
                    self.field_0x298 = 0;
                }
                self.kill(ctx);
                return true;
            }
        }

        if self.move_check(ctx.f) {
            if self.field_0x298 == 1 {
                if self.gba_link() {
                    if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                        return true;
                    }
                    self.mail_cancel();
                }
                self.field_0x298 = 0;
            }
            return true;
        }

        if self.gba_link() && self.gba_status_check(STATUS_CHANNEL_MAIL) {
            let hit = match ctx.agb.as_ref() {
                Some(agb) => {
                    (agb.is_free() || agb.get_follow_target() == 0) && self.hit_check_actor(agb.pos)
                }
                None => false,
            };
            if hit {
                let react = big_little_change(u32::from(self.msg_no)) >> 16;
                self.mail_send(react as u16, 4, self.sw1, sw0, MAIL_SFX_NONE);
                self.field_0x298 = 1;
            } else if self.field_0x298 == 1 {
                self.mail_cancel();
                self.field_0x298 = 0;
            }
        }
        true
    }

    fn exe_sub_f(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let sw0 = self.sw0;
        let condition_no = self.param_no;

        if sw0 != NO_SWITCH {
            if condition_no == 0 {
                if !ctx.f.switches.test(sw0) {
                    return true;
                }
            } else if ctx.f.switches.test(sw0) {
                self.kill(ctx);
                return true;
            }
        } else {
            self.kill(ctx);
            return true;
        }

        if self.move_check(ctx.f) {
            return true;
        }

        if self.gba_link() && self.gba_status_check(STATUS_CHANNEL_MAIL) {
            let hit = match ctx.agb.as_ref() {
                Some(agb) => {
                    agb.is_active()
                        && !agb.is_free()
                        && agb.get_follow_target() == 0
                        && self.hit_check_actor(agb.pos)
                }
                None => false,
            };
            if hit {
                let react = big_little_change(u32::from(self.msg_no)) >> 16;
                self.mail_send(react as u16, 6, self.sw1, sw0, MAIL_SFX_NONE);
            }
        }
        true
    }

    fn exe_sub_f2(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let sw0 = self.sw0;
        let condition_no = self.param_no;

        if sw0 != NO_SWITCH {
            if condition_no == 0 {
                if !ctx.f.switches.test(sw0) {
                    return true;
                }
            } else if ctx.f.switches.test(sw0) {
                if self.gba_link() {
                    if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                        return true;
                    }
                    self.mail_cancel();
                }
                self.kill(ctx);
                return true;
            }
        } else {
            if self.gba_link() {
                if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                    return true;
                }
                self.mail_cancel();
            }
            self.kill(ctx);
            return true;
        }

        if self.move_check(ctx.f) {
            return true;
        }

        if self.gba_link() && self.gba_status_check(STATUS_CHANNEL_MAIL) {
            if let Some(agb) = ctx.agb.as_mut() {
                if self.field_0x299 == 0 {
                    if agb.is_active()
                        && !agb.is_free()
                        && agb.get_follow_target() == 0
                        && self.hit_check_actor(agb.pos)
                    {
                        self.mail_send(0x5A00, 6, NO_SWITCH, NO_SWITCH, MAIL_SFX_NONE);
                        agb.on_free();
                        agb.on_hold();
                        agb.pos = Vec3::new(self.pos.x, self.pos.y + PLAYER_LIFT, self.pos.z);
                        agb.home = agb.pos;
                        agb.shape_angle_x = 0x3FFF;
                        agb.flag_0x67f = true;
                        self.orig_scale_x = self.scale.x;
                        self.orig_scale_z = self.scale.z;
                        self.scale.z = 50.0;
                        self.scale.x = 50.0;
                        self.field_0x299 += 1;
                    }
                } else if self.field_0x299 == 1 {
                    let at_player = ctx
                        .f
                        .player_pos
                        .map(|at| self.hit_check(at, HIT_PLAYER_RADIUS))
                        .unwrap_or(false);
                    if agb.is_active() && at_player {
                        let react = big_little_change(u32::from(self.msg_no)) >> 16;
                        self.mail_send(react as u16, 5, self.sw1, sw0, MAIL_SFX_NONE);
                        agb.reset_cursor(false);
                        self.field_0x299 += 1;
                    } else if self.hit_check_actor(agb.pos) {
                        self.mail_send(0x5A00, 1, NO_SWITCH, NO_SWITCH, MAIL_SFX_NONE);
                        self.field_0x298 = 1;
                    } else if self.field_0x298 == 1 {
                        self.mail_cancel();
                        self.field_0x298 = 0;
                    }
                }
            }
        }
        true
    }

    fn exe_sub_m(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let sw0 = self.sw0;
        if sw0 != NO_SWITCH && !ctx.f.switches.test(sw0) {
            if self.field_0x298 == 1 {
                if self.gba_link() && self.gba_status_check(STATUS_CHANNEL_MAIL) {
                    self.mail_cancel();
                }
                self.field_0x298 = 0;
            }
            return true;
        }

        if self.gba_link() && self.gba_status_check(STATUS_CHANNEL_MAIL) {
            let hit = match ctx.agb.as_ref() {
                Some(agb) => {
                    agb.is_active()
                        && (agb.is_free() || agb.get_follow_target() != 1)
                        && self.hit_check_actor(agb.pos)
                }
                None => false,
            };
            if hit {
                let react = big_little_change(u32::from(self.msg_no)) >> 16;
                self.mail_send(react as u16, 0x3, NO_SWITCH, NO_SWITCH, MAIL_SFX_NONE);
                self.field_0x298 = 1;
            } else if self.field_0x298 == 1 {
                self.mail_cancel();
                self.field_0x298 = 0;
            }
        }
        true
    }

    fn exe_sub_m2(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if ctx.f.stories.test(BIT_UNK_2D01) {
            if self.field_0x298 == 1 {
                if self.gba_link() {
                    if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                        return true;
                    }
                    self.mail_cancel();
                }
                self.field_0x298 = 0;
            }
            self.kill(ctx);
            return true;
        }

        if !inventory::check_get_item(&ctx.f.equipped, &ctx.f.collect, ITEM_SKULL_HAMMER) {
            return true;
        }

        if self.gba_link() && self.gba_status_check(STATUS_CHANNEL_MAIL) {
            let hit = match ctx.agb.as_ref() {
                Some(agb) => {
                    agb.is_active()
                        && (agb.is_free() || agb.get_follow_target() != 1)
                        && self.hit_check_actor(agb.pos)
                }
                None => false,
            };
            if hit {
                let react = big_little_change(u32::from(self.msg_no)) >> 16;
                self.mail_send(react as u16, 0x3, NO_SWITCH, NO_SWITCH, MAIL_SFX_NONE);
                self.field_0x298 = 1;
            } else if self.field_0x298 == 1 {
                self.mail_cancel();
                self.field_0x298 = 0;
            }
        }
        true
    }

    fn exe_sub_m3(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if ctx.f.triforce_num() == TRIFORCE_ALL {
            if self.field_0x298 == 1 {
                if self.gba_link() {
                    if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                        return true;
                    }
                    self.mail_cancel();
                }
                self.field_0x298 = 0;
            }
            self.kill(ctx);
            return true;
        }

        if !ctx.f.stories.test(BIT_UNK_1820) {
            return true;
        }

        if self.gba_link() && self.gba_status_check(STATUS_CHANNEL_MAIL) {
            let mut hit = false;
            if let Some(agb) = ctx.agb.as_ref() {
                hit = agb.is_active()
                    && (agb.is_free() || agb.get_follow_target() != 1)
                    && self.hit_check_actor(agb.pos);
            }
            if hit {
                let room = ctx.f.agb_room;
                let gba_msg_no = self.triforce_check(ctx.f, room);
                let react = big_little_change(u32::from(gba_msg_no)) >> 16;
                self.mail_send(react as u16, 0xF, NO_SWITCH, NO_SWITCH, MAIL_SFX_NONE);
                self.field_0x298 = 1;
            } else if self.field_0x298 == 1 {
                self.mail_cancel();
                self.field_0x298 = 0;
            }
        }
        true
    }

    fn exe_sub_mw(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let sw0 = self.sw0;

        if (sw0 != NO_SWITCH && !ctx.f.switches.test(sw0)) || !self.gba_link() {
            return true;
        }

        if MW_SE_FLAG.load(Ordering::Relaxed) == 0 {
            ctx.events.list.push(SimEvent::Se(SE_CHI_MEGAHORN));
            MW_SE_FLAG.store(1, Ordering::Relaxed);
        }

        let _ = ctx.f.agb_staff;

        self.eye_at(self.pos);

        if ctx.f.demo_accept {
            if let Some(at) = ctx.f.player_pos {
                let diff = self.eye_pos - at;
                let dist = (diff.x * diff.x + diff.z * diff.z).sqrt();
                if dist < EVENT_ATTN_DIST_EPS {
                    self.eye_pos.x += sin_s(ctx.f.player_yaw) * EVENT_ATTN_NUDGE;
                    self.eye_pos.z += cos_s(ctx.f.player_yaw) * EVENT_ATTN_NUDGE;
                }
            }

            if ctx.f.look_attention.is_some_and(|at| ctx.active.end_check(at)) {
                if let Some(at) = ctx.f.look_attention {
                    ctx.active.end(at);
                }
                if sw0 == NO_SWITCH || self.param_no == -1 {
                    self.kill(ctx);
                } else {
                    ctx.switches.clear(sw0);
                }
            }
        } else if let Some(at) = ctx.f.look_attention {
            let _ = ctx.active.order(&ctx.list, at, ctx.entity);
            let _ = EVT_FLAG_TALK;
        }
        true
    }

    fn exe_sub_t(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let sw0 = self.sw0;

        if sw0 != NO_SWITCH && ctx.f.switches.test(sw0) {
            if self.timer == 0 {
                if self.gba_link() {
                    if ctx.f.event_running || !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                        return true;
                    }
                    let react = big_little_change(u32::from(self.msg_no)) >> 16;
                    self.mail_send(react as u16, 0x7, NO_SWITCH, NO_SWITCH, MAIL_SFX_NONE);
                }
                self.kill(ctx);
            } else {
                self.timer -= 1;
            }
            return true;
        }

        if self.gba_link() && self.gba_status_check(STATUS_CHANNEL_BOMB) && self.cyl.tg_hit {
            let se = big_little_change(MAIL_SE_BOMB);
            MAIL_SE.store(se, Ordering::Relaxed);
            ctx.switches.set(sw0);
            if self.msg_no == NO_MSG {
                self.kill(ctx);
                return true;
            }
            self.timer = TIMER_TRIGGER;
        }
        true
    }

    fn exe_sub_s(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let sw0 = self.sw0;
        let sw1 = self.sw1;

        if sw0 != NO_SWITCH && ctx.f.switches.test(sw0) {
            self.kill(ctx);
            return true;
        }

        if ctx.f.stage_m_new_d2
            && self.room_no() == ROOM_M_NEW_D2
            && ctx.f.chests.is_open(SAVE_STAGE_CURRENT, TBOX_M_NEW_D2)
        {
            self.kill(ctx);
            return true;
        }

        if self.field_0x299 == 0 && self.gba_link() {
            if self.param_no < 0 {
                let hit = match ctx.agb.as_ref() {
                    Some(agb) => {
                        agb.is_active()
                            && (agb.is_free() || agb.get_follow_target() != 1)
                            && self.hit_check_actor(agb.pos)
                    }
                    None => false,
                };
                if hit {
                    if self.msg_no == NO_MSG {
                        ctx.switches.set(sw0);
                        if sw1 != NO_SWITCH {
                            ctx.switches.set(sw1);
                        }
                        self.kill(ctx);
                    } else if self.gba_status_check(STATUS_CHANNEL_MAIL) {
                        let react = big_little_change(u32::from(self.msg_no)) >> 16;
                        self.mail_send(react as u16, 0x9, sw1, sw0, MAIL_SFX_NONE);
                        self.field_0x299 = 1;
                    }
                }
            } else if self.timer == 0 {
                if self.msg_no == NO_MSG {
                    ctx.switches.set(sw0);
                    if sw1 != NO_SWITCH {
                        ctx.switches.set(sw1);
                    }
                    self.kill(ctx);
                } else if self.gba_status_check(STATUS_CHANNEL_MAIL) {
                    let react = big_little_change(u32::from(self.msg_no)) >> 16;
                    self.mail_send(react as u16, 0x9, sw1, sw0, MAIL_SFX_NONE);
                    self.field_0x299 = 1;
                }
            } else {
                let at_player = ctx
                    .f
                    .player_pos
                    .map(|at| self.hit_check(at, HIT_PLAYER_RADIUS))
                    .unwrap_or(false);
                if ctx.agb.is_some_and(|agb| agb.present) && at_player {
                    self.timer -= 1;
                }
            }
        }
        true
    }

    fn exe_sub_r(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let sw0 = self.sw0;

        if self.gba_link() && self.gba_status_check(STATUS_CHANNEL_MAIL) {
            if sw0 != NO_SWITCH && ctx.f.switches.test(sw0) {
                let mut item_no = self.param_no as i32;
                if item_no < 0 || item_no > i32::from(ITEM_TRIPLE_HEART) {
                    item_no = ITEM_HEART;
                }
                if item_no != i32::from(ITEM_RECOVER_FAIRY) {
                    self.pos.y += self.scale.y / 2.0;
                }
                ctx.spawns.push(Spawn {
                    pos: self.pos,
                    room: self.room_no(),
                    item: item_no as u8,
                });
                ctx.events.list.push(SimEvent::Se(SE_CHI_MEGAHORN));
                self.mail_cancel();

                self.field_0x298 = 0;
                self.kill(ctx);
                return true;
            } else if ctx.agb.as_ref().is_some_and(|agb| agb.is_active()) {
                let hit = match ctx.agb.as_ref() {
                    Some(agb) => agb.is_free() && self.hit_check_actor(agb.pos),
                    None => false,
                };
                if hit {
                    let react = big_little_change(u32::from(self.msg_no)) >> 16;
                    self.mail_send(react as u16, 0xA, NO_SWITCH, sw0, MAIL_SFX_NONE);
                    self.field_0x299 = 1;
                    self.field_0x298 = 1;
                } else {
                    if self.field_0x298 == 1 {
                        self.mail_cancel();
                        self.field_0x298 = 0;
                    }
                    self.field_0x299 = 0;
                }
            }
        }
        true
    }

    fn exe_sub_b(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let sw0 = self.sw0;
        let restriction = self.param_no;

        if restriction == RESTRICTION_FIGURE {
            if !ctx.f.stories.test(BIT_UNK_2E08) {
                let at_player = ctx
                    .f
                    .player_pos
                    .map(|at| self.hit_check(at, HIT_PLAYER_RADIUS))
                    .unwrap_or(false);
                if at_player {
                    ctx.stories.set(BIT_UNK_2E08);
                }
            }
            if ctx.agb.is_some() {
                let hit = match ctx.agb.as_ref() {
                    Some(agb) => self.hit_check_actor(agb.pos),
                    None => false,
                };
                AGB_FLAG_3_5.store(u8::from(hit), Ordering::Relaxed);

                if self.gba_link() && self.gba_status_check(STATUS_CHANNEL_MAIL) {
                    let beat = FIGURE_BEAT.load(Ordering::Relaxed);
                    if self.field_0x299 == beat {
                        let key = i32::from(self.field_0x299) - 1;
                        if let Some(&msg) = usize::try_from(key).ok().and_then(|i| BEAT_MSG.get(i)) {
                            let react = big_little_change(u32::from(msg)) >> 16;
                            self.mail_send(react as u16, 0xB, NO_SWITCH, NO_SWITCH, MAIL_SFX_NONE);
                        }
                        if self.field_0x299 < BEAT_KEYS as u8 {
                            self.field_0x299 += 1;
                        } else {
                            FIGURE_BEAT.store(BEAT_KEYS as u8 - 1, Ordering::Relaxed);
                        }
                    }

                    let dispose = FIGURE_DISPOSE.load(Ordering::Relaxed);
                    if self.timer == u32::from(dispose) {
                        let key = self.timer as i32 - 1;
                        if let Some(&msg) =
                            usize::try_from(key).ok().and_then(|i| DISPOSED_MSG.get(i))
                        {
                            let react = big_little_change(u32::from(msg)) >> 16;
                            self.mail_send(react as u16, 0xB, NO_SWITCH, NO_SWITCH, MAIL_SFX_NONE);
                        }
                        if self.timer < DISPOSE_KEYS as u32 {
                            self.timer += 1;
                        } else {
                            FIGURE_DISPOSE.store(1, Ordering::Relaxed);
                        }
                    }
                } else {
                    FIGURE_DISPOSE.store(0, Ordering::Relaxed);
                    FIGURE_BEAT.store(0, Ordering::Relaxed);
                    self.field_0x299 = 1;
                    self.timer = 1;
                }
            }
        } else if restriction == RESTRICTION_DENY_MSG {
            if let Some(agb) = ctx.agb.as_mut() {
                let at_player = ctx
                    .f
                    .player_pos
                    .map(|at| self.hit_check(at, HIT_PLAYER_RADIUS))
                    .unwrap_or(false);
                if at_player {
                    agb.flag_0x67d = true;
                    agb.cursor_message = u32::from(self.msg_no);
                    self.field_0x298 = 1;
                } else if self.field_0x298 == 1 {
                    agb.flag_0x67d = false;
                    self.field_0x298 = 0;
                }
            }
        } else if restriction == RESTRICTION_CURSOR {
            let at_player = ctx
                .f
                .player_pos
                .map(|at| self.hit_check(at, HIT_PLAYER_RADIUS))
                .unwrap_or(false);
            if at_player {
                self.kill(ctx);
                return true;
            }

            if let Some(agb) = ctx.agb.as_mut() {
                if agb.is_free() && self.hit_check_actor(agb.pos) {
                    let xz_diff = distance_xz(agb.pos, self.pos);
                    let pos_diff = self.pos - agb.pos;

                    if !self.non_circular {
                        let rad = self.scale.x;
                        if xz_diff < rad - HIT_AGB_INSIDE_MARGIN {
                            if agb.pos.y < self.pos.y + self.scale.y / 2.0 {
                                agb.home.y = self.pos.y - HIT_AGB_MOVE_GAP;
                            } else {
                                agb.home.y = self.pos.y + self.scale.y + HIT_AGB_MOVE_GAP;
                            }
                        } else {
                            agb.home.x = self.pos.x - (pos_diff.x * (rad + 1.0) / xz_diff);
                            agb.home.z = self.pos.z - (pos_diff.z * (self.scale.z + 1.0) / xz_diff);
                        }
                    } else {
                        let mut rel = self.relative_pos(agb.pos);
                        rel.y = rel.y - self.scale.y / 2.0 + HIT_AGB_MOVE_LIFT;
                        let x_diff = self.scale.x - rel.x.abs();
                        let y_diff = (self.scale.y / 2.0) - rel.y.abs() + PLAYER_LIFT;
                        let z_diff = self.scale.z - rel.z.abs();
                        if y_diff < x_diff && y_diff < z_diff {
                            if agb.pos.y < self.pos.y + self.scale.y / 2.0 {
                                agb.home.y = self.pos.y - HIT_AGB_MOVE_GAP;
                            } else {
                                agb.home.y = self.pos.y + self.scale.y + HIT_AGB_MOVE_GAP;
                            }
                        } else if x_diff < z_diff {
                            if rel.x >= 0.0 {
                                agb.home.x = (self.pos.x - pos_diff.x) + (x_diff + 1.0) * cos_s(self.shape_angle_y);
                                agb.home.z = (self.pos.z - pos_diff.z) - (x_diff + 1.0) * sin_s(self.shape_angle_y);
                            } else {
                                agb.home.x = (self.pos.x - pos_diff.x) - (x_diff + 1.0) * cos_s(self.shape_angle_y);
                                agb.home.z = (self.pos.z - pos_diff.z) + (x_diff + 1.0) * sin_s(self.shape_angle_y);
                            }
                        } else if rel.z >= 0.0 {
                            agb.home.x = (self.pos.x - pos_diff.x) + (z_diff + 1.0) * sin_s(self.shape_angle_y);
                            agb.home.z = (self.pos.z - pos_diff.z) + (z_diff + 1.0) * cos_s(self.shape_angle_y);
                        } else {
                            agb.home.x = (self.pos.x - pos_diff.x) - (z_diff + 1.0) * sin_s(self.shape_angle_y);
                            agb.home.z = (self.pos.z - pos_diff.z) - (z_diff + 1.0) * cos_s(self.shape_angle_y);
                        }
                    }
                }
            }
        } else if (restriction != RESTRICTION_TBOX && sw0 != NO_SWITCH && ctx.f.switches.test(sw0))
            || (restriction == RESTRICTION_TBOX
                && sw0 < TBOX_RANGE
                && ctx.f.chests.is_open(SAVE_STAGE_CURRENT, sw0))
        {
            if let Some(agb) = ctx.agb.as_mut() {
                if restriction == RESTRICTION_BOMB_DENY {
                    agb.off_bomb_deny();
                } else if restriction != RESTRICTION_CURSOR {
                    agb.flag_0x67b = false;
                }
            }
            self.kill(ctx);
            return true;
        } else if let Some(agb) = ctx.agb.as_mut() {
            if restriction == RESTRICTION_BOMB_DENY {
                let at_player = ctx
                    .f
                    .player_pos
                    .map(|at| self.hit_check(at, HIT_PLAYER_RADIUS))
                    .unwrap_or(false);
                if at_player {
                    agb.on_bomb_deny();
                    agb.set_deny_message(u32::from(self.msg_no));
                    self.field_0x298 = 1;
                } else if self.field_0x298 == 1 {
                    agb.off_bomb_deny();
                    self.field_0x298 = 0;
                }
            } else {
                let hit = self.hit_check_actor(agb.pos);
                if hit {
                    agb.flag_0x67b = true;
                    agb.deny_message = u32::from(self.msg_no);
                    self.field_0x298 = 1;
                } else if self.field_0x298 == 1 {
                    agb.flag_0x67b = false;
                    self.field_0x298 = 0;
                }
            }
        }
        true
    }

    fn exe_sub_d(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let sw0 = self.sw0;

        if sw0 != NO_SWITCH && ctx.f.switches.test(sw0) && self.field_0x299 == 0 {
            if let Some(agb) = ctx.agb.as_mut() {
                agb.reset_cursor(true);
            }
            self.kill(ctx);
            return true;
        }

        if self.gba_link() {
            if self.gba_status_check(STATUS_CHANNEL_MAIL) {
                if let Some(agb) = ctx.agb.as_mut() {
                    if self.field_0x299 == 0 {
                        if agb.is_active() && agb.is_free() && self.hit_check_actor(agb.pos) {
                            self.mail_send(0x5F00, 0xC, NO_SWITCH, NO_SWITCH, 0x19);
                            agb.on_hold();
                            agb.flag_0x675 = true;

                            agb.pos = self.pos;
                            agb.home = self.pos;
                            agb.shape_angle_x = -0x3FFF;
                            agb.flag_0x67f = true;
                            self.orig_scale_x = self.scale.x;
                            self.orig_scale_z = self.scale.z;
                            self.scale.z = 50.0;
                            self.scale.x = 50.0;
                            self.field_0x299 += 1;
                        }
                    } else if self.field_0x299 == 1 {
                        let at_player = ctx
                            .f
                            .player_pos
                            .map(|at| self.hit_check(at, HIT_PLAYER_RADIUS))
                            .unwrap_or(false);
                        if agb.is_active() && at_player {
                            let react = big_little_change(u32::from(self.msg_no)) >> 16;
                            self.mail_send(react as u16, 0xC, self.sw1, NO_SWITCH, 0x1B);

                            agb.shape_angle_x = 0x3FFF;
                            agb.home.y += PLAYER_LIFT;
                            agb.pos.y = agb.home.y;
                            agb.flag_0x676 = true;

                            self.field_0x299 += 1;
                        }
                    } else if self.field_0x299 == 2 && ctx.f.switches.test(self.sw1) {
                        let item_no = self.param_no as i32;
                        if item_no != i32::from(ITEM_RECOVER_FAIRY) {
                            self.pos.y += self.scale.y / 2.0;
                        }
                        if (0..ITEM_COUNT).contains(&item_no)
                            && item_no != i32::from(ITEM_HEART_PIECE)
                            && item_no != i32::from(ITEM_HEART_CONTAINER)
                            && item_no != i32::from(ITEM_SMALL_KEY)
                        {
                            ctx.spawns.push(Spawn {
                                pos: self.pos,
                                room: self.room_no(),
                                item: item_no as u8,
                            });
                            self.mail_send(NO_MSG, MAIL_REACT_NONE, NO_SWITCH, NO_SWITCH, 0x11);
                        }

                        agb.reset_cursor(true);

                        self.kill(ctx);
                        return true;
                    }
                }
            }
        } else if self.field_0x299 != 0 && self.field_0x299 < 3 {
            self.scale.x = self.orig_scale_x;
            self.scale.z = self.orig_scale_z;
            self.field_0x299 = 0;
        }
        true
    }

    fn exe_sub_fa(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let sw0 = self.sw0;
        let condition_no = self.param_no;

        if sw0 != NO_SWITCH {
            if condition_no == 0 {
                if !ctx.f.switches.test(sw0) {
                    if self.field_0x298 == 1 {
                        if self.gba_link() {
                            if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                                return true;
                            }
                            self.mail_cancel();
                        }
                        self.field_0x298 = 0;
                    }
                    return true;
                }
            } else if ctx.f.switches.test(sw0) {
                if self.field_0x298 == 1 {
                    if self.gba_link() {
                        if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                            return true;
                        }
                        self.mail_cancel();
                    }
                    self.field_0x298 = 0;
                }
                self.kill(ctx);
                return true;
            }
        } else {
            if self.field_0x298 == 1 {
                if self.gba_link() {
                    if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                        return true;
                    }
                    self.mail_cancel();
                }
                self.field_0x298 = 0;
            }
            self.kill(ctx);
            return true;
        }

        if self.move_check(ctx.f) {
            if self.field_0x298 == 1 {
                if self.gba_link() {
                    if !self.gba_status_check(STATUS_CHANNEL_MAIL) {
                        return true;
                    }
                    self.mail_cancel();
                }
                self.field_0x298 = 0;
            }
            return true;
        }

        if self.gba_link() && self.gba_status_check(STATUS_CHANNEL_MAIL) {
            let hit = match ctx.agb.as_ref() {
                Some(agb) => {
                    (agb.is_free() || agb.get_follow_target() == 0) && self.hit_check_actor(agb.pos)
                }
                None => false,
            };
            if hit {
                let react = big_little_change(u32::from(self.msg_no)) >> 16;
                self.mail_send(react as u16, 0xD, self.sw1, sw0, MAIL_SFX_NONE);
                self.field_0x298 = 1;
            } else if self.field_0x298 == 1 {
                self.mail_cancel();
                self.field_0x298 = 0;
            }
        }
        true
    }

    fn room_no(&self) -> i8 {
        self.room
    }

    fn kill(&mut self, ctx: &mut Ctx<'_>) {
        self.release(ctx);
        ctx.dying.push(ctx.entity);
    }

    fn gba_status_check(&self, _channel: u8) -> bool {
        self.gba_link()
    }

    fn eye_at(&mut self, pos: Vec3) {
        self.eye_pos = pos;
    }

    pub fn move_check(&mut self, f: &Facts) -> bool {
        match self.param_no {
            1 => {
                if f.stories.test(BIT_MET_KORL) {
                    return false;
                }
            }
            2 => {
                if f.collect.is_symbol(SYMBOL_NAYRU) {
                    return false;
                }
            }
            3 => {
                if f.stories.test(BIT_UNK_1E40) {
                    return false;
                }
            }
            4 => {
                if f.stories.test(BIT_UNK_1820) {
                    return false;
                }
            }
            5 => {
                if f.ghostship_aboard {
                    return false;
                }
            }
            6 => {
                if f.stories.test(BIT_UNLOCK_TINGLE_BALLOON_DISCOUNT) {
                    return false;
                }
            }
            7 => {
                if f.stories.test(BIT_UNLOCK_TING_DISCOUNT) {
                    return false;
                }
            }
            8 => {
                if f.stories.test(BIT_UNK_1708) {
                    return false;
                }
            }
            9 => {
                if f.chests.is_open(SAVE_STAGE_DRC, SAVE_TBOX) {
                    return false;
                }
            }
            0xA => {
                if f.chests.is_open(SAVE_STAGE_FW, SAVE_TBOX) {
                    return false;
                }
            }
            0xB => {
                if f.chests.is_open(SAVE_STAGE_TOTG, SAVE_TBOX) {
                    return false;
                }
            }
            0xC => {
                if f.chests.is_open(SAVE_STAGE_WT, SAVE_TBOX) {
                    return false;
                }
            }
            0xD => {
                if f.chests.is_open(SAVE_STAGE_ET, SAVE_TBOX) {
                    return false;
                }
            }
            0x11 => {
                if f.select_item[0] != ITEM_BOW
                    && f.select_item[1] != ITEM_BOW
                    && f.select_item[2] != ITEM_BOW
                {
                    return false;
                }
            }
            0x12 => {
                if f.select_item[0] != ITEM_BOOMERANG
                    && f.select_item[1] != ITEM_BOOMERANG
                    && f.select_item[2] != ITEM_BOOMERANG
                {
                    return false;
                }
            }
            0x13 => {
                if f.select_item[0] != ITEM_DEKU_LEAF
                    && f.select_item[1] != ITEM_DEKU_LEAF
                    && f.select_item[2] != ITEM_DEKU_LEAF
                {
                    return false;
                }
            }
            0x14 => {
                if f.select_item[0] != ITEM_GRAPPLE_HOOK
                    && f.select_item[1] != ITEM_GRAPPLE_HOOK
                    && f.select_item[2] != ITEM_GRAPPLE_HOOK
                {
                    return false;
                }
            }
            0x15 => {
                if f.select_item[0] != ITEM_BOMB_BAG
                    && f.select_item[1] != ITEM_BOMB_BAG
                    && f.select_item[2] != ITEM_BOMB_BAG
                {
                    return false;
                }
            }
            0x16 => {
                if f.select_item[0] != ITEM_HOOKSHOT
                    && f.select_item[1] != ITEM_HOOKSHOT
                    && f.select_item[2] != ITEM_HOOKSHOT
                {
                    return false;
                }
            }
            0x17 => {
                if inventory::check_get_item(&f.equipped, &f.collect, ITEM_MAGIC_ARMOR) {
                    return false;
                }
            }
            0x18 => {
                if f.collect.test(COLLECT_TING, COLLECT_TING_ITEM) {
                    return false;
                }
            }
            0x19 => {
                if f.life < f.max_life {
                    return false;
                }
            }
            0x1A => {
                if inventory::NO_ITEM != f.equipped.item(inventory::BOW_SLOT) && f.arrow_num < f.arrow_max
                {
                    return false;
                }
            }
            0x1B => {
                if inventory::check_get_item(&f.equipped, &f.collect, ITEM_BOMB_BAG)
                    && f.bomb_num < f.bomb_max
                {
                    return false;
                }
            }
            0x1C => {
                if f.life < LIFE_CONTAINER {
                    return false;
                }
            }
            0x1D => {
                if f.max_magic != MAGIC_LIMIT && f.get_magic_is_non_null() {
                    return false;
                }
            }
            0x1E => {
                if inventory::NO_ITEM != f.equipped.item(inventory::BOW_SLOT) && f.arrow_num == 0 {
                    return false;
                }
            }
            0x1F => {
                if inventory::check_get_item(&f.equipped, &f.collect, ITEM_BOMB_BAG) && f.bomb_num == 0
                {
                    return false;
                }
            }
            0x20 => {
                if f.stories.test(BIT_UNK_1708)
                    && inventory::NO_ITEM != f.equipped.item(inventory::BOW_SLOT)
                    && f.arrow_num == 0
                {
                    return false;
                }
            }
            0x21 => {
                if f.stories.test(BIT_UNK_1708)
                    && inventory::check_get_item(&f.equipped, &f.collect, ITEM_BOMB_BAG)
                    && f.bomb_num == 0
                {
                    return false;
                }
            }
            0x22 => {
                if f.rupees <= RUPEE_DONATE_MIN {
                    return false;
                }
            }
            0x23 => {
                if f.swim {
                    return false;
                }
            }
            0x24 => {
                if f.ship_ride {
                    return false;
                }
            }
            0x25 => {
                if f.ship_ride
                    && f.select_item[0] != ITEM_BOMB_BAG
                    && f.select_item[1] != ITEM_BOMB_BAG
                    && f.select_item[2] != ITEM_BOMB_BAG
                {
                    return false;
                }
            }
            0x26 => {
                if f.ship_ride
                    && f.select_item[0] != ITEM_GRAPPLE_HOOK
                    && f.select_item[1] != ITEM_GRAPPLE_HOOK
                    && f.select_item[2] != ITEM_GRAPPLE_HOOK
                {
                    return false;
                }
            }
            0x27 => {
                if f.key_num == 0 {
                    return false;
                }
            }
            0x28 => {
                if !f.mem.is_boss_key() {
                    return false;
                }
            }
            0x29 => {
                if inventory::check_get_item(&f.equipped, &f.collect, ITEM_GRAPPLE_HOOK) {
                    return false;
                }
            }
            0x2A => {
                if inventory::check_get_item(&f.equipped, &f.collect, ITEM_BOOMERANG) {
                    return false;
                }
            }
            0x2B => {
                if inventory::check_get_item(&f.equipped, &f.collect, ITEM_BOMB_BAG) {
                    return false;
                }
            }
            0x2C => {
                if inventory::NO_ITEM != f.equipped.item(inventory::BOW_SLOT) {
                    return false;
                }
            }
            0x2D => {
                if !f.machete {
                    return false;
                }
            }
            0x2E => {
                if f.machete {
                    return false;
                }
            }
            0x2F => {
                if f.collect.is_symbol(SYMBOL_DIN)
                    && !f.stories.test(BIT_UNLOCK_TING_DISCOUNT)
                {
                    return false;
                }
            }
            0x30 => {
                if f.collect.is_symbol(SYMBOL_DIN)
                    && !f.stories.test(BIT_UNLOCK_TINGLE_BALLOON_DISCOUNT)
                {
                    return false;
                }
            }
            0x31 => {
                if !f.stories.test(BIT_UNK_1708) {
                    return false;
                }
            }
            0x32 => {
                if f.collect.is_tact(TACT_NONE) {
                    return false;
                }
            }
            0x33 => {
                if !f.collect.is_tact(TACT_NONE) {
                    return false;
                }
            }
            0x34 => {
                if f.winds_dir == 0 {
                    return false;
                }
            }
            0x35 => {
                if f.winds_dir != 0 {
                    return false;
                }
            }
            0x36 => {
                if f.winds_dir == 4 {
                    return false;
                }
            }
            0x37 => {
                if (!f.winds_dir) == 4 {
                    return false;
                }
            }
            0x38 => {
                if f.winds_dir == 2 {
                    return false;
                }
            }
            0x39 => {
                if (!f.winds_dir) == 2 {
                    return false;
                }
            }
            0x3A => {
                if f.winds_dir == 6 {
                    return false;
                }
            }
            0x3B => {
                if (!f.winds_dir) == 6 {
                    return false;
                }
            }
            0x3C => {
                if FIGURE_DISPOSE.load(Ordering::Relaxed) == 1 {
                    return false;
                }
            }
            0x3D => {
                if FIGURE_DISPOSE.load(Ordering::Relaxed) == 2 {
                    return false;
                }
            }
            0x3E => {
                if FIGURE_BEAT.load(Ordering::Relaxed) == 1 {
                    return false;
                }
            }
            0x3F => {
                if FIGURE_BEAT.load(Ordering::Relaxed) == 2 {
                    return false;
                }
            }
            0x40 => {
                if FIGURE_BEAT.load(Ordering::Relaxed) == 3 {
                    return false;
                }
            }
            0x41 => {
                if FIGURE_BEAT.load(Ordering::Relaxed) == 4 {
                    return false;
                }
            }
            0x42 => {
                for slot in 0..3 {
                    if f.os_room[slot] {
                        return false;
                    }
                }
            }
            0x43 => {
                for slot in 0..3 {
                    if f.os_room[slot] {
                        return true;
                    }
                }
                return false;
            }
            0x44 => {
                if f.player == FPC_NPC_OS {
                    return false;
                }
            }
            0x45 => {
                if f.player != FPC_NPC_OS {
                    return false;
                }
            }
            0x46 => {
                if f.player == FPC_NPC_MD {
                    return false;
                }
            }
            0x47 => {
                if f.player != FPC_NPC_MD {
                    return false;
                }
            }
            0x48 => {
                if f.player == FPC_NPC_CB1 {
                    return false;
                }
            }
            0x49 => {
                if f.player != FPC_NPC_CB1 {
                    return false;
                }
            }
            0x4A => {
                if f.player == FPC_NPC_KAM {
                    return false;
                }
            }
            0x4B => {
                if f.player != FPC_NPC_KAM {
                    return false;
                }
            }
            0x4C => {
                if inventory::check_get_item(&f.equipped, &f.collect, ITEM_SKULL_HAMMER) {
                    return false;
                }
            }
            0x4D => {
                if !inventory::check_get_item(&f.equipped, &f.collect, ITEM_SKULL_HAMMER) {
                    return false;
                }
            }
            0x4E => {
                if inventory::check_get_item(&f.equipped, &f.collect, ITEM_MIRROR_SHIELD) {
                    return false;
                }
            }
            0x4F => {
                if !inventory::check_get_item(&f.equipped, &f.collect, ITEM_MIRROR_SHIELD) {
                    return false;
                }
            }
            0x50 => {
                if f.md_room {
                    return false;
                }
            }
            0x51 => {
                if !f.md_room {
                    return false;
                }
            }
            0x52 => {
                if inventory::check_get_item(&f.equipped, &f.collect, ITEM_HOOKSHOT) {
                    return false;
                }
            }
            0x53 => {
                if !inventory::check_get_item(&f.equipped, &f.collect, ITEM_HOOKSHOT) {
                    return false;
                }
            }
            0x54 => {
                if f.cb1_room {
                    return false;
                }
            }
            0x55 => {
                if !f.cb1_room {
                    return false;
                }
            }
            0x56 => {
                if !f.stories.test(BIT_UNK_2E08) {
                    return false;
                }
            }
            0x57 => {
                if !f.collect.test(COLLECT_TING, COLLECT_TING_ITEM) {
                    return false;
                }
            }
            0x58 => {
                if !inventory::check_get_item(&f.equipped, &f.collect, ITEM_IRON_BOOTS) {
                    return false;
                }
            }
            0x59 => {
                if inventory::check_get_item(&f.equipped, &f.collect, ITEM_MASTER_SWORD_2)
                    || inventory::check_get_item(&f.equipped, &f.collect, ITEM_MASTER_SWORD_3)
                {
                    return false;
                }
            }
            0x5A => {
                if !inventory::check_get_item(&f.equipped, &f.collect, ITEM_MASTER_SWORD_2)
                    && !inventory::check_get_item(&f.equipped, &f.collect, ITEM_MASTER_SWORD_3)
                {
                    return false;
                }
            }
            0x5B => {
                if inventory::check_get_item(&f.equipped, &f.collect, ITEM_MASTER_SWORD_3) {
                    return false;
                }
            }
            0x5C => {
                if !inventory::check_get_item(&f.equipped, &f.collect, ITEM_MASTER_SWORD_3) {
                    return false;
                }
            }
            0x5D => {
                if f.rupees >= RUPEE_KEY_MIN && Self::chart_short(f) {
                    return false;
                }
            }
            0x5E => {
                if f.rupees < RUPEE_KEY_MIN && Self::chart_short(f) {
                    return false;
                }
            }
            0x5F => {
                if !f.charts.has(usize::from(Self::slot(CHART_FIRST))) {
                    return false;
                }
            }
            0x60 => {
                if f.charts.has(usize::from(Self::slot(CHART_FIRST)))
                    && f.charts.triforce(Self::triforce_slot(CHART_FIRST))
                    && !f.collect.is_triforce(0)
                {
                    return false;
                }
            }
            0x61 => {
                if f.collect.is_triforce(0) {
                    return false;
                }
            }
            0x62 => {
                if !f.charts.has(usize::from(Self::slot(CHART_FIRST + 1))) {
                    return false;
                }
            }
            0x63 => {
                if f.charts.has(usize::from(Self::slot(CHART_FIRST + 1)))
                    && f.charts.triforce(Self::triforce_slot(CHART_FIRST + 1))
                    && !f.collect.is_triforce(1)
                {
                    return false;
                }
            }
            0x64 => {
                if f.collect.is_triforce(1) {
                    return false;
                }
            }
            0x65 => {
                if !f.charts.has(usize::from(Self::slot(CHART_FIRST + 2))) {
                    return false;
                }
            }
            0x66 => {
                if f.charts.has(usize::from(Self::slot(CHART_FIRST + 2)))
                    && f.charts.triforce(Self::triforce_slot(CHART_FIRST + 2))
                    && !f.collect.is_triforce(2)
                {
                    return false;
                }
            }
            0x67 => {
                if f.collect.is_triforce(2) {
                    return false;
                }
            }
            0x68 => {
                if !f.charts.has(usize::from(Self::slot(CHART_FIRST + 3))) {
                    return false;
                }
            }
            0x69 => {
                if f.charts.has(usize::from(Self::slot(CHART_FIRST + 3)))
                    && f.charts.triforce(Self::triforce_slot(CHART_FIRST + 3))
                    && !f.collect.is_triforce(3)
                {
                    return false;
                }
            }
            0x6A => {
                if f.collect.is_triforce(3) {
                    return false;
                }
            }
            0x6B => {
                if !f.charts.has(usize::from(Self::slot(CHART_FIRST + 4))) {
                    return false;
                }
            }
            0x6C => {
                if f.charts.has(usize::from(Self::slot(CHART_FIRST + 4)))
                    && f.charts.triforce(Self::triforce_slot(CHART_FIRST + 4))
                    && !f.collect.is_triforce(4)
                {
                    return false;
                }
            }
            0x6D => {
                if f.collect.is_triforce(4) {
                    return false;
                }
            }
            0x6E => {
                if !f.charts.has(usize::from(Self::slot(CHART_FIRST + 5))) {
                    return false;
                }
            }
            0x6F => {
                if f.charts.has(usize::from(Self::slot(CHART_FIRST + 5)))
                    && f.charts.triforce(Self::triforce_slot(CHART_FIRST + 5))
                    && !f.collect.is_triforce(5)
                {
                    return false;
                }
            }
            0x70 => {
                if f.collect.is_triforce(5) {
                    return false;
                }
            }
            0x71 => {
                if !f.charts.has(usize::from(Self::slot(CHART_FIRST + 6))) {
                    return false;
                }
            }
            0x72 => {
                if f.charts.has(usize::from(Self::slot(CHART_FIRST + 6)))
                    && f.charts.triforce(Self::triforce_slot(CHART_FIRST + 6))
                    && !f.collect.is_triforce(6)
                {
                    return false;
                }
            }
            0x73 => {
                if f.collect.is_triforce(6) {
                    return false;
                }
            }
            0x74 => {
                if !f.charts.has(usize::from(Self::slot(CHART_LAST))) {
                    return false;
                }
            }
            0x75 => {
                if f.charts.has(usize::from(Self::slot(CHART_LAST)))
                    && f.charts.triforce(Self::triforce_slot(CHART_LAST))
                    && !f.collect.is_triforce(7)
                {
                    return false;
                }
            }
            0x76 => {
                if f.collect.is_triforce(7) {
                    return false;
                }
            }
            0x77 => {
                if f.curse {
                    return false;
                }
            }
            0x78 => {
                if !f.curse {
                    return false;
                }
            }
            0x79 => {
                if f.confuse {
                    return false;
                }
            }
            0x7A => {
                if !f.confuse {
                    return false;
                }
            }
            0x7B => {
                if f.player == FPC_NPC_MD && f.md_flying {
                    return false;
                }
            }
            0x7C => {
                if f.player != FPC_NPC_MD || !f.md_flying {
                    return false;
                }
            }
            0x7F => {
                if f.player == FPC_NPC_MD && f.cb1_flying {
                    return false;
                }
            }
            0x80 => {
                if f.player != FPC_NPC_MD || !f.cb1_flying {
                    return false;
                }
            }
            0x7D => {
                if !f.switches.test(SWITCH_7C) {
                    return false;
                }
            }
            0x7E => {
                if inventory::check_get_item(&f.equipped, &f.collect, ITEM_SKULL_HAMMER)
                    && !f.stories.test(BIT_UNK_2D01)
                {
                    return false;
                }
            }
            0x82 => {
                if self.field_0x29b == 0 {
                    if f.md_room {
                        self.field_0x29b = 1;
                    }
                } else if !f.md_room {
                    return false;
                }
            }
            0x83 => {
                if self.field_0x29b == 0 {
                    if f.cb1_room {
                        self.field_0x29b = 1;
                    }
                } else if !f.cb1_room {
                    return false;
                }
            }
            _ => return false,
        }
        true
    }

    fn slot(no: i32) -> u8 {
        (no - 1) as u8
    }

    fn triforce_slot(no: i32) -> u8 {
        (no - 1) as u8
    }

    fn chart_short(f: &Facts) -> bool {
        (CHART_FIRST..=CHART_LAST).any(|no| {
            f.charts.has(usize::from(Self::slot(no)))
                && !f.charts.triforce(Self::triforce_slot(no))
        })
    }

    pub fn triforce_check(&mut self, f: &Facts, room_no: i32) -> u16 {
        for index in 0..TRIFORCE_SLOTS {
            if f.charts.triforce(index as u8) && !f.collect.is_triforce(index as u8) {
                return if f.stories.test(BIT_UNK_3E02) {
                    0x304
                } else {
                    0x303
                };
            }
        }

        for no in CHART_FIRST..=CHART_LAST {
            if f.charts.has(usize::from(Self::slot(no))) && !f.charts.triforce(Self::triforce_slot(no))
            {
                if RUPEE_BUY_MIN <= f.rupees {
                    return 0x305;
                }
                let rnd = self.rnd_span(TRIFORCE_RND_SPAN);
                if rnd <= TRIFORCE_RND_LOW {
                    return 0x306;
                } else if rnd <= TRIFORCE_RND_MID {
                    return 0x307;
                } else {
                    return 0x308;
                }
            }
        }

        if !f.charts.has(usize::from(Self::slot(CHART_FIRST))) {
            return 0x309;
        }
        if !f.charts.has(usize::from(Self::slot(CHART_FIRST + 1)))
            && inventory::check_get_item(&f.equipped, &f.collect, ITEM_SKULL_HAMMER)
        {
            if inventory::check_get_item(&f.equipped, &f.collect, ITEM_CABANA_DEED) {
                return if room_no == ISLE_ROOM_PRIVATE_OASIS {
                    0x30A
                } else {
                    0x30B
                };
            }
            let num = f.pendant_num(ITEM_JOY_PENDANT);
            return if num >= JOY_PENDANT_FULL { 0x30C } else { 0x30D };
        }
        if !f.charts.has(usize::from(Self::slot(CHART_FIRST + 2))) {
            if inventory::check_get_item(&f.equipped, &f.collect, ITEM_BAIT_BAG) {
                if f.bait(ITEM_HYOI_PEAR) {
                    return if room_no == ISLE_ROOM_BIRDS_PEAK {
                        0x311
                    } else {
                        0x310
                    };
                }
                return 0x30F;
            }
            return 0x30E;
        }
        if !f.charts.has(usize::from(Self::slot(CHART_FIRST + 3)))
            && inventory::check_get_item(&f.equipped, &f.collect, ITEM_HOOKSHOT)
        {
            if f.charts.is_open(usize::from(Self::slot(CHART_TRIFORCE_SLOT))) {
                if f.hour < HOUR_DAWN || f.hour >= HOUR_NIGHT_END {
                    match f.moon {
                        MOON_CRESCENT => {
                            if room_no != MOON_DEFAULT_ROOM {
                                return 0x314;
                            }
                        }
                        MOON_STAR_ISLES => {
                            if room_no != MOON_SPECTACLE_ROOM {
                                return 0x315;
                            }
                        }
                        MOON_SPECTACLE_ISLE => {
                            if room_no != MOON_STAR_BELT_ROOM {
                                return 0x316;
                            }
                        }
                        MOON_NEEDLE_ROCK => {
                            if room_no != MOON_NEEDLE_ROOM {
                                return 0x317;
                            }
                        }
                        MOON_BIRDS_PEAK => {
                            if room_no != MOON_BIRDS_PEAK_ROOM {
                                return 0x318;
                            }
                        }
                        MOON_GREAT_FISH => {
                            if room_no != MOON_THREE_EYE_ROOM {
                                return 0x319;
                            }
                        }
                        MOON_DEFAULT => {
                            if room_no != MOON_DIAMOND_ROOM {
                                return 0x31A;
                            }
                        }
                        _ => {
                            if room_no != MOON_DIAMOND_ROOM {
                                return 0x31A;
                            }
                        }
                    }
                    return 0x31B;
                }
                return 0x313;
            }
            return 0x312;
        }
        if !f.charts.has(usize::from(Self::slot(CHART_FIRST + 4))) {
            if f.stories.test(BIT_UNK_3E80) {
                return 0x31E;
            }
            return if room_no == ISLE_ROOM_NEEDLE_ROCK {
                0x31C
            } else {
                0x31D
            };
        }
        if !f.collect.test(COLLECT_TING, COLLECT_TING_ITEM) {
            if inventory::check_get_item(&f.equipped, &f.collect, ITEM_MAGIC_ARROW) {
                return 0x328;
            }
            return if f.collect.is_tact(TACT_KORL) {
                0x32A
            } else {
                0x329
            };
        }

        if !f.charts.has(usize::from(Self::slot(CHART_FIRST + 5))) {
            return if room_no == ISLE_ROOM_OUTSET {
                0x325
            } else {
                0x324
            };
        }
        if !f.charts.has(usize::from(Self::slot(CHART_FIRST + 6))) {
            return if room_no == ISLE_ROOM_STONE_WATCHER {
                0x327
            } else {
                0x326
            };
        }
        if inventory::check_get_item(&f.equipped, &f.collect, ITEM_HOOKSHOT) {
            return if room_no == ISLE_ROOM_OVERLOOK {
                0x32C
            } else {
                0x32D
            };
        }

        0x32B
    }
}

fn big_little_change(value: u32) -> u32 {
    value.swap_bytes()
}

fn distance_xz(from: Vec3, to: Vec3) -> f32 {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    (dx * dx + dz * dz).sqrt()
}

pub struct Ctx<'a> {
    pub f: &'a Facts,
    pub agb: Option<Agb>,
    pub switches: &'a mut Switches,
    pub stories: &'a mut StoryBits,
    pub active: &'a mut ActiveEvent,
    pub list: &'a EventList,
    pub events: &'a mut SimEvents,
    pub spawns: &'a mut Vec<Spawn>,
    pub dying: &'a mut Vec<Entity>,
    pub entity: Entity,
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collect: Res<'w, Collect>,
    equipped: Res<'w, Equipped>,
    charts: Res<'w, Charts>,
    chests: Res<'w, Chests>,
    mem: Res<'w, MemBits>,
    rupees: Res<'w, Rupees>,
    health: Res<'w, Health>,
    held: Res<'w, Held>,
    clock: Res<'w, Clock>,
    swim: Res<'w, Swim>,
    stage: Res<'w, super::area::StageName>,
    list: Res<'w, EventList>,
    players: Query<'w, 's, (Entity, &'static Player)>,
    ghosts: Query<'w, 's, &'static super::ghostship::Ghostship>,
    medlis: Query<'w, 's, &'static Medli>,
    makars: Query<'w, 's, &'static Makar>,
    ships: Query<'w, 's, &'static super::ship::Ship>,
    bombs: Query<'w, 's, (&'static Actor, &'static bomb::Bomb), Without<Agbsw0>>,
}

impl Facts {
    fn get_magic_is_non_null(&self) -> bool {
        true
    }

    fn triforce_num(&self) -> u8 {
        let mut num = 0_u8;
        for slot in 0..TRIFORCE_SLOTS {
            if self.collect.is_triforce(slot as u8) {
                num += 1;
            }
        }
        num
    }

    fn bait(&self, item: u8) -> bool {
        self.bait.contains(&item)
    }

    fn pendant_num(&self, item: u8) -> u8 {
        let mut count = 0_u8;
        for slot in 0..inventory::SLOT_COUNT {
            if self.equipped.item(slot) == item {
                count = count.wrapping_add(1);
            }
        }
        count
    }

    fn ocean_sv(&self, grid: usize, bit: u16) -> bool {
        self.ocean_sv
            .get(grid)
            .is_some_and(|word| word & bit != 0)
    }
}

pub fn control(
    watch: Watch<'_, '_>,
    mut switches: ResMut<Switches>,
    mut stories: ResMut<StoryBits>,
    mut active: ResMut<ActiveEvent>,
    mut events: ResMut<SimEvents>,
    info: Res<MsgGameInfo>,
    mut actors: Query<(Entity, &mut Actor, &mut Agbsw0), (With<Agbsw0>, Without<Player>)>,
    mut commands: Commands,
) {
    let Some(player_entry) = watch.players.iter().next() else {
        return;
    };
    let player_pos = Some(player_entry.1.pos);
    let player_yaw = to_s16(player_entry.1.yaw);

    let ghostship_aboard = watch.ghosts.iter().any(|ship| ship.entered);
    let (md_room, md_flying) = watch
        .medlis
        .iter()
        .next()
        .map(|medli| (medli.player_room, medli.is_flying()))
        .unwrap_or((false, false));
    let (cb1_room, cb1_flying) = watch
        .makars
        .iter()
        .next()
        .map(|makar| (makar.is_player_room(), makar.is_flying()))
        .unwrap_or((false, false));
    let ship_ride = watch.ships.iter().any(|ship| ship.riding);
    let bombs: Vec<BombView> = watch
        .bombs
        .iter()
        .map(|(actor, shell)| BombView {
            pos: actor.pos,
            mode: shell.life.mode(),
        })
        .collect();

    let held_arrow = (item::FIRST_ARROW..=item::LAST_ARROW).contains(&watch.held.item);
    let held_bomb = (item::FIRST_BOMB..=item::LAST_BOMB).contains(&watch.held.item);
    let bow_slot = watch.equipped.has(inventory::BOW_SLOT);
    let bomb_slot = watch.equipped.has(inventory::BOMB_SLOT);
    let arrow_num = if held_arrow {
        i32::from(watch.held.count)
    } else {
        0
    };
    let bomb_num = if held_bomb {
        i32::from(watch.held.count)
    } else {
        0
    };
    let arrow_max = if bow_slot { ARROW_CAP } else { 0 };
    let bomb_max = if bomb_slot { BOMB_CAP } else { 0 };
    let select_item = [inventory::NO_ITEM; ITEM_BUTTON_COUNT];

    let facts = Facts {
        stories: *stories,
        collect: *watch.collect,
        equipped: *watch.equipped,
        charts: *watch.charts,
        chests: *watch.chests,
        switches: *switches,
        mem: *watch.mem,
        rupees: watch.rupees.count,
        life: i32::from(watch.health.units),
        max_life: i32::from(watch.health.max),
        max_magic: 0,
        select_item,
        arrow_num,
        arrow_max,
        bomb_num,
        bomb_max,
        key_num: 0,
        hour: watch.clock.hour() as i32,
        moon: watch.clock.moon(),
        swim: watch.swim.active,
        ship_ride,
        winds_dir: 0,
        machete: false,
        curse: false,
        confuse: false,
        bait: [NO_BAIT; BAIT_SLOTS],
        ghostship_aboard,
        player: FPC_PLAYER,
        os_room: [false; 3],
        md_room,
        md_flying,
        cb1_room,
        cb1_flying,
        ocean_sv: [0; OCEAN_GRIDS],
        stage_m_new_d2: watch.stage.0 == STAGE_M_NEW_D2,
        agb_room: 0,
        demo_accept: false,
        event_running: info.event_run_check(),
        look_attention: watch.list.event_index(EVENT_ATTENTION),
        agb_staff: watch.list.staff_of(EVENT_STAFF),
        bombs,
        player_pos,
        player_yaw,
    };

    let mut spawns: Vec<Spawn> = Vec::new();
    let mut dying: Vec<Entity> = Vec::new();

    for (entity, mut actor, mut agb) in &mut actors {
        if !agb.entered {
            agb.entered = true;
            if !agb.ctor(&mut actor, Vec3::ONE, &facts) {
                dying.push(entity);
                continue;
            }
        }
        agb.room = actor.room;

        let mut ctx = Ctx {
            f: &facts,
            agb: if AGB_PRESENT { Some(Agb::new()) } else { None },
            switches: &mut switches,
            stories: &mut stories,
            active: &mut active,
            list: &watch.list,
            events: &mut events,
            spawns: &mut spawns,
            dying: &mut dying,
            entity,
        };
        agb.execute(&mut ctx);
        actor.pos = agb.pos;
    }

    for spawn in &spawns {
        let item = super::actor::Actor {
            name: String::from(SPAWN_ITEM_ACTOR),
            parameters: u32::from(spawn.item),
            pos: spawn.pos,
            angle: [0; 3],
            set_id: NO_SET_ID,
            room: spawn.room,
        };
        let pickup = super::pickup::Pickup::spawn(u32::from(spawn.item));
        commands.spawn((item, pickup));
    }

    for entity in dying {
        let _ = commands.entity(entity).despawn();
    }
}