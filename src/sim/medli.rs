use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::area::StageName;
use super::carry::Carry;
use super::collision::Collision;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{StoryBits, Switches};
use super::ground;
use super::water::Water;
use super::intent::{DoIntent, MoveIntent, TargetIntent};
use super::player::Player;
use super::ship::Ship;
use super::transition::{NextStage, Target};
use super::stream;
use super::walk;
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgFlags, MsgGameInfo, MsgProcs,
    fop_msgm_message_set, fop_msgm_tact_message_set,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_INPUT, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED,
    STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS, STATUS_TYPING,
};

pub const MEDLI_ACTOR: &str = "Md1";
pub const ARC_NAME: &str = "Md";
pub const ARC_NAME_SHIP: &str = "Md_ship";
pub const STAFF_NAME: &str = "Md1";
pub const PRESENT_ITEM_ACTOR: &str = "PresentItem";

pub const TALK_TYPE_SHIFT: u32 = 0x08;
pub const TALK_TYPE_MASK: u32 = 0xFF;
pub const SWITCH_SHIFT: u32 = 0x10;
pub const SWITCH_MASK: u32 = 0xFF;
pub const PRM_LO_SHIFT: u32 = 0x00;
pub const PRM_LO_MASK: u32 = 0xFF;

pub const TYPE_ATORIZK: u8 = 0;
pub const TYPE_ADANMAE: u8 = 1;
pub const TYPE_M_DRA09: u8 = 2;
pub const TYPE_SEA: u8 = 3;
pub const TYPE_EDAICHI: u8 = 4;
pub const TYPE_M_DAI: u8 = 5;
pub const TYPE_M_DAIB: u8 = 6;
pub const TYPE_SHIP_RIDE: u8 = 7;

pub const STTS_UNK1: u32 = 0x0000_0001;
pub const STTS_UNK2: u32 = 0x0000_0002;
pub const STTS_UNK4: u32 = 0x0000_0004;
pub const STTS_UNK8: u32 = 0x0000_0008;
pub const STTS_FLY: u32 = 0x0000_0010;
pub const STTS_CAM_TAG_IN: u32 = 0x0000_0020;
pub const STTS_UNK40: u32 = 0x0000_0040;
pub const STTS_UNK80: u32 = 0x0000_0080;
pub const STTS_XY_TALK: u32 = 0x0000_0100;
pub const STTS_UNK200: u32 = 0x0000_0200;
pub const STTS_UNK400: u32 = 0x0000_0400;
pub const STTS_CARRY_ACTION: u32 = 0x0000_0800;
pub const STTS_SHIP_RIDE: u32 = 0x0000_1000;
pub const STTS_LIGHT_HIT: u32 = 0x0000_2000;
pub const STTS_UNK4000: u32 = 0x0000_4000;
pub const STTS_LIGHT_BODY_HIT: u32 = 0x0000_8000;
pub const STTS_DEFAULT_TALK_XY: u32 = 0x0001_0000;
pub const STTS_UNK20000: u32 = 0x0002_0000;

pub const ATTN_LOCKON_TALK: u32 = 0x0000_0002;
pub const ATTN_ACTION_SPEAK: u32 = 0x0000_0008;
pub const ATTN_ACTION_CARRY: u32 = 0x0000_0010;
pub const ATTN_TALK_DISTANCE: u8 = 0x21;
pub const ATTN_SPEAK_DISTANCE: u8 = 0x21;
pub const ATTN_CARRY_DISTANCE: u8 = 0x27;

pub const HIO_NPC_EYE_OFFSET: f32 = -25.0;
pub const HIO_NPC_MAX_HEAD_X: i16 = 0x2328;
pub const HIO_NPC_MAX_HEAD_Y: i16 = 0x2328;
pub const HIO_NPC_MAX_BACKBONE_X: i16 = 0x0;
pub const HIO_NPC_MAX_BACKBONE_Y: i16 = 0x1F40;
pub const HIO_NPC_MIN_HEAD_X: i16 = -0x2328;
pub const HIO_NPC_MIN_HEAD_Y: i16 = -0x2328;
pub const HIO_NPC_MIN_BACKBONE_X: i16 = 0x0;
pub const HIO_NPC_MIN_BACKBONE_Y: i16 = -0x1F40;
pub const HIO_NPC_MAX_TURN_STEP: i16 = 0x1000;
pub const HIO_NPC_MAX_HEAD_TURN_VEL: i16 = 0x800;
pub const HIO_NPC_ATTN_Y_OFFSET: f32 = 130.0;
pub const HIO_NPC_MAX_ATTN_ANGLE_Y: i32 = 0x4000;
pub const HIO_NPC_MAX_ATTN_DIST_XZ: f32 = 150.0;

pub const HIO2_M04: f32 = 5.0;
pub const HIO2_M08: f32 = 17.0;
pub const HIO2_M0C: f32 = 7.0;
pub const HIO2_M10: f32 = 6.0;
pub const HIO2_M14: f32 = 0.7;
pub const HIO2_M18: f32 = 2.6;
pub const HIO2_M1C: f32 = 0.6;
pub const HIO2_M20: i16 = 3300;
pub const HIO2_M22: i16 = 2900;
pub const HIO2_M24: i16 = 5;
pub const HIO2_M26: i16 = 0x19;
pub const HIO2_M28: i16 = 0x7FFF;
pub const HIO2_M2A: i16 = 1;

pub const HIO3_M04: f32 = 0.8;
pub const HIO3_M08: f32 = 0.2;
pub const HIO3_M0C: f32 = -5.0;
pub const HIO3_M10: f32 = -5.0;
pub const HIO3_M14: f32 = 0.0;
pub const HIO3_M1E: i16 = 3800;
pub const HIO3_M20: i16 = 5800;
pub const HIO3_M22: i16 = 0x7FFF;
pub const HIO3_M24: i16 = -0x7FFF;

pub const HIO4_M4: f32 = 86.0;
pub const HIO5_M4: f32 = 16.0;
pub const HIO5_M8: f32 = 0.6;

pub const HIO6_M04: f32 = 500.0;
pub const HIO6_M08: i16 = 430;
pub const HIO6_M0A: i16 = 9000;
pub const HIO6_M0C: i16 = -5000;
pub const HIO6_M0E: i16 = -9000;
pub const HIO6_M10: i16 = 0;
pub const HIO6_M12: i16 = 0;
pub const HIO6_M14: i16 = 0;
pub const HIO6_M16: i16 = 0;

pub const HIO_M0B8: f32 = 770.0;
pub const HIO_M0BC: f32 = 250.0;
pub const HIO_M0C0: f32 = -450.0;
pub const HIO_M0C4: f32 = 90.0;
pub const HIO_M0C8: f32 = 400.0;
pub const HIO_M0CC: f32 = 600.0;
pub const HIO_M0D4: f32 = 17.0;
pub const HIO_M0D8: f32 = 0.0;
pub const HIO_M0DC: f32 = 0.5;
pub const HIO_M0E8: f32 = 15.0;
pub const HIO_M0EC: f32 = 22.0;
pub const HIO_M0F0: f32 = 9.0;
pub const HIO_M0F4: f32 = 4.0;
pub const HIO_M0F8: f32 = -1.51367;
pub const HIO_M0FC: f32 = -1.09863;
pub const HIO_M100: f32 = -0.56152;
pub const HIO_M104: f32 = 0.1;
pub const HIO_M108: f32 = 0.2;
pub const HIO_M10C: f32 = 10.0;
pub const HIO_M110: f32 = -50.0;
pub const HIO_M114: f32 = 1.3;
pub const HIO_M118: f32 = 1.0;
pub const HIO_M11C: f32 = 1.5;
pub const HIO_M120: f32 = 2.0;
pub const HIO_M124: f32 = 1.6;
pub const HIO_M130: f32 = 3.0;
pub const HIO_M134: f32 = 1.25;
pub const HIO_M138: f32 = 0.2;
pub const HIO_M13C: f32 = 100.0;
pub const HIO_M140: f32 = 50.0;
pub const HIO_M144: f32 = 9.0;
pub const HIO_M148: f32 = 0.9;
pub const HIO_M150: f32 = 10.0;
pub const HIO_M154: f32 = 5.0;
pub const HIO_M188: f32 = 1.0;
pub const HIO_M18C: f32 = -10.0;
pub const HIO_M190: f32 = 43.0;
pub const HIO_M194: f32 = 14.65;
pub const HIO_M198: f32 = 13.99;
pub const HIO_M1A8: f32 = 3.0;
pub const HIO_M1AC: f32 = 1000.0;
pub const HIO_M1B0: f32 = 1000.0;
pub const HIO_M1B4: i16 = 0x17;
pub const HIO_M1B6: i16 = 0x14;
pub const HIO_M1BA: i16 = 0x78;
pub const HIO_M1BC: i16 = 0x1F4;
pub const HIO_M1BE: i16 = 0x46;
pub const HIO_M1C0: i16 = 0x5;
pub const HIO_M1C4: i16 = 0x96;
pub const HIO_M1C6: u8 = 0x1;
pub const HIO_M1C7: u8 = 0x0;

pub const HEIGHT: f32 = 120.0;
pub const HEIGHT_SQUAT: f32 = 75.0;
pub const MAX_FALL: f32 = -100.0;
pub const CYL_RADIUS: f32 = 30.0;
pub const CPS_RADIUS: f32 = 20.0;
pub const ACCH_ROOF_HEIGHT: f32 = 120.0;
pub const ACCH_CIR0_WALL: [f32; 2] = [20.0, 20.0];
pub const ACCH_CIR1_WALL: [f32; 2] = [60.0, 20.0];
pub const ACCH_CIR1_WALL_FLY: [f32; 2] = [60.0, 60.0];

pub const ANM_WAIT: i32 = 0x00;
pub const ANM_WALK: i32 = 0x01;
pub const ANM_SQUATDOWN: i32 = 0x02;
pub const ANM_SQWAIT01: i32 = 0x03;
pub const ANM_SQWAIT02: i32 = 0x04;
pub const ANM_FIRING: i32 = 0x05;
pub const ANM_FLY01: i32 = 0x06;
pub const ANM_FLY02: i32 = 0x07;
pub const ANM_GLIDING: i32 = 0x08;
pub const ANM_FALL: i32 = 0x09;
pub const ANM_FALL02: i32 = 0x0A;
pub const ANM_LAND: i32 = 0x0B;
pub const ANM_LAND02: i32 = 0x0C;
pub const ANM_LAND03: i32 = 0x0D;
pub const ANM_KAMAEWAIT: i32 = 0x0E;
pub const ANM_KAMAE: i32 = 0x0F;
pub const ANM_PIYO2: i32 = 0x10;
pub const ANM_OJIGI: i32 = 0x11;
pub const ANM_HARPPLAY01: i32 = 0x12;
pub const ANM_HARPPLAY02: i32 = 0x13;
pub const ANM_TAKEOFF01: i32 = 0x14;
pub const ANM_TAKEOFF02: i32 = 0x15;
pub const ANM_WATASU: i32 = 0x16;
pub const ANM_TALK01: i32 = 0x17;
pub const ANM_TALK02: i32 = 0x18;
pub const ANM_TALK03: i32 = 0x19;
pub const ANM_TALK04: i32 = 0x1A;
pub const ANM_MKAMAE: i32 = 0x1B;
pub const ANM_MWAIT: i32 = 0x1C;
pub const ANM_ORIWAIT: i32 = 0x1D;
pub const ANM_SHIPWAIT: i32 = 0x1E;
pub const ANM_CAUGHT01: i32 = 0x1F;
pub const ANM_CAUGHT02: i32 = 0x20;
pub const ANM_KYOHI: i32 = 0x21;
pub const ANM_KYORO2: i32 = 0x22;
pub const ANM_SHIPWAIT02: i32 = 0x23;
pub const ANM_SHIPTALK01: i32 = 0x24;
pub const ANM_TALK05: i32 = 0x25;
pub const ANM_UNAZUKI: i32 = 0x26;
pub const ANM_COUNT: usize = 50;

pub const ANM_TBL_IDX: [u8; ANM_COUNT] = [
    0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x06, 0x07,
    0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x0F,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19,
    0x1A, 0x1B, 0x1C, 0x1B, 0x1D, 0x1E, 0x1F, 0x20, 0x17, 0x21,
    0x22, 0x1A, 0x23, 0x24, 0x25, 0x25, 0x07, 0x04, 0x00, 0x26,
];

pub const ANM_TBL_ARM: [u8; ANM_COUNT] = [
    0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x81, 0x82, 0x82, 0x83,
    0x83, 0x84, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0B,
    0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x85, 0x11, 0x12, 0x13, 0x14,
    0x15, 0x16, 0x17, 0x16, 0x18, 0x19, 0x86, 0x87, 0x12, 0x1A,
    0x1B, 0x15, 0x1C, 0x1D, 0x1E, 0x1E, 0x83, 0x04, 0x00, 0x1F,
];

pub const ANM_TBL_BTP: [u8; ANM_COUNT] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
    0x03, 0x00, 0x01, 0x02, 0x06, 0x07, 0x06, 0x05, 0x04, 0x04,
    0x08, 0x00, 0x09, 0x09, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x0C, 0x0C, 0x05, 0x0D,
    0x00, 0x05, 0x00, 0x00, 0x00, 0x05, 0x03, 0x05, 0x00, 0x0F,
];

pub const ANM_TBL_LOOP: [bool; ANM_COUNT] = [
    true, false, true, false, true, true, false, true, false, true,
    false, true, true, true, false, false, false, true, false, false,
    true, false, true, true, false, false, false, true, true, true,
    true, false, true, false, true, true, true, true, true, true,
    false, false, true, true, true, true, true, true, true, false,
];

pub const ANM_TBL_MORF: [f32; ANM_COUNT] = [
    8.0, 8.0, 4.0, 4.0, 4.0, 4.0, 0.0, 3.0, 3.0, 8.0,
    8.0, 4.0, 8.0, 8.0, 0.0, 0.0, 0.0, 8.0, 8.0, 8.0,
    8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0,
    8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0,
    8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 4.0, 0.0, 8.0,
];

pub const ANM_TBL_SPEED: [f32; ANM_COUNT] = [
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.4, 0.0, -1.4, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
];

pub const HARP_PLAY_ANM: [i32; 6] = [0x16, 0x17, 0x16, 0x17, 0x16, 0x17];
pub const HARP_SE: [u32; 12] = [
    0x5940, 0x5941, 0x5942, 0x5943, 0x5944, 0x5940, 0x5941, 0x5942, 0x5943, 0x5944, 0x5944, 0x5944,
];
pub const HARP_FRAME_11: f32 = 11.0;
pub const HARP_FRAME_35: f32 = 35.0;
pub const HARP_FRAME_38: f32 = 38.0;

pub const SE_MIRROR_REFLECT: u32 = 0x6961;
pub const SE_MIRROR_LIGHT: u32 = 0x7028;
pub const SE_CV_MD_LIFT_UP: u32 = 0x48A5;
pub const SE_CV_MD_THROW: u32 = 0x48A6;
pub const SE_CV_MD_LANDING: u32 = 0x48A7;
pub const SE_CV_MD_CRASH: u32 = 0x48A9;
pub const SE_CV_MD_LEFT_ALONE: u32 = 0x491E;
pub const SE_CV_MD_FLY_WITH_LINK: u32 = 0x491F;
pub const SE_CV_MD_FLY_END: u32 = 0x4920;
pub const SE_CM_MD_PIYO: u32 = 0x50BC;
pub const SE_CM_MD_HARP_SET_UP: u32 = 0x59A1;
pub const SE_CM_MD_HARP_TAKE_OFF: u32 = 0x59A2;
pub const SE_CTRL_NPC_TO_LINK: u32 = 0x0886;
pub const BGM_MEDORI_TAKT_8: u32 = 0x8000_0060;

pub const BIT_1101: u16 = 0x1101;
pub const BIT_1820: u16 = 0x1820;
pub const BIT_2920: u16 = 0x2920;
pub const BIT_1102: u16 = 0x1102;
pub const BIT_1104: u16 = 0x1104;
pub const BIT_1140: u16 = 0x1140;
pub const BIT_1280: u16 = 0x1280;
pub const BIT_1402: u16 = 0x1402;
pub const BIT_1504: u16 = 0x1504;
pub const BIT_1608: u16 = 0x1608;
pub const BIT_1620: u16 = 0x1620;
pub const BIT_2C08: u16 = 0x2C08;
pub const BIT_2E04: u16 = 0x2E04;
pub const BIT_2E40: u16 = 0x2E40;
pub const BIT_3310: u16 = 0x3310;
pub const BIT_3320: u16 = 0x3320;
pub const BIT_3B80: u16 = 0x3B80;
pub const BIT_4001: u16 = 0x4001;
pub const BIT_4180: u16 = 0x4180;
pub const BIT_GAVE_LETTER: u16 = 0x0E02;

pub const ITEM_WIND_WAKER: u8 = 0x22;
pub const ITEM_GOLDEN_FEATHER: u8 = 0x47;
pub const ITEM_GRAPPLING_HOOK: u8 = 0x25;
pub const ITEM_EMPTY_BOTTLE: u8 = 0x50;
pub const ITEM_FATHER_LETTER: u8 = 0x98;

pub const EVENT_ITEM_GET: &str = "Md_ItemGet";
pub const EVENT_ROPE_GET: &str = "Md_RopeGet";
pub const EVENT_MD_FLY: &str = "MD_FLY";
pub const EVENT_MD_CLIFF: &str = "md_cliff";
pub const EVENT_FLY2: &str = "Md_Fly2";
pub const EVENT_TACT: &str = "Md_Tact";
pub const EVENT_TACT_TRUE: &str = "Md_TactTrue";
pub const EVENT_HARP_TALK: &str = "Md_HarpTalk";
pub const EVENT_OPTION_CHAR_END: &str = "OPTION_CHAR_END";
pub const EVENT_DEFAULT_TALK_XY: &str = "DEFAULT_TALK_XY";

pub const EVENT_NAMES: [&str; EVENT_SLOTS] = [
    EVENT_ITEM_GET,
    EVENT_ROPE_GET,
    EVENT_MD_FLY,
    EVENT_MD_CLIFF,
    EVENT_FLY2,
    EVENT_TACT,
    EVENT_TACT_TRUE,
    EVENT_HARP_TALK,
    EVENT_OPTION_CHAR_END,
    EVENT_DEFAULT_TALK_XY,
];

pub const CUT_NAMES: [&str; 21] = [
    "WAIT",
    "CREATEITEM",
    "MSG_SET",
    "MSG_END",
    "POS_MOV",
    "FLY",
    "GLIDING",
    "LANDING",
    "WALK",
    "DASH",
    "END",
    "TACT",
    "TAKEOFF",
    "Onetime",
    "QUAKE",
    "HARPPLAY",
    "OFF_PLAYER",
    "ON_PLAYER",
    "TURN",
    "SET_ANM",
    "LOOK_DOWN",
];
pub const CUT_LOOK_UP: &str = "LOOK_UP";

pub fn cut_of(name: &str) -> Option<Cut> {
    if name == CUT_LOOK_UP {
        return Some(Cut::LookUp);
    }
    CUT_NAMES
        .iter()
        .position(|cut| *cut == name)
        .and_then(|index| CUTS.get(index).copied())
}
pub const CUT_SLOTS: usize = 22;

pub const MSG_TACT: u32 = 0x5AC;
pub const MSG_SHIP_TALK: u32 = 0x19EF;
pub const MSG_LETTER_SEA: u32 = 0x17DC;
pub const MSG_LETTER_ATORIZK_CLEAR: u32 = 0x1805;
pub const MSG_LETTER_FIRST: u32 = 0x17D5;
pub const MSG_ADANMAE_PIYO_TALK: u32 = 0x17F6;
pub const MSG_ADANMAE_HARP: u32 = 0x17F7;
pub const MSG_ADANMAE_FIRST: u32 = 0x17E6;
pub const MSG_DRA09_HARP: u32 = 0x17E5;
pub const MSG_DRA09_FIRST: u32 = 0x17DD;
pub const MSG_SEA_FEATHER_DONE: u32 = 0x1A00;
pub const MSG_SEA_FEATHER_SEEN: u32 = 0x19F8;
pub const MSG_SEA_FIRST: u32 = 0x19FC;
pub const MSG_SEA_TACT_TRUE: u32 = 0x19D9;
pub const MSG_SEA_TACT: u32 = 0x19D6;
pub const MSG_SEA_HARP_TALK: u32 = 0x19D2;
pub const MSG_SEA_HARP_WALK: u32 = 0x19F6;
pub const MSG_SEA_OPEN: u32 = 0x19C9;
pub const MSG_FIELD_FIRST: u32 = 0x1A02;
pub const MSG_TACT_MATCH: u32 = 0x19DD;
pub const MSG_TACT_MISS: u32 = 0x19DA;
pub const MSG_TACT_STOP: u32 = 0x19DC;

pub const STEP_STARTING: i8 = 0;
pub const STEP_ONGOING_1: i8 = 1;
pub const STEP_ONGOING_2: i8 = 2;
pub const STEP_ONGOING_3: i8 = 3;
pub const STEP_ONGOING_4: i8 = 4;
pub const STEP_ENDING: i8 = -1;

pub const MODE_TALK: i8 = 1;
pub const MODE_TALK_ITEM: i8 = 2;
pub const MODE_TALK_RETRY: i8 = 3;
pub const MODE_EVENT0: i8 = 4;
pub const MODE_EVENT1: i8 = 5;
pub const MODE_EVENT2: i8 = 6;
pub const MODE_EVENT3: i8 = 7;
pub const MODE_EVENT4: i8 = 8;
pub const MODE_EVENT5: i8 = 9;
pub const MODE_EVENT6: i8 = 10;
pub const MODE_EVENT7: i8 = 11;
pub const MODE_EVENT8: i8 = 12;

pub const EVENT_SLOTS: usize = 10;
pub const NO_STATUS: u8 = 0xFF;
pub const NO_INDEX: i16 = -1;
pub const NO_STAFF: usize = usize::MAX;
pub const NO_EVENT: usize = usize::MAX;

