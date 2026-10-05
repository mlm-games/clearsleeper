use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::StoryBits;
use super::intent::{DoIntent, TargetIntent};
use super::inventory::{Equipped, Got};
use super::pickup::Rupees;
use super::player::Player;
use super::walk::{from_s16, to_s16};
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgFlags, MsgGameInfo, MsgProcs,
    fop_msgm_message_set,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS, STATUS_PREPARING,
};

pub const ZUNARI_ACTOR: &str = "Rsh1";
pub const ARC_NAME: &str = "Rsh";
pub const STAFF_NAME: &str = "Rsh1";
pub const EVENT_GET_DEMO: &str = "RSH_GET_DEMO";
pub const EVENT_SHOP_OUT: &str = "RSH_SHOP_OUT";
pub const PRESENT_ITEM_ACTOR: &str = "PresentItem";
pub const TALK_RANGE: f32 = HIO_MAX_ATTN_DIST_XZ;

pub const VARIANT_SHIFT: u32 = 0x14;
pub const VARIANT_MASK: u32 = 0xF;
pub const PATH_SHIFT: u32 = 0x10;
pub const PATH_MASK: u32 = 0xFF;

pub const HIO_EYE_OFFSET: f32 = -20.0;
pub const HIO_MAX_HEAD_X: i32 = 0x200;
pub const HIO_MAX_HEAD_Y: i32 = 0x200;
pub const HIO_MAX_BACKBONE_X: i32 = 0x1388;
pub const HIO_MAX_BACKBONE_Y: i32 = 0x1770;
pub const HIO_MIN_HEAD_X: i32 = -0x200;
pub const HIO_MIN_HEAD_Y: i32 = -0x200;
pub const HIO_MIN_BACKBONE_X: i32 = -5000;
pub const HIO_MIN_BACKBONE_Y: i32 = -6000;
pub const HIO_MAX_TURN_STEP: i32 = 0x1000;
pub const HIO_MAX_HEAD_TURN_VEL: i16 = 0x800;
pub const HIO_ATTN_Y_OFFSET: f32 = 80.0;
pub const HIO_MAX_ATTN_ANGLE_Y: i32 = 0x4000;
pub const HIO_MAX_ATTN_DIST_XZ: f32 = 400.0;
pub const HIO_CURSOR_SCALE_X: f32 = 1.0;
pub const HIO_CURSOR_SCALE_Y: f32 = 0.9;
pub const HIO_CURSOR_SCALE_Z: f32 = 0.5;
pub const HIO_CURSOR_STEP_X: f32 = 40.0;
pub const HIO_CURSOR_STEP_Y: f32 = 30.0;
pub const HIO_AIM_REACH: f32 = 1300.0;
pub const HIO_CYL_R1: f32 = 100.0;
pub const HIO_CYL_R2: f32 = 180.0;
pub const HIO_CYL_H: f32 = 190.0;
pub const HIO_CHASE_STEP: f32 = 7.5;
pub const HIO_SRC_RADIUS: f32 = 60.0;
pub const HIO_SRC_HEIGHT: f32 = 80.0;

pub const ATTENTION_BONUS_DIST: f32 = 40.0;
pub const ATTENTION_BONUS_ANGLE: i32 = 0x71C;

pub const SHOP_COUNT_LAST: u8 = 11;
pub const ROTEN_FIRST: u8 = 0x8C;
pub const ROTEN_LAST: u8 = 0x97;
pub const ROTEN_ROWS: usize = 4;
pub const ROTEN_ROW_LEN: usize = 3;
pub const ROTEN_COUNT: usize = ROTEN_ROWS * ROTEN_ROW_LEN;
pub const BAG_SLOTS: usize = 8;
pub const SHOP_MAX_BAG: i32 = 3;
pub const SHOP_COUNT_MANY: i32 = 8;
pub const SHOP_COUNT_SOME: i32 = 1;
pub const SHOP_STALL_FOUR: i32 = 3;
pub const SHOP_STALL_COUNT_MIN: i32 = 1;

pub const SHOP_MSG_FIRST: u32 = 0x286B;
pub const SHOP_MSG_LAST: u32 = 0x2882;
pub const SHOP_ERR_FIRST: u32 = 0x2868;
pub const SHOP_ERR_LAST: u32 = 0x286A;
pub const SHOP_CHOOSE: u32 = 0x2863;
pub const SHOP_SOLD_OUT: u32 = 0x2884;

pub const EVENT_BIT_0E08: u16 = 0x0E08;
pub const EVENT_BIT_1108: u16 = 0x1108;
pub const EVENT_BIT_1110: u16 = 0x1110;
pub const EVENT_BIT_2420: u16 = 0x2420;
pub const EVENT_BIT_2D01: u16 = 0x2D01;

pub const ITEM_MAGIC_ARMOR: u8 = 0x2A;
pub const ITEM_SAIL: u8 = 0x78;
pub const ITEM_DELIVERY_BAG: u8 = 0x30;
pub const ITEM_TOWN_FLOWER: u8 = 0x8C;
pub const ITEM_NONE: u8 = 0xFF;

pub const GOT_MAGIC_ARMOR: usize = 10;
pub const GOT_SAIL: usize = 1;
pub const GOT_DELIVERY_BAG: usize = 18;
pub const GOT_BIT: u8 = 0;

pub const SE_SHOP_BOUGHT: u32 = 0x087F;

pub const MSG_SAIL_TOTAL: u32 = 0x2889;
pub const MSG_SAIL_SHORT: u32 = 0x288A;
pub const MSG_SAIL_DECLINE: u32 = 0x288B;
pub const MSG_SAIL_DONE: u32 = 0x288D;
pub const MSG_SAIL_OPEN: u32 = 0x288C;
pub const MSG_STALL_MOVE: u32 = 0x2862;
pub const MSG_STALL_OPEN: u32 = 0x2865;
pub const MSG_STALL_BACK: u32 = 0x2866;
pub const MSG_STALL_LEAVE: u32 = 0x2867;
pub const MSG_STALL_BACK_2: u32 = 0x2883;
pub const MSG_ARMOR_FULL: u32 = 0x285D;
pub const MSG_SAIL_TALK: u32 = 0x2886;
pub const MSG_SAIL_GIVE: u32 = 0x288E;
pub const MSG_SAIL_AFTER: u32 = 0x288F;
pub const MSG_BAG_TALK: u32 = 0x2890;
pub const MSG_ARMOR_GIVE: u32 = 0x285F;
pub const MSG_ARMOR_AFTER: u32 = 0x2860;
pub const MSG_FLOWER_GIVE: u32 = 0x2856;
pub const MSG_FLOWER_AFTER: u32 = 0x2857;
pub const MSG_BUY_FIRST: u32 = 0x286B;
pub const MSG_BUY_LAST: u32 = 0x2882;
pub const MSG_SOLD_OUT_TALK: u32 = 0x2884;
pub const MSG_NO_PEONIES: u32 = 0x2849;
pub const MSG_HAS_PEONIES: u32 = 0x2858;
pub const MSG_ALL_EIGHT: u32 = 0x283E;
pub const MSG_SOME: u32 = 0x2840;
pub const MSG_HAVE_PEONY: u32 = 0x2842;
pub const MSG_FIRST_TALK: u32 = 0x2845;
pub const MSG_RETURN_TALK: u32 = 0x284B;
pub const MSG_STALL_TALK: u32 = 0x2859;
pub const MSG_RETURNING: u32 = 0x283D;
pub const MSG_RETURNING_2: u32 = 0x283F;
pub const MSG_RETURNING_3: u32 = 0x2841;
pub const MSG_RETURNING_4: u32 = 0x2843;
pub const MSG_BUY_ASK: u32 = 0x284C;
pub const MSG_BUY_YES: u32 = 0x2853;
pub const MSG_BUY_YES_NEXT: u32 = 0x2855;
pub const MSG_BUY_NO: u32 = 0x2854;
pub const MSG_RETURN_SKIP: u32 = 0x284E;
pub const MSG_BACK: [u32; 4] = [0x283D, 0x283F, 0x2841, 0x2843];
pub const SHOP_ERROR_MSGS: [u32; 4] = [0x2868, 0x2869, 0x286A, 0x2884];
pub const SHOP_ERR_MAX: u32 = 0x2869;
pub const SHOP_ERR_LOW: u32 = 0x2868;
pub const MSG_RUPEES_LOW: u32 = 0x284A;
pub const MSG_RUPEES_OK: u32 = 0x284D;

