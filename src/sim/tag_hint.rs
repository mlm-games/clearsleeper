use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::carry::{Carry, Carryable};
use super::collision::Collision;
use super::door::StayRoom;
use super::event::{ActiveEvent, EventList};
use super::event_manager::EventManager;
use super::events::{SimEvent, SimEvents};
use super::flags::{Collect, EventFlags, MemBits, OrderFlags, Switches};
use super::intent::DoIntent;
use super::inventory::{self, Equipped};
use super::movebox::Movebox;
use super::obj_doguu::Text;
use super::player::Player;
use super::walk;
use crate::assets::MsgArchives;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{FPC_M_ERROR_PROCESS_ID, MsgGameInfo, MsgProcs};
use crate::text::{STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS};

pub const TAG_HINT_ACTORS: [&str; 2] = ["TagHt", "TagHt2"];

pub fn is_tag_hint(name: &str) -> bool {
    TAG_HINT_ACTORS.contains(&name)
}

pub const TYPE_SHIFT: u32 = 0; // <- d_a_tag_hint.cpp:37
pub const TYPE_MASK: u32 = 0x3F; // <- d_a_tag_hint.cpp:37
pub const TYPE2_SHIFT: u32 = 6; // <- d_a_tag_hint.cpp:42
pub const TYPE2_MASK: u32 = 0x3; // <- d_a_tag_hint.cpp:42
pub const SWBIT_SHIFT: u32 = 8; // <- d_a_tag_hint.cpp:27
pub const SWBIT2_SHIFT: u32 = 16; // <- d_a_tag_hint.cpp:32
pub const EVENT_NO_SHIFT: u32 = 24; // <- d_a_tag_hint.cpp:22
pub const FIELD_MASK: u32 = 0xFF; // <- d_a_tag_hint.cpp:22

pub const NO_SWITCH: u8 = 0xFF; // <- d_a_tag_hint.cpp:68
pub const NO_EVENT_FLAG: u16 = 0xFFFF; // <- d_a_tag_hint.cpp:78
pub const MESSAGE_STATUS_NONE: u8 = 0xFF; // <- d_a_tag_hint.cpp:497

pub const FLAG_00000001: u16 = 0x0001; // <- d_a_tag_hint.cpp:543

pub const EVENT_0401: u32 = 0x0401; // <- d_save_event_flag.inc:38
pub const EVENT_0404: u32 = 0x0404; // <- d_save_event_flag.inc:36
pub const EVENT_1801: u32 = 0x1801; // <- d_save_event_flag.inc:188
pub const EVENT_2A10: u32 = 0x2A10; // <- d_save_event_flag.inc:321
pub const EVENT_3C01: u32 = 0x3C01; // <- d_save_event_flag.inc:464
pub const EVENT_3F20: u32 = 0x3F20; // <- d_save_event_flag.inc:482
pub const EVENT_4020: u32 = 0x4020; // <- d_save_event_flag.inc:488
pub const EVENT_A507: u32 = 0xA507; // <- d_save_event_flag.inc:542
pub const EVENT_REG_PAGE: u8 = 0xA5; // <- d_save.cpp:1209 (UNK_A507 >> 8)
pub const EVENT_REG_MASK: u8 = 0x07; // <- d_save.cpp:1209 (UNK_A507 & 0xFF)

pub const TMP_0304: u32 = 0x0304; // <- d_save_event_tmp_flag.inc:30
pub const TMP_0308: u32 = 0x0308; // <- d_save_event_tmp_flag.inc:29
pub const TMP_BYTES: usize = 0x100; // <- d_save.h:722

pub const ITEM_BOMB_10: u8 = 0x0C; // <- d_item_data.h:19
pub const ITEM_POWER_BRACELETS: u8 = 0x28; // <- d_item_data.h:47

pub const SE_ITM_OMAMORI_BLINK: u32 = 0x0852; // <- JAZelAudio_SE.h:91
pub const SE_ITM_OMAMORI_ST_TALK: u32 = 0x0853; // <- JAZelAudio_SE.h:92
pub const SE_ITM_OMAMORI_TETLA: u32 = 0x0854; // <- JAZelAudio_SE.h:93
pub const SE_ITM_OMAMORI_ED_TALK: u32 = 0x0855; // <- JAZelAudio_SE.h:94
pub const VOICE_SURPRISE_INDEX: u32 = 0x1C; // <- d_a_tag_hint.cpp:560

pub const STAFF_TAG_HINT: &str = "TAGHINT"; // <- d_a_tag_hint.cpp:464
pub const EVENT_HIRL_HINT: &str = "HIRL_HINT"; // <- d_a_tag_hint.cpp:324
pub const EVENT_DEFAULT_HINT2: &str = "DEFAULT_HINT2"; // <- d_a_tag_hint.cpp:327
pub const EVENT_DEFAULT_HINT: &str = "DEFAULT_HINT"; // <- d_a_tag_hint.cpp:330