pub const PY_STTS0_800000: u32 = 0x0080_0000;
pub const PIYO2_TALK_MAX: u8 = 3;
pub const FLY_MAX_TIMER: i16 = 15 * 30;
pub const HARP_RETRY_FRAMES: i16 = 0x0C;
pub const HARP_RESUME_FRAMES: i16 = 0x78;
pub const ESCAPE_FRAMES: i16 = 0x1E;
pub const PIYO_FRAMES: i16 = 0x78;
pub const WAIT_HARP_FRAMES: i16 = 150;
pub const DEMO_FLY_DELETE: i16 = 0x3C;
pub const DEMO_FLY_POS: Vec3 = Vec3::new(-1511.0, 14500.0, 150.0);
pub const DEMO_FLY_ALT: f32 = 14500.0;
pub const DEMO_FLY_ANM_READY: i32 = 0x18;
pub const DEMO_FLY_ANM_WING: i32 = 0x19;
pub const DEMO_FLY_ANM_GLIDE: i32 = 0x0B;
pub const DEMO_FLY_ANM_UP: i32 = 0x0A;
pub const FLY_CEILING_LIFT: f32 = 1500.0;
pub const DEMO_FLY_YAW: i16 = 0x5700;
pub const DEMO_FLY_SPEED: f32 = 17.0;

pub const GLIDE_STEPS: i16 = 2;
pub const GLIDE_WING_FRAME: f32 = 23.0;
pub const WIND_ABS_MIN: f32 = 1e-6;
pub const WIND_DOT_MIN: f32 = 0.7071;
pub const ROUTE_PROBE_UP: f32 = 80.0;
pub const ROUTE_DROP_LIMIT: f32 = -100.0;
pub const ROUTE_SQUARE_LIMIT: f32 = 600.0;
pub const ROUTE_DIST_LIMIT: f32 = -10000000.0;
pub const ROUTE_JUMP_FLOOR: f32 = 0.0;
pub const ROUTE_JUMP_MAX: f32 = 80.0;
pub const ROUTE_JUMP_SCALE: f32 = 3.6;
pub const ROUTE_SNAP_MIN: f32 = -30.1;
pub const ROUTE_SNAP_MAX: f32 = 0.001;
pub const SEARCH_SPEED_MIN: f32 = 0.5;
pub const SEARCH_SPEED_MAX: f32 = 90000.0;
pub const SEARCH_SPEED_DIV: f32 = 90000.0;
pub const SEARCH_ANGLE_TOL: i16 = 0x2000;
pub const SEARCH_RND_MIN: i32 = 8;
pub const SEARCH_RND_SPAN: i32 = 20;
pub const HARP_RND_SPAN: i32 = 4;
pub const HARP_SE_RND_SPAN: i32 = 10;

pub const VELOCITY_EPS: f32 = 0.001;
pub const SPEED_EPS: f32 = 0.05;
pub const IS_ZERO: f32 = 8e-11;
pub const NORMALIZE_EPS: f32 = 1e-4;
pub const HEAD_SCALE: i16 = 4;
pub const HEAD_MAX_STEP: i16 = 0x800;
pub const TURN_SCALE: i16 = 8;
pub const TURN_MAX_STEP: i16 = 0x2000;
pub const TURN_MIN_STEP: i16 = 0x400;
pub const WAIST_SCALE: i16 = 4;
pub const WAIST_MAX_STEP: i16 = 0xC00;
pub const WAIST_MIN_STEP: i16 = 0x180;
pub const TILT_SCALE: i16 = 8;
pub const TILT_MAX_STEP: i16 = 0x2000;
pub const TILT_MIN_STEP: i16 = 0x400;
pub const TILT_BEND_LIMIT: i16 = 0x6000;
pub const TILT_BEND_MAX: i16 = 0x2000;
pub const TILT_SIDE: i16 = -0x2000;

pub const CARRY_TUNE_PITCH: f32 = 0.0;
pub const CARRY_HARP_X: f32 = 0.37;
pub const CARRY_HARP_Y: f32 = 3.81;
pub const CARRY_HARP_Z: f32 = -11.13;
pub const CARRY_HARP_ROT_X: i16 = 0x1B24;
pub const CARRY_HARP_ROT_Y: i16 = -0x2913;
pub const CARRY_HARP_ROT_Z: i16 = -0x1982;
pub const BODY_HARP_X: f32 = 2.2;
pub const BODY_HARP_Y: f32 = -11.681;
pub const BODY_HARP_Z: f32 = 1.3;
pub const BODY_HARP_ROT_X: i16 = 0x3F11;
pub const BODY_HARP_ROT_Y: i16 = 0x2894;
pub const BODY_HARP_ROT_Z: i16 = 0x7F7E;
pub const BASE_LIFT: f32 = 95.0;
pub const SHIP_OFFSET_X: f32 = HIO_M190;
pub const SHIP_OFFSET_Y: f32 = HIO_M194;
pub const SHIP_OFFSET_Z: f32 = HIO_M198;
pub const SHIP_ROT_Y: i16 = -0x45B0;
pub const SHIP_TURN: i16 = -0x4000;

pub const ATTN_BONUS_DIST: f32 = 40.0;
pub const ATTN_BONUS_ANGLE: i32 = 0x71C;
pub const PIYO_TALK_ANIM: i32 = 0x1B;
pub const PIYO_TALK_ATTR: u8 = 0x14;
pub const SHADOW_LIFT: f32 = 150.0;
pub const SHADOW_RADIUS: f32 = 800.0;
pub const SHADOW_DEPTH: f32 = 20.0;
pub const NORMAL_EPS: f32 = 0.001;
pub const ANGLE_HALF: i16 = 0x4000;
pub const DUST_SCALE_X: f32 = 1.25;
pub const DUST_SCALE_Y: f32 = 1.5;
pub const DUST_SCALE_Z: f32 = 1.0;
pub const FALL_COUNT_MAX: u8 = 30;
pub const GROUND_CODE_LIMIT: i32 = 4;
pub const DEMO_RANGE_LO: f32 = 600.0;
pub const DEMO_RANGE_HI: f32 = 700.0;
pub const LOOK_DOWN_OFFSET: f32 = -50.0;
pub const TURN_JOINT_MIN_Y: i16 = -500;

pub const NEXT_STAGE_SEA: &str = "sea";
pub const NEXT_STAGE_EXIT: u8 = 0xE3;
pub const NEXT_STAGE_ROOM: u8 = 13;
pub const NEXT_STAGE_START: u8 = 8;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
pub const RND_SEED: i32 = 100;

pub const MSG_ANM: [i32; 44] = [
    0x00, 0x02, 0x02, 0x03, 0x04, 0x05, 0x06, 0x0B, 0x07, 0x09, 0x0C, 0x0D, 0x0E, 0x0F, 0x11, 0x14,
    0x12, 0x1B, 0x1C, 0x15, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1D, 0x1A, 0x10, 0x1F, 0x20, 0x22, 0x1E,
    0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x31,
];

pub const PIYO2_MSG: [u32; 3] = [0x17F5, 0x1803, 0x1804];

pub const BTP_REPEAT_MIN: f32 = 30.0;
pub const BTP_REPEAT_SPAN: f32 = 60.0;

pub fn is_medli(name: &str) -> bool {
    name == MEDLI_ACTOR
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    None,
    Wait,
    HarpWait,
    Talk,
    ShipTalk,
    Kyohi,
    Ship,
    Mwait,
    Squatdown,
    Sqwait01,
    Carry,
    Throw,
    Gliding,
    Fall,
    Fall02,
    WallHit,
    Land01,
    Land02,
    Land03,
    Piyo2,
    Delete,
    DemoFly,
    Search,
    Hit,
    Jump,
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStep {
    None,
    Wait,
    Walk,
    Hit,
    Jump,
    Fly,
    Land,
    Mkamae,
    Carry,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Arg {
    None,
    Speed(f32),
    Angle(i16),
    Flag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cut {
    Wait,
    CreateItem,
    MsgSet,
    MsgEnd,
    PosMov,
    Fly,
    Gliding,
    Landing,
    Walk,
    Dash,
    End,
    Tact,
    TakeOff,
    Onetime,
    Quake,
    HarpPlay,
    OffPlayer,
    OnPlayer,
    Turn,
    SetAnm,
    LookDown,
    LookUp,
}

pub const CUTS: [Cut; CUT_SLOTS] = [
    Cut::Wait,
    Cut::CreateItem,
    Cut::MsgSet,
    Cut::MsgEnd,
    Cut::PosMov,
    Cut::Fly,
    Cut::Gliding,
    Cut::Landing,
    Cut::Walk,
    Cut::Dash,
    Cut::End,
    Cut::Tact,
    Cut::TakeOff,
    Cut::Onetime,
    Cut::Quake,
    Cut::HarpPlay,
    Cut::OffPlayer,
    Cut::OnPlayer,
    Cut::Turn,
    Cut::SetAnm,
    Cut::LookDown,
    Cut::LookUp,
];

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Medli {
    pub entered: bool,
    pub talk_type: u8,
    pub switch_no: u8,
    pub prm_lo: u8,
    pub stts: u32,
    pub attn_flags: u32,
    pub effect: u8,
    pub step: Step,
    pub step_stat: i8,
    pub arg: Arg,
    pub player_step: PlayerStep,
    pub player_stat: i8,
    pub anm: i32,
    pub anm_body: i8,
    pub anm_frame: f32,
    pub anm_start: f32,
    pub anm_running: bool,
    pub anm_rate: f32,
    pub run_rate: f32,
    pub btp_tbl: u8,
    pub btp_repeat: f32,
    pub btp_looping: bool,
    pub msg_no: u32,
    pub msg_id: u32,
    pub msg_attr: u8,
    pub mode: i8,
    pub cur_event: i8,
    pub event_idx: [i16; EVENT_SLOTS],
    pub accepted: bool,
    pub return_link: bool,
    pub npc_not_change: bool,
    pub call_command: bool,
    pub staff_idx: usize,
    pub draw_off: bool,
    pub home: Vec3,
    pub home_yaw: i16,
    pub eye: Vec3,
    pub attn_base: Vec3,
    pub attn_pos: Vec3,
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub max_fall: f32,
    pub height: f32,
    pub offset: Vec3,
    pub normal: Vec3,
    pub start_pos: Vec3,
    pub turn_offset: Vec3,
    pub turn_pos: Vec3,
    pub block_pos: Vec3,
    pub block_dir: i8,
    pub max_speed: f32,
    pub walk_speed: f32,
    pub waist_speed: f32,
    pub turn_vel: i16,
    pub waist_rot_x: i16,
    pub waist_rot_y: i16,
    pub waist_angle: i16,
    pub wait_timer: i16,
    pub anim_ended: bool,
    pub attn_count: u8,
    pub attn_seen: bool,
    pub npc_flip: u8,
    pub search_timer: i16,
    pub was_airborne: u8,
    pub piyo_count: u8,
    pub harp_delay: u8,
    pub harp_se: u32,
    pub old_light_body_hit: bool,
    pub timer_a: i16,
    pub timer_b: i16,
    pub glide_flag: i16,
    pub glide_timer: i16,
    pub wind_flag: i16,
    pub wind_timer: i16,
    pub start_y: f32,
    pub wind_ramp: f32,
    pub jnt_angles: [[i16; 2]; 2],
    pub jnt_min: [[i16; 2]; 2],
    pub jnt_max: [[i16; 2]; 2],
    pub jnt_step: [[i16; 2]; 2],
    pub jnt_trn: bool,
    pub light_seen: bool,
    pub wind_vec: Vec3,
    pub wind_power: f32,
    pub home_room: i8,
    pub room_visible: bool,
    pub ship_head_hidden: bool,
    pub flying: bool,
    pub mirror: bool,
    pub sea_talk: bool,
    pub player_room: bool,
    pub flying_timer: i16,
    pub seed: [i32; 3],
    pub cond_talk: bool,
    pub do_held: bool,
    pub ground_miss: u8,
    pub a_button: bool,
    pub b_button: bool,
    pub pos: Vec3,
    pub yaw: i16,
    pub shape: [i16; 3],
    pub ground_hit: bool,
    pub ground_y: f32,
    pub old: Vec3,
    pub link: Option<Vec3>,
    pub link_hidden: bool,
    pub pending_se: u32,
    pub stick_angle: i16,
    pub player_yaw: i16,
    pub player_shape: i16,
    pub frame_ended: bool,
    pub ripple: bool,
}

impl Medli {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            entered: false,
            talk_type: ((parameters >> TALK_TYPE_SHIFT) & TALK_TYPE_MASK) as u8,
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
            prm_lo: ((parameters >> PRM_LO_SHIFT) & PRM_LO_MASK) as u8,
            stts: 0,
            attn_flags: ATTN_LOCKON_TALK | ATTN_ACTION_SPEAK,
            effect: 0,
            step: Step::None,
            step_stat: STEP_STARTING,
            arg: Arg::None,
            player_step: PlayerStep::None,
            player_stat: STEP_STARTING,
            anm: 0,
            anm_body: -1,
            anm_frame: 0.0,
            anm_start: 0.0,
            anm_running: false,
            anm_rate: 1.0,
            run_rate: 0.0,
            btp_tbl: 0,
            btp_repeat: 0.0,
            btp_looping: false,
            msg_no: 0,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            msg_attr: NO_STATUS,
            mode: 0,
            cur_event: -1,
            event_idx: [NO_INDEX; EVENT_SLOTS],
            accepted: false,
            return_link: false,
            npc_not_change: false,
            call_command: false,
            staff_idx: NO_STAFF,
            draw_off: false,
            home: Vec3::ZERO,
            home_yaw: 0,
            eye: Vec3::ZERO,
            attn_base: Vec3::ZERO,
            attn_pos: Vec3::ZERO,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: HIO_M0F4,
            max_fall: MAX_FALL,
            height: HEIGHT,
            offset: Vec3::ZERO,
            normal: Vec3::ZERO,
            start_pos: Vec3::ZERO,
            turn_offset: Vec3::new(0.0, 0.0, 50.0),
            turn_pos: Vec3::ZERO,
            block_pos: Vec3::ZERO,
            block_dir: 0,
            max_speed: HIO2_M08,
            walk_speed: 0.0,
            waist_speed: 0.0,
            turn_vel: 0,
            waist_rot_x: 0,
            waist_rot_y: 0,
            waist_angle: 0,
            wait_timer: 0,
            anim_ended: false,
            attn_count: 0,
            attn_seen: false,
            npc_flip: 0,
            search_timer: 0,
            was_airborne: 0,
            piyo_count: 0,
            harp_delay: 0,
            harp_se: HARP_SE[0],
            old_light_body_hit: false,
            timer_a: 0,
            timer_b: 0,
            glide_flag: 0,
            glide_timer: 0,
            wind_flag: 0,
            wind_timer: 0,
            start_y: 0.0,
            wind_ramp: 0.0,
            jnt_angles: [[0; 2]; 2],
            jnt_min: [
                [HIO_NPC_MIN_HEAD_X, HIO_NPC_MIN_HEAD_Y],
                [HIO_NPC_MIN_BACKBONE_X, HIO_NPC_MIN_BACKBONE_Y],
            ],
            jnt_max: [
                [HIO_NPC_MAX_HEAD_X, HIO_NPC_MAX_HEAD_Y],
                [HIO_NPC_MAX_BACKBONE_X, HIO_NPC_MAX_BACKBONE_Y],
            ],
            jnt_step: [[HIO_NPC_MAX_TURN_STEP; 2]; 2],
            jnt_trn: false,
            light_seen: false,
            wind_vec: Vec3::ZERO,
            wind_power: 0.0,
            home_room: -1,
            room_visible: true,
            ship_head_hidden: false,
            flying: false,
            mirror: false,
            sea_talk: false,
            player_room: false,
            flying_timer: 0,
            seed: [RND_SEED; 3],
            cond_talk: false,
            do_held: false,
            ground_miss: 0,
            a_button: false,
            b_button: false,
            pos: Vec3::ZERO,
            yaw: 0,
            shape: [0; 3],
            ground_hit: false,
            ground_y: f32::NEG_INFINITY,
            old: Vec3::ZERO,
            link: None,
            link_hidden: false,
            pending_se: 0,
            stick_angle: 0,
            player_yaw: 0,
            player_shape: 0,
            frame_ended: false,
            ripple: false,
        }
    }

    pub fn is_type_atorizk(&self) -> bool {
        self.talk_type == TYPE_ATORIZK
    }

    pub fn is_type_adanmae(&self) -> bool {
        self.talk_type == TYPE_ADANMAE
    }

    pub fn is_type_dra09(&self) -> bool {
        self.talk_type == TYPE_M_DRA09
    }

    pub fn is_type_sea(&self) -> bool {
        self.talk_type == TYPE_SEA
    }

    pub fn is_type_edaichi(&self) -> bool {
        self.talk_type == TYPE_EDAICHI
    }

    pub fn is_type_m_dai(&self) -> bool {
        self.talk_type == TYPE_M_DAI
    }

    pub fn is_type_m_daib(&self) -> bool {
        self.talk_type == TYPE_M_DAIB
    }

    pub fn is_type_ship_ride(&self) -> bool {
        self.talk_type == TYPE_SHIP_RIDE
    }

    pub fn is_type_field(&self) -> bool {
        self.is_type_m_dai() || self.is_type_edaichi()
    }

    pub fn set_stts(&mut self, bits: u32) {
        self.stts |= bits;
    }

    pub fn clr_stts(&mut self, bits: u32) {
        self.stts &= !bits;
    }

    pub fn has_stts(&self, bits: u32) -> bool {
        self.stts & bits != 0
    }

    pub fn is_flying(&self) -> bool {
        self.flying
    }

    pub fn is_mirror(&self) -> bool {
        self.mirror
    }

    pub fn is_sea_talk(&self) -> bool {
        self.sea_talk
    }

    pub fn is_player_room(&self) -> bool {
        self.player_room
    }

    pub fn is_xy_talk(&self) -> bool {
        self.has_stts(STTS_XY_TALK)
    }

    pub fn is_default_talk_xy(&self) -> bool {
        self.has_stts(STTS_DEFAULT_TALK_XY)
    }

    pub fn is_ship_ride(&self) -> bool {
        self.has_stts(STTS_SHIP_RIDE)
    }

    pub fn is_light_hit(&self) -> bool {
        self.has_stts(STTS_LIGHT_HIT)
    }

    pub fn is_light_body_hit(&self) -> bool {
        self.has_stts(STTS_LIGHT_BODY_HIT)
    }

    pub fn is_cam_tag_in(&self) -> bool {
        self.has_stts(STTS_CAM_TAG_IN)
    }

    pub fn is_no_carry(&self) -> bool {
        self.has_stts(STTS_CARRY_ACTION)
    }

    pub fn is_event_accepted(&self) -> bool {
        self.accepted
    }

    pub fn drawn(&self) -> bool {
        if self.is_ship_ride() {
            return !self.ship_head_hidden;
        }
        self.home_room >= 0 && self.room_visible
    }

    pub fn head_x(&self) -> i16 {
        self.jnt_angles[0][0]
    }

    pub fn head_y(&self) -> i16 {
        self.jnt_angles[0][1]
    }

    pub fn backbone_x(&self) -> i16 {
        self.jnt_angles[1][0]
    }

    pub fn backbone_y(&self) -> i16 {
        self.jnt_angles[1][1]
    }

    pub fn waist_x(&self) -> i16 {
        self.waist_rot_x
    }

    pub fn waist_y(&self) -> i16 {
        self.waist_rot_y
    }

    pub fn set_attn_bit(&mut self, bits: u32) {
        self.attn_flags |= bits;
    }

    pub fn clr_attn_bit(&mut self, bits: u32) {
        self.attn_flags &= !bits;
    }

    pub fn inc_attn_count(&mut self) {
        if self.attn_count != 0xFF {
            self.attn_count += 1;
        }
    }

    pub fn count_piyo_talk(&mut self) {
        self.piyo_count = self.piyo_count.wrapping_add(1);
        if self.piyo_count >= PIYO2_TALK_MAX {
            self.piyo_count = 0;
        }
    }

    pub fn calc_timer(value: &mut i16) -> bool {
        if *value != 0 {
            *value -= 1;
        }
        *value == 0
    }

    pub fn clear_jnt_ang(&mut self) {
        self.jnt_angles[0][1] = 0;
        self.jnt_angles[0][0] = 0;
        self.jnt_angles[1][1] = 0;
        self.jnt_angles[1][0] = 0;
    }
}

pub fn angle_distance(a: i16, b: i16) -> u16 {
    a.wrapping_sub(b).unsigned_abs()
}

pub fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

pub fn abs2(v: Vec3) -> f32 {
    v.x * v.x + v.y * v.y + v.z * v.z
}

pub fn abs2_xz(v: Vec3) -> f32 {
    v.x * v.x + v.z * v.z
}

pub fn abs_xz(v: Vec3) -> f32 {
    abs2_xz(v).sqrt()
}

pub fn normalize_zp(v: Vec3) -> Vec3 {
    let len = abs_xz(v);
    if !(len.abs() < NORMALIZE_EPS) {
        Vec3::new(v.x / len, v.y, v.z / len)
    } else {
        Vec3::ZERO
    }
}

pub fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        let mul = RND_MUL.get(lane).copied().unwrap_or(1);
        let modu = RND_MOD.get(lane).copied().unwrap_or(1);
        if let Some(slot) = seed.get_mut(lane) {
            *slot = (*slot * mul) % modu;
        }
    }
    let a = (seed.first().copied().unwrap_or(0)) as f32 / RND_SCALE[0];
    let b = (seed.get(1).copied().unwrap_or(0)) as f32 / RND_SCALE[1];
    let c = (seed.get(2).copied().unwrap_or(0)) as f32 / RND_SCALE[2];
    (a + b + c) % 1.0
}

pub fn rnd_f(seed: &mut [i32; 3], range: f32) -> f32 {
    rnd(seed) * range
}