pub const MSG_CHAIN: [u32; 21] = [
    0x2845, 0x2846, 0x2847, 0x2848, 0x284B, 0x284D, 0x284E, 0x284F, 0x2850, 0x2851, 0x2852, 0x2855,
    0x2857, 0x2859, 0x285A, 0x285D, 0x285E, 0x2886, 0x2887, 0x2888, 0x288D,
];

pub const MSG_STICK_MOVE: [u32; 12] = [
    0x286B, 0x286D, 0x286F, 0x2871, 0x2873, 0x2875, 0x2877, 0x2879, 0x287B, 0x287D, 0x287F, 0x2881,
];

pub const MSG_SHOP_BUY: [u32; 12] = [
    0x286C, 0x286E, 0x2870, 0x2872, 0x2874, 0x2876, 0x2878, 0x287A, 0x287C, 0x287E, 0x2880, 0x2882,
];

pub const BUY_ERROR_MAXIMUM: u8 = 0x04;
pub const BUY_ERROR_RUPEES: u8 = 0x20;

pub const MODE_NONE: i8 = 0;
pub const MODE_TALK: i8 = 1;
pub const MODE_BUMPED: i8 = 2;
pub const MODE_FACE_ON: i8 = 3;
pub const MODE_GIVE_DEMO: i8 = 4;
pub const MODE_SHOP_OUT: i8 = 5;

pub const PHASE_WAIT: i8 = 1;
pub const PHASE_TALK: i8 = 2;

pub const TALK_INIT: i8 = 0;
pub const TALK_CREATE: i8 = 1;
pub const TALK_NORMAL: i8 = 2;
pub const TALK_SHOP: i8 = 3;
pub const TALK_FINISHED: i8 = -1;

pub const NO_STATUS: u8 = 0xFF;
pub const NO_INDEX: i16 = -1;
pub const NO_STAFF: usize = usize::MAX;
pub const NO_EVENT: usize = usize::MAX;
pub const NO_POINT: u8 = 0xFF;
pub const NO_SHOP: i32 = -1;
pub const PATH_POINTS: usize = 5;
pub const PATH_MARK_COUNT: usize = 4;
pub const TURN_SCALE: i16 = 8;
pub const TURN_MAX_STEP: i16 = 0x1000;
pub const HEAD_TURN_SCALE: i16 = 4;
pub const HEAD_TURN_MAX: i16 = 0x800;
pub const FACE_MAX_HI: i16 = 0x1800;
pub const FACE_MAX_LO: i16 = -0x2800;
pub const CAM_TALK_FRAMES: f32 = 45.0;
pub const ITEM_ZOOM_DIST: f32 = 125.0;
pub const EYE_LIFT: f32 = 170.0;
pub const GRAVITY: f32 = -30.0;
pub const BUMP_HEIGHT_TOL: f32 = 25.0;
pub const TALK_HEIGHT_TOL: f32 = 100.0;
pub const TRIG_AMPLITUDE: f32 = 140.0;
pub const CHASE_SCALE: f32 = 0.25;
pub const CHASE_MIN_STEP: f32 = 1.0;
pub const REG10_UNRESOLVED: f32 = 0.0;
pub const CHASE_MAX_STEP: f32 = REG10_UNRESOLVED + 5.0;
pub const STROLL_AMPLITUDE: f32 = REG10_UNRESOLVED + 100.0;
pub const CULL_LOOKAT_DIST: f32 = REG10_UNRESOLVED + 5000.0;
pub const CULL_SIZE_FAR: f32 = 0.25;
pub const BTP_RANDOM: f32 = 100.0;
pub const BTP_DELAY: i16 = 30;
pub const ANM_END_FRAME: f32 = 8.0;
pub const CAM_DATA_STALL: i32 = 8;
pub const CAM_DATA_SHOP: i32 = 8;
pub const CAM_TALK_A_IDX: i32 = 9;
pub const CAM_TALK_B_IDX: i32 = 10;

pub const REL_CAM_CTR_A: Vec3 = Vec3::new(96.0, -46.0, -26.0);
pub const REL_CAM_EYE_A: Vec3 = Vec3::new(-156.0, -16.0, 166.0);
pub const REL_CAM_CTR_B: Vec3 = Vec3::new(2.0, -32.0, 29.0);
pub const REL_CAM_EYE_B: Vec3 = Vec3::new(144.0, 19.0, 215.0);

pub const CAM_TALK_A: [u32; 7] = [0x2846, 0x284E, 0x2858, 0x285A, 0x285E, 0x2887, 0x2888];
pub const CAM_TALK_B: [u32; 4] = [0x2848, 0x2851, 0x2852, 0x288D];

pub const IN_CHK_POS1: [Vec3; 4] = [
    Vec3::new(919.0, 670.0, -205111.0),
    Vec3::new(1466.0, 670.0, -204723.0),
    Vec3::new(1090.0, 670.0, -204173.0),
    Vec3::new(603.0, 670.0, -204461.0),
];

pub const IN_CHK_POS2: [Vec3; 4] = [
    Vec3::new(1466.0, 670.0, -204723.0),
    Vec3::new(1700.0, 670.0, -204518.0),
    Vec3::new(1423.0, 670.0, -204174.0),
    Vec3::new(1125.0, 670.0, -204337.0),
];

pub const BASE_TALK_AREA: [Vec3; 4] = [
    Vec3::new(580.0, 671.0, -204393.0),
    Vec3::new(539.0, 671.0, -204321.0),
    Vec3::new(930.0, 671.0, -204114.0),
    Vec3::new(971.0, 671.0, -204189.0),
];

pub const SHOP_BUYER_POS: [Vec3; 4] = [
    Vec3::new(1314.0, 672.0, -204426.0),
    Vec3::new(1544.0, 672.0, -204214.0),
    Vec3::new(1775.0, 672.0, -204029.0),
    Vec3::new(1985.0, 672.0, -203786.0),
];

pub const STALL_Y: [i16; ROTEN_ROWS] = [0xE800, 0xE400, 0xE000, 0xDC00];