pub const ACTION_TABLE: [&str; 5] = [
    // <- d_a_tag_hint.cpp:533-539
    "WAIT",
    "MESSAGE",
    "MESSAGE2",
    "SURPRIS",
    "MESSAGE3",
];
pub const ACT_WAIT: i32 = 0; // <- d_a_tag_hint.cpp:534
pub const ACT_MESSAGE: i32 = 1; // <- d_a_tag_hint.cpp:535
pub const ACT_MESSAGE2: i32 = 2; // <- d_a_tag_hint.cpp:536
pub const ACT_SURPRIS: i32 = 3; // <- d_a_tag_hint.cpp:537
pub const ACT_MESSAGE3: i32 = 4; // <- d_a_tag_hint.cpp:538

pub const MSG_HINT_CONTINUED: u32 = 0x454; // <- d_a_tag_hint.cpp:599

pub const PROC_WAIT: u8 = 0; // <- d_a_tag_hint.cpp:690
pub const PROC_LIGHT: u8 = 1; // <- d_a_tag_hint.cpp:694
pub const PROC_ARRIVAL: u8 = 2; // <- d_a_tag_hint.cpp:698
pub const PROC_HUNT: u8 = 3; // <- d_a_tag_hint.cpp:702
pub const PROC_EVENT: u8 = 4; // <- d_a_tag_hint.cpp:706

pub const TALK_PHASE_INIT: i8 = 0; // <- d_a_tag_hint.cpp:492
pub const TALK_PHASE_START: i8 = 1; // <- d_a_tag_hint.cpp:501
pub const TALK_PHASE_OPEN: i8 = 2; // <- d_a_tag_hint.cpp:511
pub const TALK_PHASE_DONE: i8 = -1; // <- d_a_tag_hint.cpp:522

pub const PRIORITY_SPEAK: i32 = 0x1EA; // <- f_pc_draw_priority.h:515
pub const PRIORITY_DEFAULT: i32 = 0x1FF; // <- f_pc_draw_priority.h:517
pub const EVENT_IDX_SPEAK: usize = 0x1EA; // <- f_op_actor_mng.cpp:678

pub const TIMER_TYPE_1: i16 = 600; // <- d_a_tag_hint.cpp:90
pub const TIMER_TYPE_4: i16 = 0xE10; // <- d_a_tag_hint.cpp:93
pub const TIMER_TYPE_8: i16 = 0xE10; // <- d_a_tag_hint.cpp:96
pub const TIMER_TYPE_9: i16 = 600; // <- d_a_tag_hint.cpp:102
pub const TIMER_TYPE_12: i16 = 0x708; // <- d_a_tag_hint.cpp:110
pub const TIMER_TYPE_14: i16 = 0x708; // <- d_a_tag_hint.cpp:113
pub const TIMER_TYPE_15: i16 = 900; // <- d_a_tag_hint.cpp:116

pub const RANGE_SCALE: f32 = 100.0; // <- d_a_tag_hint.cpp:742-743
pub const SEARCH_STEP: f32 = 120.0; // <- d_a_tag_hint.cpp:416
pub const DARK_STEP: f32 = 0.05; // <- d_a_tag_hint.cpp:584
pub const DARK_MIN: f32 = 0.15; // <- d_a_tag_hint.cpp:582
pub const DARK_LOW: f32 = 0.2; // <- d_a_tag_hint.cpp:583
pub const DARK_MAX: f32 = 0.95; // <- d_a_tag_hint.cpp:664
pub const DARK_FULL: f32 = 1.0; // <- d_a_tag_hint.cpp:667
pub const DARK_SCALE: f32 = 1.1764705; // <- d_a_tag_hint.cpp:433
pub const DARK_BG_SCALE: f32 = 0.85; // <- d_a_tag_hint.cpp:438
pub const DARK_FOG_NEAR: f32 = 300.0; // <- d_a_tag_hint.cpp:436
pub const DARK_FOG_FAR: f32 = 2000.0; // <- d_a_tag_hint.cpp:436
pub const DARK_FOG_MID: f32 = 3000.0; // <- d_a_tag_hint.cpp:439
pub const DARK_FOG_MID_BASE: f32 = 500.0; // <- d_a_tag_hint.cpp:439

pub const LIGHT_LIFT: f32 = 5.66; // <- d_a_tag_hint.cpp:355
pub const LIGHT_X_COS: f32 = 10.25; // <- d_a_tag_hint.cpp:356
pub const LIGHT_X_SIN: f32 = 11.13; // <- d_a_tag_hint.cpp:356
pub const LIGHT_POWER: f32 = 139.7; // <- d_a_tag_hint.cpp:340
pub const LIGHT_POWER_E: f32 = 270.7; // <- d_a_tag_hint.cpp:345
pub const LIGHT_FLUCTUATION: f32 = 93.8; // <- d_a_tag_hint.cpp:341
pub const LIGHT_COLOR_P: [u16; 3] = [0x5C, 0xAE, 0xFF]; // <- d_a_tag_hint.cpp:337-339
pub const LIGHT_COLOR_E: [u16; 3] = [0xBA, 0xFF, 0xFF]; // <- d_a_tag_hint.cpp:342-344