pub fn rnd_value(seed: &mut [i32; 3], low: i32, span: i32) -> i32 {
    let value = rnd(seed);
    low + (value * span as f32) as i32
}

pub fn player_eye_pos(player: Vec3) -> Vec3 {
    Vec3::new(player.x, player.y + HIO_NPC_EYE_OFFSET, player.z)
}

impl Medli {
    pub fn jnt_set_param(
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
        self.jnt_max[1][0] = max_backbone_x;
        self.jnt_max[1][1] = max_backbone_y;
        self.jnt_min[1][0] = min_backbone_x;
        self.jnt_min[1][1] = min_backbone_y;
        self.jnt_max[0][0] = max_head_x;
        self.jnt_max[0][1] = max_head_y;
        self.jnt_min[0][0] = min_head_x;
        self.jnt_min[0][1] = min_head_y;
        for row in self.jnt_step.iter_mut() {
            row[0] = max_turn_step;
            row[1] = max_turn_step;
        }
    }

    pub fn jnt_default_param(&mut self) {
        self.jnt_set_param(
            HIO_NPC_MAX_BACKBONE_X,
            HIO_NPC_MAX_BACKBONE_Y,
            HIO_NPC_MIN_BACKBONE_X,
            HIO_NPC_MIN_BACKBONE_Y,
            HIO_NPC_MAX_HEAD_X,
            HIO_NPC_MAX_HEAD_Y,
            HIO_NPC_MIN_HEAD_X,
            HIO_NPC_MIN_HEAD_Y,
            HIO_NPC_MAX_TURN_STEP,
        );
    }

    pub fn jnt_ship_param(&mut self) {
        self.jnt_set_param(
            HIO6_M10,
            HIO6_M12,
            HIO6_M14,
            HIO6_M16,
            HIO6_M08,
            HIO6_M0A,
            HIO6_M0C,
            HIO6_M0E,
            HIO_NPC_MAX_TURN_STEP,
        );
    }

    fn jnt_limiter(&self, target: i16, slot: usize, axis: usize) -> i16 {
        let max = self.jnt_max.get(slot).and_then(|r| r.get(axis)).copied().unwrap_or(0);
        let min = self.jnt_min.get(slot).and_then(|r| r.get(axis)).copied().unwrap_or(0);
        if target > max { max } else if target < min { min } else { target }
    }

    fn jnt_ang_calc(&self, out: &mut i16, target: i16, speed: i16, max_vel: i16) -> bool {
        let orig = *out - target;
        let diff = walk::add_calc_angle_s(orig, 0, speed, max_vel, 0x60);
        if i32::from(diff.unsigned_abs()) > i32::from(speed) {
            *out = out.wrapping_add(diff.wrapping_sub(orig));
        } else {
            *out = target;
        }
        target.wrapping_sub(*out) == 0
    }

    fn jnt_follow(&mut self, out: &mut i16, target: i16, max_vel: i16) -> bool {
        let orig = *out;
        self.jnt_ang_calc(out, target, 4, max_vel);
        let delta = out.wrapping_sub(orig);
        let want = target.wrapping_sub(*out);
        let mut sum = 0_i16;
        let mut total = 0_i16;
        for slot in (0..2).rev() {
            let angle = self.jnt_angles[slot][1];
            sum = sum.wrapping_add(angle);
            if (delta > 0 && sum > want) || (delta < 0 && sum < want) {
                let value = self.jnt_limiter(want.wrapping_sub(total), slot, 1);
                self.jnt_angles[slot][1] = value;
            }
            total = total.wrapping_add(self.jnt_angles[slot][1]);
        }
        target != *out
    }

    fn jnt_move(&mut self, param_1: i16, axis: usize) -> bool {
        let mut rest = param_1;
        let mut angles = [0_i16; 2];
        for slot in 0..2 {
            let mut angle = rest;
            angle = self.jnt_limiter(angle, slot, axis);
            rest -= angle;
            angles[slot] = angle;
            let current = self.jnt_angles[slot][axis];
            let max_step = self.jnt_step[slot][axis];
            self.jnt_angles[slot][axis] = walk::add_calc_angle_s(current, angle, 4, max_step, 1);
        }
        let mut done = 0;
        for slot in 0..2 {
            let max_step = self.jnt_step[slot][axis];
            if angle_distance(angles[slot], self.jnt_angles[slot][axis]) <= (max_step / 2) as u16 {
                done += 1;
            }
        }
        done == 2
    }

    pub fn jnt_look_at(
        &mut self,
        out: &mut i16,
        dst: Option<Vec3>,
        src: Vec3,
        default_y: i16,
        max_vel: i16,
        hold: bool,
    ) {
        let (target_y, target_x) = match dst {
            Some(point) => {
                let delta = point - src;
                let dist_xz = (delta.x * delta.x + delta.z * delta.z).sqrt();
                (
                    walk::to_s16(delta.x.atan2(delta.z)),
                    walk::to_s16(delta.y.atan2(dist_xz)),
                )
            }
            None => (default_y, 0),
        };
        let delta_y = target_y.wrapping_sub(*out);
        let max_y = self.jnt_max[1][1].wrapping_add(self.jnt_max[0][1]);
        let min_y = self.jnt_min[1][1].wrapping_add(self.jnt_min[0][1]);
        let done = self.jnt_move(delta_y, 1);
        if (self.jnt_trn || (delta_y >= max_y || delta_y <= min_y) && done) && !hold {
            self.jnt_trn = self.jnt_follow(out, target_y, max_vel);
        }
        self.jnt_move(target_x, 0);
    }

    pub fn jnt_set_trn(&mut self) {
        self.jnt_trn = true;
    }

    pub fn jnt_trn_chk(&self) -> bool {
        self.jnt_trn
    }
}

pub struct Text<'a> {
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub flags: &'a mut MsgFlags,
    pub one: &'a mut Msg1Box,
    pub two: &'a mut Msg2Box,
    pub clear_count: u8,
    pub archive: Option<&'a dyn MsgArchive>,
}

impl Text<'_> {
    pub fn proc_name(&self) -> Option<u16> {
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

    pub fn select_num(&self) -> u8 {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.select_num,
            _ => 0,
        }
    }

    pub fn message_set(&mut self, msg_no: u32, look_at: Vec3) -> u32 {
        let Some(archive) = self.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        fop_msgm_message_set(
            self.procs,
            self.info,
            archive,
            self.clear_count,
            msg_no,
            None,
            look_at,
        )
    }

    pub fn tact_message_set(&mut self) -> u32 {
        let Some(archive) = self.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        let clear_count = self.clear_count;
        fop_msgm_tact_message_set(
            self.procs,
            self.flags,
            self.info,
            archive,
            clear_count,
        )
    }
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub player: Option<Vec3>,
    pub player_yaw: i16,
    pub player_shape: i16,
    pub player_speed_f: f32,
    pub link: Option<Vec3>,
    pub stick: f32,
    pub stick_angle: i16,
    pub trig_a: bool,
    pub trig_b: bool,
    pub is_player: bool,
    pub carried: bool,
    pub ground_y: Option<f32>,
    pub ground_normal: Vec3,
    pub ground_normal_opt: Option<Vec3>,
    pub ground_code: Option<i32>,
    pub wall_hit: bool,
    pub roof_hit: bool,
    pub ground_room: Option<i8>,
    pub actor_room: i8,
    pub link_room: i8,
    pub trig_r: bool,
    pub trig_start: bool,
    pub wall_angle: i16,
    pub in_water: bool,
    pub water_height: f32,
    pub ship_pos: Option<Vec3>,
    pub ship_yaw: i16,
    pub ship_speed: f32,
    pub room_drawn: bool,
    pub cancel_button: bool,
    pub pos_move: bool,
    pub ship_riding: bool,
    pub staff_cut: Option<usize>,
    pub stories: &'a mut StoryBits,
    pub switches: &'a mut Switches,
    pub events: &'a mut SimEvents,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub text: &'a mut Text<'a>,
    pub give_item: Option<(Vec3, u8)>,
    pub goto_sea: bool,
    pub stage: &'a str,
    pub talk_request: bool,
    pub talk_xy: bool,
    pub door_command: bool,
    pub demo_accepted: bool,
    pub wait_cut: bool,
    pub drop_carry: bool,
    pub special_ground: bool,
    pub wind_hit: bool,
    pub wind_vec: Vec3,
    pub auto_jump: i32,
    pub wall_normal: Option<Vec3>,
    pub probe_y: Option<f32>,
    pub cross_normal: Option<Vec3>,
    pub drop_normal: Option<Vec3>,
    pub player_locked: bool,
    pub attention_lock: Option<Vec3>,
    pub link_hidden: bool,
    pub link_move: Option<(Vec3, i16)>,
    pub cut_end_mode: i32,
    pub tact_song: i32,
    pub tact_ended: bool,
    pub link_eye: Option<Vec3>,
    pub move_block: Option<(i8, Vec3)>,
    pub xy_end: bool,
    pub reset_event: bool,
    pub delete_me: bool,
}

impl Ctx<'_> {
    pub fn event_run(&self) -> bool {
        self.text.info.event_run_check()
    }
}

pub const BTP_FLAG: [u8; 16] = [0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0, 0, 1, 1];

impl Medli {
    pub fn init(&mut self, actor: &Actor, ctx: &mut Ctx<'_>) {
        self.attn_flags = ATTN_LOCKON_TALK | ATTN_ACTION_SPEAK;
        self.mode = 0;
        self.cur_event = 0;
        self.harp_delay = 0;
        self.harp_se = HARP_SE[0];
        self.turn_offset = Vec3::new(0.0, 0.0, 50.0);
        self.piyo_count = 0;
        self.stts = 0;
        self.gravity = HIO_M0F4;
        self.step = if self.is_type_m_daib() {
            Step::HarpWait
        } else {
            Step::Wait
        };
        self.step_stat = STEP_STARTING;
        self.flying = false;
        self.mirror = false;
        self.height = HEIGHT;
        self.run_rate = 0.0;
        self.anm_rate = 0.0;
        if ctx.stories.test(BIT_1620) {
            if self.is_type_ship_ride() {
                self.set_stts(STTS_SHIP_RIDE);
                self.set_npc_action(Step::Ship, Arg::None);
            }
        } else {
            self.npc_not_change = true;
        }
        if self.is_type_edaichi() {
            self.call_command = true;
        }
        self.attn_base = actor.pos;
        self.eye = actor.pos;
        self.home = actor.pos;
        self.home_yaw = actor.angle[1];
        if self.prm_lo & 1 != 0 {
            self.stts = STTS_UNK8;
        }
        self.effect = 0;
        self.cur_event = -1;
        self.staff_idx = ctx.list.staff_of(STAFF_NAME).unwrap_or(NO_STAFF);
        for slot in 0..EVENT_SLOTS {
            let name = EVENT_NAMES.get(slot).copied().unwrap_or("");
            self.event_idx[slot] = ctx
                .list
                .event_index(name)
                .and_then(|index| i16::try_from(index).ok())
                .unwrap_or(NO_INDEX);
        }
    }

    pub fn set_npc_action(&mut self, step: Step, arg: Arg) {
        self.flying = false;
        self.player_step = PlayerStep::None;
        if self.step != step {
            self.step = step;
            self.step_stat = STEP_STARTING;
            self.timer_a = 0;
            self.timer_b = 0;
            self.glide_flag = 0;
            self.glide_timer = 0;
            self.start_y = 0.0;
            self.arg = arg;
        }
    }

    pub fn set_player_action(&mut self, step: PlayerStep, arg: Arg) {
        self.step = Step::None;
        if self.player_step != step {
            self.player_step = step;
            self.player_stat = STEP_STARTING;
            self.timer_a = 0;
            self.timer_b = 0;
            self.glide_flag = 0;
            self.glide_timer = 0;
            self.start_y = 0.0;
            self.arg = arg;
        }
    }

    pub fn has_arg(&self) -> bool {
        self.arg != Arg::None
    }

    pub fn get_anm_type(attr: u8) -> i32 {
        let index = usize::from(attr);
        if index < MSG_ANM.len() {
            MSG_ANM.get(index).copied().unwrap_or(-1)
        } else {
            -1
        }
    }

    pub fn set_anm(&mut self, index: i32) -> bool {
        let slot = usize::try_from(index).unwrap_or(usize::MAX);
        let table_idx = ANM_TBL_IDX.get(slot).copied().unwrap_or(0);
        let arm_idx = ANM_TBL_ARM.get(slot).copied().unwrap_or(0);
        let btp_idx = ANM_TBL_BTP.get(slot).copied().unwrap_or(0);
        let table_speed = ANM_TBL_SPEED.get(slot).copied().unwrap_or(1.0);
        let mut _speed = table_speed;
        let mut _morf = ANM_TBL_MORF.get(slot).copied().unwrap_or(8.0);
        if index == 6 {
            _speed = HIO_M118;
        } else if index == 9 || index == 10 {
            _speed = HIO_M11C;
        } else if index == 0x2E {
            _speed = HIO_M124;
        } else if index == 0x28 {
            _morf = HIO5_M4;
            _speed = HIO5_M8;
        }
        if index == 7 || index == 8 {
            _morf = HIO_M1A8;
        }
        self.anm = index;
        let mut ret = false;
        if self.anm_body != table_idx as i8 || self.anm_rate != table_speed {
            self.anm_body = table_idx as i8;
            self.anim_ended = false;
            self.anm_frame = 0.0;
            self.anm_start = 0.0;
            self.anm_rate = table_speed;
            let wing = arm_idx >= 0x80;
            ret = wing;
            if index == 0x12 || index == 0x1F {
                self.harp_delay = 0;
            } else if index == 0x13 || index == 0x21 {
                self.harp_delay = 0;
            }
        }
        if btp_idx != self.btp_tbl {
            self.init_tex_pattern_anm(btp_idx);
        }
        let body = i32::from(self.anm_body);
        if body == 0x0E
            || body == 0x12
            || body == 0x13
            || body == 0x1A
            || body == 0x1C
            || body == 0x25
        {
            self.set_stts(STTS_UNK80);
        } else if body != 0x0F && body != 0x1B {
            self.clr_stts(STTS_UNK80);
        }
        if body == 0x1B || body == 0x1C {
            self.set_stts(STTS_UNK4000);
        } else {
            self.clr_stts(STTS_UNK4000);
        }
        ret
    }

    pub fn init_tex_pattern_anm(&mut self, index: u8) {
        let slot = usize::from(index);
        self.btp_tbl = index;
        self.btp_repeat = 0.0;
        self.btp_looping = BTP_FLAG.get(slot).copied().unwrap_or(0) == 1;
    }

    pub fn play_tex_pattern_anm(&mut self) {
        if self.btp_looping {
            return;
        }
        if self.btp_repeat > 0.0 {
            self.btp_repeat -= 1.0;
            if self.btp_repeat < 0.0 {
                self.btp_repeat = 0.0;
            }
            return;
        }
        self.btp_repeat = ((BTP_REPEAT_MIN + rnd_f(&mut self.seed, BTP_REPEAT_SPAN)) as u8) as f32;
    }

    pub fn set_harp_play_num(&mut self) {
        let value = rnd_value(&mut self.seed, 0, HARP_RND_SPAN);
        let index = usize::try_from(value).unwrap_or(usize::MAX);
        let anm = HARP_PLAY_ANM.get(index).copied().unwrap_or(0);
        self.set_anm(anm);
    }

    pub fn change_caught02(&mut self) {
        self.set_anm(ANM_CAUGHT02);
        self.set_stts(STTS_UNK1);
    }

    pub fn chk_attention(
        &self,
        link: Option<Vec3>,
        pos: Vec3,
        angle: i16,
        wide: i32,
        dra09_letter: bool,
    ) -> bool {
        let Some(link) = link else {
            return false;
        };
        let mut max_dist = HIO_NPC_MAX_ATTN_DIST_XZ;
        let mut max_angle = HIO_NPC_MAX_ATTN_ANGLE_Y;
        if self.is_ship_ride() {
            max_dist = HIO6_M04;
        } else if self.is_type_dra09() && dra09_letter {
            max_dist = HIO_M0B8;
        }
        let delta = link - pos;
        let dist_xz = (delta.x * delta.x + delta.z * delta.z).sqrt();
        let mut target = walk::to_s16(delta.x.atan2(delta.z));
        if self.attn_seen {
            max_dist += ATTN_BONUS_DIST;
            max_angle += ATTN_BONUS_ANGLE;
        }
        target = target.wrapping_sub(angle);
        if wide != 0 {
            max_dist > dist_xz && delta.y < HIO_M0BC && delta.y > HIO_M0C0
        } else {
            max_angle > i32::from(target.unsigned_abs())
                && max_dist > dist_xz
                && delta.y < HIO_M0BC
                && delta.y > HIO_M0C0
        }
    }

    pub fn chk_area(&self, link: Option<Vec3>, pos: Vec3) -> bool {
        let Some(link) = link else {
            return false;
        };
        let delta = Vec3::new(link.x - pos.x, 0.0, link.z - pos.z);
        let dist = (delta.x * delta.x + delta.z * delta.z).sqrt();
        HIO_NPC_MAX_ATTN_DIST_XZ + ATTN_BONUS_DIST > dist
    }
}

impl Medli {
    pub fn next_msg_status(&mut self, msg_no: &mut u32, select_num: u8, ctx: &mut Ctx<'_>) -> u8 {
        let mut status = STATUS_MSG_CONTINUES;
        match *msg_no {
            0x5AC => status = STATUS_MSG_ENDS,
            0x17D5 | 0x1805 => *msg_no = 0x17D6,
            0x17D6 => *msg_no = 0x17D7,
            0x17D7 => *msg_no = 0x17D8,
            0x17D8 => {
                self.mode = MODE_EVENT0;
                status = STATUS_MSG_ENDS;
            }
            0x17D9 => *msg_no = 0x1802,
            0x1802 => *msg_no = 0x17DA,
            0x17DA => *msg_no = 0x17DB,
            0x17DB => {
                ctx.stories.set(BIT_GAVE_LETTER);
                status = STATUS_MSG_ENDS;
            }
            0x17DD => *msg_no = 0x17DE,
            0x17DE => {
                self.mode = MODE_EVENT1;
                status = STATUS_MSG_ENDS;
            }
            0x17DF => *msg_no = 0x17E0,
            0x17E0 => *msg_no = 0x17E1,
            0x17E1 => *msg_no = 0x17E2,
            0x17E2 => *msg_no = 0x17E3,
            0x17E4 => {
                ctx.stories.set(BIT_1101);
                status = STATUS_MSG_ENDS;
            }
            0x17E5 => {
                ctx.stories.set(BIT_1280);
                status = STATUS_MSG_ENDS;
            }
            0x17E6 => *msg_no = 0x17E7,
            0x17E7 => *msg_no = 0x17E8,
            0x17E8 => *msg_no = 0x17E9,
            0x17E9 => *msg_no = 0x17EA,
            0x17EA => *msg_no = 0x17EB,
            0x17EB => *msg_no = 0x17EC,
            0x17EC => *msg_no = 0x17ED,
            0x17ED => *msg_no = 0x17EE,
            0x17EE => {
                if select_num == 1 {
                    *msg_no = 0x17EF;
                } else {
                    *msg_no = 0x17F0;
                }
            }
            0x17EF => *msg_no = 0x17ED,
            0x17F0 => *msg_no = 0x17F1,
            0x17F1 => {
                self.mode = MODE_EVENT3;
                status = STATUS_MSG_ENDS;
            }
            0x17F3 => *msg_no = 0x17F4,
            0x17F4 => {
                ctx.stories.set(BIT_1104);
                self.set_stts(STTS_UNK40);
                status = STATUS_MSG_ENDS;
            }
            0x17F7 => *msg_no = 0x17F8,
            0x17F8 => {
                self.set_stts(STTS_UNK40);
                status = STATUS_MSG_ENDS;
            }
            0x17FA => *msg_no = 0x17FB,
            0x17FB => *msg_no = 0x17FC,
            0x17FC => *msg_no = 0x17FD,
            0x17FD => *msg_no = 0x17FE,
            0x17FE => *msg_no = 0x17FF,
            0x1800 => *msg_no = 0x1801,
            0x1801 => {
                ctx.stories.set(BIT_1102);
                status = STATUS_MSG_ENDS;
            }
            0x19C9 => *msg_no = 0x19CA,
            0x19CA => *msg_no = 0x19CB,
            0x19CB => *msg_no = 0x19CC,
            0x19CC => *msg_no = 0x19CD,
            0x19CD => *msg_no = 0x19CE,
            0x19CE => *msg_no = 0x19CF,
            0x19CF => *msg_no = 0x19D0,
            0x19D0 => *msg_no = 0x19D1,
            0x19D1 => {
                ctx.stories.set(BIT_1402);
                self.sea_talk = true;
                status = STATUS_MSG_ENDS;
            }
            0x19D2 => *msg_no = 0x19D3,
            0x19D3 => *msg_no = 0x19D4,
            0x19D4 => *msg_no = 0x19D5,
            0x19D5 => *msg_no = 0x19D6,
            0x19D6 => *msg_no = 0x19D7,
            0x19D7 => *msg_no = 0x19D8,
            0x19D8 => {
                ctx.stories.set(BIT_1504);
                status = STATUS_MSG_ENDS;
            }
            0x19D9 => status = STATUS_MSG_ENDS,
            0x19DA => *msg_no = 0x19DB,
            0x19DD => *msg_no = 0x19DE,
            0x19E0 => *msg_no = 0x19E1,
            0x19E1 => *msg_no = 0x19E2,
            0x19E2 => *msg_no = 0x19E3,
            0x19E3 => *msg_no = 0x19E4,
            0x19E5 => *msg_no = 0x19E6,
            0x19E7 => *msg_no = 0x19E8,
            0x19F6 => *msg_no = 0x19F7,
            0x19F7 => *msg_no = 0x19D2,
            0x19F8 => *msg_no = 0x19F9,
            0x19F9 => *msg_no = 0x19FA,
            0x19FA => *msg_no = 0x19FB,
            0x19FB | 0x19FF => {
                ctx.stories.set(BIT_2E40);
                self.sea_talk = true;
                ctx.stories.set(BIT_2C08);
                status = STATUS_MSG_ENDS;
            }
            0x19FC => *msg_no = 0x19FD,
            0x19FD => *msg_no = 0x19FE,
            0x19FE => *msg_no = 0x19FF,
            0x1A00 => *msg_no = 0x1A01,
            0x1A01 => {
                ctx.stories.set(BIT_2C08);
                status = STATUS_MSG_ENDS;
            }
            0x1A02 => {
                ctx.stories.set(BIT_3B80);
                status = STATUS_MSG_ENDS;
            }
            _ => status = STATUS_MSG_ENDS,
        }
        status
    }