pub const CUT_NAMES: [&str; 4] = ["TALKMSG", "CONTINUE_TALK", "SETANM", "TURN"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proc {
    None,
    Wait,
    PlShopOut,
    GetDemo,
    Event,
    Dummy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookTarget {
    None,
    Player,
    Item,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Zunari {
    pub variant: i8,
    pub proc_state: Proc,
    pub action: i8,
    pub mode: i8,
    pub phase: i8,
    pub prev_phase: u8,
    pub talk_phase: i8,
    pub cam_state: i8,
    pub anm: i8,
    pub anm_frame: f32,
    pub anm_count: i8,
    pub head_x: i16,
    pub head_y: i16,
    pub backbone_x: i16,
    pub backbone_y: i16,
    pub tex_anm: i8,
    pub btp_frame: u8,
    pub btp_timer: i16,
    pub shop_idx: i32,
    pub shop_count: i32,
    pub shop_items: Option<usize>,
    pub shop_selected: i16,
    pub shop_select_item_no: u8,
    pub shop_cursor_moved: bool,
    pub shop_cam_data: i32,
    pub shop_stock: [u8; ROTEN_COUNT],
    pub stall_y: i16,
    pub cam_action: i8,
    pub path_index: i8,
    pub path_no: i32,
    pub path_ready: bool,
    pub path_point_pos: [Vec3; PATH_POINTS],
    pub path_point_mark: [i8; PATH_POINTS],
    pub msg_no: u32,
    pub shop_msg_no: u32,
    pub forced_msg_no: u32,
    pub ev_msg_end: u32,
    pub msg_id: u32,
    pub item_no: u8,
    pub shop_out_event: i16,
    pub get_demo_event: i16,
    pub staff_idx: usize,
    pub roten_reserve: u16,
    pub bag_reserve: [u8; BAG_SLOTS],
    pub attention: bool,
    pub talking: bool,
    pub talk_condition: bool,
    pub speak_ordered: bool,
    pub player_in_shop: bool,
    pub look_target: LookTarget,
    pub look_aim: Vec3,
    pub look_enable: bool,
    pub look_base: i16,
    pub look_at_max_vel: i16,
    pub entered: bool,
    pub head_turning: bool,
    pub do_held: bool,
    pub send_button: bool,
    pub home: Vec3,
    pub home_angle: i16,
    pub attn_base_pos: Vec3,
    pub attn_pos: Vec3,
    pub eye_pos: Vec3,
    pub push_anchor: Vec3,
    pub push_offset: Vec3,
}

impl Zunari {
    pub fn spawn(parameters: u32) -> Self {
        let variant = match (parameters >> VARIANT_SHIFT) & VARIANT_MASK {
            0 => 0,
            _ => 0,
        };
        Self {
            variant: variant as i8,
            proc_state: Proc::None,
            action: 0,
            mode: MODE_NONE,
            phase: PHASE_WAIT,
            prev_phase: PHASE_WAIT as u8,
            talk_phase: TALK_INIT,
            cam_state: 0,
            anm: -1,
            anm_frame: 0.0,
            anm_count: 0,
            head_x: 0,
            head_y: 0,
            backbone_x: 0,
            backbone_y: 0,
            tex_anm: -1,
            btp_frame: 0,
            btp_timer: 0,
            shop_idx: NO_SHOP,
            shop_count: 0,
            shop_items: None,
            shop_selected: -1,
            shop_select_item_no: ITEM_NONE,
            shop_cursor_moved: false,
            shop_cam_data: CAM_DATA_STALL,
            shop_stock: [0; ROTEN_COUNT],
            stall_y: 0,
            cam_action: 0,
            path_index: 0,
            path_no: 0,
            path_ready: false,
            path_point_pos: [Vec3::ZERO; PATH_POINTS],
            path_point_mark: [0; PATH_POINTS],
            msg_no: 0,
            shop_msg_no: 0,
            forced_msg_no: 0,
            ev_msg_end: 0,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            item_no: ITEM_NONE,
            shop_out_event: NO_INDEX,
            get_demo_event: NO_INDEX,
            staff_idx: NO_STAFF,
            roten_reserve: 0,
            bag_reserve: [0; BAG_SLOTS],
            attention: false,
            talking: false,
            talk_condition: false,
            speak_ordered: false,
            player_in_shop: false,
            look_target: LookTarget::None,
            look_aim: Vec3::ZERO,
            look_enable: true,
            look_base: 0,
            look_at_max_vel: 0,
            entered: false,
            head_turning: false,
            do_held: false,
            send_button: false,
            home: Vec3::ZERO,
            home_angle: 0,
            attn_base_pos: Vec3::ZERO,
            attn_pos: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            push_anchor: Vec3::ZERO,
            push_offset: Vec3::ZERO,
        }
    }

    pub fn path_index_of(&self, slot: i32) -> i8 {
        let at = usize::try_from(slot).unwrap_or(usize::MAX);
        self.path_point_mark.get(at).copied().unwrap_or(0)
    }

    pub fn roten_item_no(&self, index: usize) -> u8 {
        ROTEN_FIRST.wrapping_add(index as u8)
    }

    pub fn set_roten_reserve(&mut self, bit: u8) {
        if bit <= SHOP_COUNT_LAST {
            self.roten_reserve |= 1 << bit;
        }
    }

    pub fn reserve_bag_item(&mut self, item: u8) {
        if let Some(slot) = self.bag_reserve.iter_mut().find(|slot| **slot == 0) {
            *slot = item;
        }
    }

    pub fn owns_magic_armor(&self, got: &Got) -> bool {
        got.has(GOT_MAGIC_ARMOR, GOT_BIT)
    }

    pub fn owns_sail(&self, got: &Got) -> bool {
        got.has(GOT_SAIL, GOT_BIT)
    }

    pub fn owns_delivery_bag(&self, equipped: &Equipped) -> bool {
        equipped.item(GOT_DELIVERY_BAG) == ITEM_DELIVERY_BAG
    }

    pub fn owns_sail_slot(&self, equipped: &Equipped) -> bool {
        equipped.item(GOT_SAIL) == ITEM_SAIL
    }

    pub fn count_shop(&self, equipped: &Equipped) -> i32 {
        let mut result = 0;
        let mut i: u8 = 0;
        while i <= SHOP_COUNT_LAST {
            if self.roten_reserve & (1 << i) != 0 {
                result += 1;
            } else if i == 0 && self.owns_delivery_bag(equipped) {
                result += 1;
            }
            i += 1;
        }
        result
    }

    pub fn roten_item_num_in_bag(&self) -> i32 {
        let mut result = 0;
        let mut i = ROTEN_FIRST;
        while i <= ROTEN_LAST {
            for slot in self.bag_reserve.iter() {
                if *slot == i {
                    result += 1;
                }
            }
            i += 1;
        }
        result
    }

    pub fn check_roten_item_get(&self, index: u8, equipped: &Equipped) -> bool {
        if index != 0 {
            self.roten_reserve & (1 << index) != 0
        } else {
            self.owns_delivery_bag(equipped)
        }
    }
}

pub const IS_ZERO: f32 = 8e-11;

fn is_zero(value: f32) -> bool {
    value.abs() < IS_ZERO
}

fn ssin(angle: i16) -> f32 {
    from_s16(angle).sin()
}

fn scos(angle: i16) -> f32 {
    from_s16(angle).cos()
}

fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn abs_xz(v: Vec3) -> f32 {
    (v.x * v.x + v.z * v.z).sqrt()
}

fn add_calc_angle_s2(value: i16, target: i16, scale: i16, max_step: i16) -> i16 {
    let diff = target.wrapping_sub(value);
    let step = diff / scale;
    if step > max_step {
        value.wrapping_add(max_step)
    } else if step < -max_step {
        value.wrapping_sub(max_step)
    } else {
        value.wrapping_add(step)
    }
}

fn chase_pos_xz(value: &mut Vec3, target: Vec3, step: f32) -> bool {
    let mut diff = *value - target;
    diff.y = 0.0;
    let dist = abs_xz(diff);
    if step != 0.0 {
        if is_zero(dist) || dist <= step {
            *value = target;
            return true;
        }
        *value -= diff * (step / dist);
    } else if is_zero(dist) {
        return true;
    }
    false
}

fn add_calc_pos(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32, min_step: f32) -> f32 {
    if *value != target {
        let mut diff = *value - target;
        let mut step = diff.length();
        if step < min_step {
            *value = target;
        } else {
            step *= scale;
            diff *= scale;
            if !is_zero(step) {
                if step > max_step {
                    diff *= max_step / step;
                } else if step < min_step {
                    diff *= min_step / step;
                }
                *value -= diff;
            } else {
                *value = target;
            }
        }
    }
    (*value - target).length()
}

fn winding_count(table: &[Vec3; 4], link_pos: Vec3) -> i32 {
    let mut l = 0;
    let mut j: usize = 0;
    let mut k: usize = 1;
    while j < 4 {
        let temp2 = table.get(j).copied().unwrap_or(Vec3::ZERO) - link_pos;
        let temp3 = table.get(k).copied().unwrap_or(Vec3::ZERO) - link_pos;
        let cross = temp2.cross(temp3);
        if cross.y > 0.0 {
            l += 1;
        }
        k += 1;
        j += 1;
        if k > 3 {
            k = 0;
        }
    }
    l
}

pub fn shop_msg_check(msg_no: u32) -> bool {
    (SHOP_MSG_FIRST <= msg_no && msg_no <= SHOP_MSG_LAST)
        || (SHOP_ERR_FIRST <= msg_no && msg_no <= SHOP_ERR_LAST)
        || msg_no == SHOP_CHOOSE
        || msg_no == SHOP_SOLD_OUT
}

pub fn shop_stick_move_msg_check(msg_no: u32) -> bool {
    (SHOP_MSG_FIRST <= msg_no && msg_no <= SHOP_MSG_LAST && msg_no % 2 != 0)
        || msg_no == SHOP_CHOOSE
}

pub fn check_roten_base_talk_area(player: Vec3) -> bool {
    let l = winding_count(&BASE_TALK_AREA, player);
    l == 4 || l == 0
}

impl Zunari {
    pub fn check_create_in_shop_player(&self, actor: &Actor, player: Vec3) -> bool {
        let dir = Vec3::new(-scos(actor.angle[1]), 0.0, ssin(actor.angle[1]));
        let diff = player - actor.pos;
        for table in [&IN_CHK_POS1, &IN_CHK_POS2] {
            let l = winding_count(table, player);
            if (l == 4 || l == 0) && diff.dot(dir) > 0.0 && diff.y.abs() < BUMP_HEIGHT_TOL {
                return true;
            }
        }
        false
    }

    pub fn set_anm(&mut self, index: i8) {
        if index != self.anm || self.anm == -1 {
            self.anm = index;
            self.anm_frame = 0.0;
        }
    }

    pub fn set_tex_anm(&mut self, index: i8) {
        if index != self.tex_anm || self.tex_anm == -1 {
            self.tex_anm = index;
            self.btp_frame = 0;
            self.btp_timer = 0;
        }
    }

    pub fn set_anm_from_msg_tag(&mut self, attr: u8) {
        match attr {
            0 => self.set_anm(0),
            1 => self.set_anm(1),
            2 => self.set_anm(2),
            3 => self.set_anm(3),
            4 => {
                self.set_anm(4);
                self.anm_count = 2;
            }
            5 => self.set_anm(5),
            6 => self.set_anm(6),
            _ => {}
        }
        if self.anm == 4 && self.anm_frame == ANM_END_FRAME - 1.0 {
            self.anm_count -= 1;
            if self.anm_count <= 0 {
                self.set_anm(0);
                self.anm_count = 0;
            }
        }
    }

    pub fn advance_anm(&mut self) {
        if self.anm >= 0 {
            self.anm_frame += 1.0;
            if self.anm_frame >= ANM_END_FRAME {
                self.anm_frame = 0.0;
            }
        }
    }

    pub fn chk_attention(&self, pos: Vec3, player: Vec3, adjustment: i16, attentive: bool) -> bool {
        let mut max_dist = HIO_MAX_ATTN_DIST_XZ;
        let mut max_angle: i32 = HIO_MAX_ATTN_ANGLE_Y;
        let pos_diff = player - pos;
        let temp_abs_xz = abs_xz(pos_diff);
        let mut angle = to_s16(pos_diff.x.atan2(pos_diff.z));
        if attentive {
            max_dist += ATTENTION_BONUS_DIST;
            max_angle += ATTENTION_BONUS_ANGLE;
        }
        angle = angle.wrapping_sub(adjustment);
        max_angle > i32::from(angle.unsigned_abs()) && max_dist > temp_abs_xz
    }

    pub fn talk_init(&mut self) {
        self.talk_phase = TALK_INIT;
    }

    pub fn set_attention(&mut self, actor: &Actor) {
        self.attn_pos = Vec3::new(
            self.attn_base_pos.x,
            self.attn_base_pos.y + HIO_ATTN_Y_OFFSET,
            self.attn_base_pos.z,
        );
        self.eye_pos = Vec3::new(actor.pos.x, actor.pos.y + EYE_LIFT, actor.pos.z);
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

    pub fn message_rupee(&self) -> i16 {
        self.info.message_rupee
    }
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub player: Option<Vec3>,
    pub send_button: bool,
    pub talk_request: bool,
    pub b_button: bool,
    pub events: &'a mut SimEvents,
    pub stories: &'a mut StoryBits,
    pub rupees: &'a mut Rupees,
    pub got: &'a mut Got,
    pub equipped: &'a mut Equipped,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub text: &'a mut Text<'a>,
    pub give_item: Option<(Vec3, u8)>,
}

pub const ITEM_BIRD_BAIT_5: u8 = 0x82;
pub const ITEM_HYOI_PEAR: u8 = 0x83;
pub const ITEM_SHOP_GURU: u8 = 0x97;
pub const BUY_ERROR_MAX_ITEM: i32 = -1;

pub fn shop_max_check(item: u8) -> bool {
    if item == ITEM_TOWN_FLOWER {
        item == ITEM_SHOP_GURU
    } else {
        false
    }
}

pub fn shop_bought_error_status(item: u8, buy_price: i32, rupees: u32) -> u8 {
    let mut status = 0_u8;
    if shop_max_check(item) {
        status |= BUY_ERROR_MAXIMUM;
    }
    let price = if buy_price == BUY_ERROR_MAX_ITEM { 0 } else { buy_price };
    if i32::try_from(rupees).unwrap_or(i32::MAX) < price {
        status |= BUY_ERROR_RUPEES;
    }
    status
}

pub fn buy_msg_no(item: u8) -> u32 {
    MSG_BUY_FIRST.wrapping_add(1 + 2 * u32::from(item.wrapping_sub(ROTEN_FIRST)))
}

pub fn show_msg_no(item: u8) -> u32 {
    MSG_BUY_FIRST + 2 * u32::from(item.wrapping_sub(ROTEN_FIRST))
}

pub fn check_item_get(item: u8, got: &Got, roten: u16) -> bool {
    if (ROTEN_FIRST..=ROTEN_LAST).contains(&item) {
        roten & (1 << (item - ROTEN_FIRST)) != 0
    } else if item == ITEM_MAGIC_ARMOR {
        got.has(GOT_MAGIC_ARMOR, GOT_BIT)
    } else if item == ITEM_SAIL {
        got.has(GOT_SAIL, GOT_BIT)
    } else if item == ITEM_DELIVERY_BAG {
        got.has(GOT_DELIVERY_BAG, GOT_BIT)
    } else {
        true
    }
}

impl Zunari {
    pub fn exec_item_get(&mut self, item: u8, got: &mut Got, equipped: &mut Equipped) {
        if (ROTEN_FIRST..=ROTEN_LAST).contains(&item) {
            let bit = item - ROTEN_FIRST;
            self.set_roten_reserve(bit);
            self.reserve_bag_item(item);
            return;
        }
        if item == ITEM_MAGIC_ARMOR {
            got.gain(GOT_MAGIC_ARMOR, GOT_BIT);
            equipped.set(GOT_MAGIC_ARMOR, ITEM_MAGIC_ARMOR);
        } else if item == ITEM_SAIL {
            equipped.set(GOT_SAIL, ITEM_SAIL);
            got.gain(GOT_SAIL, GOT_BIT);
        } else if item == ITEM_DELIVERY_BAG {
            got.gain(GOT_DELIVERY_BAG, GOT_BIT);
            equipped.set(GOT_DELIVERY_BAG, ITEM_DELIVERY_BAG);
        }
    }

    pub fn select_item_no(&self) -> u8 {
        let stall = usize::try_from(self.shop_idx).unwrap_or(usize::MAX);
        let at = stall * ROTEN_ROW_LEN + usize::try_from(self.shop_selected).unwrap_or(usize::MAX);
        self.shop_stock.get(at).copied().unwrap_or(0)
    }

    pub fn next_msg_status(&mut self, actor: &mut Actor, msg_no: &mut u32, ctx: &mut Ctx) -> u8 {
        let mut status = STATUS_MSG_CONTINUES;
        if MSG_CHAIN.contains(msg_no) {
            *msg_no = msg_no.wrapping_add(1);
            return status;
        }
        if *msg_no == MSG_SAIL_OPEN {
            *msg_no = MSG_SAIL_TOTAL;
            return status;
        }
        if *msg_no == MSG_SAIL_TOTAL {
            if ctx.text.select_num() == 0 {
                let msg_rupee = i32::from(ctx.text.message_rupee());
                let have = i32::try_from(ctx.rupees.count).unwrap_or(i32::MAX);
                if msg_rupee > have {
                    *msg_no = MSG_SAIL_SHORT;
                    return status;
                }
                ctx.rupees.count = ctx
                    .rupees
                    .count
                    .saturating_sub(u32::try_from(msg_rupee).unwrap_or(0));
                *msg_no = MSG_SAIL_DONE;
                return status;
            }
            *msg_no = MSG_SAIL_DECLINE;
            return status;
        }
        if *msg_no == MSG_STALL_MOVE {
            if !self.shop_pos_move(actor, ctx) {
                status = STATUS_MSG_DISPLAYED;
            } else {
                self.set_anm(0);
                *msg_no = msg_no.wrapping_add(1);
            }
            return status;
        }
        if MSG_BACK.contains(msg_no) {
            *msg_no = msg_no.wrapping_sub(1);
            return status;
        }
        if *msg_no == MSG_BUY_ASK {
            *msg_no = if ctx.text.select_num() == 0 {
                MSG_RUPEES_OK
            } else {
                MSG_RUPEES_LOW
            };
            return status;
        }
        if *msg_no == MSG_BUY_YES {
            *msg_no = if ctx.text.select_num() == 0 {
                MSG_BUY_YES_NEXT
            } else {
                MSG_BUY_NO
            };
            return status;
        }
        if *msg_no == MSG_BUY_NO {
            *msg_no = MSG_RETURN_SKIP;
            return status;
        }
        if *msg_no == MSG_STALL_BACK {
            *msg_no = SHOP_CHOOSE;
            return status;
        }
        if *msg_no == SHOP_CHOOSE {
            if ctx.b_button {
                *msg_no = MSG_STALL_LEAVE;
            } else {
                status = STATUS_MSG_DISPLAYED;
            }
            return status;
        }
        if MSG_STICK_MOVE.contains(msg_no) {
            if ctx.b_button {
                *msg_no = MSG_STALL_LEAVE;
            } else {
                *msg_no = msg_no.wrapping_add(1);
            }
            return status;
        }
        if MSG_SHOP_BUY.contains(msg_no) {
            if self.shop_items.is_some() {
                if ctx.b_button {
                    *msg_no = msg_no.wrapping_sub(1);
                    return status;
                }
                if ctx.text.select_num() == 0 {
                    let msg_rupee = i32::from(ctx.text.message_rupee());
                    let item = self.select_item_no();
                    let bought = shop_bought_error_status(item, msg_rupee, ctx.rupees.count);
                    if bought & BUY_ERROR_RUPEES != 0 {
                        *msg_no = SHOP_ERR_LOW;
                        return status;
                    }
                    if bought & BUY_ERROR_MAXIMUM != 0 {
                        *msg_no = SHOP_ERR_MAX;
                        return status;
                    }
                    if self.roten_item_num_in_bag() >= SHOP_MAX_BAG {
                        *msg_no = SHOP_SOLD_OUT;
                        return status;
                    }
                    ctx.events.list.push(SimEvent::Se(SE_SHOP_BOUGHT));
                    self.shop_select_item_no = item;
                    ctx.rupees.count = ctx
                        .rupees
                        .count
                        .saturating_sub(u32::try_from(msg_rupee).unwrap_or(0));
                    if !check_item_get(item, ctx.got, self.roten_reserve) {
                        self.mode = MODE_GIVE_DEMO;
                        status = STATUS_MSG_ENDS;
                        return status;
                    }
                    self.exec_item_get(item, ctx.got, ctx.equipped);
                    *msg_no = SHOP_ERR_LAST;
                    return status;
                }
                *msg_no = msg_no.wrapping_sub(1);
                return status;
            }
            return status;
        }
        if SHOP_ERROR_MSGS.contains(msg_no) {
            self.shop_selected = -1;
            *msg_no = SHOP_CHOOSE;
            return status;
        }
        if *msg_no == MSG_NO_PEONIES {
            ctx.stories.set(EVENT_BIT_1110);
            status = STATUS_MSG_ENDS;
            return status;
        }
        if *msg_no == MSG_HAS_PEONIES {
            ctx.stories.set(EVENT_BIT_1108);
            status = STATUS_MSG_ENDS;
            return status;
        }
        STATUS_MSG_ENDS
    }

    pub fn get_msg(&mut self, ctx: &mut Ctx) -> u32 {
        if self.forced_msg_no != 0 {
            let msg_no = self.forced_msg_no;
            self.forced_msg_no = 0;
            return msg_no;
        }
        if !self.owns_magic_armor(ctx.got) && self.count_shop(ctx.equipped) >= SHOP_STALL_FOUR {
            return MSG_ARMOR_FULL;
        }
        if !self.owns_sail_slot(ctx.equipped) {
            if ctx.stories.test(EVENT_BIT_2420) {
                return MSG_SAIL_OPEN;
            }
            ctx.stories.set(EVENT_BIT_2420);
            return MSG_SAIL_TALK;
        }
        if !self.owns_delivery_bag(ctx.equipped) {
            return MSG_BAG_TALK;
        }
        if self.shop_idx == NO_SHOP {
            if ctx.stories.test(EVENT_BIT_2D01)
                && !ctx.stories.test(EVENT_BIT_0E08)
                && ctx.stories.test(EVENT_BIT_1108)
            {
                return MSG_STALL_TALK;
            }
            let shop_cnt = self.count_shop(ctx.equipped);
            if shop_cnt >= SHOP_COUNT_MANY {
                return MSG_ALL_EIGHT;
            }
            if shop_cnt > SHOP_COUNT_SOME {
                return MSG_SOME;
            }
            if ctx.stories.test(EVENT_BIT_1108) {
                return MSG_HAVE_PEONY;
            }
            if !ctx.stories.test(EVENT_BIT_1110) {
                MSG_FIRST_TALK
            } else {
                MSG_RETURN_TALK
            }
        } else if !ctx.stories.test(EVENT_BIT_1108) {
            MSG_STALL_OPEN
        } else {
            MSG_STALL_MOVE
        }
    }
}

impl Zunari {
    pub fn shop_pos_move(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        if self.shop_idx != NO_SHOP {
            let slot = usize::try_from(self.shop_idx).unwrap_or(usize::MAX);
            let temp = SHOP_BUYER_POS.get(slot).copied().unwrap_or(Vec3::ZERO);
            if chase_pos_xz(&mut actor.pos, temp, HIO_CHASE_STEP) {
                self.set_anm(0);
                return true;
            }
            let target = target_angle_y(actor.pos, temp);
            actor.angle[1] = add_calc_angle_s2(actor.angle[1], target, TURN_SCALE, TURN_MAX_STEP);
            self.set_anm(5);
        } else {
            return true;
        }
        false
    }

    pub fn get_aim_shop_pos_idx(&self, player: Vec3) -> usize {
        let mut result_index = 1_usize;
        let mut fcond = HIO_AIM_REACH;
        let condition = if self.shop_count < SHOP_STALL_COUNT_MIN {
            SHOP_STALL_COUNT_MIN
        } else {
            self.shop_count
        };
        let mut i: usize = 0;
        loop {
            let mut temp = player;
            let base = self.path_point_pos.get(i).copied().unwrap_or(Vec3::ZERO);
            temp -= base;
            let mag = abs_xz(temp);
            if mag < fcond {
                fcond = mag;
                result_index = i;
            }
            if i >= usize::try_from(condition).unwrap_or(usize::MAX) {
                break;
            }
            i += 1;
        }
        result_index
    }

    pub fn path_move(&mut self, actor: &mut Actor, ctx: &Ctx, aim: Option<usize>) -> bool {
        if !self.path_ready {
            return false;
        }
        let temp4 = match aim {
            Some(value) => i32::try_from(value).unwrap_or(0),
            None => {
                let player = ctx.player.unwrap_or(Vec3::ZERO);
                i32::try_from(self.get_aim_shop_pos_idx(player)).unwrap_or(0)
            }
        };
        let mark = self.path_index_of(temp4);
        let temp = if self.path_index == mark {
            self.path_index
        } else if self.path_index < mark {
            self.path_index + 1
        } else {
            self.path_index - 1
        };
        let at = usize::try_from(temp).unwrap_or(usize::MAX);
        let point = self.path_point_pos.get(at).copied().unwrap_or(Vec3::ZERO);
        let mark_at = self.path_point_mark.get(at).copied().unwrap_or(NO_POINT);
        let chase_result = chase_pos_xz(&mut actor.pos, point, HIO_CHASE_STEP);
        if chase_result {
            self.path_index = temp;
            if mark_at != NO_POINT {
                let temp2 = i32::from(mark_at) - 1;
                let temp3 = temp2 + 9;
                self.shop_idx = temp2;
                if self.shop_idx != NO_SHOP {
                    self.shop_items = usize::try_from(self.shop_idx).ok();
                    self.shop_cam_data = self.shop_idx + CAM_DATA_SHOP;
                } else {
                    self.shop_items = None;
                }
                self.shop_cam_data = temp3;
                if temp4 == temp2 + 1 {
                    actor.angle[1] = add_calc_angle_s2(
                        actor.angle[1],
                        self.home_angle,
                        TURN_SCALE,
                        TURN_MAX_STEP,
                    );
                    return true;
                }
            }
        }
        let target = target_angle_y(actor.pos, point);
        actor.angle[1] = add_calc_angle_s2(actor.angle[1], target, TURN_SCALE, TURN_MAX_STEP);
        false
    }

    pub fn path_get(&mut self, parameters: u32) {
        self.path_no = (((parameters >> PATH_SHIFT) & PATH_MASK) & 0xFF) as i32;
        self.path_ready = false;
        self.shop_idx = 0;
    }

    pub fn create_shop_list(&mut self, equipped: &Equipped) {
        let mut k: usize = 0;
        let mut i: isize = 0;
        while i < ROTEN_ROWS as isize {
            let row = usize::try_from(i).unwrap_or(usize::MAX);
            let mut j: usize = 0;
            self.stall_y = STALL_Y.get(row).copied().unwrap_or(0);
            while j < ROTEN_ROW_LEN && k < ROTEN_COUNT {
                let idx = row * ROTEN_ROW_LEN + j;
                let item = self.roten_item_no(k);
                if self.check_roten_item_get(u8::try_from(k).unwrap_or(0), equipped) {
                    if let Some(slot) = self.shop_stock.get_mut(idx) {
                        *slot = item;
                    }
                    j += 1;
                }
                k += 1;
            }
            if j < ROTEN_ROW_LEN {
                if j == 0 {
                    i -= 1;
                }
                break;
            } else if i == ROTEN_ROWS as isize - 1 {
                break;
            }
            i += 1;
        }
        self.shop_count = i32::try_from(i + 1).unwrap_or(0);
        self.shop_count = if self.shop_count > SHOP_STALL_COUNT_MIN {
            self.shop_count
        } else {
            SHOP_STALL_COUNT_MIN
        };
    }

    pub fn wait01(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        if self.talking {
            self.prev_phase = self.phase as u8;
            self.phase = PHASE_TALK;
            return;
        }
        if self.cyl_hit(actor, ctx) {
            self.mode = MODE_BUMPED;
            self.player_in_shop = true;
            self.forced_msg_no = MSG_STALL_BACK_2;
            return;
        }
        let path_move_res = self.path_move(actor, ctx, None);
        if !path_move_res {
            self.set_anm(5);
            self.mode = MODE_NONE;
        } else {
            self.set_anm(0);
        }
        if self.attention {
            let abs_diff = match ctx.player {
                Some(player) => (actor.pos.y - player.y).abs(),
                None => 0.0,
            };
            if path_move_res && abs_diff < TALK_HEIGHT_TOL {
                self.mode = MODE_FACE_ON;
            } else if self.mode == MODE_FACE_ON {
                self.mode = MODE_NONE;
            }
        }
    }

    pub fn cyl_hit(&self, actor: &Actor, ctx: &Ctx) -> bool {
        let Some(player) = ctx.player else {
            return false;
        };
        let dx = player.x - actor.pos.x;
        let dz = player.z - actor.pos.z;
        let (radius, height) = if self.proc_state == Proc::PlShopOut {
            (HIO_CYL_R2, HIO_CYL_H)
        } else {
            (HIO_CYL_R1, HIO_CYL_H)
        };
        let half = height * 0.5;
        dx * dx + dz * dz < radius * radius
            && player.y - actor.pos.y <= half
            && player.y - actor.pos.y >= -half
    }
}

impl Zunari {
    pub fn normal_talk(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> u8 {
        let status = ctx.text.status().unwrap_or(NO_STATUS);
        if status == STATUS_MSG_DISPLAYED {
            let mut next_no = self.msg_no;
            let next = self.next_msg_status(actor, &mut next_no, ctx);
            self.msg_no = next_no;
            ctx.text.set_status(next);
            if next == STATUS_MSG_CONTINUES {
                let msg_no = self.msg_no;
                ctx.text.message_set(msg_no, Vec3::ZERO);
            }
        } else if status == STATUS_BOX_CLOSED {
            ctx.text.set_status(STATUS_MSG_DESTROYED);
        } else if status == STATUS_PREPARING {
            ctx.text.flags.demo_msg_flag_on();
        }
        if ctx.send_button {
            self.msg_no = ctx.text.msg_no();
            self.shop_cam_data = talk_cam_slot(self.msg_no);
        }
        status
    }

    pub fn shop_talk(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> u8 {
        let status = ctx.text.status().unwrap_or(NO_STATUS);
        if status == STATUS_MSG_DISPLAYED || status == STATUS_MSG_CONTINUES {
            if self.shop_cursor_moved {
                self.shop_cursor_moved = false;
            } else {
                let mut next_no = self.msg_no;
                let next = self.next_msg_status(actor, &mut next_no, ctx);
                self.shop_msg_no = next_no;
                ctx.text.set_status(next);
                if next == STATUS_MSG_CONTINUES {
                    let msg_no = self.shop_msg_no;
                    ctx.text.message_set(msg_no, Vec3::ZERO);
                }
            }
        } else if status == STATUS_BOX_CLOSED {
            ctx.text.set_status(STATUS_MSG_DESTROYED);
            self.shop_selected = -1;
        } else if status == STATUS_PREPARING {
            ctx.text.flags.demo_msg_flag_on();
        }
        status
    }

    pub fn talk(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> u8 {
        let mut status = NO_STATUS;
        if self.talk_phase == TALK_INIT {
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
            self.msg_no = self.get_msg(ctx);
            self.shop_msg_no = 0;
            self.talk_phase = TALK_CREATE;
            self.shop_select_item_no = ITEM_NONE;
        } else if self.talk_phase != TALK_FINISHED {
            if self.msg_id == FPC_M_ERROR_PROCESS_ID {
                let msg_no = self.msg_no;
                let eye = self.eye_pos;
                self.msg_id = ctx.text.message_set(msg_no, eye);
            } else if ctx.text.procs.search_by_id(self.msg_id).is_none() {
                if !shop_stick_move_msg_check(self.msg_no) {
                    self.talk_phase = TALK_NORMAL;
                } else {
                    self.talk_phase = TALK_SHOP;
                }
            } else {
                let attr = ctx.text.info.mesg_anime;
                self.set_anm_from_msg_tag(attr);
                ctx.text.info.set_mesg_anime_attr_info(0);
                match self.talk_phase {
                    TALK_NORMAL => status = self.normal_talk(actor, ctx),
                    TALK_SHOP => status = self.shop_talk(actor, ctx),
                    _ => {}
                }
                if ctx.send_button {
                    self.msg_no = ctx.text.msg_no();
                    self.talk_phase = if shop_stick_move_msg_check(self.msg_no) {
                        TALK_SHOP
                    } else {
                        TALK_NORMAL
                    };
                }
            }
        }
        status
    }

    pub fn talk01(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> u8 {
        let result = self.talk(actor, ctx);
        if result == STATUS_BOX_CLOSED {
            self.phase = self.prev_phase as i8;
            self.talking = false;
            self.set_anm(0);
            if self.msg_no == MSG_FLOWER_GIVE {
                self.mode = MODE_GIVE_DEMO;
                self.item_no = ITEM_TOWN_FLOWER;
            } else if self.msg_no == MSG_SAIL_GIVE {
                self.mode = MODE_GIVE_DEMO;
                self.item_no = ITEM_SAIL;
            } else if self.msg_no == MSG_ARMOR_GIVE {
                self.mode = MODE_GIVE_DEMO;
                self.item_no = ITEM_MAGIC_ARMOR;
            } else if self.msg_no == MSG_STALL_BACK_2 && self.player_in_shop {
                set_action(self, actor, ctx, Proc::PlShopOut);
            }
        }
        result
    }

    pub fn wait_action(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        if self.action == 0 {
            self.phase = PHASE_WAIT;
            self.action = 1;
        } else if self.action != -1 {
            let sum = actor.angle[1]
                .wrapping_add(self.head_y)
                .wrapping_add(self.backbone_y);
            self.attention = ctx
                .player
                .is_some_and(|player| self.chk_attention(actor.pos, player, sum, false));
            self.head_turning = self.attention;
            match self.phase {
                PHASE_WAIT => self.wait01(actor, ctx),
                PHASE_TALK => {
                    self.talk01(actor, ctx);
                }
                _ => {}
            }
        }
    }
}

fn talk_cam_slot(msg_no: u32) -> i32 {
    if CAM_TALK_A.contains(&msg_no) {
        CAM_TALK_A_IDX
    } else if CAM_TALK_B.contains(&msg_no) {
        CAM_TALK_B_IDX
    } else {
        CAM_DATA_STALL
    }
}

impl Zunari {
    pub fn pl_shop_out_action(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        if self.action == 0 {
            self.set_anm(6);
            self.push_anchor = actor.pos;
            self.push_anchor.x -= ssin(actor.angle[1]) * TRIG_AMPLITUDE;
            self.push_anchor.z -= scos(actor.angle[1]) * TRIG_AMPLITUDE;
            self.player_in_shop = false;
            self.action = 1;
        } else if self.action != -1 {
            let temp_78c = i32::try_from(self.shop_count).unwrap_or(0);
            let player = ctx.player.unwrap_or(Vec3::ZERO);
            let target = target_angle_y(actor.pos, player);
            if self.path_move(actor, ctx, usize::try_from(temp_78c).ok()) {
                let mut temp = Vec3::ZERO;
                let calc_pos = add_calc_pos(
                    &mut self.push_anchor,
                    actor.pos,
                    CHASE_SCALE,
                    CHASE_MAX_STEP,
                    CHASE_MIN_STEP,
                );
                actor.angle[1] =
                    add_calc_angle_s2(actor.angle[1], target, HEAD_TURN_SCALE, TURN_MAX_STEP);
                if calc_pos < CHASE_MIN_STEP {
                    add_calc_pos(
                        &mut self.push_offset,
                        temp,
                        CHASE_SCALE,
                        CHASE_MAX_STEP,
                        CHASE_MIN_STEP,
                    );
                    if self.push_offset.length() > CHASE_MIN_STEP {
                        self.set_anm(5);
                    } else {
                        self.set_anm(0);
                        set_action(self, actor, ctx, Proc::Wait);
                    }
                } else {
                    temp.x += STROLL_AMPLITUDE * ssin(actor.angle[1]);
                    temp.z += STROLL_AMPLITUDE * scos(actor.angle[1]);
                    add_calc_pos(
                        &mut self.push_offset,
                        temp,
                        CHASE_SCALE,
                        CHASE_MAX_STEP,
                        CHASE_MIN_STEP,
                    );
                }
            } else {
                self.push_anchor = actor.pos;
                self.push_anchor.x -= ssin(actor.angle[1]) * TRIG_AMPLITUDE;
                self.push_anchor.z -= scos(actor.angle[1]) * TRIG_AMPLITUDE;
            }
        }
    }

    pub fn get_demo_action(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        if self.action == 0 {
            self.phase = self.prev_phase as i8;
            if self.item_no != ITEM_NONE {
                ctx.give_item = Some((actor.pos, self.item_no));
            }
            self.action = 1;
        } else if self.action != -1 && self.get_demo_event >= 0 {
            let index = usize::try_from(self.get_demo_event as u16).unwrap_or(usize::MAX);
            if ctx.active.end_check(index) {
                self.mode = MODE_TALK;
                let item = self.item_no;
                if item == ITEM_TOWN_FLOWER {
                    self.forced_msg_no = MSG_FLOWER_AFTER;
                } else if item == ITEM_SAIL {
                    self.forced_msg_no = MSG_SAIL_AFTER;
                } else if item == ITEM_MAGIC_ARMOR {
                    self.forced_msg_no = MSG_ARMOR_AFTER;
                }
                self.item_no = ITEM_NONE;
                set_action(self, actor, ctx, Proc::Wait);
            }
        }
    }

    pub fn dummy_action(&mut self) {
        if self.action == 0 {
            self.action = 1;
        }
    }

    pub fn event_order(&mut self, ctx: &mut Ctx) {
        self.talk_condition = false;
        self.speak_ordered = false;
        if self.mode == MODE_SHOP_OUT {
            if self.shop_out_event < 0 {
                return;
            }
            let index = usize::try_from(self.shop_out_event as u16).unwrap_or(usize::MAX);
            if ctx.active.stage_of(index).is_none() {
                ctx.active.order(ctx.list, index, ctx.entity);
            }
        } else if self.mode == MODE_GIVE_DEMO {
            if self.get_demo_event < 0 {
                return;
            }
            let index = usize::try_from(self.get_demo_event as u16).unwrap_or(usize::MAX);
            if ctx.active.stage_of(index).is_none() {
                ctx.active.order(ctx.list, index, ctx.entity);
            }
        } else if matches!(self.mode, MODE_TALK | MODE_BUMPED | MODE_FACE_ON) {
            if self.mode != MODE_FACE_ON
                || self.shop_idx != NO_SHOP
                || ctx
                    .player
                    .is_some_and(check_roten_base_talk_area)
            {
                self.talk_condition = true;
            }
            if matches!(self.mode, MODE_TALK | MODE_BUMPED) {
                self.speak_ordered = true;
            }
        }
    }

    pub fn check_order(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        let demo_accepted = self.demo_accepted(ctx);
        if demo_accepted {
            if self.mode == MODE_SHOP_OUT {
                self.mode = MODE_NONE;
                set_action(self, actor, ctx, Proc::Event);
            } else if self.mode == MODE_GIVE_DEMO {
                self.mode = MODE_NONE;
                set_action(self, actor, ctx, Proc::GetDemo);
            }
        } else if ctx.talk_request {
            if matches!(self.mode, MODE_TALK | MODE_BUMPED | MODE_FACE_ON) {
                let temp = match ctx.player {
                    Some(player) => target_angle_y(actor.pos, player).wrapping_sub(self.home_angle),
                    None => 0,
                };
                if temp > FACE_MAX_HI || temp < FACE_MAX_LO {
                    self.forced_msg_no = MSG_STALL_BACK_2;
                } else if self.mode != MODE_BUMPED {
                    self.cam_action = 1;
                }
                self.mode = MODE_NONE;
                self.talking = true;
                self.talk_init();
            }
        } else {
            self.cam_action = 0;
        }
    }

    pub fn demo_accepted(&self, ctx: &Ctx) -> bool {
        let index = if self.mode == MODE_SHOP_OUT {
            self.shop_out_event
        } else if self.mode == MODE_GIVE_DEMO {
            self.get_demo_event
        } else {
            return false;
        };
        if index < 0 {
            return false;
        }
        ctx.active
            .start_check(usize::try_from(index as u16).unwrap_or(usize::MAX))
    }

    pub fn look_back(&mut self, actor: &Actor, ctx: &Ctx) {
        let mut target = LookTarget::None;
        let mut aim = Vec3::ZERO;
        let mut enable = true;
        match self.phase {
            PHASE_WAIT => {
                if self.attention {
                    aim = player_eye_pos(ctx.player.unwrap_or(Vec3::ZERO));
                    target = LookTarget::Player;
                }
                self.set_tex_anm(0);
            }
            PHASE_TALK => {
                if self.shop_items.is_some() {
                    if self.cam_action != 0 {
                        aim = player_eye_pos(ctx.player.unwrap_or(Vec3::ZERO));
                        enable = false;
                    } else if self.shop_selected < 0 {
                        aim = self.item_zoom_pos();
                        enable = false;
                    } else {
                        aim = self.eye_pos;
                        target = LookTarget::Item;
                    }
                }
            }
            _ => {}
        }
        if self.head_turning {
            self.look_at_max_vel = add_calc_angle_s2(
                self.look_at_max_vel,
                HIO_MAX_HEAD_TURN_VEL,
                HEAD_TURN_SCALE,
                HEAD_TURN_MAX,
            );
        } else {
            self.look_at_max_vel = 0;
        }
        self.look_target = target;
        self.look_aim = aim;
        self.look_enable = enable;
        self.look_base = actor.angle[1];
    }

    pub fn item_zoom_pos(&self) -> Vec3 {
        self.eye_pos + Vec3::new(0.0, ITEM_ZOOM_DIST, 0.0)
    }

    pub fn play_tex_pattern_anm(&mut self) {
        if self.btp_timer != 0 {
            self.btp_timer -= 1;
        }
        if self.btp_timer == 0 {
            if i32::from(self.btp_frame) >= BTP_FRAME_MAX {
                self.btp_frame = self.btp_frame.wrapping_sub(BTP_FRAME_MAX as u8);
                self.btp_timer = BTP_DELAY;
            } else {
                self.btp_frame = self.btp_frame.wrapping_add(1);
            }
        }
    }

    pub fn create_init(&mut self, actor: &mut Actor, parameters: u32, ctx: &mut Ctx) {
        self.home = actor.pos;
        self.home_angle = actor.angle[1];
        if self.variant == 0 {
            set_action(self, actor, ctx, Proc::Wait);
        }
        self.attn_base_pos = actor.pos;
        self.cam_state = 0;
        self.shop_cursor_moved = false;
        self.forced_msg_no = 0;
        self.shop_select_item_no = ITEM_NONE;
        self.shop_cam_data = CAM_DATA_STALL;
        self.cam_action = 0;
        self.create_shop_list(ctx.equipped);
        self.shop_idx = NO_SHOP;
        if self.shop_idx != NO_SHOP {
            self.shop_items = usize::try_from(self.shop_idx).ok();
            self.shop_cam_data = self.shop_idx + CAM_DATA_SHOP;
        } else {
            self.shop_items = None;
        }
        self.path_get(parameters);
        self.push_anchor = Vec3::ZERO;
        self.push_offset = Vec3::ZERO;
        self.player_in_shop = false;
        if ctx
            .player
            .is_some_and(|player| self.check_create_in_shop_player(actor, player))
        {
            self.mode = MODE_SHOP_OUT;
            self.shop_out_event = ctx
                .list
                .event_index(EVENT_SHOP_OUT)
                .map(|index| i16::try_from(index).unwrap_or(NO_INDEX))
                .unwrap_or(NO_INDEX);
            set_action(self, actor, ctx, Proc::Dummy);
        } else {
            self.mode = MODE_NONE;
        }
    }

    pub fn event_action(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        if self.action == 0 {
            self.phase = self.prev_phase as i8;
            self.mode = MODE_NONE;
            self.action = 1;
        } else if self.action != -1 {
            if self.shop_out_event >= 0 {
                let index = usize::try_from(self.shop_out_event as u16).unwrap_or(usize::MAX);
                if ctx.active.end_check(index) {
                    self.mode = MODE_NONE;
                    set_action(self, actor, ctx, Proc::Wait);
                }
            }
            self.look_back(actor, ctx);
        }
    }

    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        self.play_tex_pattern_anm();
        self.advance_anm();
        self.check_order(actor, ctx);
        run_proc(self, actor, ctx);
        self.event_order(ctx);
        self.look_back(actor, ctx);
        self.set_attention(actor);
    }
}

fn player_eye_pos(player: Vec3) -> Vec3 {
    Vec3::new(player.x, player.y + HIO_EYE_OFFSET, player.z)
}

pub const BTP_FRAME_MAX: i32 = 2;

pub fn is_zunari(name: &str) -> bool {
    name == ZUNARI_ACTOR
}

fn set_action(z: &mut Zunari, actor: &mut Actor, ctx: &mut Ctx, next: Proc) {
    if z.proc_state == next {
        return;
    }
    z.proc_state = next;
    z.action = 0;
    run_proc(z, actor, ctx);
}

fn run_proc(z: &mut Zunari, actor: &mut Actor, ctx: &mut Ctx) {
    match z.proc_state {
        Proc::Wait => z.wait_action(actor, ctx),
        Proc::PlShopOut => z.pl_shop_out_action(actor, ctx),
        Proc::GetDemo => z.get_demo_action(actor, ctx),
        Proc::Event => z.event_action(actor, ctx),
        Proc::Dummy => z.dummy_action(),
        Proc::None => {}
    }
}

#[derive(SystemParam)]
pub struct World<'w> {
    pub stories: ResMut<'w, StoryBits>,
    pub rupees: ResMut<'w, Rupees>,
    pub got: ResMut<'w, Got>,
    pub equipped: ResMut<'w, Equipped>,
    pub events: ResMut<'w, SimEvents>,
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub do_intent: Res<'w, DoIntent>,
    pub target: Res<'w, TargetIntent>,
}

#[derive(SystemParam)]
pub struct Msg<'w, 's> {
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub flags: ResMut<'w, MsgFlags>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub text_flags: Res<'w, TextFlags>,
    pub archive: Option<Res<'w, MsgArchives>>,
    pub players: Query<'w, 's, &'static Player>,
}

pub fn control(
    world: World,
    msg: Msg,
    mut actors: Query<(Entity, &mut Actor, &mut Zunari)>,
    mut commands: Commands,
) {
    let player = msg.players.iter().next().copied();
    let b_button = world.target.0;
    let do_pressed = world.do_intent.0;
    let clear_count = msg.text_flags.clear_count;
    for (entity, actor, zunari) in &mut actors {
        let actor = &mut *actor;
        let zunari = &mut *zunari;
        let pressed = do_pressed && !zunari.do_held;
        zunari.do_held = do_pressed;
        zunari.send_button = pressed;
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            flags: &mut msg.flags,
            one: &mut msg.one,
            two: &mut msg.two,
            clear_count,
            archive: msg.archive.as_deref().map(|archive| archive as &dyn MsgArchive),
        };
        let talk_request = pressed
            && player.is_some_and(|player| {
                let gap = player.pos - actor.pos;
                gap.x * gap.x + gap.z * gap.z < TALK_RANGE * TALK_RANGE
            })
            && matches!(zunari.mode, MODE_TALK | MODE_BUMPED | MODE_FACE_ON);
        let mut ctx = Ctx {
            entity,
            player: player.map(|link| link.pos),
            send_button: zunari.send_button,
            talk_request,
            b_button,
            events: &mut world.events,
            stories: &mut world.stories,
            rupees: &mut world.rupees,
            got: &mut world.got,
            equipped: &mut world.equipped,
            list: &world.list,
            active: &mut world.active,
            text: &mut text,
            give_item: None,
        };
        if !zunari.entered {
            zunari.entered = true;
            zunari.home = actor.pos;
            zunari.home_angle = actor.angle[1];
            zunari.get_demo_event = ctx
                .list
                .event_index(EVENT_GET_DEMO)
                .and_then(|index| i16::try_from(index).ok())
                .unwrap_or(NO_INDEX);
            zunari.staff_idx = ctx.list.staff_of(STAFF_NAME).unwrap_or(NO_STAFF);
            let parameters = actor.parameters;
            zunari.create_init(actor, parameters, &mut ctx);
        } else {
            zunari.execute(actor, &mut ctx);
        }
        if let Some((pos, item)) = ctx.give_item {
            zunari.exec_item_get(item, ctx.got, ctx.equipped);
            ctx.events.list.push(SimEvent::Collect);
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
        }
    }
}