pub const SEARCH_ANGLE_TABLE: [i16; 8] = [0x00, 0x2000, -0x2000, 0x4000, -0x4000, 0x6000, -0x6000, 0x7FFF]; // <- d_a_tag_hint.cpp:399
pub const SEARCH_ANGLE_TABLE_GRAB: [i16; 8] =
    [0x4000, -0x4000, 0x6000, -0x6000, 0x7FFF, -0x2000, 0x2000, 0x00]; // <- d_a_tag_hint.cpp:400
pub const SEARCH_ANGLE_COUNT: usize = 8; // <- d_a_tag_hint.cpp:407

pub const CULL_MIN: [f32; 3] = [-150.0, 0.0, -150.0]; // <- f_op_actor_mng.cpp:563
pub const CULL_MAX: [f32; 3] = [150.0, 300.0, 150.0]; // <- f_op_actor_mng.cpp:563

pub const EVT_CND_CANTALK: u16 = 0x0001; // <- f_op_actor.h:145

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagHintTmp {
    pub bytes: [u8; TMP_BYTES],
}

impl Default for TagHintTmp {
    fn default() -> Self {
        Self {
            bytes: [0_u8; TMP_BYTES],
        }
    }
}

impl TagHintTmp {
    fn slot(&self, flag: u32) -> Option<(usize, u8)> {
        let byte = (flag >> 8) as usize;
        if byte >= TMP_BYTES {
            return None;
        }
        Some((byte, (flag & 0xFF) as u8))
    }

    pub fn test(&self, flag: u32) -> bool {
        match self.slot(flag) {
            Some((byte, mask)) => self.bytes[byte] & mask != 0,
            None => false,
        }
    }

    pub fn set(&mut self, flag: u32) {
        if let Some((byte, mask)) = self.slot(flag) {
            self.bytes[byte] |= mask;
        }
    }