    pub fn get_msg(&mut self, ctx: &mut Ctx<'_>) -> u32 {
        let mut msg_no = 0;
        if self.is_ship_ride() {
            msg_no = MSG_SHIP_TALK;
        } else if self.is_type_atorizk() {
            if ctx.stories.test(BIT_GAVE_LETTER) {
                msg_no = MSG_LETTER_SEA;
            } else if ctx.text.clear_count != 0 {
                msg_no = MSG_LETTER_ATORIZK_CLEAR;
            } else {
                msg_no = MSG_LETTER_FIRST;
            }
        } else if self.is_type_adanmae() {
            if self.has_stts(STTS_UNK40) {
                if self.anm_body == 0x10 {
                    let count = usize::from(self.piyo_count);
                    msg_no = PIYO2_MSG.get(count).copied().unwrap_or(0x17F5);
                    self.count_piyo_talk();
                } else {
                    msg_no = MSG_ADANMAE_PIYO_TALK;
                }
            } else if ctx.stories.test(BIT_1104) {
                msg_no = MSG_ADANMAE_HARP;
            } else {
                msg_no = MSG_ADANMAE_FIRST;
            }
        } else if self.is_type_dra09() {
            if ctx.stories.test(BIT_1101) {
                msg_no = MSG_DRA09_HARP;
            } else {
                msg_no = MSG_DRA09_FIRST;
            }
        } else if self.is_type_sea() {
            if self.is_xy_talk() {
                if self.is_default_talk_xy() {
                    if ctx.stories.test(BIT_2E40) {
                        msg_no = MSG_SEA_FEATHER_DONE;
                    } else if ctx.stories.test(BIT_2C08) {
                        msg_no = MSG_SEA_FEATHER_SEEN;
                    } else {
                        msg_no = MSG_SEA_FIRST;
                    }
                } else if ctx.stories.test(BIT_1504) {
                    msg_no = MSG_SEA_TACT_TRUE;
                } else {
                    msg_no = MSG_SEA_TACT;
                }
            } else if ctx.stories.test(BIT_1402) {
                if self.sea_talk {
                    msg_no = MSG_SEA_HARP_TALK;
                } else {
                    msg_no = MSG_SEA_HARP_WALK;
                    self.sea_talk = true;
                }
            } else {
                msg_no = MSG_SEA_OPEN;
            }
        } else if self.is_type_field() {
            msg_no = MSG_FIELD_FIRST;
        }
        msg_no
    }

    pub fn talk_init(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.msg_id == FPC_M_ERROR_PROCESS_ID {
            if self.msg_no == MSG_TACT {
                self.msg_id = ctx.text.tact_message_set();
            } else {
                let msg_no = self.msg_no;
                let look = self.eye;
                self.msg_id = ctx.text.message_set(msg_no, look);
            }
            false
        } else {
            ctx.text.procs.search_by_id(self.msg_id).is_some()
        }
    }

    pub fn talk(&mut self, tact: bool, cancel: bool, ctx: &mut Ctx<'_>) -> bool {
        let msg_status = ctx.text.status().unwrap_or(NO_STATUS);
        let msg_anm_atr = ctx.text.info.mesg_anime;
        if msg_status == STATUS_MSG_DISPLAYED {
            let mut next_no = self.msg_no;
            let select = ctx.text.select_num();
            let next = self.next_msg_status(&mut next_no, select, ctx);
            self.msg_no = next_no;
            ctx.text.set_status(next);
            if next == STATUS_MSG_CONTINUES {
                let msg_no = self.msg_no;
                ctx.text.message_set(msg_no, Vec3::ZERO);
                self.msg_attr = 0;
            }
        } else if msg_status == STATUS_INPUT {
            if tact {
                if cancel {
                    ctx.text.set_status(STATUS_MSG_ENDS);
                    ctx.text.flags.message_send_on();
                    self.set_stts(STTS_UNK200);
                }
                if self.has_stts(STTS_UNK400) {
                    ctx.text.set_status(STATUS_MSG_ENDS);
                    ctx.text.flags.message_send_on();
                }
            }
        } else if msg_status == STATUS_TYPING {
            if self.msg_attr == 0 && !ctx.carried && !self.is_ship_ride() && self.msg_no != 0x05AC {
                let anm = Medli::get_anm_type(msg_anm_atr);
                if anm >= 0 {
                    self.set_anm(anm);
                }
            }
        } else if msg_status == STATUS_BOX_CLOSED {
            ctx.text.set_status(STATUS_MSG_DESTROYED);
            self.clr_stts(STTS_XY_TALK);
            return true;
        }
        if !ctx.carried && !self.is_ship_ride() && self.msg_no != 0x05AC {
            if self.msg_attr == 1 && msg_anm_atr == PIYO_TALK_ATTR && self.anim_ended {
                self.set_anm(PIYO_TALK_ANIM);
                self.msg_attr = 1;
            }
        }
        false
    }
}

impl Medli {
    pub fn look_back(&mut self, use_player: bool, waist: bool, enable: bool) {
        let mut dst = None;
        let mut src = Vec3::ZERO;
        let desired = self.shape[1];
        if self.jnt_trn_chk() {
            let vel = if waist { HIO_M1BC } else { HIO_NPC_MAX_HEAD_TURN_VEL };
            stream::add_calc_angle_s2(&mut self.turn_vel, vel, HEAD_SCALE, HEAD_MAX_STEP);
        } else {
            self.turn_vel = 0;
        }
        if use_player {
            if let Some(link) = self.link {
                dst = Some(player_eye_pos(link));
                src = Vec3::new(self.pos.x, self.eye.y, self.pos.z);
            }
        }
        let mut yaw = self.shape[1];
        self.jnt_look_at(&mut yaw, dst, src, desired, self.turn_vel, enable);
        self.shape[1] = yaw;
    }

    pub fn look_back_at(&mut self, target: Option<Vec3>, waist: bool, enable: bool) {
        let mut dst = None;
        let mut src = Vec3::ZERO;
        let desired = self.shape[1];
        if self.jnt_trn_chk() {
            stream::add_calc_angle_s2(
                &mut self.turn_vel,
                HIO_NPC_MAX_HEAD_TURN_VEL,
                HEAD_SCALE,
                HEAD_MAX_STEP,
            );
        } else {
            self.turn_vel = 0;
        }
        if waist {
            dst = target;
            src = Vec3::new(self.pos.x, self.eye.y, self.pos.z);
        }
        let mut yaw = self.shape[1];
        self.jnt_look_at(&mut yaw, dst, src, desired, self.turn_vel, enable);
        self.shape[1] = yaw;
    }

    pub fn look_back_waist(&mut self, angle: i16, speed: f32) -> bool {
        let diff = angle.wrapping_sub(self.shape[1]);
        let rad = diff as f32 * walk::RAD_PER_UNIT;
        let rise = HIO3_M04 * (speed * rad.cos());
        let yaw = HIO3_M08 * (speed * rad.sin());
        let want = if rise < 0.0 {
            (HIO3_M20 as f32 * rise) as i16
        } else {
            (HIO3_M1E as f32 * rise) as i16
        };
        let want_y = (HIO3_M22 as f32 * yaw) as i16;
        let got_x =
            walk::add_calc_angle_s(self.waist_rot_x, want, WAIST_SCALE, WAIST_MAX_STEP, WAIST_MIN_STEP);
        self.waist_rot_x = got_x;
        let got_y =
            walk::add_calc_angle_s(self.waist_rot_y, want_y, WAIST_SCALE, WAIST_MAX_STEP, WAIST_MIN_STEP);
        self.waist_rot_y = got_y;
        got_x == 0 && got_y == 0
    }

    pub fn set_attention(&mut self, active: bool, pos: Vec3, is_player: bool) {
        if !active && self.attn_count >= 2 {
            return;
        }
        self.eye = Vec3::new(self.eye.x, self.eye.y, self.eye.z);
        let lift = if self.is_flying() {
            70.0
        } else if is_player {
            110.0
        } else {
            HIO_NPC_ATTN_Y_OFFSET
        };
        self.attn_pos = Vec3::new(pos.x, pos.y + lift, pos.z);
    }

    pub fn ship_ride_check(&mut self) -> bool {
        if self.is_type_ship_ride() {
            self.set_npc_action(Step::Ship, Arg::None);
            return true;
        }
        false
    }

    pub fn is_fall_action(&self) -> bool {
        self.player_step == PlayerStep::Jump
            || self.step == Step::Fall
            || self.step == Step::Fall02
    }

    pub fn return_link_check(&self, ground: bool, trig_r: bool, trig_start: bool, ctx: &Ctx<'_>) -> bool {
        if !ctx.event_run() && (trig_r || trig_start) && ground {
            return true;
        }
        false
    }

    pub fn wall_hit_check(&self, wall: bool, wall_angle: i16) -> i16 {
        if wall {
            return wall_angle;
        }
        -1
    }

    pub fn light_hit_check(&mut self) -> bool {
        let light_hit = self.light_seen;
        self.clr_stts(STTS_LIGHT_BODY_HIT);
        if light_hit {
            self.set_stts(STTS_LIGHT_BODY_HIT);
        }
        if light_hit {
            self.set_stts(STTS_LIGHT_HIT);
        } else {
            self.clr_stts(STTS_LIGHT_HIT);
        }
        light_hit
    }

    pub fn wait_ground_check(&mut self, special: bool, is_player: bool) {
        if self.ground_hit {
            if !self.is_type_dra09() {
                if special {
                    self.shape[1] = walk::add_calc_angle_s(
                        self.shape[1],
                        target_angle_y(self.pos, self.block_pos),
                        TURN_SCALE,
                        TURN_MAX_STEP,
                        TURN_MIN_STEP,
                    );
                    self.speed_f += 0.5;
                    self.speed_f = self.speed_f.min(HIO2_M08);
                } else {
                    self.speed_f = 0.0;
                }
            }
        } else if !is_player {
            self.set_npc_action(Step::Fall, Arg::None);
        }
    }

    pub fn chk_adanmae_demo_order(&mut self) -> bool {
        if self.is_type_adanmae() {
            let ground_h = self.ground_y;
            if self.is_cam_tag_in() && (DEMO_RANGE_LO..=DEMO_RANGE_HI).contains(&ground_h) {
                self.mode = MODE_EVENT2;
                self.gravity = HIO_M0F4;
                self.max_fall = MAX_FALL;
                if self.ground_hit {
                    self.speed_f = 0.0;
                }
                return true;
            }
        }
        false
    }

    pub fn npc_call(&mut self, ctx: &mut Ctx<'_>) -> i32 {
        if !ctx.stories.test(BIT_1620) {
            return 0;
        }
        let Some(player) = ctx.player else {
            return 0;
        };
        let dist_sq = abs2(player - self.pos);
        if !self.call_command {
            if dist_sq < HIO_M0C8 * HIO_M0C8 {
                return 1;
            }
            0
        } else {
            let temp = 2.0 * HIO_M0C4;
            if dist_sq >= temp * temp {
                self.set_npc_action(Step::Search, Arg::None);
            }
            1
        }
    }
}

impl Medli {
    pub fn check_collision(&mut self, as_player: bool) -> bool {
        if !self.is_type_field() {
            return false;
        }
        let Some(player) = self.link else {
            return false;
        };
        let mut delta = Vec3::new(self.pos.x - player.x, 0.0, self.pos.z - player.z);
        if abs_xz(delta) < HIO_NPC_MAX_ATTN_DIST_XZ + ATTN_BONUS_DIST {
            delta = normalize_zp(delta);
            let angle = if delta.x.abs() < NORMAL_EPS && delta.z.abs() < NORMAL_EPS {
                0
            } else {
                walk::to_s16(delta.x.atan2(delta.z))
            };
            if as_player {
                self.set_player_action(PlayerStep::Hit, Arg::None);
            } else {
                self.set_npc_action(Step::Hit, Arg::Angle(angle));
            }
            return true;
        }
        false
    }

    pub fn carry_check(&mut self, carried: bool) {
        if carried {
            self.set_npc_action(Step::Carry, Arg::None);
        }
    }

    pub fn is_tag_check_ok(&self, carried_by_player: bool) -> bool {
        if self.step == Step::Wait || self.step == Step::Search {
            return true;
        }
        if self.step == Step::Carry && !carried_by_player {
            return true;
        }
        false
    }

    pub fn restart_point(&mut self, point: i16) {
        self.timer_a = point;
        self.set_npc_action(Step::Wait, Arg::None);
    }

    pub fn set_message_animation(&mut self, attr: u8, carried: bool, ctx: &Ctx<'_>) {
        if carried {
            return;
        }
        if !ctx.event_run() {
            return;
        }
        if self.staff_idx == NO_STAFF {
            return;
        }
        if !ctx.wait_cut {
            return;
        }
        let anm = Medli::get_anm_type(attr);
        if anm >= 0 {
            self.set_anm(anm);
        }
    }

    pub fn check_command_talk(&mut self, ctx: &Ctx<'_>) -> bool {
        if ctx.talk_request {
            if ctx.talk_xy {
                self.set_stts(STTS_XY_TALK);
                if !self.is_default_talk_xy() {
                    if self.mode == MODE_TALK_RETRY {
                        self.mode = 0;
                    }
                    return false;
                }
            } else {
                self.clr_stts(STTS_XY_TALK);
            }
            return true;
        }
        self.clr_stts(STTS_XY_TALK);
        false
    }

    pub fn check_order(&mut self, ctx: &mut Ctx<'_>) {
        if ctx.talk_request && (self.mode == MODE_TALK || self.mode == MODE_TALK_ITEM || self.mode == MODE_TALK_RETRY)
        {
            self.mode = 0;
            if ctx.talk_xy {
                self.set_stts(STTS_XY_TALK);
                if self.is_default_talk_xy() {
                    self.set_npc_action(Step::Talk, Arg::None);
                }
            } else if self.is_ship_ride() {
                self.set_npc_action(Step::ShipTalk, Arg::None);
            } else {
                self.set_npc_action(Step::Talk, Arg::None);
            }
            ctx.drop_carry = true;
        }
    }

    pub fn xy_check_cb(&mut self, item: u8, ctx: &mut Ctx<'_>) -> bool {
        if self.is_type_sea() {
            if item == ITEM_WIND_WAKER {
                return true;
            }
            if item == ITEM_GOLDEN_FEATHER {
                if !ctx.stories.test(BIT_2E40) || self.sea_talk {
                    return true;
                }
            }
        } else if self.is_type_m_dai() || self.is_type_edaichi() {
            if item == ITEM_GOLDEN_FEATHER && !ctx.stories.test(BIT_3B80) {
                return true;
            }
        }
        if item == ITEM_GOLDEN_FEATHER {
            ctx.stories.set(BIT_2C08);
        }
        false
    }

    pub fn xy_event_cb(&mut self, item: u8) -> i32 {
        if item == ITEM_WIND_WAKER {
            self.clr_stts(STTS_DEFAULT_TALK_XY);
            return i32::from(self.event_idx[5]);
        } else if item == ITEM_GOLDEN_FEATHER {
            self.set_stts(STTS_DEFAULT_TALK_XY);
        }
        -1
    }