    pub fn clear(&mut self, flag: u32) {
        if let Some((byte, mask)) = self.slot(flag) {
            self.bytes[byte] &= !mask;
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct TagHintLight {
    pub point_pos: Vec3,
    pub point_r: u16,
    pub point_g: u16,
    pub point_b: u16,
    pub point_power: f32,
    pub point_fluctuation: f32,
    pub point_live: bool,
    pub eff_pos: Vec3,
    pub eff_r: u16,
    pub eff_g: u16,
    pub eff_b: u16,
    pub eff_power: f32,
    pub eff_fluctuation: f32,
    pub eff_live: bool,
}

impl Default for TagHintLight {
    fn default() -> Self {
        Self {
            point_pos: Vec3::ZERO,
            point_r: 0,
            point_g: 0,
            point_b: 0,
            point_power: 0.0,
            point_fluctuation: 0.0,
            point_live: false,
            eff_pos: Vec3::ZERO,
            eff_r: 0,
            eff_g: 0,
            eff_b: 0,
            eff_power: 0.0,
            eff_fluctuation: 0.0,
            eff_live: false,
        }
    }
}

impl TagHintLight {
    fn init(&mut self) {
        self.point_r = LIGHT_COLOR_P[0];
        self.point_g = LIGHT_COLOR_P[1];
        self.point_b = LIGHT_COLOR_P[2];
        self.point_power = LIGHT_POWER;
        self.point_fluctuation = LIGHT_FLUCTUATION;
        self.eff_r = LIGHT_COLOR_E[0];
        self.eff_g = LIGHT_COLOR_E[1];
        self.eff_b = LIGHT_COLOR_E[2];
        self.eff_power = LIGHT_POWER_E;
        self.eff_fluctuation = LIGHT_FLUCTUATION;
    }

    fn cut(&mut self) {
        self.point_live = false;
        self.eff_live = false;
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct TagHintFog {
    pub vrbox_ratio: f32,
    pub fogcol_ratio: f32,
    pub bgcol_ratio: f32,
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_alpha: f32,
}

impl Default for TagHintFog {
    fn default() -> Self {
        Self {
            vrbox_ratio: 1.0,
            fogcol_ratio: 1.0,
            bgcol_ratio: 1.0,
            fog_start: DARK_FOG_NEAR,
            fog_end: DARK_FOG_FAR,
            fog_alpha: 1.0,
        }
    }
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TagHintMsg {
    pub id: u32,
    pub found: bool,
}

#[derive(Resource, Debug, Default)]
pub struct TagHintRequest {
    pub z_hint: Option<Entity>,
    pub priority: i32,
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TagHintPlayerRoom {
    pub goat: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct TagHint {
    pub event_no: u8,
    pub swbit: u8,
    pub swbit2: u8,
    pub kind: u8,
    pub kind2: u8,
    pub message: u16,
    pub event_flag: u16,
    pub cur_proc: u8,
    pub flags: u16,
    pub dark: f32,
    pub timer: i16,
    pub event_id: i16,
    pub grab: bool,
    pub radius2: f32,
    pub height: f32,
    pub msg_no: u32,
    pub staff: i32,
    pub talk_phase: i8,
    pub talk_msg: u32,
    pub entered: bool,
}

impl TagHint {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            event_no: ((parameters >> EVENT_NO_SHIFT) & FIELD_MASK) as u8,
            swbit: ((parameters >> SWBIT_SHIFT) & FIELD_MASK) as u8,
            swbit2: ((parameters >> SWBIT2_SHIFT) & FIELD_MASK) as u8,
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            kind2: ((parameters >> TYPE2_SHIFT) & TYPE2_MASK) as u8,
            message: 0,
            event_flag: NO_EVENT_FLAG,
            cur_proc: PROC_WAIT,
            flags: 0,
            dark: 0.0,
            timer: 0,
            event_id: 0,
            grab: false,
            radius2: 0.0,
            height: 0.0,
            msg_no: 0,
            staff: -1,
            talk_phase: TALK_PHASE_INIT,
            talk_msg: 0,
            entered: false,
        }
    }

    pub fn chk_flag(&self, flg: u16) -> bool {
        self.flags & flg == flg
    }

    pub fn set_flag(&mut self, flg: u16) {
        self.flags |= flg;
    }

    pub fn set_actio(&mut self, action: u8) {
        self.cur_proc = action;
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        // fopAc_CULLBOX_6_e  <- d_a_tag_hint.cpp:795 -> f_op_actor_mng.cpp:563
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn get_event_no(&self) -> u8 {
        self.event_no
    }

    pub fn get_swbit(&self) -> u8 {
        self.swbit
    }

    pub fn get_swbit2(&self) -> u8 {
        self.swbit2
    }

    pub fn get_type(&self) -> u8 {
        self.kind
    }

    pub fn get_type2(&self) -> u8 {
        self.kind2
    }

    pub fn get_message(&self) -> u16 {
        self.message
    }

    pub fn get_event_flag(&self) -> u16 {
        self.event_flag
    }

    pub fn get_msg(&self) -> u32 {
        self.msg_no
    }

    pub fn next_msg_status(&mut self, _msg_no: &mut u32) -> u8 {
        STATUS_MSG_ENDS
    }

    pub fn talk_init(&mut self) {
        self.talk_phase = TALK_PHASE_INIT;
    }

    pub fn get_priority(&self) -> i32 {
        match self.get_type() {
            2 => PRIORITY_SPEAK,
            _ => PRIORITY_DEFAULT,
        }
    }

    pub fn range_check_local(&self, pos: Vec3, current: Vec3) -> bool {
        let mut sp20 = pos - current;
        if sp20.y < 0.0 {
            sp20.y = -sp20.y;
        }
        sp20.x * sp20.x + sp20.z * sp20.z < self.radius2 && sp20.y <= self.height
    }

    pub fn dark_proc(&mut self, fog: &mut TagHintFog) {
        fog.vrbox_ratio = self.dark;
        fog.fogcol_ratio = self.dark;
        let f_var1 = (self.dark - DARK_MIN) * DARK_SCALE;
        if f_var1 > 0.5 {
            fog.fog_start = DARK_FOG_NEAR;
            fog.fog_end = DARK_FOG_FAR;
            fog.fog_alpha = 2.0 - f_var1 * 2.0;
        } else {
            fog.bgcol_ratio = f_var1 * DARK_BG_SCALE * 2.0 + DARK_MIN;
            fog.fog_start = DARK_FOG_NEAR;
            fog.fog_end = f_var1 * DARK_FOG_MID + DARK_FOG_MID_BASE;
            fog.fog_alpha = 1.0;
        }
    }

    pub fn set_light_pos(&self, right_hand: Vec3, shape_angle: i16, light: &mut TagHintLight) {
        let (sin, cos) = walk::from_s16(shape_angle).sin_cos();
        let mut pos = right_hand;
        pos.y += LIGHT_LIFT;
        pos.x += cos * LIGHT_X_COS + sin * LIGHT_X_SIN;
        pos.z += -sin * LIGHT_X_COS + cos * LIGHT_X_SIN;
        light.point_pos = pos;
        light.eff_pos = pos;
    }

    pub fn make_light(&self, right_hand: Vec3, shape_angle: i16, light: &mut TagHintLight) {
        light.init();
        self.set_light_pos(right_hand, shape_angle, light);
        light.point_live = true;
        light.eff_live = true;
    }

    pub fn delete_light(&self, light: &mut TagHintLight) {
        light.cut();
    }

    pub fn set_player_angle(&self, attn: Vec3, shape_angle: i16, ctx: &mut Ctx<'_>) {
        for i in 0..SEARCH_ANGLE_COUNT {
            let u_var3 = if !self.grab {
                shape_angle.wrapping_add(SEARCH_ANGLE_TABLE[i])
            } else {
                shape_angle.wrapping_add(SEARCH_ANGLE_TABLE_GRAB[i])
            };
            let rad = walk::from_s16(u_var3);
            let sp08 = Vec3::new(
                attn.x + rad.sin() * SEARCH_STEP,
                attn.y,
                attn.z + rad.cos() * SEARCH_STEP,
            );
            if !ctx.line_crossed(attn, sp08) {
                ctx.set_player_angle(u_var3);
                return;
            }
        }
        ctx.set_player_angle(shape_angle);
    }

    pub fn delete(&mut self, ctx: &mut Ctx<'_>) {
        match self.get_type() {
            9 => ctx.tmp.clear(TMP_0308),
            17 => ctx.flags.clear(EVENT_4020),
            _ => {}
        }
    }
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub pos: Vec3,
    pub room: i8,
    pub room_player: bool,
    pub stay_room: i8,
    pub boxes: &'a [(Vec3, Vec3)],
    pub flags: &'a mut EventFlags,
    pub switches: &'a mut Switches,
    pub tmp: &'a mut TagHintTmp,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub manager: &'a mut EventManager,
    pub events: &'a mut SimEvents,
    pub light: &'a mut TagHintLight,
    pub fog: &'a mut TagHintFog,
    pub held: &'a mut TagHintMsg,
    pub request: &'a mut TagHintRequest,
    pub order: &'a mut OrderFlags,
    pub equip: &'a Equipped,
    pub collect: &'a Collect,
    pub boss_enemy: bool,
    pub carrying: bool,
    pub carrying_barrel: bool,
    pub carrying_wear: bool,
    pub grab_actor: bool,
    pub player: Option<&'a mut Player>,
    pub player_yaw: i16,
    pub collision: Option<&'a Collision>,
    pub talk_command: bool,
    pub text: Text<'a>,
}

impl Ctx<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn line_crossed(&self, from: Vec3, to: Vec3) -> bool {
        match self.collision {
            Some(collision) => collision.slide(from, to) != to,
            None => false,
        }
    }

    fn set_player_angle(&mut self, angle: i16) {
        if let Some(player) = self.player.as_deref_mut() {
            player.yaw = walk::from_s16(angle);
        }
    }

    fn event_reset(&mut self) {
        self.manager.exception.init();
    }

    fn order_event(&mut self, index: i32) {
        let Ok(index) = usize::try_from(index) else {
            return;
        };
        if index == 0 {
            return;
        }
        if self.active.stage_of(index).is_none() {
            let entity = self.entity;
            self.active.order(self.list, index, entity);
        }
    }

    fn equipped_item(&self, slot: u8) -> u8 {
        self.equip.item(usize::from(slot))
    }

    fn checked_item(&self, item: u8) -> bool {
        inventory::check_get_item(self.equip, self.collect, item)
    }

    fn event_reg(&self) -> u8 {
        let byte = usize::from(EVENT_REG_PAGE);
        match self.flags.words().get(byte) {
            Some(value) => value & EVENT_REG_MASK,
            None => 0,
        }
    }

    fn event_run_check(&self) -> bool {
        self.text.info.event_run_check()
    }

    fn attention_pos(&self) -> Vec3 {
        match self.player.as_ref() {
            Some(player) => player.pos,
            None => Vec3::ZERO,
        }
    }

    fn event_index(&mut self, name: &str, event_no: u8) -> i16 {
        let index = self.manager.get_event_idx(Some(name), event_no);
        i16::try_from(index).unwrap_or(0)
    }

    fn my_staff_id(&mut self, name: &str) -> i32 {
        self.manager.get_my_staff_id(name, None, 0)
    }

    fn act_index(&mut self, staff: i32) -> i32 {
        self.manager.get_my_act_idx(staff, &ACTION_TABLE, false, 0)
    }

    fn is_advance(&self, staff: i32) -> bool {
        self.manager.get_is_advance(staff)
    }

    fn cut_end(&mut self, staff: i32) {
        self.manager.cut_end(staff);
    }
}

impl TagHint {
    fn range_check(&self, ctx: &Ctx<'_>) -> bool {
        let pos = ctx.player.as_ref().map(|player| player.pos);
        match pos {
            Some(pos) => self.range_check_local(pos, ctx.pos),
            None => false,
        }
    }

    fn arrival_terms(&mut self, ctx: &mut Ctx<'_>) -> i32 {
        let u_var1 = self.get_swbit2() as i32;
        let u_var4 = self.get_event_flag();
        let u_var2 = self.get_type();

        match u_var2 {
            8 | 10 | 16 => {}
            _ => {
                if u_var1 != NO_SWITCH as i32 && !ctx.switches.test(u_var1 as u8) {
                    return 0;
                }
            }
        }

        match u_var2 {
            2 | 8 | 10 => {
                if u_var4 != NO_EVENT_FLAG && !ctx.flags.test(u_var4 as u32) {
                    return 0;
                }
            }
            0x11 => {
                if !ctx.flags.test(EVENT_4020) {
                    return 0;
                }
            }
            _ => {}
        }

        match u_var2 {
            1 => {
                self.timer = TIMER_TYPE_1;
            }
            4 => {
                self.timer = TIMER_TYPE_4;
            }
            8 => {
                self.timer = TIMER_TYPE_8;
            }
            9 => {
                if !ctx.tmp.test(TMP_0308) {
                    return 0;
                }
                self.timer = TIMER_TYPE_9;
            }
            0xb => {
                if !ctx.room_player || ctx.stay_room != ctx.room {
                    return 0;
                }
            }
            0xc => {
                self.timer = TIMER_TYPE_12;
            }
            0xe => {
                self.timer = TIMER_TYPE_14;
            }
            0xf => {
                self.timer = TIMER_TYPE_15;
            }
            _ => {
                self.timer = 0;
            }
        }
        1
    }

    fn wait_terms(&mut self, ctx: &mut Ctx<'_>) -> i32 {
        let u_var1 = self.get_swbit() as i32;
        let u_var2 = self.get_swbit2() as i32;
        let u_var5 = self.get_event_flag();
        let u_var3 = self.get_type();

        if u_var1 != NO_SWITCH as i32 && ctx.switches.test(u_var1 as u8) {
            return 1;
        }

        match u_var3 {
            0 | 3 | 4 | 5 => {
                if u_var5 != NO_EVENT_FLAG && ctx.flags.test(u_var5 as u32) {
                    return 1;
                }
            }
            _ => {}
        }

        match u_var3 {
            1 => {
                if self.move_box_check(ctx) {
                    if u_var1 != NO_SWITCH as i32 {
                        ctx.switches.set(u_var1 as u8);
                    }
                    ctx.flags.set(EVENT_0404);
                    return 1;
                }
            }
            6 => {
                if ctx.flags.test(EVENT_2A10) {
                    return 1;
                }
            }
            7 => {
                if u_var2 != NO_SWITCH as i32 && !ctx.switches.test(u_var2 as u8) {
                    return 1;
                }
            }
            9 => {
                if ctx.tmp.test(TMP_0304) {
                    return 1;
                }
                if ctx.flags.test(EVENT_1801) {
                    return 1;
                }
            }
            8 | 10 | 0x10 => {
                if u_var2 != NO_SWITCH as i32 && ctx.switches.test(u_var2 as u8) {
                    return 1;
                }
            }
            0xb => {
                if ctx.equipped_item(ITEM_BOMB_10) != inventory::NO_ITEM {
                    return 1;
                }
            }
            0xd => {
                if ctx.boss_enemy {
                    return 1;
                }
            }
            0xe => {
                if ctx.flags.test(EVENT_3C01) {
                    return 1;
                }
            }
            0xf => {
                if ctx.flags.test(EVENT_3F20) {
                    return 1;
                }
            }
            0x11 => {
                if ctx.checked_item(ITEM_POWER_BRACELETS) {
                    return 1;
                }
            }
            _ => {}
        }
        0
    }

    fn move_box_check(&self, ctx: &Ctx<'_>) -> bool {
        for (pos, home_pos) in ctx.boxes.iter() {
            if self.range_check_local(*home_pos, ctx.pos) {
                let sp14 = *pos - *home_pos;
                if sp14.x * sp14.x + sp14.z * sp14.z > 0.0 {
                    return true;
                }
            }
        }
        false
    }

    fn other_check(&mut self, ctx: &mut Ctx<'_>) -> i32 {
        let kind = self.get_type();

        match kind {
            4 | 8 | 9 | 12 => {
                if self.timer > 0 {
                    self.timer -= 1;
                    return 0;
                }
            }
            _ => {}
        }

        if !self.range_check(ctx) {
            return 0;
        }

        match kind {
            0 => {
                if ctx.carrying_barrel || ctx.carrying_wear {
                    return 0;
                }
            }
            1 => {
                ctx.flags.set(EVENT_0404);
                if !ctx.flags.test(EVENT_0401) {
                    return 0;
                }
            }
            6 => {
                if ctx.event_reg() < 3 {
                    return 0;
                }
            }
            14 | 15 => {
                if ctx.event_run_check() {
                    return 0;
                }
            }
            _ => {}
        }

        if self.timer > 0 {
            self.timer -= 1;
            0
        } else {
            1
        }
    }

    fn make_event_id(&mut self, ctx: &mut Ctx<'_>) {
        let u_var2 = self.get_type();
        if u_var2 == 10 {
            self.event_id = ctx.event_index(EVENT_HIRL_HINT, self.get_event_no());
            self.grab = false;
        } else if ctx.grab_actor {
            self.event_id = ctx.event_index(EVENT_DEFAULT_HINT2, self.get_event_no());
            self.grab = true;
        } else {
            self.event_id = ctx.event_index(EVENT_DEFAULT_HINT, self.get_event_no());
            self.grab = false;
        }
    }

    fn start_proc(&mut self, ctx: &mut Ctx<'_>) {
        let u_var2 = self.get_swbit() as i32;
        let kind = self.get_type();

        match kind {
            0 => {
                ctx.flags.set(EVENT_0401);
            }
            6 => {}
            12 => {}
            _ => {
                if u_var2 != NO_SWITCH as i32 {
                    ctx.switches.set(u_var2 as u8);
                }
            }
        }

        self.staff = ctx.my_staff_id(STAFF_TAG_HINT);
        self.dark = DARK_FULL;

        if kind != 10 && self.get_type2() & 1 == 0 {
            ctx.se(SE_ITM_OMAMORI_ST_TALK);
            let attn = ctx.attention_pos();
            let yaw = ctx.player_yaw;
            self.set_player_angle(attn, yaw, ctx);
        }

        let u_var5 = self.get_event_flag();
        match kind {
            5 => {
                ctx.flags.set(u_var5 as u32);
            }
            _ => {}
        }
    }

    fn talk(&mut self, ctx: &mut Ctx<'_>) -> u8 {
        let mut status = MESSAGE_STATUS_NONE;
        if self.talk_phase == TALK_PHASE_INIT {
            ctx.held.id = FPC_M_ERROR_PROCESS_ID;
            self.talk_msg = self.get_msg();
            self.talk_phase = TALK_PHASE_START;
        } else if self.talk_phase != TALK_PHASE_DONE {
            if ctx.held.id == FPC_M_ERROR_PROCESS_ID {
                ctx.held.id = ctx.text.message_set_actor(self.talk_msg, ctx.pos);
            } else {
                match self.talk_phase {
                    TALK_PHASE_START => {
                        if ctx.text.alive(ctx.held.id) {
                            self.talk_phase = TALK_PHASE_OPEN;
                        }
                    }
                    TALK_PHASE_OPEN => {
                        status = ctx.text.status(ctx.held.id);
                        if status == STATUS_MSG_DISPLAYED {
                            let mut msg_no = self.talk_msg;
                            let next = self.next_msg_status(&mut msg_no);
                            ctx.text.set_status(ctx.held.id, next);
                            if next == STATUS_MSG_CONTINUES {
                                self.talk_msg = msg_no;
                                ctx.text.message_set(msg_no);
                            }
                        } else if status == STATUS_BOX_CLOSED {
                            ctx.text.set_status(ctx.held.id, STATUS_MSG_DESTROYED);
                            self.talk_phase = TALK_PHASE_DONE;
                        }
                    }
                    _ => {}
                }
            }
        }
        status
    }

    fn action_event(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let i_var2 = ctx.act_index(self.staff);
        if ctx.is_advance(self.staff) {
            self.set_flag(FLAG_00000001);

            match i_var2 {
                ACT_MESSAGE => {
                    self.talk_init();
                    let attn = ctx.attention_pos();
                    let yaw = ctx.player_yaw;
                    self.make_light(attn, yaw, ctx.light);
                    ctx.se(SE_ITM_OMAMORI_TETLA);
                }
                ACT_MESSAGE2 => {
                    self.talk_init();
                    ctx.se(SE_ITM_OMAMORI_ST_TALK);
                    ctx.se(SE_ITM_OMAMORI_TETLA);
                }
                ACT_SURPRIS => {
                    ctx.se(SE_ITM_OMAMORI_BLINK);
                    ctx.se(VOICE_SURPRISE_INDEX);
                }
                ACT_MESSAGE3 => {
                    self.talk_init();
                    ctx.se(SE_ITM_OMAMORI_TETLA);
                }
                _ => {}
            }
        }

        match i_var2 {
            ACT_MESSAGE => {
                let s_var3 = self.talk(ctx);
                if s_var3 == STATUS_BOX_CLOSED {
                    ctx.se(SE_ITM_OMAMORI_ED_TALK);
                    ctx.cut_end(self.staff);
                    self.set_actio(PROC_LIGHT);
                    self.delete_light(ctx.light);
                    ctx.event_reset();
                }
                if self.dark > DARK_MIN && i_var2 == ACT_MESSAGE {
                    if self.dark > DARK_LOW {
                        self.dark -= DARK_STEP;
                    } else {
                        self.dark = DARK_MIN;
                    }
                }
                self.dark_proc(&mut *ctx.fog);
            }
            ACT_MESSAGE2 => {
                let s_var3 = self.talk(ctx);
                if self.chk_flag(FLAG_00000001) {
                    if s_var3 != STATUS_BOX_CLOSED {
                        return true;
                    }
                    ctx.se(SE_ITM_OMAMORI_ED_TALK);
                    self.msg_no = MSG_HINT_CONTINUED;
                    ctx.cut_end(self.staff);
                } else {
                    ctx.cut_end(self.staff);
                }
            }
            ACT_MESSAGE3 => {
                let s_var3 = self.talk(ctx);
                if s_var3 != STATUS_BOX_CLOSED {
                    return true;
                }
                self.set_actio(PROC_WAIT);
                ctx.event_reset();
            }
            _ => {
                ctx.cut_end(self.staff);
            }
        }

        true
    }

    fn action_hunt(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if ctx.talk_command {
            self.set_actio(PROC_EVENT);
            self.start_proc(ctx);
            self.msg_no = u32::from(self.get_message());
        } else if self.wait_terms(ctx) != 0 {
            self.set_actio(PROC_WAIT);
        } else if self.other_check(ctx) != 0 {
            self.make_event_id(ctx);
            if self.get_type2() & 1 != 0 {
                ctx.order_event(EVENT_IDX_SPEAK as i32);
            } else {
                ctx.request.z_hint = Some(ctx.entity);
                ctx.request.priority = self.get_priority();
            }
            ctx.order.set(EVT_CND_CANTALK);
        }
        true
    }

    fn action_arrival(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let _ = self.get_swbit();
        if self.wait_terms(ctx) != 0 {
            self.set_actio(PROC_WAIT);
        } else if self.arrival_terms(ctx) != 0 {
            self.make_event_id(ctx);
            self.set_actio(PROC_HUNT);
            self.action_hunt(ctx);
        }
        true
    }

    fn action_light(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.dark < DARK_MAX {
            self.dark += DARK_STEP;
        } else {
            self.dark = DARK_FULL;
            self.set_actio(PROC_WAIT);
        }
        self.dark_proc(&mut *ctx.fog);
        true
    }

    fn action_wait(&mut self, _ctx: &mut Ctx<'_>) -> bool {
        true
    }

    fn execute(&mut self, ctx: &mut Ctx<'_>) -> bool {
        match self.cur_proc {
            PROC_WAIT => {
                self.action_wait(ctx);
            }
            PROC_LIGHT => {
                self.action_light(ctx);
            }
            PROC_ARRIVAL => {
                self.action_arrival(ctx);
            }
            PROC_HUNT => {
                self.action_hunt(ctx);
            }
            PROC_EVENT => {
                self.action_event(ctx);
            }
            _ => {}
        }
        true
    }

    pub fn create(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.radius2 = (actor.scale.x * actor.scale.x) * (RANGE_SCALE * RANGE_SCALE);
        self.height = actor.scale.y * RANGE_SCALE;

        self.message = actor.angle[0] as u16;
        self.event_flag = actor.angle[2] as u16;

        let u_var2 = self.get_swbit() as i32;
        match self.get_type() {
            1 => {
                if ctx.flags.test(EVENT_0404) && u_var2 != NO_SWITCH as i32 {
                    ctx.switches.set(u_var2 as u8);
                }
            }
            _ => {}
        }

        if self.wait_terms(ctx) != 0 {
            self.set_actio(PROC_WAIT);
        } else {
            self.set_actio(PROC_ARRIVAL);
        }

        actor.shape_angle[2] = 0;
        actor.shape_angle[0] = 0;
        actor.angle[2] = 0;
        actor.angle[0] = 0;
        self.timer = 0;
        self.entered = true;
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    tags: Query<'w, 's, (Entity, &'static mut Actor, &'static mut TagHint), Without<Player>>,
    players: Query<'w, 's, &'static mut Player>,
    boxes: Query<'w, 's, &'static Movebox>,
    barrels: Query<'w, 's, (Entity, &'static Carryable)>,
    list: Res<'w, EventList>,
    active: ResMut<'w, ActiveEvent>,
    manager: ResMut<'w, EventManager>,
    events: ResMut<'w, SimEvents>,
    flags: ResMut<'w, EventFlags>,
    switches: ResMut<'w, Switches>,
    tmp: ResMut<'w, TagHintTmp>,
    light: ResMut<'w, TagHintLight>,
    fog: ResMut<'w, TagHintFog>,
    held: ResMut<'w, TagHintMsg>,
    request: ResMut<'w, TagHintRequest>,
    order: ResMut<'w, OrderFlags>,
    player_room: Res<'w, TagHintPlayerRoom>,
    stay: Res<'w, StayRoom>,
    carry: Res<'w, Carry>,
    bits: Res<'w, MemBits>,
    do_intent: Res<'w, DoIntent>,
    collect: Res<'w, Collect>,
    equipped: Res<'w, Equipped>,
    collision: Option<Res<'w, Collision>>,
    procs: ResMut<'w, MsgProcs>,
    info: ResMut<'w, MsgGameInfo>,
    one: ResMut<'w, Msg1Box>,
    two: ResMut<'w, Msg2Box>,
    archive: Option<Res<'w, MsgArchives>>,
}

pub fn control(mut watch: Watch<'_, '_>) {
    let boxes: Vec<(Vec3, Vec3)> = watch
        .boxes
        .iter()
        .map(|block| (block.pos, block.home_pos))
        .collect();
    let archive = watch.archive.as_deref().map(|held| held as &dyn MsgArchive);
    let collision = watch.collision.as_deref();
    let boss_enemy = watch.bits.is_stage_boss_enemy();
    let carrying = watch.carry.held.is_some();
    let carrying_barrel = watch
        .barrels
        .iter()
        .any(|(_entity, kind)| *kind == Carryable::Barrel);
    let talk_command = watch.do_intent.0;
    let player_yaw = walk::to_s16(watch.players.iter().next().map(|p| p.yaw).unwrap_or(0.0));
    let room_player = watch.player_room.goat;
    let stay = watch.stay.0;
    for (entity, mut actor, mut tag) in &mut watch.tags {
        let mut player = watch.players.iter_mut().next();
        {
            let text = Text {
                procs: &mut watch.procs,
                info: &mut watch.info,
                one: &mut watch.one,
                two: &mut watch.two,
                archive,
            };
            let mut ctx = Ctx {
                entity,
                pos: actor.pos,
                room: actor.room,
                boxes: &boxes,
                flags: &mut watch.flags,
                switches: &mut watch.switches,
                tmp: &mut watch.tmp,
                list: &watch.list,
                active: &mut watch.active,
                manager: &mut watch.manager,
                events: &mut watch.events,
                light: &mut watch.light,
                fog: &mut watch.fog,
                held: &mut watch.held,
                request: &mut watch.request,
                order: &mut watch.order,
                room_player,
                stay_room: stay,
                equip: &watch.equipped,
                collect: &watch.collect,
                boss_enemy,
                carrying,
                carrying_barrel,
                carrying_wear: false,
                grab_actor: carrying,
                player: player.as_deref_mut(),
                player_yaw,
                collision,
                talk_command,
                text,
            };
            if !tag.entered {
                tag.create(&mut *actor, &mut ctx);
            }
            tag.execute(&mut ctx);
        }
    }
}