    pub fn event_order(&mut self, ctx: &mut Ctx<'_>) {
        match self.mode {
            MODE_TALK => {
                self.cond_talk = true;
            }
            MODE_TALK_ITEM | MODE_TALK_RETRY => {
                self.cond_talk = true;
            }
            m if m >= MODE_EVENT0 => {
                let slot = usize::try_from(i32::from(m) - i32::from(MODE_EVENT0)).unwrap_or(usize::MAX);
                let index = self.event_idx.get(slot).copied().unwrap_or(NO_INDEX);
                self.cur_event = i8::try_from(slot).unwrap_or(-1);
                if index >= 0 {
                    let at = usize::from(index as u16);
                    if ctx.active.stage_of(at).is_none() {
                        ctx.active.order(ctx.list, at, ctx.entity);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn return_link_player(&mut self) {
        self.flying = false;
        self.mirror = false;
    }

    pub fn event_accept_block(&mut self, ctx: &mut Ctx<'_>) {
        if !ctx.demo_accepted || self.mode == 0 {
            return;
        }
        if self.mode == MODE_EVENT8 {
            let index = self.event_idx.get(8).copied().unwrap_or(NO_INDEX);
            if index >= 0 {
                let at = usize::from(index as u16);
                if ctx.active.start_check(at) || ctx.active.end_check(at) {
                    self.cond_talk = true;
                    ctx.events.list.push(SimEvent::Se(SE_CTRL_NPC_TO_LINK));
                } else {
                    self.return_link = false;
                    self.mode = 0;
                }
            }
        }
        if self.mode != 0 {
            self.accepted = true;
            self.mode = 0;
        }
    }
}

impl Medli {
    pub fn set_normal_speed_f(
        &mut self,
        f1: f32,
        f2: f32,
        scale: f32,
        max_step: f32,
        min_step: f32,
        wall_angle: Option<i16>,
    ) {
        let mut want = self.max_speed * f1;
        if let Some(wall) = wall_angle {
            let angle = self
                .yaw
                .wrapping_add(-0x8000)
                .wrapping_sub(wall);
            if angle.unsigned_abs() < ANGLE_HALF as u16 {
                want *= 1.0 - HIO2_M1C * walk::from_s16(angle).cos();
            }
        }
        let mut drive = f2;
        let mut target;
        let step;
        if want < self.walk_speed {
            let gap = self.walk_speed - want;
            step = if gap > max_step { max_step } else { gap };
            let step = if step < min_step { min_step } else { step };
            drive = 0.0;
            target = want;
        } else {
            step = max_step;
            target = 0.0;
        }
        if drive.abs() >= IS_ZERO {
            self.walk_speed += drive;
            if self.walk_speed > want {
                self.walk_speed = want;
            }
        } else {
            walk::add_calc(&mut self.walk_speed, target, scale, step, min_step);
        }
    }

    pub fn set_speed_and_angle_normal(&mut self, f1: f32, target: i16, wall_angle: Option<i16>) {
        let mut drive = 0.0;
        if f1 > SPEED_EPS {
            self.yaw = walk::add_calc_angle_s(
                self.yaw,
                target,
                HIO2_M24,
                HIO2_M20,
                HIO2_M22,
            );
            let mut align = walk::from_s16(target.wrapping_sub(self.yaw)).cos();
            if align < 0.0 {
                align = 0.0;
            }
            drive = HIO2_M04 * f1 * align;
        }
        self.set_normal_speed_f(f1, drive, HIO2_M14, HIO2_M0C, HIO2_M10, wall_angle);
    }

    pub fn walk_proc(&mut self, f1: f32, target: i16, wall_angle: Option<i16>) {
        self.max_speed = HIO2_M08;
        self.set_speed_and_angle_normal(f1, target, wall_angle);
        let ratio = (self.walk_speed / self.max_speed).abs();
        self.anm_rate = ratio * HIO2_M18;
        self.run_rate = ratio;
    }

    pub fn route_ang_check(&mut self, normal: Vec3, angle: &mut i16, player_dy: f32) {
        let temp = self.normal.cross(normal);
        let mut next = walk::to_s16(temp.x.atan2(temp.z));
        if (self.normal.y >= 1.0 && angle_distance(next, *angle) > ANGLE_HALF as u16)
            || player_dy * temp.y < 0.0
        {
            next = next.wrapping_add(-0x8000);
        }
        *angle = next;
    }

    pub fn route_wall_check(&mut self, cross: Option<Vec3>, angle: &mut i16, player_dy: f32) {
        if let Some(normal) = cross {
            self.route_ang_check(normal, angle, player_dy);
        }
    }

    pub fn check_forward_ground_y(
        &self,
        angle: i16,
        wall_normal: Option<Vec3>,
        probe: Option<f32>,
    ) -> f32 {
        if let Some(normal) = wall_normal {
            let facing = walk::to_s16(normal.x.atan2(normal.z));
            if angle_distance(angle, facing) > ANGLE_HALF as u16 {
                return probe.unwrap_or(ROUTE_DIST_LIMIT);
            }
        }
        ROUTE_DIST_LIMIT
    }

    pub fn check_wall_jump(
        &self,
        angle: i16,
        wall_normal: Option<Vec3>,
        probe: Option<f32>,
    ) -> f32 {
        let temp = self.check_forward_ground_y(angle, wall_normal, probe) - self.pos.y;
        if ROUTE_JUMP_FLOOR < temp && temp < ROUTE_JUMP_MAX {
            return temp.sqrt() * ROUTE_JUMP_SCALE;
        }
        -1.0
    }

    #[allow(clippy::too_many_arguments)]
    pub fn route_check(
        &mut self,
        dist_sq: f32,
        angle: &mut i16,
        wall_hit: bool,
        wall_normal: Option<Vec3>,
        probe: Option<f32>,
        cross: Option<Vec3>,
        drop_normal: Option<Vec3>,
        player_dy: f32,
    ) -> bool {
        if !self.ground_hit {
            let temp = self.pos;
            self.pos = self.old;
            self.speed_f = 0.0;
            self.was_airborne = 1;
            if let Some(normal) = drop_normal
                && angle_distance(*angle, walk::to_s16(normal.x.atan2(normal.z)))
                    > ANGLE_HALF as u16
            {
                return true;
            }
            if self.ground_y - temp.y < ROUTE_DROP_LIMIT {
                return false;
            }
            self.set_npc_action(Step::Jump, Arg::Speed(20.0));
        } else if wall_hit {
            let jump = self.check_wall_jump(*angle, wall_normal, probe);
            if jump >= 0.0 {
                self.set_npc_action(Step::Jump, Arg::Speed(jump));
                return true;
            }
            if dist_sq > ROUTE_SQUARE_LIMIT * ROUTE_SQUARE_LIMIT {
                return false;
            }
            self.route_wall_check(cross, angle, player_dy);
        }
        true
    }

    pub fn wind_proc(&mut self, hit: bool, hit_vec: Vec3) -> i16 {
        let mut blown = 1_i16;
        let mut vec = Vec3::ZERO;
        let mut power = 0.0;
        if self.has_stts(STTS_UNK8) {
            vec = self.wind_vec;
            power = self.wind_power;
        }
        if HIO_M1C6 != 0 && hit {
            vec = normalize_zp(hit_vec);
            power = HIO_M148;
        }
        let mut flat = Vec3::new(vec.x, 0.0, vec.z);
        let face = Vec3::new(
            walk::from_s16(self.shape[1]).sin(),
            0.0,
            walk::from_s16(self.shape[1]).cos(),
        );
        let mut scale = HIO_M134;
        if !(abs2_xz(flat).abs() < WIND_ABS_MIN) {
            flat = normalize_zp(flat);
            if !(face.x * flat.x + face.z * flat.z > WIND_DOT_MIN) {
                blown = 0;
                scale = HIO_M130;
            }
        }
        if blown == 0 {
            let limit = power * scale;
            self.wind_ramp += HIO_M138;
            if self.wind_ramp > limit {
                self.wind_ramp = limit;
            }
            power = self.wind_ramp;
        } else {
            power *= scale;
        }
        let flow = self.speed + vec * power;
        self.yaw = walk::to_s16(flow.x.atan2(flow.z));
        self.speed_f = abs_xz(flow).min(HIO_M13C);
        self.speed.y = flow.y;
        if self.glide_flag == 0 {
            self.max_fall = HIO_M110;
            self.gravity = HIO_M114;
        }
        if !(power.abs() < WIND_ABS_MIN) {
            let timer = calc_timer_val(&mut self.wind_timer);
            let window = HIO_M1B4.wrapping_sub(HIO_M1B6);
            if blown == 1 && timer > 0 && timer < window && (HIO_M1C7 != 0 || self.glide_flag == 0)
            {
                self.max_fall = HIO_M140;
                self.gravity = HIO_M144;
            }
        }
        blown
    }
}

pub fn calc_timer_val(value: &mut i16) -> i16 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

impl Medli {
    pub fn look_zero(&mut self) {
        let mut yaw = self.shape[1];
        self.jnt_look_at(
            &mut yaw,
            None,
            Vec3::ZERO,
            self.shape[1],
            0,
            false,
        );
        self.shape[1] = yaw;
    }

    pub fn look_target(&mut self, dst: Vec3, max_vel: i16) {
        let mut yaw = self.shape[1];
        self.jnt_look_at(&mut yaw, Some(dst), self.pos, self.shape[1], max_vel, false);
        self.shape[1] = yaw;
    }

    pub fn npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step == Step::None {
            self.speed_f = 0.0;
            self.step = if self.anm == i32::from(ANM_MWAIT) {
                Step::Mwait
            } else {
                Step::Wait
            };
        }
        let mut done = false;
        match self.step {
            Step::Wait => done = self.wait_npc_action(ctx),
            Step::HarpWait => done = self.harp_wait_npc_action(ctx),
            Step::Talk => done = self.talk_npc_action(ctx),
            Step::ShipTalk => done = self.ship_talk_npc_action(ctx),
            Step::Kyohi => done = self.kyohi_npc_action(ctx),
            Step::Ship => done = self.ship_npc_action(ctx),
            Step::Mwait => done = self.mwait_npc_action(ctx),
            Step::Squatdown => done = self.squatdown_npc_action(ctx),
            Step::Sqwait01 => done = self.sqwait01_npc_action(ctx),
            Step::Carry => done = self.carry_npc_action(ctx),
            Step::Throw => done = self.throw_npc_action(ctx),
            Step::Gliding => done = self.gliding_npc_action(ctx),
            Step::Fall => done = self.fall_npc_action(ctx),
            Step::Fall02 => done = self.fall02_npc_action(ctx),
            Step::WallHit => done = self.wall_hit_npc_action(ctx),
            Step::Land01 => done = self.land01_npc_action(ctx),
            Step::Land02 => done = self.land02_npc_action(ctx),
            Step::Land03 => done = self.land03_npc_action(ctx),
            Step::Piyo2 => done = self.piyo2_npc_action(ctx),
            Step::Delete => done = self.delete_npc_action(ctx),
            Step::DemoFly => done = self.demo_fly_npc_action(ctx),
            Step::Search => done = self.search_npc_action(ctx),
            Step::Hit => done = self.hit_npc_action(ctx),
            Step::Jump => done = self.jump_npc_action(ctx),
            Step::Escape => done = self.escape_npc_action(ctx),
            Step::None => {}
        }
        done
    }

    pub fn wait_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.clr_attn_bit(ATTN_ACTION_CARRY);
            if self.is_type_sea() {
                let body = i32::from(self.anm_body);
                if self.anm == 0x1E
                    || self.anm == 0x29
                    || body == 0x0E
                    || body == 0x12
                    || body == 0x13
                    || body == 0x1A
                    || body == 0x25
                {
                    self.set_harp_play_num();
                } else {
                    self.set_anm(0x12);
                }
            } else if self.is_type_dra09() && !ctx.stories.test(BIT_1140) {
                self.set_anm(0x22);
            } else if self.is_type_field() {
                if !self.has_stts(STTS_UNK80) {
                    self.set_anm(0);
                }
            } else {
                self.set_anm(0);
            }
            self.max_fall = MAX_FALL;
            self.gravity = HIO_M0F4;
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_FLY);
            if !self.is_type_sea() {
                self.set_stts(STTS_UNK4);
            } else {
                self.clr_stts(STTS_UNK4);
            }
            self.shape[0] = 0;
            self.shape[2] = 0;
            self.speed_f = 0.0;
            self.height = HEIGHT;
            self.timer_a = WAIT_HARP_FRAMES;
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            if self.anim_ended {
                if self.anm == 0x12 {
                    self.set_harp_play_num();
                } else if self.anm == 0x13 || self.anm == 0x21 {
                    self.set_anm(0);
                } else if self.anm == 0x16 || self.anm == 0x17 || self.anm == 0x11 {
                    self.set_harp_play_num();
                }
            }
            if self.is_type_adanmae() && ctx.stories.test(BIT_1102) {
                return true;
            }
            let mut sp08 = 0_i32;
            let dra09_letter = ctx.stories.test(BIT_1101);
            if self.is_type_sea() && !ctx.stories.test(BIT_1402) {
                if let Some(player) = ctx.player {
                    let delta = player - self.pos;
                    let dist_xz2 = delta.x * delta.x + delta.z * delta.z;
                    let dy = delta.y;
                    if dist_xz2 < HIO_M0CC * HIO_M0CC && dy < HIO_M0BC && dy > HIO_M0C0 {
                        self.mode = MODE_EVENT7;
                    }
                }
            } else {
                let angle = self
                    .shape[1]
                    .wrapping_add(self.head_y())
                    .wrapping_add(self.backbone_y());
                self.clr_attn_bit(ATTN_LOCKON_TALK | ATTN_ACTION_SPEAK);
                self.attn_seen = self.chk_attention(ctx.link, self.pos, angle, 1, dra09_letter);
                sp08 = i32::from(self.attn_seen);
                if sp08 != 0 {
                    if self.is_type_atorizk() || self.is_type_adanmae() || self.is_type_sea() {
                        self.set_attn_bit(ATTN_LOCKON_TALK | ATTN_ACTION_SPEAK);
                    }
                    if self.is_type_atorizk() || self.is_type_adanmae() {
                        self.mode = MODE_TALK_ITEM;
                    } else if self.is_type_dra09() {
                        if ctx.stories.test(BIT_1140) {
                            if dra09_letter {
                                self.set_attn_bit(ATTN_ACTION_SPEAK);
                                if ctx.text.info.check_player_status0(PY_STTS0_800000) {
                                    ctx.stories.set(BIT_1280);
                                }
                                self.mode = if ctx.stories.test(BIT_1280) {
                                    MODE_TALK_ITEM
                                } else {
                                    MODE_TALK
                                };
                            } else {
                                self.set_attn_bit(ATTN_LOCKON_TALK | ATTN_ACTION_SPEAK);
                                self.mode = MODE_TALK_ITEM;
                            }
                        } else {
                            self.set_anm(0x22);
                        }
                    } else if self.is_type_sea() {
                        if self.mode == 0 && ctx.stories.test(BIT_1402) {
                            self.mode = MODE_TALK_RETRY;
                        }
                    } else {
                        self.set_attn_bit(ATTN_ACTION_CARRY);
                    }
                }
                sp08 = self.npc_call(ctx);
            }
            if self.is_type_dra09() && dra09_letter && ctx.switches.test(self.switch_no) {
                self.set_npc_action(Step::DemoFly, Arg::None);
            }
            if self.is_type_field() {
                if self.anm == 0 {
                    if Medli::calc_timer(&mut self.timer_a) {
                        self.set_anm(40);
                    }
                } else if self.anm == 0x28 && self.anim_ended {
                    self.set_anm(0);
                    self.timer_a = ((rnd_f(&mut self.seed, 180.0)) as i16).wrapping_add(60);
                }
            }
            if self.is_type_atorizk() {
                if self.chk_area(ctx.player, self.pos) {
                    self.look_back(sp08 != 0, false, true);
                } else {
                    self.look_back(false, false, false);
                }
            } else if self.is_type_dra09() {
                if ctx.stories.test(BIT_1140) {
                    self.look_back(true, false, false);
                    self.set_anm(0);
                } else {
                    self.look_back(true, false, true);
                }
            } else {
                if self.anm == 0x16 || self.anm == 0x17 {
                    self.look_zero();
                }
                if self.chk_area(ctx.player, self.pos) {
                    if self.is_type_sea() {
                        self.look_back(sp08 != 0, false, true);
                    } else {
                        self.look_back(sp08 != 0, false, false);
                    }
                } else {
                    self.look_back(false, false, false);
                }
            }
            self.wait_ground_check(ctx.special_ground, ctx.is_player);
            self.yaw = self.shape[1];
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn harp_wait_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.set_harp_play_num();
            self.max_fall = MAX_FALL;
            self.gravity = HIO_M0F4;
            self.stts = 0;
            self.set_stts(STTS_UNK80);
            self.shape[0] = 0;
            self.shape[2] = 0;
            self.speed_f = 0.0;
            self.height = HEIGHT;
            self.step_stat += 1;
            self.timer_a = 0;
            self.timer_b = HARP_RETRY_FRAMES;
        } else if self.step_stat != STEP_ENDING {
            if self.step_stat == STEP_ONGOING_1 {
                if self.anim_ended {
                    self.timer_b -= 1;
                    if self.timer_b != 0 {
                        self.set_harp_play_num();
                    } else {
                        self.set_anm(0x11);
                        self.timer_b = HARP_RESUME_FRAMES;
                        self.step_stat += 1;
                    }
                }
            } else {
                self.timer_b -= 1;
                if self.timer_b == 0 {
                    self.set_harp_play_num();
                    self.timer_b = HARP_RETRY_FRAMES;
                    self.step_stat = STEP_ONGOING_1;
                }
            }
            if self.is_type_m_daib()
                && self.timer_b == HARP_RETRY_FRAMES
                && (self.anm == 0x16 || self.anm == 0x17)
                && self.anm_frame == HARP_FRAME_11
            {
                ctx.events.list.push(SimEvent::Bgm(BGM_MEDORI_TAKT_8));
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }
}

impl Medli {
    pub fn talk_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
            self.msg_no = self.get_msg(ctx);
            self.clr_attn_bit(ATTN_ACTION_CARRY);
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_FLY);
            self.set_stts(STTS_UNK4);
            self.attn_count = 0;
            self.height = HEIGHT;
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            let angle = self
                .shape[1]
                .wrapping_add(self.head_y())
                .wrapping_add(self.backbone_y());
            let dra09_letter = ctx.stories.test(BIT_1101);
            self.attn_seen = self.chk_attention(ctx.link, self.pos, angle, 0, dra09_letter);
            if self.step_stat == STEP_ONGOING_1 {
                if !self.is_xy_talk() || ctx.xy_end {
                    self.step_stat = STEP_ONGOING_2;
                }
            } else if self.step_stat == STEP_ONGOING_2 {
                if self.talk_init(ctx) {
                    self.step_stat = STEP_ONGOING_3;
                }
            } else if self.step_stat == STEP_ONGOING_3 {
                if self.talk(false, false, ctx) {
                    if self.is_type_atorizk() || self.is_type_dra09() {
                        self.step_stat = STEP_ONGOING_4;
                    } else if self.is_type_adanmae() {
                        if ctx.stories.test(BIT_1104) {
                            self.set_npc_action(Step::Squatdown, Arg::None);
                        }
                    } else {
                        self.set_npc_action(Step::Wait, Arg::None);
                    }
                    ctx.reset_event = true;
                }
            } else if self.is_type_atorizk() {
                if ctx.stories.test(BIT_GAVE_LETTER) {
                    self.set_npc_action(Step::Wait, Arg::None);
                }
            } else if self.is_type_dra09() && dra09_letter {
                self.set_npc_action(Step::Wait, Arg::None);
            }
            self.jnt_set_trn();
            self.look_back(true, false, false);
            self.yaw = self.shape[1];
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn ship_talk_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.npc_not_change = true;
            self.set_stts(STTS_SHIP_RIDE);
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
            self.msg_no = self.get_msg(ctx);
            self.set_anm(0x2b);
            self.attn_count = 0;
            self.step_stat += 1;
        } else if self.step_stat == STEP_ENDING {
            self.clr_stts(STTS_SHIP_RIDE);
        } else {
            let angle = self
                .shape[1]
                .wrapping_add(self.head_y())
                .wrapping_add(self.backbone_y());
            let dra09_letter = ctx.stories.test(BIT_1101);
            self.attn_seen = self.chk_attention(ctx.link, self.pos, angle, 0, dra09_letter);
            if self.step_stat == STEP_ONGOING_1 {
                if self.talk_init(ctx) {
                    self.step_stat = STEP_ONGOING_2;
                }
            } else if self.talk(false, false, ctx) {
                ctx.reset_event = true;
                self.set_npc_action(Step::Ship, Arg::None);
            }
            self.look_back(true, false, true);
            self.yaw = self.shape[1];
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn kyohi_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_FLY);
            self.set_stts(STTS_UNK4);
            self.set_anm(0x27);
            self.speed_f = 0.0;
            self.height = HEIGHT;
            self.timer_a = 2;
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            if self.anim_ended {
                self.timer_a -= 1;
                if self.timer_a == 0 {
                    self.set_npc_action(Step::Wait, Arg::None);
                }
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn ship_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.npc_not_change = true;
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_FLY);
            self.set_stts(STTS_UNK4);
            self.set_anm(0x23);
            self.speed_f = 0.0;
            self.height = HEIGHT;
            if ctx.ship_pos.is_some() {
                self.set_stts(STTS_SHIP_RIDE);
            }
            self.step_stat += 1;
        } else if self.step_stat == STEP_ENDING {
            self.clr_stts(STTS_SHIP_RIDE);
        } else {
            self.clr_attn_bit(ATTN_LOCKON_TALK | ATTN_ACTION_SPEAK);
            if ctx.ship_pos.is_some() {
                self.set_stts(STTS_SHIP_RIDE);
                if ctx.ship_speed.abs() < VELOCITY_EPS {
                    self.set_anm(0x2a);
                    if !ctx.ship_riding {
                        let angle = self
                            .shape[1]
                            .wrapping_add(self.head_y())
                            .wrapping_add(self.backbone_y());
                        let dra09_letter = ctx.stories.test(BIT_1101);
                        self.attn_seen =
                            self.chk_attention(ctx.link, self.pos, angle, 0, dra09_letter);
                        if self.attn_seen {
                            self.set_attn_bit(ATTN_LOCKON_TALK | ATTN_ACTION_SPEAK);
                            self.mode = MODE_TALK_ITEM;
                        }
                    }
                } else {
                    self.set_anm(0x23);
                }
            }
            if self.anm == 0x23 {
                self.look_zero();
            } else {
                self.look_back(true, false, true);
            }
            self.yaw = self.shape[1];
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn mwait_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.set_attn_bit(ATTN_ACTION_CARRY);
            self.call_command = false;
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_UNK4 | STTS_FLY);
            self.set_anm(0x20);
            self.shape[0] = 0;
            self.shape[2] = 0;
            self.gravity = HIO_M0F4;
            self.max_fall = MAX_FALL;
            self.speed_f = 0.0;
            self.speed.y = 0.0;
            self.height = HEIGHT;
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            if self.step_stat == STEP_ONGOING_1 {
                if !self.is_light_body_hit() {
                    self.step_stat = STEP_ONGOING_3;
                } else {
                    self.clr_attn_bit(ATTN_LOCKON_TALK | ATTN_ACTION_SPEAK);
                    let dist_sq = ctx
                        .player
                        .map_or(0.0, |player| abs2(player - self.pos));
                    if !self.call_command {
                        if dist_sq < HIO_M0C8 * HIO_M0C8 {
                            let _ = self.npc_call(ctx);
                        }
                    } else {
                        self.step_stat += 1;
                        if dist_sq < (HIO_M0C4 * 2.0) * (HIO_M0C4 * 2.0) {
                            self.step_stat += 1;
                        }
                    }
                }
                if self.step_stat != STEP_ONGOING_1 {
                    self.waist_angle = 0;
                    self.waist_speed = 0.0;
                    self.waist_rot_x = 0;
                    self.waist_rot_y = 0;
                    self.set_anm(0x21);
                    self.anim_ended = false;
                }
            } else if self.look_back_waist(self.waist_angle, self.waist_speed) && self.anim_ended {
                self.waist_rot_x = 0;
                self.waist_rot_y = 0;
                if self.step_stat == STEP_ONGOING_2 {
                    self.set_npc_action(Step::Search, Arg::None);
                } else {
                    self.set_anm(0);
                    self.set_npc_action(Step::Wait, Arg::None);
                }
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn squatdown_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_FLY);
            self.set_stts(STTS_UNK4);
            self.set_anm(3);
            self.speed_f = 0.0;
            self.height = HEIGHT_SQUAT;
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            if self.anim_ended {
                self.set_npc_action(Step::Sqwait01, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn sqwait01_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.set_attn_bit(ATTN_ACTION_CARRY);
            if ctx.stories.test(BIT_GAVE_LETTER) {
                self.clr_attn_bit(ATTN_LOCKON_TALK | ATTN_ACTION_SPEAK);
            }
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_FLY);
            self.set_stts(STTS_UNK4);
            self.set_anm(4);
            self.max_fall = MAX_FALL;
            self.gravity = HIO_M0F4;
            self.speed_f = 0.0;
            self.height = HEIGHT_SQUAT;
            self.shape[0] = 0;
            self.shape[2] = 0;
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            if i8::try_from(self.talk_type).unwrap_or(0) >= 4 {
                self.set_npc_action(Step::Wait, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn piyo2_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.set_attn_bit(ATTN_LOCKON_TALK | ATTN_ACTION_SPEAK);
            self.max_fall = MAX_FALL;
            self.gravity = HIO_M0F4;
            self.set_anm(0x14);
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_UNK4 | STTS_FLY);
            self.shape[0] = 0;
            self.shape[2] = 0;
            self.speed_f = 0.0;
            self.height = HEIGHT;
            self.timer_a = HIO_M1BA;
            if self.is_type_edaichi() || self.is_type_m_dai() {
                self.timer_a = self.timer_a >> 1;
            }
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            ctx.events.list.push(SimEvent::Se(SE_CM_MD_PIYO));
            let angle = self
                .shape[1]
                .wrapping_add(self.head_y())
                .wrapping_add(self.backbone_y());
            if Medli::calc_timer(&mut self.timer_a) {
                self.set_npc_action(Step::Wait, Arg::None);
            }
            let dra09_letter = ctx.stories.test(BIT_1101);
            self.attn_seen = self.chk_attention(ctx.link, self.pos, angle, 0, dra09_letter);
            if self.attn_seen {
                if self.is_type_atorizk()
                    || self.is_type_adanmae()
                    || self.is_type_dra09()
                    || self.is_type_sea()
                {
                    self.set_attn_bit(ATTN_LOCKON_TALK | ATTN_ACTION_SPEAK);
                } else {
                    self.clr_attn_bit(ATTN_LOCKON_TALK | ATTN_ACTION_SPEAK);
                }
                if self.is_type_adanmae() {
                    self.mode = MODE_TALK_ITEM;
                }
            }
            let _ = self.npc_call(ctx);
            self.look_back(true, true, false);
            self.yaw = self.shape[1];
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn delete_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            ctx.delete_me = true;
        }
        false
    }
}

impl Medli {
    pub fn demo_fly_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.attn_flags = 0;
            self.step_stat += 1;
            self.yaw = DEMO_FLY_YAW;
        } else if self.step_stat == STEP_ENDING {
        } else {
            self.attn_flags = 0;
            if self.step_stat == STEP_ONGOING_1 {
                if self.jnt_trn_chk() {
                    stream::add_calc_angle_s2(
                        &mut self.turn_vel,
                        HIO_NPC_MAX_HEAD_TURN_VEL,
                        HEAD_SCALE,
                        HEAD_MAX_STEP,
                    );
                } else {
                    self.turn_vel = 0;
                    self.set_anm(DEMO_FLY_ANM_READY);
                    self.step_stat += 1;
                }
                self.look_target(DEMO_FLY_POS, self.turn_vel);
            } else if self.step_stat == STEP_ONGOING_2 {
                if self.anim_ended {
                    self.set_anm(DEMO_FLY_ANM_WING);
                    self.set_stts(STTS_UNK1);
                    self.speed.y = 0.0;
                    self.gravity = 1.0;
                    self.max_fall = 100.0;
                    self.step_stat += 1;
                }
            } else if self.step_stat == STEP_ONGOING_3 {
                if self.anim_ended {
                    self.set_anm(DEMO_FLY_ANM_UP);
                }
                if self.pos.y > DEMO_FLY_ALT {
                    self.set_anm(DEMO_FLY_ANM_GLIDE);
                    self.speed.y = 0.0;
                    self.gravity = 0.0;
                    self.max_fall = 0.0;
                    self.speed_f = DEMO_FLY_SPEED;
                    self.timer_a = DEMO_FLY_DELETE;
                    self.step_stat += 1;
                }
            } else if self.step_stat == STEP_ONGOING_4 && Medli::calc_timer(&mut self.timer_a) {
                ctx.delete_me = true;
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn carry_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.npc_not_change = true;
            self.clr_attn_bit(ATTN_ACTION_CARRY);
            self.call_command = false;
            self.timer_a = self.shape[1].wrapping_sub(ctx.player_shape);
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_UNK4 | STTS_FLY);
            let se;
            if self.is_no_carry() {
                self.set_anm(0x24);
                self.set_stts(STTS_UNK1);
                se = SE_CV_MD_CRASH;
            } else {
                if self.anm == 4 {
                    self.set_anm(5);
                } else {
                    self.set_anm(3);
                    self.anim_ended = false;
                }
                se = SE_CV_MD_LIFT_UP;
            }
            ctx.events.list.push(SimEvent::Se(se));
            self.height = HEIGHT_SQUAT;
            self.step_stat += 1;
        } else if self.step_stat == STEP_ENDING {
            self.npc_not_change = false;
            self.offset.y = 0.0;
            self.offset.z = 0.0;
            self.clr_stts(STTS_CARRY_ACTION);
        } else {
            self.attn_count = 0;
            let mut b_var1 = false;
            if ctx.carried {
                self.set_restart(2);
            }
            if !self.is_no_carry() {
                let flying = ctx.auto_jump;
                if self.step_stat == STEP_ONGOING_1 {
                    if flying > 0 {
                        self.step_stat = STEP_ONGOING_2;
                        self.set_anm(9);
                        self.set_stts(STTS_UNK1);
                        self.anim_ended = false;
                        ctx.events.list.push(SimEvent::Se(SE_CV_MD_FLY_WITH_LINK));
                        if ctx.stories.test(BIT_4001) {
                            ctx.stories.set(BIT_4180);
                        }
                    } else if self.anm == 5 {
                    } else if self.anim_ended {
                        self.set_anm(5);
                    }
                } else if self.step_stat == STEP_ONGOING_2 {
                    if flying < 0x1e {
                        self.step_stat = STEP_ONGOING_3;
                        self.anm_rate = HIO_M120;
                    }
                } else if flying <= 0 {
                    self.step_stat = STEP_ONGOING_1;
                    self.effect |= 1;
                    self.set_anm(5);
                    self.clr_stts(STTS_UNK1);
                    b_var1 = true;
                    ctx.events.list.push(SimEvent::Se(SE_CV_MD_FLY_END));
                }
            }
            if !ctx.carried {
                if self.is_no_carry() {
                    if self.speed_f > 0.0 {
                        self.set_npc_action(Step::Throw, Arg::None);
                    } else {
                        self.set_npc_action(Step::Land03, Arg::None);
                    }
                } else if self.has_stts(STTS_UNK1) || b_var1 {
                    let ground_h = self.ground_y;
                    if self.pos.y - ground_h >= HIO_M1AC {
                        self.set_npc_action(Step::Fall02, Arg::None);
                    } else {
                        self.set_npc_action(Step::Fall, Arg::None);
                    }
                } else if self.speed_f > 0.0 {
                    self.set_npc_action(Step::Throw, Arg::None);
                } else {
                    self.set_npc_action(Step::Sqwait01, Arg::None);
                }
            }
            self.look_zero();
            self.yaw = self.shape[1];
        }
        false
    }

    pub fn set_restart(&mut self, point: i16) {
        let _ = point;
    }

    pub fn throw_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            ctx.events.list.push(SimEvent::Se(SE_CV_MD_THROW));
            self.wind_flag = 1;
            self.wind_timer = HIO_M1B4;
            self.glide_flag = 0;
            self.wind_ramp = 0.0;
            self.speed_f = HIO_M0E8;
            self.speed.y = HIO_M0EC;
            self.max_fall = HIO_M110;
            self.gravity = HIO_M114;
            self.start_pos = self.pos;
            self.clr_stts(STTS_UNK4);
            self.set_stts(STTS_UNK1 | STTS_FLY);
            self.set_anm(6);
            self.step_stat += 1;
        } else if self.step_stat == STEP_ENDING {
        } else {
            let frame = self.anm_frame;
            if frame == GLIDE_WING_FRAME && self.speed.y < HIO_M0F0 {
                self.speed.y = HIO_M0F0;
            }
            if frame > HIO_M154 {
                let result = self.wind_proc(ctx.wind_hit, ctx.wind_vec);
                if result == 0 {
                    self.wind_flag = 0;
                }
            }
            if self.ground_hit {
                self.set_npc_action(Step::Land01, Arg::None);
            } else if ctx.wall_hit {
                self.set_npc_action(Step::WallHit, Arg::None);
            } else if self.anim_ended {
                self.set_npc_action(Step::Gliding, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn gliding_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            if self.wind_flag == 1 {
                self.glide_flag = 0;
                self.set_anm(0xb);
            } else {
                self.glide_flag = 1;
                self.set_anm(10);
            }
            self.glide_timer = GLIDE_STEPS;
            self.clr_stts(STTS_UNK4);
            self.set_stts(STTS_UNK1 | STTS_FLY);
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            if !self.chk_adanmae_demo_order() {
                if self.ground_hit {
                    if self.is_type_adanmae() {
                        self.set_npc_action(Step::Land01, Arg::Flag);
                    } else {
                        self.set_npc_action(Step::Land01, Arg::None);
                    }
                } else if ctx.wall_hit {
                    if self.is_type_adanmae() {
                        self.set_npc_action(Step::WallHit, Arg::Flag);
                    } else {
                        self.set_npc_action(Step::WallHit, Arg::None);
                    }
                } else if self.glide_flag == 0 {
                    let result = self.wind_proc(ctx.wind_hit, ctx.wind_vec);
                    if self.wind_flag != result {
                        if result == 0 {
                            self.set_anm(10);
                            self.glide_flag = 1;
                        } else {
                            self.set_anm(0xb);
                        }
                    }
                    self.wind_flag = result;
                } else if self.anim_ended {
                    self.glide_timer -= 1;
                    if self.glide_timer != 0 {
                        self.set_anm(10);
                    } else {
                        if self.is_type_adanmae() {
                            self.set_npc_action(Step::Fall02, Arg::Flag);
                        } else {
                            self.set_npc_action(Step::Fall02, Arg::None);
                        }
                        return true;
                    }
                }
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn fall_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.clr_stts(STTS_UNK1 | STTS_UNK4);
            self.set_stts(STTS_FLY);
            self.set_anm(0xc);
            self.height = HEIGHT;
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            if self.ground_hit {
                self.set_npc_action(Step::Land01, Arg::None);
            } else if ctx.wall_hit {
                self.set_npc_action(Step::WallHit, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn fall02_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.clr_stts(STTS_UNK1 | STTS_UNK4);
            self.set_stts(STTS_FLY);
            self.max_fall = HIO_M110;
            self.gravity = HIO_M114;
            self.height = HEIGHT;
            self.set_anm(0xd);
            self.step_stat += 1;
            if self.has_arg() {
                self.timer_a = 1;
            } else {
                self.timer_a = 0;
            }
            self.start_y = self.pos.y;
        } else if self.step_stat != STEP_ENDING {
            if self.timer_a != 0 {
                if !self.chk_adanmae_demo_order() && self.ground_hit {
                    self.set_npc_action(Step::Land03, Arg::None);
                }
            } else if self.ground_hit {
                self.set_npc_action(Step::Land03, Arg::None);
            } else if ctx.wall_hit {
                self.set_npc_action(Step::WallHit, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn wall_hit_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            ctx.events.list.push(SimEvent::Se(SE_CV_MD_CRASH));
            self.max_fall = MAX_FALL;
            self.gravity = HIO_M0F4;
            self.set_anm(0xd);
            self.shape[0] = 0;
            self.shape[2] = 0;
            if ctx.wall_hit {
                self.yaw = ctx.wall_angle;
            }
            self.speed_f = HIO_M150;
            self.clr_stts(STTS_UNK1 | STTS_UNK4);
            self.set_stts(STTS_UNK2 | STTS_FLY);
            self.height = HEIGHT;
            self.step_stat += 1;
            if self.has_arg() {
                self.timer_a = 1;
            } else {
                self.timer_a = 0;
            }
        } else if self.step_stat != STEP_ENDING {
            if self.timer_a != 0 {
                if !self.chk_adanmae_demo_order() && self.ground_hit {
                    self.set_npc_action(Step::Land02, Arg::None);
                }
            } else if self.ground_hit {
                self.set_npc_action(Step::Land02, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn land01_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            ctx.events.list.push(SimEvent::Se(SE_CV_MD_LANDING));
            if self.has_stts(STTS_UNK1) {
                self.effect |= 1;
            }
            self.max_fall = MAX_FALL;
            self.gravity = HIO_M0F4;
            self.set_anm(0xe);
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_UNK4 | STTS_FLY);
            self.shape[0] = 0;
            self.shape[2] = 0;
            self.speed_f = 0.0;
            self.height = HEIGHT;
            self.step_stat += 1;
            if self.has_arg() {
                self.timer_a = 1;
            } else {
                self.timer_a = 0;
            }
        } else if self.step_stat != STEP_ENDING {
            if self.timer_a != 0 {
                if !self.chk_adanmae_demo_order() && self.anim_ended {
                    self.set_npc_action(Step::Wait, Arg::None);
                }
            } else if self.anim_ended {
                self.set_npc_action(Step::Wait, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn land02_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.max_fall = MAX_FALL;
            self.gravity = HIO_M0F4;
            self.set_anm(0xf);
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_UNK4 | STTS_FLY);
            self.shape[0] = 0;
            self.shape[2] = 0;
            self.speed_f = 0.0;
            self.height = HEIGHT;
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            if self.anim_ended {
                self.set_npc_action(Step::Piyo2, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn land03_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.max_fall = MAX_FALL;
            self.gravity = HIO_M0F4;
            self.set_anm(0xf);
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_UNK4 | STTS_FLY);
            self.shape[0] = 0;
            self.shape[2] = 0;
            self.speed_f = 0.0;
            self.height = HEIGHT;
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            if self.anim_ended {
                self.set_npc_action(Step::Wait, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn jump_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_UNK4 | STTS_FLY);
            self.set_stts(STTS_UNK4);
            if let Arg::Speed(speed) = self.arg {
                self.speed.y = speed;
            }
            self.shape[0] = 0;
            self.shape[2] = 0;
            self.speed_f = 4.0;
            self.gravity = HIO_M0F4;
            self.max_fall = MAX_FALL;
            self.set_anm(0xc);
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            if self.step_stat == STEP_ONGOING_1 {
                if self.ground_hit {
                    self.speed_f = 0.0;
                    self.set_anm(0xe);
                    self.anim_ended = false;
                    self.step_stat += 1;
                }
            } else if self.anim_ended {
                self.speed_f = 0.0;
                self.set_npc_action(Step::Wait, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }
}

impl Medli {
    pub fn search_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.set_stts(STTS_UNK4);
            self.set_attn_bit(ATTN_ACTION_CARRY);
            if self.anm == 0x11 || self.anm == 0x16 || self.anm == 0x17 {
                self.set_anm(0x13);
                self.step_stat += 1;
            } else if self.anm == 0x20 {
                self.set_anm(0x21);
                self.anim_ended = false;
                self.step_stat += 1;
            } else {
                self.set_anm(2);
                self.step_stat += 2;
            }
            if !self.ground_hit {
                let delta = self.ground_y - self.pos.y;
                if delta < 0.0 && delta >= ROUTE_SNAP_MIN {
                    self.pos.y = self.ground_y;
                    self.ground_hit = true;
                    self.speed.y = 0.0;
                }
            }
        } else if self.step_stat != STEP_ENDING {
            if self.step_stat == STEP_ONGOING_1 {
                if self.anim_ended {
                    self.set_anm(2);
                    self.step_stat += 1;
                }
            } else {
                self.attn_seen = true;
                let mut temp = 0.0;
                let mut angle = 0_i16;
                let mut adjusted = 0_i16;
                let mut dist_sq = 0.0;
                if let Some(player) = ctx.player {
                    let delta = player - self.pos;
                    dist_sq = delta.x * delta.x + delta.z * delta.z;
                    if dist_sq < HIO_M0C4 * HIO_M0C4 {
                        temp = 0.0;
                    } else {
                        let mut span = dist_sq - HIO_M0C4 * HIO_M0C4;
                        if span > SEARCH_SPEED_MAX {
                            span = SEARCH_SPEED_MAX;
                        }
                        temp = span / SEARCH_SPEED_DIV;
                        if temp < SEARCH_SPEED_MIN {
                            temp = SEARCH_SPEED_MIN;
                        }
                    }
                    angle = target_angle_y(self.pos, player);
                    adjusted = angle;
                }
                let routed = self.route_check(
                    dist_sq,
                    &mut adjusted,
                    ctx.wall_hit,
                    ctx.wall_normal,
                    ctx.probe_y,
                    ctx.cross_normal,
                    ctx.drop_normal,
                    ctx.player.map_or(0.0, |player| player.y - self.pos.y),
                );
                let route_ok =
                    routed && angle_distance(angle, adjusted) <= SEARCH_ANGLE_TOL as u16;
                if ctx.door_command || !route_ok || ctx.player_locked {
                    temp = 0.0;
                    self.call_command = false;
                    ctx.events.list.push(SimEvent::Se(SE_CV_MD_LEFT_ALONE));
                    if !route_ok {
                        self.set_npc_action(Step::Kyohi, Arg::None);
                        return true;
                    }
                } else if !ctx.door_command {
                    self.set_restart(2);
                    if Medli::calc_timer(&mut self.search_timer) {
                        self.npc_flip ^= 1;
                        self.search_timer = rnd_value(
                            &mut self.seed,
                            SEARCH_RND_MIN,
                            SEARCH_RND_SPAN,
                        ) as i16;
                    }
                }
                let wall = if ctx.wall_hit {
                    Some(ctx.wall_angle)
                } else {
                    None
                };
                self.walk_proc(temp, adjusted, wall);
                self.shape[1] = walk::add_calc_angle_s(
                    self.shape[1],
                    self.yaw,
                    HIO2_M24,
                    HIO2_M20 * 2,
                    HIO2_M22 * 2,
                );
                let mut temp_angle = self.shape[1];
                let aim = player_eye_pos(ctx.player.unwrap_or(Vec3::ZERO));
                let mut src = self.pos;
                src.y = self.eye.y;
                self.jnt_look_at(
                    &mut temp_angle,
                    Some(aim),
                    src,
                    self.shape[1],
                    HIO_NPC_MAX_TURN_STEP,
                    false,
                );
                if self.speed_f < VELOCITY_EPS {
                    self.set_npc_action(Step::Wait, Arg::None);
                } else {
                    self.shape[1] = temp_angle;
                }
                self.look_back(true, false, false);
                self.set_attention(true, self.pos, ctx.is_player);
            }
        }
        false
    }

    pub fn hit_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            ctx.events.list.push(SimEvent::Se(SE_CV_MD_CRASH));
            let angle = match self.arg {
                Arg::Angle(value) => value,
                _ => 0,
            };
            self.yaw = angle;
            self.speed_f = 10.0;
            self.speed.y = 20.0;
            self.clr_attn_bit(ATTN_ACTION_CARRY);
            self.clr_stts(STTS_UNK1 | STTS_UNK4);
            self.set_stts(STTS_UNK2);
            self.height = HEIGHT;
            self.set_anm(0xD);
            self.step_stat += 1;
        } else if self.step_stat != STEP_ENDING {
            if self.ground_hit {
                self.speed_f = 0.0;
                self.set_npc_action(Step::Land03, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn escape_npc_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.step_stat == STEP_STARTING {
            self.set_stts(STTS_UNK4);
            self.set_attn_bit(ATTN_ACTION_CARRY);
            if self.anm == 0x11 || self.anm == 0x16 || self.anm == 0x17 {
                self.set_anm(0x13);
                self.step_stat += 1;
            } else if self.anm == 0x20 {
                self.set_anm(0x21);
                self.anim_ended = false;
                self.step_stat += 1;
            } else {
                self.set_anm(2);
                self.step_stat += 2;
            }
            if !self.ground_hit {
                let delta = self.ground_y - self.pos.y;
                if delta < 0.0 && delta >= ROUTE_SNAP_MIN {
                    self.pos.y = self.ground_y;
                    self.ground_hit = true;
                    self.speed.y = 0.0;
                }
            }
            self.timer_a = 0;
            self.timer_b = ESCAPE_FRAMES;
        } else if self.step_stat != STEP_ENDING {
            if self.step_stat == STEP_ONGOING_1 {
                if self.anim_ended {
                    self.set_anm(2);
                    self.step_stat += 1;
                }
            } else {
                self.attn_seen = true;
                Medli::calc_timer(&mut self.timer_b);
                if self.block_dir != 0 {
                    let sign = if self.block_dir < 0 { -0x4000_i16 } else { 0x4000_i16 };
                    self.timer_a = sign.wrapping_add(walk::to_s16(
                        self.block_pos.x.atan2(self.block_pos.z),
                    ));
                } else if self.timer_b == 0 {
                    self.set_npc_action(Step::Wait, Arg::None);
                    return true;
                }
                let mut local = self.timer_a;
                let wall = if ctx.wall_hit {
                    Some(ctx.wall_angle)
                } else {
                    None
                };
                let before = local;
                let _ = self.route_check(
                    250000.0,
                    &mut local,
                    ctx.wall_hit,
                    ctx.wall_normal,
                    ctx.probe_y,
                    ctx.cross_normal,
                    ctx.drop_normal,
                    ctx.player.map_or(0.0, |player| player.y - self.pos.y),
                );
                let _ = angle_distance(before, local);
                self.walk_proc(1.0, local, wall);
                self.shape[1] = walk::add_calc_angle_s(
                    self.shape[1],
                    self.yaw,
                    HIO2_M24,
                    HIO2_M20 * 2,
                    HIO2_M22 * 2,
                );
                if self.speed_f < VELOCITY_EPS {
                    self.set_npc_action(Step::Wait, Arg::None);
                }
                self.look_back(false, false, false);
                self.set_attention(true, self.pos, ctx.is_player);
            }
        }
        false
    }
}

impl Medli {
    pub fn player_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.player_step == PlayerStep::None {
            self.speed_f = 0.0;
            self.set_player_action(PlayerStep::Wait, Arg::None);
        }
        match self.player_step {
            PlayerStep::Wait => self.wait_player_action(ctx),
            PlayerStep::Walk => self.walk_player_action(ctx),
            PlayerStep::Hit => self.hit_player_action(ctx),
            PlayerStep::Jump => self.jump_player_action(ctx),
            PlayerStep::Fly => self.fly_player_action(ctx),
            PlayerStep::Land => self.land_player_action(ctx),
            PlayerStep::Mkamae => self.mkamae_player_action(ctx),
            PlayerStep::Carry => self.carry_player_action(ctx),
            PlayerStep::None => false,
        }
    }

    pub fn get_stick_ang_y(&self, absolute: bool) -> i16 {
        if absolute {
            self.stick_angle
        } else {
            self.stick_angle.wrapping_add(-0x8000).wrapping_add(self.player_yaw)
        }
    }

    pub fn calc_stick_pos(&self, angle: i16, lock: Option<Vec3>) -> (i32, Vec3) {
        if let Some(eye) = lock {
            return (1, eye);
        }
        (
            0,
            Vec3::new(
                self.pos.x + 100.0 * walk::from_s16(angle).sin(),
                self.pos.y,
                self.pos.z + 100.0 * walk::from_s16(angle).cos(),
            ),
        )
    }

    pub fn fly_check(&mut self) -> bool {
        if !self.a_button {
            return false;
        }
        if self.is_light_body_hit() {
            self.set_player_action(PlayerStep::Mkamae, Arg::None);
        } else {
            self.set_player_action(PlayerStep::Fly, Arg::None);
        }
        true
    }

    pub fn wait_player_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.player_stat == STEP_STARTING {
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_FLY);
            self.set_stts(STTS_UNK4);
            self.set_anm(0);
            self.speed_f = 0.0;
            self.height = HEIGHT;
            self.waist_angle = 0;
            self.waist_speed = 0.0;
            self.waist_rot_x = 0;
            self.waist_rot_y = 0;
            self.player_stat += 1;
        } else if self.player_stat != STEP_ENDING && !self.fly_check() {
            if ctx.stick >= HIO_M104 || ctx.attention_lock.is_some() {
                let stick_angle = self.get_stick_ang_y(false);
                self.yaw = walk::add_calc_angle_s(
                    self.yaw,
                    stick_angle,
                    HIO2_M26,
                    HIO2_M28,
                    HIO2_M2A,
                );
                let (temp, stick_pos) = self.calc_stick_pos(stick_angle, ctx.attention_lock);
                if temp == 0 {
                    self.shape[1] = self.yaw;
                } else if ctx.stick >= HIO_M108 {
                    self.shape[1] = self.yaw;
                }
                let mut temp_angle = self.shape[1];
                self.jnt_look_at(
                    &mut temp_angle,
                    Some(stick_pos),
                    self.pos,
                    self.shape[1],
                    HIO_NPC_MAX_TURN_STEP,
                    false,
                );
                if temp > 0 {
                    self.shape[1] = temp_angle;
                }
                self.yaw = self.shape[1];
                if ctx.stick >= HIO_M108 && temp == 0 {
                    self.yaw = stick_angle;
                    self.set_player_action(PlayerStep::Walk, Arg::None);
                }
            } else {
                self.look_zero();
                self.wait_ground_check(ctx.special_ground, ctx.is_player);
                self.yaw = self.shape[1];
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn walk_player_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.player_stat == STEP_STARTING {
            self.set_stts(STTS_UNK4);
            self.set_anm(2);
            self.max_speed = HIO2_M08;
            self.height = HEIGHT;
            self.player_stat += 1;
        } else if self.player_stat != STEP_ENDING && !self.fly_check() {
            let stick = ctx.stick;
            let stick_angle = self.get_stick_ang_y(false);
            let wall = if ctx.wall_hit {
                Some(ctx.wall_angle)
            } else {
                None
            };
            self.walk_proc(stick, stick_angle, wall);
            let (focus, stick_pos) = self.calc_stick_pos(stick_angle, ctx.attention_lock);
            if focus == 0 {
                self.shape[1] = walk::add_calc_angle_s(
                    self.shape[1],
                    self.yaw,
                    HIO2_M24,
                    HIO2_M20 * 2,
                    HIO2_M22 * 2,
                );
            }
            let mut temp_angle = self.shape[1];
            self.jnt_look_at(
                &mut temp_angle,
                Some(stick_pos),
                self.pos,
                self.shape[1],
                HIO_NPC_MAX_TURN_STEP,
                false,
            );
            if focus > 0 {
                self.shape[1] = temp_angle;
            }
            if self.speed_f < VELOCITY_EPS || focus != 0 {
                self.set_player_action(PlayerStep::Wait, Arg::None);
            } else if !self.ground_hit {
                self.set_player_action(PlayerStep::Jump, Arg::Speed(10.0));
            } else if ctx.wall_hit {
                let jump = self.check_wall_jump(self.yaw, ctx.wall_normal, ctx.probe_y);
                if jump >= 0.0 {
                    self.set_player_action(PlayerStep::Jump, Arg::Speed(jump));
                    return true;
                }
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn hit_player_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.player_stat == STEP_STARTING {
            ctx.events.list.push(SimEvent::Se(SE_CV_MD_CRASH));
            let angle = match self.arg {
                Arg::Angle(value) => value,
                _ => 0,
            };
            self.yaw = angle;
            self.speed_f = 10.0;
            self.speed.y = 20.0;
            self.max_fall = MAX_FALL;
            self.gravity = HIO_M0F4;
            self.clr_attn_bit(ATTN_ACTION_CARRY);
            self.clr_stts(STTS_UNK1 | STTS_UNK4);
            self.set_stts(STTS_UNK2);
            self.height = HEIGHT;
            self.set_anm(0xd);
            self.player_stat += 1;
        } else if self.player_stat != STEP_ENDING {
            if self.player_stat == STEP_ONGOING_1 {
                if self.ground_hit {
                    self.speed_f = 0.0;
                    self.set_anm(0xf);
                    self.anim_ended = false;
                    self.player_stat += 1;
                }
            } else if self.anim_ended {
                self.return_link = true;
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn jump_player_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.player_stat == STEP_STARTING {
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_UNK4 | STTS_FLY);
            self.set_stts(STTS_UNK4);
            if let Arg::Speed(speed) = self.arg {
                self.speed.y = speed;
            }
            self.shape[0] = 0;
            self.shape[2] = 0;
            self.gravity = HIO_M0F4;
            self.max_fall = MAX_FALL;
            self.set_anm(0xc);
            self.height = HEIGHT;
            self.player_stat += 1;
            self.start_y = self.pos.y;
        } else if self.player_stat != STEP_ENDING {
            if self.player_stat == STEP_ONGOING_1 {
                if self.ground_hit {
                    if self.start_y - self.pos.y >= HIO_M1B0 {}
                    self.speed_f = 0.0;
                    self.set_anm(0xe);
                    self.anim_ended = false;
                    self.player_stat += 1;
                }
            } else if self.anim_ended {
                self.set_player_action(PlayerStep::Wait, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn land_player_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.player_stat == STEP_STARTING {
            ctx.events.list.push(SimEvent::Se(SE_CV_MD_LANDING));
            if self.has_stts(STTS_UNK1) {
                self.effect |= 1;
            }
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_UNK4 | STTS_FLY);
            self.set_anm(0xe);
            self.shape[0] = 0;
            self.shape[2] = 0;
            self.gravity = HIO_M0F4;
            self.max_fall = MAX_FALL;
            self.speed_f = 0.0;
            self.speed.y = 0.0;
            self.height = HEIGHT;
            self.player_stat += 1;
        } else if self.player_stat != STEP_ENDING {
            if self.anim_ended {
                self.speed_f = 0.0;
                self.set_player_action(PlayerStep::Wait, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn mkamae_player_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.player_stat == STEP_STARTING {
            self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_UNK4 | STTS_FLY);
            self.set_anm(0x1F);
            self.shape[0] = 0;
            self.shape[2] = 0;
            self.gravity = HIO_M0F4;
            self.max_fall = MAX_FALL;
            self.speed_f = 0.0;
            self.speed.y = 0.0;
            self.height = HEIGHT;
            self.player_stat += 1;
        } else if self.player_stat == STEP_ENDING {
            self.mirror = false;
        } else {
            if self.player_stat == STEP_ONGOING_1 {
                if self.anim_ended {
                    self.speed_f = 0.0;
                    self.set_anm(0x20);
                    self.mirror = true;
                    self.player_stat += 1;
                }
            } else if self.player_stat == STEP_ONGOING_2 {
                if ctx.trig_b || !self.is_light_body_hit() {
                    self.set_anm(0x21);
                    self.anim_ended = false;
                    self.mirror = false;
                    self.player_stat += 1;
                } else {
                    if ctx.stick >= HIO_M104 || ctx.attention_lock.is_some() {
                        self.waist_angle = self.get_stick_ang_y(false);
                        self.waist_speed = ctx.stick;
                    } else {
                        self.waist_angle = 0;
                        self.waist_speed = 0.0;
                    }
                    self.look_back_waist(self.waist_angle, self.waist_speed);
                }
            } else if self.anim_ended {
                self.set_player_action(PlayerStep::Wait, Arg::None);
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn carry_player_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.player_stat == STEP_STARTING {
            self.clr_stts(STTS_UNK2 | STTS_UNK4 | STTS_FLY);
            self.set_anm(0x24);
            self.set_stts(STTS_UNK1);
            ctx.events.list.push(SimEvent::Se(SE_CV_MD_LIFT_UP));
            self.height = HEIGHT_SQUAT;
            self.player_stat += 1;
        }
        false
    }
}

impl Medli {
    pub fn fly_player_action(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.player_stat == STEP_STARTING {
            self.set_anm(0x18);
            self.start_y = self.pos.y + FLY_CEILING_LIFT;
            self.timer_a = HIO_M1BE;
            self.timer_b = HIO_M1C0;
            self.flying_timer = FLY_MAX_TIMER;
            self.speed_f = 0.0;
            self.height = HEIGHT;
            self.flying = true;
            self.player_stat += 1;
        } else if self.player_stat == STEP_ENDING {
            self.flying = false;
        } else {
            if ctx.trig_a {
                self.timer_b = HIO_M1C0;
            } else {
                Medli::calc_timer(&mut self.timer_b);
            }
            if ctx.trig_a {
                self.timer_a = HIO_M1BE;
            } else {
                Medli::calc_timer(&mut self.timer_a);
            }
            if self.player_stat == STEP_ONGOING_1 {
                if self.anim_ended {
                    self.set_stts(STTS_UNK1);
                    self.set_anm(0x19);
                    self.speed_f = 0.0;
                    self.speed.y = 20.0;
                    self.gravity = HIO_M188;
                    self.max_fall = HIO_M18C;
                    self.timer_a = HIO_M1BE;
                    self.timer_b = HIO_M1C0;
                    self.player_stat += 1;
                }
            } else {
                Medli::calc_timer(&mut self.flying_timer);
                if ctx.stories.test(BIT_3320) && self.flying_timer < 300 {
                    ctx.stories.set(BIT_3310);
                }
                if self.flying_timer == 0 || self.timer_a == 0 || ctx.trig_b {
                    if self.has_stts(STTS_UNK1) {
                        self.effect |= 1;
                    }
                    self.set_player_action(PlayerStep::Jump, Arg::None);
                    if self.flying_timer == 0 {
                        ctx.events.list.push(SimEvent::Se(SE_CV_MD_FLY_END));
                    }
                    return true;
                }
                if self.anm != 0x2e && self.anim_ended {
                    self.set_anm(7);
                }
                if self.flying_timer < HIO_M1C4 && self.anm == 7 {
                    self.set_anm(0x2e);
                }
                if ctx.stick >= HIO_M104 || ctx.attention_lock.is_some() {
                    let mut drive = ctx.stick * HIO_M10C;
                    if 0.0 < drive && drive < HIO_M0D8 {
                        drive = HIO_M0D8;
                    }
                    let stick_angle = self.get_stick_ang_y(false);
                    let _ = angle_distance(stick_angle, self.yaw);
                    let step_rate = HIO_M0DC;
                    if ctx.stick >= HIO_M104 {
                        self.yaw = walk::add_calc_angle_s(
                            self.yaw,
                            stick_angle,
                            TURN_SCALE,
                            TURN_MAX_STEP,
                            TURN_MIN_STEP,
                        );
                    }
                    let (focus, stick_pos) = self.calc_stick_pos(stick_angle, ctx.attention_lock);
                    if focus == 0 {
                        self.shape[1] = walk::add_calc_angle_s(
                            self.shape[1],
                            self.yaw,
                            TURN_SCALE,
                            TURN_MAX_STEP,
                            TURN_MIN_STEP,
                        );
                    }
                    let mut temp_angle = self.shape[1];
                    self.jnt_look_at(
                        &mut temp_angle,
                        Some(stick_pos),
                        self.pos,
                        self.shape[1],
                        HIO_NPC_MAX_TURN_STEP,
                        false,
                    );
                    if focus > 0 {
                        self.shape[1] = temp_angle;
                    } else if ctx.stick < HIO_M104 {
                        self.yaw = walk::add_calc_angle_s(
                            self.yaw,
                            self.shape[1],
                            TURN_SCALE,
                            TURN_MAX_STEP,
                            TURN_MIN_STEP,
                        );
                    }
                    if drive > HIO_M0D4 {
                        drive = HIO_M0D4;
                    }
                    walk::chase_f(&mut self.speed_f, drive, step_rate);
                } else {
                    self.look_zero();
                    self.yaw = self.shape[1];
                }
                self.gravity = HIO_M188;
                if self.timer_b != 0 {
                    self.max_fall = 10.0;
                } else {
                    self.max_fall = HIO_M18C;
                }
                if self.pos.y >= self.start_y {
                    self.pos.y = self.start_y;
                }
                if self.ground_hit {
                    self.flying_timer = 0;
                    self.set_player_action(PlayerStep::Land, Arg::None);
                }
            }
            if !self.has_stts(STTS_UNK2) {
                if self.has_stts(STTS_UNK1) {
                    let mut tilt = self.shape[1].wrapping_sub(self.yaw);
                    if tilt < TILT_BEND_LIMIT && tilt > -TILT_BEND_LIMIT {
                        if tilt > TILT_BEND_MAX {
                            tilt = TILT_BEND_MAX;
                        } else if tilt < -TILT_BEND_MAX {
                            tilt = -TILT_BEND_MAX;
                        }
                        self.shape[2] = walk::add_calc_angle_s(
                            self.shape[2],
                            tilt,
                            TILT_SCALE,
                            TILT_MAX_STEP,
                            TILT_MIN_STEP,
                        );
                        self.shape[0] = walk::add_calc_angle_s(
                            self.shape[0],
                            0,
                            TILT_SCALE,
                            TILT_MAX_STEP,
                            TILT_MIN_STEP,
                        );
                    } else {
                        self.shape[2] = walk::add_calc_angle_s(
                            self.shape[2],
                            0,
                            TILT_SCALE,
                            TILT_MAX_STEP,
                            TILT_MIN_STEP,
                        );
                        self.shape[0] = walk::add_calc_angle_s(
                            self.shape[0],
                            TILT_SIDE,
                            TILT_SCALE,
                            TILT_MAX_STEP,
                            TILT_MIN_STEP,
                        );
                    }
                } else {
                    self.yaw = self.shape[1];
                    self.shape[0] = 0;
                    self.shape[2] = 0;
                }
            }
            self.set_attention(true, self.pos, ctx.is_player);
        }
        false
    }

    pub fn tilt(&mut self) {
        if !self.has_stts(STTS_UNK2) && self.has_stts(STTS_UNK1) {
            let mut tilt = self.shape[1].wrapping_sub(self.yaw);
            if tilt < TILT_BEND_LIMIT && tilt > -TILT_BEND_LIMIT {
                if tilt > TILT_BEND_MAX {
                    tilt = TILT_BEND_MAX;
                } else if tilt < -TILT_BEND_MAX {
                    tilt = -TILT_BEND_MAX;
                }
                self.shape[2] = walk::add_calc_angle_s(
                    self.shape[2],
                    tilt,
                    TILT_SCALE,
                    TILT_MAX_STEP,
                    TILT_MIN_STEP,
                );
                self.shape[0] = walk::add_calc_angle_s(
                    self.shape[0],
                    0,
                    TILT_SCALE,
                    TILT_MAX_STEP,
                    TILT_MIN_STEP,
                );
            } else {
                self.shape[2] = walk::add_calc_angle_s(
                    self.shape[2],
                    0,
                    TILT_SCALE,
                    TILT_MAX_STEP,
                    TILT_MIN_STEP,
                );
                self.shape[0] = walk::add_calc_angle_s(
                    self.shape[0],
                    TILT_SIDE,
                    TILT_SCALE,
                    TILT_MAX_STEP,
                    TILT_MIN_STEP,
                );
            }
        } else if !self.has_stts(STTS_UNK1) {
            self.yaw = self.shape[1];
            self.shape[0] = 0;
            self.shape[2] = 0;
        }
    }
}

fn deg2s(deg: f32) -> i16 {
    walk::to_s16(deg * walk::RAD_PER_UNIT / 180.0)
}

impl Medli {
    pub fn initial_wait(&mut self, cut: usize, ctx: &mut Ctx<'_>) {
        match ctx.list.cut_number(cut, "Disp") {
            Some(disp) => {
                self.draw_off = disp == 0;
            }
            None => {
                self.draw_off = false;
            }
        }
        self.wait_timer = ctx
            .list
            .cut_number(cut, "Timer")
            .and_then(|value| i16::try_from(value).ok())
            .unwrap_or(0);
        self.speed_f = 0.0;
    }

    pub fn action_wait(&mut self, _cut: usize) -> bool {
        Medli::calc_timer(&mut self.wait_timer)
    }

    pub fn initial_letter(&mut self, _cut: usize, ctx: &mut Ctx<'_>) {
        self.clr_stts(STTS_UNK1 | STTS_UNK2 | STTS_FLY);
        self.set_stts(STTS_UNK4);
        self.speed_f = 0.0;
        self.height = HEIGHT_SQUAT;
        let item = if self.cur_event == 0 {
            ITEM_FATHER_LETTER
        } else if self.cur_event == 1 {
            ITEM_GRAPPLING_HOOK
        } else if self.cur_event == 2 {
            ITEM_EMPTY_BOTTLE
        } else {
            ITEM_FATHER_LETTER
        };
        ctx.give_item = Some((self.pos, item));
        ctx.reset_event = true;
    }

    pub fn initial_msg_set(&mut self, cut: usize, ctx: &mut Ctx<'_>) {
        if self.msg_no == 0x19DD {
            ctx.stories.set(BIT_1620);
            ctx.stories.set(BIT_1608);
            ctx.goto_sea = true;
        } else {
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
            if let Some(value) = ctx.list.cut_number(cut, "MsgNo") {
                self.msg_no = u32::try_from(value).unwrap_or(0);
                if self.msg_no == 0x19D6 && ctx.stories.test(BIT_1504) {
                    self.msg_no = 0x19D9;
                }
            }
        }
    }

    pub fn action_msg_set(&mut self, ctx: &mut Ctx<'_>) -> bool {
        self.look_back(true, false, false);
        self.set_attention(true, self.pos, ctx.is_player);
        self.talk_init(ctx)
    }

    pub fn action_msg_end(&mut self, cut: usize, ctx: &mut Ctx<'_>) -> bool {
        let result = self.talk(false, false, ctx);
        if result {
            self.initial_end(cut, ctx.list, ctx);
        }
        self.look_back(true, false, false);
        self.set_attention(true, self.pos, ctx.is_player);
        result
    }

    pub fn initial_move_pos(&mut self, cut: usize, ctx: &mut Ctx<'_>) {
        let target = ctx.list.cut_text(cut, "Target").map(str::to_owned);
        if let Some(target) = target {
            if target == "@PLAYER"
                && let Some(link) = ctx.link_eye
            {
                let mut kind = target.clone();
                if let Some(text) = ctx.list.cut_text(cut, "Type") {
                    kind = text.to_owned();
                }
                let heading = target_angle_y(self.pos, link);
                if kind == "LINK_DIST" {
                    if let Some(dist) = ctx.list.cut_float(cut, "Dist") {
                        let sin = dist * walk::from_s16(heading).sin();
                        let cos = dist * walk::from_s16(heading).cos();
                        let mut pos = Vec3::new(
                            self.pos.x + sin,
                            self.pos.y + 50.0,
                            self.pos.z + cos,
                        );
                        pos.y = ctx.ground_y.unwrap_or(self.pos.y);
                        ctx.link_move = Some((pos, self.player_yaw));
                    }
                    if let Some(angle) = ctx.list.cut_float(cut, "Angle") {
                        let value = deg2s(angle).wrapping_add(-0x8000).wrapping_add(heading);
                        if let Some((pos, _)) = ctx.link_move {
                            ctx.link_move = Some((pos, value));
                        }
                    }
                } else if kind == "DIST" {
                    if let Some(dist) = ctx.list.cut_float(cut, "Dist") {
                        let sin = dist * walk::from_s16(self.player_shape).sin();
                        let cos = dist * walk::from_s16(self.player_shape).cos();
                        self.pos = Vec3::new(link.x + sin, link.y, link.z + cos);
                    }
                    if let Some(angle) = ctx.list.cut_float(cut, "Angle") {
                        let value = deg2s(angle)
                            .wrapping_add(-0x8000)
                            .wrapping_add(self.player_shape);
                        self.yaw = value;
                        self.shape[1] = value;
                    }
                } else {
                    if let Some(angle) = ctx.list.cut_float(cut, "Angle") {
                        let value = deg2s(angle).wrapping_add(self.player_shape);
                        self.yaw = value;
                        self.shape[1] = value;
                    }
                    if let Some(offset) = ctx.list.cut_vec(cut, "Pos") {
                        self.pos = Vec3::new(link.x + offset.x, link.y + offset.y, link.z + offset.z);
                    }
                }
            }
        } else {
            if let Some(angle) = ctx.list.cut_float(cut, "Angle") {
                let value = deg2s(angle);
                self.yaw = value;
                self.shape[1] = value;
            }
            if let Some(pos) = ctx.list.cut_vec(cut, "Pos") {
                self.pos = pos;
            }
        }
        self.gravity = HIO_M0F4;
        self.max_fall = MAX_FALL;
        self.speed_f = 0.0;
        self.speed.y = 0.0;
        if ctx.list.cut_text(cut, "Name") == Some("WAIT") {
            self.set_anm(0);
        }
        match ctx.list.cut_text(cut, "ShipRide") {
            Some("on") => self.set_stts(STTS_SHIP_RIDE),
            Some("off") => self.clr_stts(STTS_SHIP_RIDE),
            _ => {}
        }
    }

    pub fn initial_fly(&mut self) {
        self.set_anm(0x19);
        self.set_stts(STTS_UNK1);
        self.speed.y = 20.0;
        self.gravity = HIO_M188;
        self.max_fall = 10.0;
        self.wait_timer = 60 * 30;
    }

    pub fn action_fly(&mut self, cut: usize, ctx: &mut Ctx<'_>) -> bool {
        let high = ctx.list.cut_float(cut, "HIGH").unwrap_or(0.0);
        if self.pos.y > high {
            return true;
        }
        if self.anm == 0x19 && self.anim_ended {
            self.set_anm(9);
        }
        self.set_attention(true, self.pos, ctx.is_player);
        if ctx.roof_hit {
            return true;
        }
        Medli::calc_timer(&mut self.wait_timer)
    }

    pub fn initial_gliding(&mut self) {
        self.gravity = HIO_M0F4;
        self.max_fall = MAX_FALL;
    }

    pub fn action_gliding(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.ground_hit {
            ctx.events.list.push(SimEvent::Se(SE_CV_MD_LANDING));
            return true;
        }
        if !self.is_cam_tag_in() {
            self.speed_f = 0.0;
        }
        self.set_attention(true, self.pos, ctx.is_player);
        false
    }

    pub fn initial_landing(&mut self) {
        self.max_fall = MAX_FALL;
        self.speed_f = 0.0;
        self.shape[0] = 0;
        self.shape[2] = 0;
        self.clr_stts(STTS_UNK1);
        self.set_anm(0x10);
        self.anim_ended = false;
    }

    pub fn action_landing(&mut self) -> bool {
        if self.anim_ended {
            self.set_anm(0);
            return true;
        }
        false
    }

    pub fn initial_walk(&mut self) {
        self.speed_f = 0.0;
        self.wait_timer = 20 * 30;
    }

    pub fn action_walk(&mut self, cut: usize, ctx: &mut Ctx<'_>) -> bool {
        self.walk_event(cut, None, ctx)
    }

    pub fn action_dash(&mut self, cut: usize, ctx: &mut Ctx<'_>) -> bool {
        self.walk_event(cut, Some((17.0, 1.0)), ctx)
    }

    fn walk_event(
        &mut self,
        cut: usize,
        dash: Option<(f32, f32)>,
        ctx: &mut Ctx<'_>,
    ) -> bool {
        let Some(goal) = ctx.list.cut_vec(cut, "Pos") else {
            return true;
        };
        let delta = goal - self.pos;
        if abs_xz(delta) < 10.0 {
            self.speed_f = 0.0;
            return true;
        }
        self.yaw = walk::to_s16(delta.x.atan2(delta.z));
        if self.jnt_trn_chk() {
            stream::add_calc_angle_s2(
                &mut self.turn_vel,
                HIO_NPC_MAX_HEAD_TURN_VEL,
                HEAD_SCALE,
                HEAD_MAX_STEP,
            );
        } else {
            self.turn_vel = 0;
            let (speed, rate) = dash.unwrap_or((
                ctx.list.cut_float(cut, "Speed").unwrap_or(5.0),
                ctx.list.cut_float(cut, "RunRate").unwrap_or(0.0),
            ));
            self.speed_f = speed;
            self.run_rate = rate;
            self.set_anm(2);
        }
        self.jnt_set_trn();
        let mut yaw = self.shape[1];
        self.jnt_look_at(&mut yaw, Some(goal), self.pos, self.yaw, self.turn_vel, false);
        self.shape[1] = yaw;
        self.set_attention(true, self.pos, ctx.is_player);
        if ctx.wall_hit {
            self.speed_f = 0.0;
            self.pos = goal;
            return true;
        }
        if !Medli::calc_timer(&mut self.wait_timer) {
            return false;
        }
        self.speed_f = 0.0;
        self.pos = goal;
        true
    }

    pub fn initial_end(&mut self, cut: usize, list: &EventList, ctx: &mut Ctx<'_>) {
        let mode = list.cut_number(cut, "EndMode").unwrap_or(0);
        match mode {
            -2 => {
                if self.msg_no == 0x19de {
                    ctx.stories.set(BIT_1620);
                    ctx.stories.set(BIT_1608);
                    ctx.goto_sea = true;
                }
                ctx.reset_event = true;
                if self.has_stts(STTS_UNK80) {
                    self.set_harp_play_num();
                }
                self.set_npc_action(Step::Wait, Arg::None);
            }
            3 => {
                self.set_npc_action(Step::Delete, Arg::None);
            }
            2 => {
                self.clr_stts(STTS_UNK1);
                self.set_npc_action(Step::Squatdown, Arg::None);
            }
            _ => {
                self.clr_stts(STTS_UNK1);
                if self.has_stts(STTS_UNK80) {
                    self.set_harp_play_num();
                } else {
                    self.set_anm(0);
                }
                self.set_npc_action(Step::Wait, Arg::None);
            }
        }
    }

    pub fn action_tact(&mut self, cut: usize, ctx: &mut Ctx<'_>) -> bool {
        let prm = ctx.list.cut_number(cut, "prm0").unwrap_or(0);
        let song = ctx.tact_song;
        if song >= 0 {
            self.set_stts(STTS_UNK400);
        }
        let result = self.talk(true, ctx.cancel_button, ctx);
        if result {
            if self.has_stts(STTS_UNK400) {
                self.clr_stts(STTS_UNK400);
                self.msg_no = if song == prm { MSG_TACT_MATCH } else { MSG_TACT_MISS };
            } else {
                self.mode = MODE_EVENT6;
                if self.has_stts(STTS_UNK200) {
                    self.msg_no = MSG_TACT_STOP;
                    self.clr_stts(STTS_UNK200);
                }
                self.draw_off = false;
                ctx.reset_event = true;
            }
        }
        result
    }

    pub fn initial_take_off(&mut self) {
        self.set_anm(0x18);
        self.anim_ended = false;
    }

    pub fn action_take_off(&mut self) -> bool {
        if self.anim_ended {
            return true;
        }
        self.set_attention(true, self.pos, false);
        false
    }

    pub fn initial_onetime(&mut self, _cut: usize) {
        self.set_anm(0x1A);
        self.anim_ended = false;
    }

    pub fn action_onetime(&mut self) -> bool {
        if self.anim_ended {
            if self.anm == 0x1A {
                self.set_anm(0);
            } else {
                return true;
            }
        }
        self.set_attention(true, self.pos, false);
        false
    }

    pub fn initial_quake(&mut self, _cut: usize) {}

    pub fn initial_harp_play(&mut self) {
        self.set_harp_play_num();
    }

    pub fn action_harp_play(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.anim_ended {
            self.set_harp_play_num();
        }
        ctx.tact_ended
    }

    pub fn initial_off_link(&mut self) {
        self.link_hidden = true;
    }

    pub fn initial_on_link(&mut self) {
        self.link_hidden = false;
    }

    pub fn initial_turn(&mut self, cut: usize, ctx: &mut Ctx<'_>) {
        self.wait_timer = ctx
            .list
            .cut_number(cut, "Timer")
            .and_then(|value| i16::try_from(value).ok())
            .unwrap_or(0);
        self.turn_pos = ctx
            .list
            .cut_vec(cut, "Pos")
            .unwrap_or(Vec3::ZERO);
    }

    pub fn action_turn(&mut self, cut: usize, ctx: &mut Ctx<'_>) -> bool {
        self.set_attention(true, self.pos, ctx.is_player);
        let target = ctx.list.cut_text(cut, "Target").unwrap_or("@PLAYER");
        let kind = ctx.list.cut_number(cut, "Type").unwrap_or(0);
        self.jnt_set_param(
            HIO_NPC_MAX_BACKBONE_X,
            HIO_NPC_MAX_BACKBONE_Y,
            HIO_NPC_MIN_BACKBONE_X,
            HIO_NPC_MIN_BACKBONE_Y,
            HIO_NPC_MAX_HEAD_X,
            HIO_NPC_MAX_HEAD_Y,
            TURN_JOINT_MIN_Y,
            HIO_NPC_MIN_HEAD_Y,
            HIO_NPC_MAX_TURN_STEP,
        );
        let mut flag = kind;
        if flag == 2 {
            self.jnt_set_trn();
            flag = 0;
        }
        if target == "@PLAYER" {
            self.look_back(flag != 0, false, false);
        } else {
            self.look_back_at(Some(self.turn_pos), true, flag != 0);
        }
        !Medli::calc_timer(&mut self.wait_timer)
    }

    pub fn initial_set_anm(&mut self, cut: usize, list: &EventList) {
        if let Some(value) = list.cut_number(cut, "Number") {
            let attr = u8::try_from(value).unwrap_or(0);
            let result = self.set_anm(Medli::get_anm_type(attr));
            if result {
                self.set_stts(STTS_UNK1);
            } else {
                self.clr_stts(STTS_UNK1);
            }
        }
        self.speed_f = 0.0;
        self.clear_jnt_ang();
    }

    pub fn initial_look_down(&mut self) {
        self.set_anm(0x2F);
    }

    pub fn initial_look_up(&mut self) {
        self.set_anm(5);
    }

    pub fn action_look_down(&mut self) -> bool {
        self.turn_offset.y = LOOK_DOWN_OFFSET;
        let mut base = Vec3::new(self.pos.x, self.eye.y, self.pos.z);
        let desired = self.shape[1];
        let rotated = Vec3::new(
            self.turn_offset.x * walk::from_s16(desired).cos()
                + self.turn_offset.z * walk::from_s16(desired).sin(),
            self.turn_offset.y,
            -self.turn_offset.x * walk::from_s16(desired).sin()
                + self.turn_offset.z * walk::from_s16(desired).cos(),
        );
        base += rotated;
        let mut yaw = self.shape[1];
        self.jnt_look_at(
            &mut yaw,
            Some(base),
            Vec3::new(self.pos.x, self.eye.y, self.pos.z),
            desired,
            self.turn_vel,
            false,
        );
        self.shape[1] = yaw;
        true
    }
}

impl Medli {
    pub fn check_frame(&self, frame: f32) -> bool {
        (self.anm_frame - frame).abs() < 1.0
    }

    pub fn animation_play(&mut self, ctx: &mut Ctx<'_>) {
        self.old_light_body_hit = self.is_light_body_hit();
        self.anm_frame += self.anm_rate;
        self.anim_ended = self.frame_ended;
        self.frame_ended = false;
        let index = self.event_idx.get(5).copied().unwrap_or(NO_INDEX);
        let at = if index >= 0 {
            Some(usize::from(index as u16))
        } else {
            None
        };
        let blocked = ctx.event_run()
            && at.is_some_and(|at| ctx.active.start_check(at) || ctx.active.end_check(at));
        if self.is_type_sea()
            && !blocked
            && (self.anm == 0x16 || self.anm == 0x17)
        {
            let hit = if self.anm == 0x16 {
                self.check_frame(HARP_FRAME_11) || self.check_frame(HARP_FRAME_38)
            } else {
                self.check_frame(HARP_FRAME_11) || self.check_frame(HARP_FRAME_35)
            };
            if hit {
                if self.harp_delay == 0 {
                    let value = rnd_value(&mut self.seed, 0, HARP_SE_RND_SPAN);
                    let index = usize::try_from(value).unwrap_or(usize::MAX);
                    self.harp_se = HARP_SE.get(index).copied().unwrap_or(HARP_SE[0]);
                    self.harp_delay = 2;
                } else {
                    self.harp_delay -= 1;
                }
                ctx.events.list.push(SimEvent::Se(self.harp_se));
            }
        } else {
            self.harp_delay = 0;
        }
        if self.anm_rate < 0.0 {
            if self.anm_start < self.anm_frame {
                self.anim_ended = true;
            }
        } else if self.anm_start > self.anm_frame {
            self.anim_ended = true;
        }
        self.anm_start = self.anm_frame;
        let body = i32::from(self.anm_body);
        if body == 0x1B || body == 0x0F {
            if self.anm_frame < 10.0 {
                self.clr_stts(STTS_UNK80);
            } else {
                self.set_stts(STTS_UNK80);
            }
        }
        self.play_tex_pattern_anm();
    }

    pub fn gravity_step(&mut self) {
        if self.max_fall < self.speed.y {
            self.speed.y -= self.gravity;
            if self.speed.y < self.max_fall {
                self.speed.y = self.max_fall;
            }
        } else if self.max_fall > self.speed.y {
            self.speed.y += self.gravity;
            if self.speed.y > self.max_fall {
                self.speed.y = self.max_fall;
            }
        }
    }

    pub fn ground_probe(&mut self, ctx: &mut Ctx<'_>) {
        if let Some(height) = ctx.ground_y {
            self.ground_y = height;
            self.ground_hit = true;
            if let Some(normal) = ctx.ground_normal_opt {
                self.normal = normal;
            }
        } else {
            self.ground_y = f32::NEG_INFINITY;
            self.ground_hit = false;
        }
    }

    pub fn event_proc(&mut self, ctx: &mut Ctx<'_>) -> bool {
        self.event_accept_block(ctx);
        if !ctx.event_run() || self.check_command_talk(ctx) {
            return false;
        }
        if self.is_event_accepted() {
            let index = self.cur_event;
            if index >= 0 {
                let at = usize::from(index as u16);
                if ctx.active.end_check(at) {
                    ctx.reset_event = true;
                    self.accepted = false;
                    if index == 8 {
                        self.return_link_player();
                        self.return_link = false;
                    }
                    self.cur_event = -1;
                }
            }
            return true;
        }
        if ctx.staff_cut.is_some() {
            return true;
        }
        if !ctx.door_command {
            return true;
        }
        false
    }

    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) -> bool {
        self.pos = actor.pos;
        self.yaw = actor.angle[1];
        self.shape = actor.angle;
        self.old = self.pos;
        self.link = ctx.player;
        self.clr_stts(STTS_UNK20000);
        self.old_light_body_hit = self.is_light_body_hit();
        self.player_room = ctx.actor_room == ctx.link_room;
        self.ground_probe(ctx);
        if self.is_ship_ride() {
            self.jnt_ship_param();
            if let Some(ship) = ctx.ship_pos {
                self.pos = ship;
                let value = ctx.ship_yaw.wrapping_add(SHIP_TURN);
                self.shape[1] = value;
                self.yaw = value;
                self.set_attention(true, self.pos, ctx.is_player);
            }
        } else if !self.is_type_m_daib() {
            self.jnt_default_param();
            if !self.is_type_ship_ride() && !self.return_link && !ctx.carried {
                if !ctx.pos_move {
                    self.gravity_step();
                    self.speed.x = self.speed_f * walk::from_s16(self.yaw).sin();
                    self.speed.z = self.speed_f * walk::from_s16(self.yaw).cos();
                    self.pos += self.speed;
                }
                self.set_stts(STTS_UNK20000);
                self.ripple =
                    ctx.in_water && ctx.water_height > self.pos.y + 5.0;
                if !self.ground_hit
                    && (self.player_step == PlayerStep::Walk
                        || self.step == Step::Search
                        || self.step == Step::Wait)
                {
                    let delta = self.ground_y - self.pos.y;
                    if delta < ROUTE_SNAP_MAX && delta >= ROUTE_SNAP_MIN && self.speed.y < ROUTE_SNAP_MAX
                    {
                        self.pos.y = self.ground_y;
                        self.ground_hit = true;
                        self.speed.y = 0.0;
                    }
                }
                self.ground_probe(ctx);
                self.was_airborne = u8::from(self.ground_hit);
                if self.ground_y == f32::NEG_INFINITY || ctx.ground_code == Some(GROUND_CODE_LIMIT) {
                    if !self.is_fall_action() {
                        self.ground_miss = 0;
                    } else if self.ground_miss < FALL_COUNT_MAX {
                        self.ground_miss += 1;
                    } else {
                        self.pos = self.home;
                        self.shape[1] = self.home_yaw;
                        self.speed_f = 0.0;
                        self.ground_miss = 0;
                    }
                } else {
                    self.ground_miss = 0;
                }
            }
        }
        self.light_hit_check();
        if !ctx.pos_move {
            self.animation_play(ctx);
        }
        if !self.event_proc(ctx) {
            if ctx.is_player {
                if self.return_link {
                    self.mode = MODE_EVENT8;
                } else {
                    if self.is_no_carry() {
                        self.set_player_action(PlayerStep::Carry, Arg::None);
                        self.return_link = true;
                    }
                    self.player_action(ctx);
                    if self.return_link_check(
                        self.ground_hit,
                        ctx.trig_r,
                        ctx.trig_start,
                        ctx,
                    ) {
                        self.return_link = true;
                    } else {
                        self.check_collision(true);
                    }
                }
            } else {
                if let Some((dir, pos)) = ctx.move_block {
                    self.block_dir = dir;
                    self.block_pos = pos;
                    if dir != 0 {
                        self.set_npc_action(Step::Escape, Arg::None);
                    }
                }
                self.carry_check(ctx.carried);
                self.check_order(ctx);
                let _ = self.npc_action(ctx);
                self.check_collision(false);
                self.tilt();
            }
        }
        self.event_order(ctx);
        self.effect = 0;
        actor.pos = self.pos;
        actor.angle = self.shape;
        actor.room = self.room_of(ctx);
        ctx.delete_me
    }

    pub fn room_of(&self, ctx: &Ctx<'_>) -> i8 {
        ctx.ground_room.unwrap_or(ctx.actor_room)
    }

}

pub const COLLECT_MEDLI_ITEM: u8 = 2;

pub fn create_ok(stories: &StoryBits, stage: &str, collect_slot_2: bool) -> bool {
    if collect_slot_2 {
        return stage == "M_DaiB";
    }
    match stage {
        "sea" => !stories.test(BIT_2E04) && stories.test(BIT_1820),
        "Atorizk" => !stories.test(BIT_2E04),
        "Adanmae" => !stories.test(BIT_2E04),
        "M_Dra09" => !stories.test(BIT_2E04) && !stories.test(BIT_1101),
        "Edaichi" => stories.test(BIT_2E04) && !stories.test(BIT_2920),
        "M_Dai" => stories.test(BIT_2E04) && stories.test(BIT_2920),
        "M_DaiB" => false,
        _ => true,
    }
}

#[derive(SystemParam)]
pub struct World<'w> {
    pub stories: ResMut<'w, StoryBits>,
    pub switches: ResMut<'w, Switches>,
    pub events: ResMut<'w, SimEvents>,
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub carry: Res<'w, Carry>,
    pub water: Res<'w, Water>,
    pub stage: Res<'w, StageName>,
    pub text_flags: Res<'w, TextFlags>,
    pub next: ResMut<'w, NextStage>,
    pub archive: Option<Res<'w, MsgArchives>>,
}

#[derive(SystemParam)]
pub struct Dialogue<'w, 's> {
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub flags: ResMut<'w, MsgFlags>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub players: Query<'w, 's, &'static Player>,
    pub ships: Query<'w, 's, &'static Ship>,
}

pub fn control(
    mut world: World,
    mut dialogue: Dialogue,
    intent: Res<MoveIntent>,
    do_intent: Res<DoIntent>,
    target: Res<TargetIntent>,
    collision: Option<Res<Collision>>,
    mut actors: Query<(Entity, &mut Actor, &mut Medli), Without<Player>>,
    mut commands: Commands,
) {
    let World {
        mut stories,
        mut switches,
        mut events,
        list,
        mut active,
        carry,
        water,
        stage,
        text_flags,
        mut next,
        archive,
    } = world;
    let player = dialogue.players.iter().next().copied();
    let ship = dialogue.ships.iter().next().copied();
    let solid = collision.as_deref();
    let stick = intent.axis.length().min(1.0);
    let stick_angle = walk::to_s16(intent.axis.x.atan2(intent.axis.y));
    let clear_count = text_flags.clear_count;
    let archive: Option<&dyn MsgArchive> = archive
        .as_deref()
        .map(|value| value as &dyn MsgArchive);
    for (entity, mut actor, mut medli) in &mut actors {
        let actor = &mut *actor;
        let medli = &mut *medli;
        let press = do_intent.0 && !medli.do_held;
        medli.do_held = do_intent.0;
        medli.stick_angle = stick_angle;
        medli.player_yaw = player.map_or(0, |link| walk::to_s16(link.yaw));
        medli.player_shape = player.map_or(0, |link| walk::to_s16(link.face));
        let mut text = Text {
            procs: &mut *dialogue.procs,
            info: &mut *dialogue.info,
            flags: &mut *dialogue.flags,
            one: &mut *dialogue.one,
            two: &mut *dialogue.two,
            clear_count,
            archive,
        };
        let ground_y = ground::surface(solid, actor.pos);
        let ground_normal = ground::slope_normal(solid, actor.pos);
        let room = actor.room;
        let talk_range = HIO_M0C8;
        let talk_request = press
            && player.is_some_and(|link| {
                let gap = link.pos - actor.pos;
                gap.x * gap.x + gap.z * gap.z < talk_range * talk_range
            });
        let staff_cut = if medli.staff_idx == NO_STAFF {
            None
        } else {
            Some(medli.staff_idx)
        };
        let mut ctx = Ctx {
            entity,
            player: player.map(|link| link.pos),
            player_yaw: player.map_or(0, |link| walk::to_s16(link.yaw)),
            player_shape: player.map_or(0, |link| walk::to_s16(link.face)),
            player_speed_f: player.map_or(0.0, |link| link.speed),
            link: player.map(|link| link.pos),
            link_eye: player.map(|link| link.pos),
            stick,
            stick_angle,
            trig_a: do_intent.0,
            trig_b: target.0,
            trig_r: false,
            trig_start: false,
            is_player: false,
            carried: carry.held == Some(entity),
            ground_y,
            ground_normal,
            ground_normal_opt: ground_y.map(|_| ground_normal),
            ground_code: None,
            ground_room: None,
            actor_room: room,
            link_room: room,
            wall_hit: false,
            wall_angle: 0,
            wall_normal: None,
            roof_hit: false,
            probe_y: None,
            cross_normal: None,
            drop_normal: None,
            in_water: water.in_area,
            water_height: water.surface,
            ship_pos: None,
            ship_yaw: 0,
            ship_speed: ship.map_or(0.0, |ship| ship.speed),
            ship_riding: ship.is_some_and(|ship| ship.riding),
            room_drawn: true,
            cancel_button: target.0,
            staff_cut,
            stories: &mut *stories,
            switches: &mut *switches,
            events: &mut *events,
            list: &*list,
            active: &mut *active,
            text: &mut text,
            give_item: None,
            goto_sea: false,
            stage: stage.0.as_str(),
            talk_request,
            talk_xy: false,
            door_command: false,
            demo_accepted: false,
            wait_cut: false,
            drop_carry: false,
            special_ground: false,
            xy_end: false,
            reset_event: false,
            delete_me: false,
            wind_hit: false,
            wind_vec: Vec3::ZERO,
            auto_jump: -1,
            player_locked: false,
            attention_lock: None,
            link_move: None,
            tact_song: -1,
            tact_ended: true,
            link_hidden: false,
            pos_move: false,
            cut_end_mode: 0,
            move_block: None,
        };
        if !medli.entered {
            medli.entered = true;
            medli.home = actor.pos;
            medli.home_yaw = actor.angle[1];
            medli.home_room = actor.room;
            medli.init(actor, &mut ctx);
        } else {
            medli.execute(actor, &mut ctx);
        }
        if let Some((pos, item)) = ctx.give_item {
            let _ = commands.spawn((
                Actor {
                    name: PRESENT_ITEM_ACTOR.to_owned(),
                    parameters: u32::from(item),
                    pos,
                    angle: [0; 3],
                    set_id: u16::MAX,
                    room: actor.room,
                },
                super::event::PresentItem {
                    item,
                    hidden: false,
                },
            ));
            ctx.events.list.push(SimEvent::Collect);
        }
        if ctx.goto_sea {
            next.exit = Some(NEXT_STAGE_EXIT);
            next.target = Some(Target {
                stage: NEXT_STAGE_SEA.to_owned(),
                room: NEXT_STAGE_ROOM,
                start: NEXT_STAGE_START,
            });
        }
        if ctx.delete_me {
            let _ = commands.entity(entity).despawn();
        }
    }
}
