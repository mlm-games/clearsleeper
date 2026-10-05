use bevy_ecs::prelude::*;
use glam::Vec3;

use super::msgdata::MsgLenState;
use super::msglayout::MsgLayoutState;
use super::msgget::{ItemMsgGet, MsgArchive, MsgGet};
use super::{
    FontSlot, ICON_ARROW_DOWN, ICON_ARROW_LEFT, ICON_ARROW_RIGHT, ICON_ARROW_UP,
    ICON_FLASHING_A_BUTTON, ICON_FLASHING_A_BUTTON_2, ICON_HEART, ICON_HEART_2, ICON_MAIN_STICK,
    ICON_MAIN_STICK_DOWN, ICON_MAIN_STICK_LEFT, ICON_MAIN_STICK_LEFT_RIGHT, ICON_MAIN_STICK_RIGHT,
    ICON_MAIN_STICK_UP, ICON_MAIN_STICK_UP_DOWN, ICON_MUSIC_NOTE, ICON_MUSIC_NOTE_2, ICON_NONE,
    ICON_SELECT_YOKO_LEFT, ICON_SELECT_YOKO_RIGHT, ICON_SLOTS, ICON_STARBURST, ICON_STARBURST_2,
    MSG_MAX_SELECT, MesgEntry, MsgData, STATUS_BOX_CLOSING, STATUS_BOX_OPENING, STATUS_CLOSE_WAIT,
    STATUS_SCOPE_ACTIVE, STATUS_SCOPE_WAIT,
};

pub const FPC_M_ERROR_PROCESS_ID: u32 = 0xFFFF_FFFF;

pub const FPC_NM_MSG: u16 = 0x01E7;
pub const FPC_NM_MSG2: u16 = 0x01E8;
pub const FPC_NM_SCP: u16 = 0x01F3;
pub const FPC_NM_PB: u16 = 0x01F4;

pub const DA_PY_STTS0_TELESCOPE_LOOK: u32 = 0x0020_0000;
pub const DA_EVT_MODE_NONE: u8 = 0;

const GROUP_SCP: u32 = 0x63;
const GROUP_PB: u32 = 0x59;
const TACT_MSG_NO: u32 = 0x5AC;
const FALLBACK_MSG_NO: u32 = 1;
const MESG_ANIME_TAG_CLEAR: u8 = 0xFF;
const MESG_CAM_TAG_CLEAR: i32 = -1;
const SEND_SPEED: i32 = 2;
const SPACE_TIMER: i32 = 0;
const SELECT_HEADER: &str = "\x1bCC[000000FF]\x1bGM[0]";
const SELECT_CURSOR_TARGET_LINES: i32 = 2;
const DRAW_SDW_OFFSET_X: i32 = 2;
const DRAW_SDW_OFFSET_Y: i32 = 1;
const DRAW_ICON_OFFSET_Y: i32 = -1;
const ROTATE_STEP: f32 = -90.0;
const ARROW_LEFT_ANGLE: f32 = 270.0;
const ARROW_RIGHT_ANGLE: f32 = 90.0;
const ARROW_UP_ANGLE: f32 = 180.0;
const HEAP_LOCK_ALLOWED: [u8; 3] = [7, 8, 9];
const BLACK_CLEAR: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
const SDW_WHITE: [u8; 4] = [0x00, 0x00, 0x00, 0xFF];
const BLEND_FULL: [f32; 4] = [0.0, 1.0, 1.0, 1.0];
const BLEND_ALPHA: u8 = 0xFF;

const BUTTON_TEX: [&str; 24] = [
    "font_00.bti",
    "font_01.bti",
    "font_09.bti",
    "font_04.bti",
    "font_05.bti",
    "font_02.bti",
    "font_03.bti",
    "font_06.bti",
    "font_08.bti",
    "font_07_01.bti",
    "font_10.bti",
    "font_10.bti",
    "font_10.bti",
    "font_10.bti",
    "font_07_01.bti",
    "font_07_01.bti",
    "font_07_01.bti",
    "font_07_01.bti",
    "font_07_01.bti",
    "font_07_01.bti",
    "font_12.bti",
    "font_13.bti",
    "font_14.bti",
    "font_15.bti",
];

const BUTTON_W: [[u8; 4]; 24] = [
    [0x00, 0xFF, 0xB4, 0xFF],
    [0xFF, 0x32, 0x32, 0xFF],
    [0xFF, 0xC8, 0x32, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0x96, 0xFF, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xFF, 0xC8, 0x32, 0xFF],
    [0xFF, 0xC8, 0x32, 0xFF],
    [0xFF, 0xC8, 0x32, 0xFF],
    [0xFF, 0xC8, 0x32, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0x00, 0xFF, 0xB4, 0xFF],
    [0xFF, 0x32, 0x32, 0xFF],
    [0xFF, 0xFF, 0xFF, 0xFF],
    [0xFF, 0xFF, 0xFF, 0xFF],
];



pub const J2D_MIRROR_Y: u8 = 1 << 0;
pub const J2D_MIRROR_X: u8 = 1 << 1;

pub const J2D_BIND_BOTTOM: u8 = 1 << 0;
pub const J2D_BIND_TOP: u8 = 1 << 1;
pub const J2D_BIND_RIGHT: u8 = 1 << 2;
pub const J2D_BIND_LEFT: u8 = 1 << 3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MyPicture {
    pub m124: f32,
    pub m128: f32,
    pub m12c: f32,
    pub m130: f32,
    pub m134: u8,
    pub global_bounds: [f32; 2],
    pub bounds_size: [f32; 2],
    pub num_texture: u8,
    pub binding: u8,
    pub mirror: u8,
    pub tumble: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PicQuad {
    pub pos: [[f32; 2]; 4],
    pub uv: [[f32; 2]; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MyPictureDraw {
    TexCoord(PicQuad),
    FullSet {
        pos: [f32; 2],
        size: [f32; 2],
        binding: u8,
        mirror: u8,
        tumble: bool,
    },
}

impl MyPicture {
    pub fn draw_self(&self, i_posx: f32, i_posy: f32) -> Option<MyPictureDraw> {
        if self.num_texture == 0 {
            return None;
        }
        Some(self.draw_full_set2(
            self.global_bounds[0] + i_posx,
            self.global_bounds[1] + i_posy,
            self.bounds_size[0],
            self.bounds_size[1],
            self.binding,
            self.mirror,
            self.tumble,
        ))
    }

    pub fn draw_full_set2(
        &self,
        i_posx: f32,
        i_posy: f32,
        i_width: f32,
        i_height: f32,
        i_binding: u8,
        i_mirror: u8,
        i_tumble: bool,
    ) -> MyPictureDraw {
        let mut s0 = self.m124;
        let mut s1 = self.m12c;
        let mut t0 = self.m128;
        let mut t1 = self.m130;

        if i_mirror & J2D_MIRROR_X != 0 {
            std::mem::swap(&mut s0, &mut s1);
        }
        if i_mirror & J2D_MIRROR_Y != 0 {
            std::mem::swap(&mut t0, &mut t1);
        }

        if self.m134 == 0 {
            return MyPictureDraw::FullSet {
                pos: [i_posx, i_posy],
                size: [i_width, i_height],
                binding: i_binding,
                mirror: i_mirror,
                tumble: i_tumble,
            };
        }

        let uv = if i_tumble {
            [[s0, t1], [s0, t0], [s1, t1], [s1, t0]]
        } else {
            [[s0, t0], [s1, t0], [s0, t1], [s1, t1]]
        };
        MyPictureDraw::TexCoord(PicQuad {
            pos: [
                [0.0, 0.0],
                [i_width, 0.0],
                [i_width, i_height],
                [0.0, i_height],
            ],
            uv,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cxy {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FopMsgPrmClass {
    pub mp_actor: Option<u32>,
    pub m_pos: Vec3,
    pub m_msg_no: u32,
    pub field_0x14: u32,
    pub field_0x18: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FopMsgPrmTimer {
    pub base: FopMsgPrmClass,
    pub m_timer_mode: i32,
    pub m_limit_time_ms: u16,
    pub m_show_type: u8,
    pub m_icon_type: u8,
    pub m_timer_pos: Cxy,
    pub m_rupee_pos: Cxy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FopMsgPrmMGameTerm {
    pub base: FopMsgPrmClass,
    pub field_0x1c: u32,
    pub field_0x20: u32,
    pub field_0x24: u16,
    pub field_0x26: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MsgAppend {
    #[default]
    None,
    Class(FopMsgPrmClass),
    MGameTerm(FopMsgPrmMGameTerm),
    Timer(FopMsgPrmTimer),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FopMsgMUnkStruct {
    pub field_0x0: u16,
    pub field_0x2: u16,
    pub field_0x4: u16,
    pub field_0x6: u16,
    pub field_0x8: u16,
    pub field_0xa: u16,
    pub field_0xc: i16,
    pub field_0xe: u16,
    pub field_0x10: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MsgTextBox {
    pub font: usize,
    pub char_space: f32,
    pub line_space: f32,
    pub width: f32,
    pub font_size: f32,
    pub bounds_x: i16,
    pub bounds_y: i16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectMessageGet {
    pub sp40: FopMsgMUnkStruct,
    pub shift_y: f32,
    pub out: [Option<String>; MSG_MAX_SELECT],
    pub select_message: [Option<String>; MSG_MAX_SELECT],
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MsgFlags {
    pub demo_msg: bool,
    pub tact_msg: bool,
    pub next_msg: bool,
    pub force_send: bool,
    pub message_send: bool,
}

impl MsgFlags {
    pub fn demo_msg_flag_on(&mut self) {
        self.demo_msg = true;
    }

    pub fn demo_msg_flag_off(&mut self) {
        self.demo_msg = false;
    }

    pub fn demo_msg_flag_check(&self) -> bool {
        self.demo_msg
    }

    pub fn tact_msg_flag_on(&mut self) {
        self.tact_msg = true;
    }

    pub fn tact_msg_flag_off(&mut self) {
        self.tact_msg = false;
    }

    pub fn tact_msg_flag_check(&self) -> bool {
        self.tact_msg
    }

    pub fn next_msg_flag_off(&mut self) {
        self.next_msg = false;
    }

    pub fn next_msg_flag_check(&self) -> bool {
        self.next_msg
    }

    pub fn force_send_on(&mut self, info: &MsgGameInfo) -> bool {
        if info.scope_mesg_status == STATUS_CLOSE_WAIT {
            self.force_send = true;
            return true;
        }
        false
    }

    pub fn force_send_off(&mut self) {
        self.force_send = false;
    }

    pub fn check_force_send(&self) -> bool {
        self.force_send
    }

    pub fn message_send_on(&mut self) {
        self.message_send = true;
    }

    pub fn message_send_off(&mut self) {
        self.message_send = false;
    }

    pub fn check_message_send(&self) -> bool {
        self.message_send
    }
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MsgGameInfo {
    pub heap_lock_flag: u8,
    pub player_status0: u32,
    pub event_mode: u8,
    pub scope_mesg_status: u8,
    pub mesg_status: u8,
    pub mesg_anime: u8,
    pub mesg_anime_tag: u8,
    pub mesg_cam_basic_id: i32,
    pub mesg_cam_id: i32,
    pub message_rupee: i16,
}

impl MsgGameInfo {
    pub fn check_player_status0(&self, flag: u32) -> bool {
        self.player_status0 & flag != 0
    }

    pub fn event_run_check(&self) -> bool {
        self.event_mode != DA_EVT_MODE_NONE
    }

    pub fn set_mesg_anime_attr_info(&mut self, id: u8) {
        self.mesg_anime = id;
    }

    pub fn clear_mesg_anime_tag_info(&mut self) {
        self.mesg_anime_tag = MESG_ANIME_TAG_CLEAR;
    }

    pub fn set_mesg_camera_attr_info(&mut self, id: i32) {
        self.mesg_cam_basic_id = id;
    }

    pub fn clear_mesg_camera_tag_info(&mut self) {
        self.mesg_cam_id = MESG_CAM_TAG_CLEAR;
    }

    pub fn set_message_rupee(&mut self, count: i16) {
        self.message_rupee = count;
    }

    pub fn get_scope_mode(&mut self) -> bool {
        if self.check_player_status0(DA_PY_STTS0_TELESCOPE_LOOK)
            && self.scope_mesg_status == STATUS_SCOPE_ACTIVE
            && !self.event_run_check()
        {
            self.scope_mesg_status = STATUS_SCOPE_WAIT;
            return true;
        }
        if self.scope_mesg_status == STATUS_BOX_CLOSING {
            self.mesg_status = STATUS_SCOPE_WAIT;
            return true;
        }
        false
    }

    pub fn release_scope_mode(&mut self) -> bool {
        if self.scope_mesg_status == STATUS_SCOPE_WAIT {
            self.scope_mesg_status = STATUS_SCOPE_ACTIVE;
            return true;
        }
        false
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MsgProc {
    pub pid: u32,
    pub name: u16,
    pub append: MsgAppend,
    pub msg_no: u32,
    pub field_0xf0: u32,
    pub layer: i32,
}

#[derive(Resource, Debug, Clone)]
pub struct MsgProcs {
    pub i_msg_id: u32,
    next_pid: u32,
    procs: Vec<MsgProc>,
}

impl Default for MsgProcs {
    fn default() -> Self {
        Self {
            i_msg_id: FPC_M_ERROR_PROCESS_ID,
            next_pid: 0,
            procs: Vec::new(),
        }
    }
}

impl MsgProcs {
    pub fn set_error(&mut self) {
        self.i_msg_id = FPC_M_ERROR_PROCESS_ID;
    }

    pub fn procs(&self) -> &[MsgProc] {
        &self.procs
    }

    pub fn spawn(&mut self, name: u16, append: MsgAppend) -> u32 {
        let pid = self.next_pid;
        self.next_pid = self.next_pid.wrapping_add(1);
        self.procs.push(MsgProc {
            pid,
            name,
            append,
            ..MsgProc::default()
        });
        pid
    }

    pub fn search_by_id(&self, i_pid: u32) -> Option<&MsgProc> {
        self.procs.iter().find(|p| p.pid == i_pid)
    }

    pub fn search_by_id_mut(&mut self, i_pid: u32) -> Option<&mut MsgProc> {
        self.procs.iter_mut().find(|p| p.pid == i_pid)
    }

    pub fn search_by_name(&self, i_name: u16) -> Option<&MsgProc> {
        self.procs.iter().find(|p| p.name == i_name)
    }

    pub fn is_executing(&self, i_pid: u32) -> bool {
        self.search_by_id(i_pid).is_some()
    }

    pub fn get_append(&self, i_pid: u32) -> Option<&MsgAppend> {
        self.search_by_id(i_pid).map(|p| &p.append)
    }

    pub fn delete(&mut self, i_pid: u32) {
        if let Some(at) = self.procs.iter().position(|p| p.pid == i_pid) {
            self.procs.remove(at);
        }
    }
}

pub fn fop_msgm_set_stage_layer(procs: &mut MsgProcs, stage_proc: u32, i_proc: u32) -> i32 {
    let layer = procs.search_by_id(stage_proc).map(|p| p.layer).unwrap_or(0);
    if let Some(target) = procs.search_by_id_mut(i_proc) {
        target.layer = layer;
    }
    layer
}

pub fn create_append(
    i_actor: Option<u32>,
    i_pos: Option<Vec3>,
    i_msg_no: Option<u32>,
    p4: Option<u32>,
    p5: i32,
) -> FopMsgPrmClass {
    FopMsgPrmClass {
        mp_actor: i_actor,
        m_pos: i_pos.unwrap_or(Vec3::ZERO),
        m_msg_no: i_msg_no.unwrap_or(0),
        field_0x14: p4.unwrap_or(0),
        field_0x18: p5,
    }
}

pub fn create_mgame_term_append(
    param_1: i16,
    param_2: i16,
    param_3: i32,
    param_4: i32,
    param_5: i32,
) -> FopMsgPrmMGameTerm {
    FopMsgPrmMGameTerm {
        base: create_append(None, None, None, None, param_5),
        field_0x1c: param_3 as u32,
        field_0x20: param_4 as u32,
        field_0x24: param_1 as u16,
        field_0x26: param_2 as u16,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn create_timer_append(
    i_mode: i32,
    i_limit_time_ms: u16,
    i_show_type: u8,
    i_icon_type: u8,
    i_pos_x: f32,
    i_pos_y: f32,
    i_rupee_pos_x: f32,
    i_rupee_pos_y: f32,
    param_9: i32,
) -> FopMsgPrmTimer {
    FopMsgPrmTimer {
        base: create_append(None, None, None, None, param_9),
        m_timer_mode: i_mode,
        m_limit_time_ms: i_limit_time_ms,
        m_show_type: i_show_type,
        m_icon_type: i_icon_type,
        m_timer_pos: Cxy {
            x: i_pos_x,
            y: i_pos_y,
        },
        m_rupee_pos: Cxy {
            x: i_rupee_pos_x,
            y: i_rupee_pos_y,
        },
    }
}

pub fn fop_msgm_create(
    procs: &mut MsgProcs,
    i_proc_name: u16,
    i_actor: Option<u32>,
    i_pos: Option<Vec3>,
    i_msg_no: Option<u32>,
    param_5: Option<u32>,
) -> u32 {
    let append = create_append(i_actor, i_pos, i_msg_no, param_5, -1);
    procs.spawn(i_proc_name, MsgAppend::Class(append))
}

pub fn fop_mgame_term_create(
    procs: &mut MsgProcs,
    i_proc_name: u16,
    param_2: i16,
    param_3: i16,
    param_4: i32,
    param_5: i32,
) -> u32 {
    let append = create_mgame_term_append(param_2, param_3, param_4, param_5, -1);
    procs.spawn(i_proc_name, MsgAppend::MGameTerm(append))
}

#[allow(clippy::too_many_arguments)]
pub fn fop_timer_create(
    procs: &mut MsgProcs,
    i_proc_name: u16,
    i_mode: u8,
    i_limit_time_ms: u16,
    i_show_type: u8,
    i_icon_type: u8,
    i_pos_x: f32,
    i_pos_y: f32,
    i_rupee_pos_x: f32,
    i_rupee_pos_y: f32,
) -> u32 {
    let append = create_timer_append(
        i_mode as i32,
        i_limit_time_ms,
        i_show_type,
        i_icon_type,
        i_pos_x,
        i_pos_y,
        i_rupee_pos_x,
        i_rupee_pos_y,
        -1,
    );
    procs.spawn(i_proc_name, MsgAppend::Timer(append))
}

fn textbox_is_msg2(textbox_type: u8) -> bool {
    textbox_type == 2 || textbox_type == 6 || textbox_type == 7
}

pub fn fop_msgm_message_type_select<F: MsgArchive + ?Sized>(
    procs: &mut MsgProcs,
    info: &mut MsgGameInfo,
    archive: &F,
    clear_count: u8,
    i_actor: Option<u32>,
    i_pos: Option<Vec3>,
    i_msg_no: &mut u32,
) -> u32 {
    if *i_msg_no >> 0x10 == GROUP_SCP {
        return fop_msgm_create(
            procs,
            FPC_NM_SCP,
            i_actor,
            i_pos,
            Some(*i_msg_no),
            Some(*i_msg_no),
        );
    }
    if *i_msg_no >> 0x10 == GROUP_PB {
        return fop_msgm_create(
            procs,
            FPC_NM_PB,
            i_actor,
            i_pos,
            Some(*i_msg_no),
            Some(*i_msg_no),
        );
    }

    let mut msg_get = MsgGet::default();
    let Some(header) = msg_get.get_mesg_header(archive, clear_count, *i_msg_no) else {
        *i_msg_no = FALLBACK_MSG_NO;
        return fop_msgm_create(
            procs,
            FPC_NM_MSG,
            i_actor,
            i_pos,
            Some(*i_msg_no),
            Some(*i_msg_no),
        );
    };
    if msg_get.get_message(header).is_some() {
        let proc_name = match msg_get.get_mesg_entry(header) {
            Some(entry) => {
                info.set_mesg_anime_attr_info(entry.initial_animation);
                info.set_mesg_camera_attr_info(entry.initial_camera as i32);
                info.set_message_rupee(entry.item_price);
                if textbox_is_msg2(entry.textbox_type) {
                    FPC_NM_MSG2
                } else {
                    FPC_NM_MSG
                }
            }
            None => FPC_NM_MSG,
        };
        return fop_msgm_create(
            procs,
            proc_name,
            i_actor,
            i_pos,
            Some(*i_msg_no),
            Some(*i_msg_no),
        );
    }
    *i_msg_no = FALLBACK_MSG_NO;
    fop_msgm_create(
        procs,
        FPC_NM_MSG,
        i_actor,
        i_pos,
        Some(*i_msg_no),
        Some(*i_msg_no),
    )
}

fn heap_lock_allows(flag: u8) -> bool {
    flag == 0 || HEAP_LOCK_ALLOWED.contains(&flag)
}

fn update_or_retire(procs: &mut MsgProcs, pid: u32, i_msg_no: u32) {
    if procs.search_by_id(pid).is_none() {
        procs.set_error();
        return;
    }
    if let Some(p_msg) = procs.search_by_id_mut(pid) {
        p_msg.msg_no = i_msg_no;
        p_msg.field_0xf0 = i_msg_no;
    }
    if procs.search_by_name(FPC_NM_SCP).is_some() || procs.search_by_name(FPC_NM_PB).is_some() {
        procs.delete(pid);
        procs.set_error();
    }
}

fn message_set_dispatch<F: MsgArchive + ?Sized>(
    procs: &mut MsgProcs,
    info: &mut MsgGameInfo,
    archive: &F,
    clear_count: u8,
    i_msg_no: u32,
    i_actor: Option<u32>,
    i_look_at_pos: Vec3,
) -> u32 {
    if !heap_lock_allows(info.heap_lock_flag) {
        return FPC_M_ERROR_PROCESS_ID;
    }

    info.clear_mesg_anime_tag_info();
    info.clear_mesg_camera_tag_info();

    let pid = procs.i_msg_id;
    if pid == FPC_M_ERROR_PROCESS_ID {
        let mut msg_no = i_msg_no;
        procs.i_msg_id = fop_msgm_message_type_select(
            procs,
            info,
            archive,
            clear_count,
            i_actor,
            Some(i_look_at_pos),
            &mut msg_no,
        );
    } else if procs.is_executing(pid) {
        update_or_retire(procs, pid, i_msg_no);
    } else {
        let mut msg_no = i_msg_no;
        procs.i_msg_id = fop_msgm_message_type_select(
            procs,
            info,
            archive,
            clear_count,
            i_actor,
            Some(i_look_at_pos),
            &mut msg_no,
        );
    }

    procs.i_msg_id
}

pub fn fop_msgm_message_set<F: MsgArchive + ?Sized>(
    procs: &mut MsgProcs,
    info: &mut MsgGameInfo,
    archive: &F,
    clear_count: u8,
    i_msg_no: u32,
    i_actor: Option<u32>,
    i_look_at_pos: Vec3,
) -> u32 {
    message_set_dispatch(
        procs,
        info,
        archive,
        clear_count,
        i_msg_no,
        i_actor,
        i_look_at_pos,
    )
}

pub fn fop_msgm_message_set_actor<F: MsgArchive + ?Sized>(
    procs: &mut MsgProcs,
    info: &mut MsgGameInfo,
    archive: &F,
    clear_count: u8,
    i_msg_no: u32,
    i_actor: u32,
    i_eye_pos: Vec3,
) -> u32 {
    message_set_dispatch(
        procs,
        info,
        archive,
        clear_count,
        i_msg_no,
        Some(i_actor),
        i_eye_pos,
    )
}

pub fn fop_msgm_message_set_look_at<F: MsgArchive + ?Sized>(
    procs: &mut MsgProcs,
    info: &mut MsgGameInfo,
    archive: &F,
    clear_count: u8,
    i_msg_no: u32,
    i_look_at_pos: Vec3,
) -> u32 {
    message_set_dispatch(
        procs,
        info,
        archive,
        clear_count,
        i_msg_no,
        None,
        i_look_at_pos,
    )
}

pub fn fop_msgm_scope_message_set(
    procs: &mut MsgProcs,
    info: &mut MsgGameInfo,
    i_msg_no: u32,
) -> u32 {
    let pid = procs.i_msg_id;
    if !procs.is_executing(pid) {
        return procs.i_msg_id;
    }
    if procs.search_by_id(pid).is_none() {
        procs.set_error();
        return procs.i_msg_id;
    }
    if info.check_player_status0(DA_PY_STTS0_TELESCOPE_LOOK)
        && info.scope_mesg_status == STATUS_SCOPE_ACTIVE
    {
        info.scope_mesg_status = STATUS_BOX_OPENING;
    }
    if let Some(p_msg) = procs.search_by_id_mut(pid) {
        p_msg.msg_no = i_msg_no;
        p_msg.field_0xf0 = i_msg_no;
    }
    procs.i_msg_id
}

pub fn fop_msgm_tact_message_set<F: MsgArchive + ?Sized>(
    procs: &mut MsgProcs,
    flags: &mut MsgFlags,
    info: &mut MsgGameInfo,
    archive: &F,
    clear_count: u8,
) -> u32 {
    if !heap_lock_allows(info.heap_lock_flag) {
        return FPC_M_ERROR_PROCESS_ID;
    }

    let msg_no_temp = TACT_MSG_NO;
    let pid = procs.i_msg_id;
    if pid == FPC_M_ERROR_PROCESS_ID {
        let mut msg_no = msg_no_temp;
        procs.i_msg_id = fop_msgm_message_type_select(
            procs,
            info,
            archive,
            clear_count,
            None,
            Some(Vec3::ZERO),
            &mut msg_no,
        );
        flags.tact_msg_flag_on();
    } else if procs.is_executing(pid) {
        update_or_retire(procs, pid, msg_no_temp);
    } else {
        let mut msg_no = msg_no_temp;
        procs.i_msg_id = fop_msgm_message_type_select(
            procs,
            info,
            archive,
            clear_count,
            None,
            Some(Vec3::ZERO),
            &mut msg_no,
        );
        flags.tact_msg_flag_on();
    }

    procs.i_msg_id
}

pub fn fop_msgm_message_get<F: MsgArchive + ?Sized>(
    msg: &MsgData,
    archive: &F,
    clear_count: u8,
    i_dest: &mut String,
    i_msg_no: u32,
    player_name: &str,
    pal_language: i32,
) {
    let mut msg_get = ItemMsgGet::default();
    let Some(head) = msg_get.get_mesg_header(archive, clear_count, i_msg_no) else {
        return;
    };
    let Some(src) = msg_get.get_message(head) else {
        return;
    };
    i_dest.clear();
    msg.get_string(i_dest, src, i_msg_no, player_name, pal_language);
}

pub fn fop_msgm_password_get<F: MsgArchive + ?Sized>(
    msg: &MsgData,
    archive: &F,
    clear_count: u8,
    i_dest: &mut String,
    i_msg_no: u32,
    player_name: &str,
    pal_language: i32,
) {
    fop_msgm_message_get(
        msg,
        archive,
        clear_count,
        i_dest,
        i_msg_no,
        player_name,
        pal_language,
    );
}

pub fn dummy_float0(arg: i32) -> Option<f32> {
    match arg {
        0 => Some(0.0),
        1 => Some(0.5),
        _ => None,
    }
}

pub fn dummy_float1(arg: i32) -> Option<f64> {
    match arg {
        0 => Some(4503601774854144.0),
        1 => Some(0.5),
        2 => Some(3.0),
        _ => None,
    }
}

pub fn dummy_float2(arg: i32) -> Option<f32> {
    match arg {
        0 => Some(2.0),
        _ => None,
    }
}

fn select_icon_pos(temp_r3: i16, temp_r3_3: i16, temp_r5: i32, icon_pos_y: i32) -> i32 {
    temp_r3_3 as i32 + temp_r5 * ((SELECT_CURSOR_TARGET_LINES - temp_r3 as i32) + icon_pos_y * 2)
}

pub fn fop_msgm_select_message_get<F: FontSlot + ?Sized>(
    font: &F,
    archive: &dyn MsgArchive,
    len_state: &MsgLenState<'_>,
    layout_state: &MsgLayoutState<'_>,
    i_text_pane: &MsgTextBox,
    i_ruby_pane: &MsgTextBox,
    i_msg_no: u32,
) -> Option<SelectMessageGet> {
    let mut msg_get = ItemMsgGet::default();
    let head = msg_get.get_mesg_header(archive, len_state.clear_count, i_msg_no)?;
    let bmg_data = msg_get.get_message(head)?.to_vec();
    let entry: MesgEntry = msg_get.get_mesg_entry(head)?;

    let mut msg_data = MsgData::new_data_proc();
    msg_data.set_bmg(bmg_data);
    msg_data.out = [
        Some(SELECT_HEADER.to_string()),
        Some(String::new()),
        Some(String::new()),
        Some(String::new()),
    ];
    msg_data.font = i_text_pane.font;
    msg_data.ruby_font = i_ruby_pane.font;
    msg_data.char_space = i_text_pane.char_space as i32;
    msg_data.ruby_char_space = i_ruby_pane.char_space as i32;
    msg_data.line_space = i_text_pane.line_space as i32;
    msg_data.entry = entry;
    msg_data.font_size = i_text_pane.font_size as i32;
    msg_data.ruby_font_size = i_ruby_pane.font_size as i32;
    msg_data.line_width = i_text_pane.width as i32;
    msg_data.center_line_width = i_text_pane.width as i32;
    msg_data.send_speed = SEND_SPEED;
    msg_data.space_timer = SPACE_TIMER;
    msg_data.short_cut();
    msg_data.space_flag = 0;

    msg_data.string_length(font, *len_state);
    msg_data.string_shift();
    msg_data.icon_idx_refresh();

    let temp_r3 = msg_data.line_count as i16;
    msg_data.line_count = 0;

    let shift_y =
        (SELECT_CURSOR_TARGET_LINES - temp_r3 as i32) as f32 * (i_text_pane.line_space / 2.0);

    msg_data.string_set(font, layout_state);

    let temp_r5 = (i_text_pane.line_space / 2.0) as i32;
    let temp_r6 = i_text_pane.bounds_x;
    let temp_r3_3 = i_text_pane.bounds_y;

    let mut sp40 = FopMsgMUnkStruct::default();
    let mut var_r7 = false;
    for i in 0..ICON_SLOTS {
        let icon_num = msg_data.icon_num[i];
        if icon_num == ICON_NONE {
            continue;
        }
        if icon_num != ICON_SELECT_YOKO_LEFT && icon_num != ICON_SELECT_YOKO_RIGHT {
            continue;
        }
        if icon_num == ICON_SELECT_YOKO_RIGHT {
            sp40.field_0x10 = var_r7 as u8;
        }
        let pos_x = msg_data.icon_pos_x[i];
        let pos_y = msg_data.icon_pos_y[i];
        let x = (temp_r6 as i32 + pos_x) as u16;
        let y = select_icon_pos(temp_r3, temp_r3_3, temp_r5, pos_y);
        if !var_r7 {
            sp40.field_0x0 = x;
            sp40.field_0x4 = y as u16;
            var_r7 = true;
        } else {
            sp40.field_0x2 = x;
            sp40.field_0x8 = x;
            sp40.field_0x6 = y as i16 as u16;
            sp40.field_0xc = y as i16;
        }
    }

    sp40.field_0xa = (temp_r6 as f32 + msg_data.now_cursor_pos) as u16;
    sp40.field_0xe = sp40.field_0xc as u16;

    Some(SelectMessageGet {
        sp40,
        shift_y,
        out: std::mem::take(&mut msg_data.out),
        select_message: std::mem::take(&mut msg_data.select_message),
    })
}

pub use super::msghio::MsgMeterHio as MeterMsgHio;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MsgIconPic {
    pub visible: bool,
    pub tex: Option<&'static str>,
    pub icon_no: u8,
    pub white: [u8; 4],
    pub black: [u8; 4],
    pub blend_color: [f32; 4],
    pub blend_alpha: [f32; 4],
    pub alpha: u8,
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub rot_z: f32,
}

impl Default for MsgIconPic {
    fn default() -> Self {
        Self {
            visible: false,
            tex: None,
            icon_no: ICON_NONE,
            white: [0xFF; 4],
            black: [0xFF; 4],
            blend_color: [0.0; 4],
            blend_alpha: [0.0; 4],
            alpha: BLEND_ALPHA,
            pos: [0.0; 2],
            size: [0.0; 2],
            rot_z: 0.0,
        }
    }
}

impl MsgIconPic {
    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn blend_draw(&mut self, data: &'static str) {
        self.visible = true;
        self.tex = Some(data);
    }

    pub fn set_white(&mut self, white: [u8; 4]) {
        self.white = white;
    }

    pub fn set_black(&mut self, black: [u8; 4]) {
        self.black = black;
    }

    pub fn set_blend_ratio(&mut self, f1: f32, f2: f32, f3: f32, f4: f32) {
        self.blend_color = [f1, f2, f3, f4];
        self.blend_alpha = [f1, f2, f3, f4];
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.pos = [x, y];
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.size = [w, h];
    }

    pub fn rotate(&mut self, angle: f32) {
        self.rot_z = angle;
    }

    pub fn draw(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.move_to(x, y);
        self.resize(w, h);
    }
}

pub fn dummy(pane: &mut MsgIconPic) {
    pane.resize(0.0, 0.0);
}

fn normalize_icon(i_icon_no: u8) -> u8 {
    match i_icon_no {
        ICON_FLASHING_A_BUTTON_2 => ICON_FLASHING_A_BUTTON,
        ICON_HEART_2 => ICON_HEART,
        ICON_MUSIC_NOTE_2 => ICON_MUSIC_NOTE,
        ICON_STARBURST_2 => ICON_STARBURST,
        other => other,
    }
}

fn icon_uses_msg_color(i_icon_no: u8) -> bool {
    if matches!(
        i_icon_no,
        ICON_ARROW_LEFT | ICON_ARROW_RIGHT | ICON_ARROW_UP | ICON_ARROW_DOWN
    ) {
        return true;
    }
    i_icon_no == 0x15 || i_icon_no == 0x17
}

fn icon_white(i_color: u32, i_icon_no: u8) -> [u8; 4] {
    if icon_uses_msg_color(i_icon_no) {
        [
            (i_color >> 0x18) as u8,
            (i_color >> 0x10) as u8,
            (i_color >> 0x8) as u8,
            0xFF,
        ]
    } else {
        BUTTON_W
            .get(i_icon_no as usize)
            .copied()
            .unwrap_or([0xFF; 4])
    }
}

pub fn fop_msgm_out_font_tex(i_icon_no: i32) -> Option<&'static str> {
    if i_icon_no < 0 {
        return None;
    }
    BUTTON_TEX.get(i_icon_no as usize).copied()
}

pub fn fop_msgm_out_font_color_white(i_no: i32) -> Option<[u8; 4]> {
    if i_no < 0 {
        return None;
    }
    BUTTON_W.get(i_no as usize).copied()
}

pub fn fop_msgm_out_font_set(
    i_icon_pic: &mut MsgIconPic,
    i_sdw_pic: &mut MsgIconPic,
    i_timer: &mut i16,
    i_color: u32,
    i_icon_no: u8,
) {
    let i_icon_no = normalize_icon(i_icon_no);
    let tex = fop_msgm_out_font_tex(i_icon_no as i32).unwrap_or_default();

    i_icon_pic.show();
    i_sdw_pic.show();
    i_icon_pic.blend_draw(tex);
    i_sdw_pic.blend_draw(tex);
    i_icon_pic.set_white(icon_white(i_color, i_icon_no));
    i_icon_pic.set_black(BLACK_CLEAR);
    i_sdw_pic.set_white(SDW_WHITE);
    i_sdw_pic.set_black(BLACK_CLEAR);
    i_icon_pic.set_blend_ratio(BLEND_FULL[0], BLEND_FULL[1], BLEND_FULL[2], BLEND_FULL[3]);
    i_sdw_pic.set_blend_ratio(BLEND_FULL[0], BLEND_FULL[1], BLEND_FULL[2], BLEND_FULL[3]);

    i_icon_pic.icon_no = i_icon_no;
    i_sdw_pic.icon_no = i_icon_no;

    *i_timer = 0;
}

pub fn fop_msgm_out_font_set1(
    i_icon_pic: &mut MsgIconPic,
    i_timer: &mut i16,
    i_color: u32,
    i_icon_no: u8,
) {
    let i_icon_no = normalize_icon(i_icon_no);
    let tex = fop_msgm_out_font_tex(i_icon_no as i32).unwrap_or_default();

    i_icon_pic.show();
    i_icon_pic.blend_draw(tex);
    i_icon_pic.set_white(icon_white(i_color, i_icon_no));
    i_icon_pic.set_black(BLACK_CLEAR);
    i_icon_pic.set_blend_ratio(BLEND_FULL[0], BLEND_FULL[1], BLEND_FULL[2], BLEND_FULL[3]);

    i_icon_pic.icon_no = i_icon_no;

    *i_timer = 0;
}

pub fn fop_msgm_value_increase(i_max: i32, i_value: i32, i_mode: u8) -> Option<f32> {
    if i_max <= 0 {
        return Some(1.0);
    }
    let value = if i_value < 0 {
        0
    } else if i_value > i_max {
        i_max
    } else {
        i_value
    };
    let v = value as f32 / i_max as f32;
    match i_mode {
        0 => Some(v * v),
        1 => Some(v.sqrt()),
        2 => Some(v),
        3 => Some((v * 2.0 - 1.0) * 2.0 - 1.0),
        _ => None,
    }
}

struct AnimePhases {
    temp: i16,
    temp2: i16,
    temp3: i16,
    temp4: i16,
    temp5: i16,
}

fn anime_phases(hio: &MeterMsgHio, param_4: i16) -> AnimePhases {
    let temp = ((hio.field_0x88 as i32)
        + hio.field_0x87 as i32
        + hio.field_0x89 as i32
        + hio.field_0x87 as i32) as i16;
    let temp2 = ((hio.field_0x88 as i32) + temp as i32 * param_4 as i32) as i16;
    let temp3 = ((hio.field_0x87 as i32) + temp2 as i32) as i16;
    let temp4 = ((hio.field_0x89 as i32) + temp3 as i32) as i16;
    let temp5 = ((hio.field_0x87 as i32) + temp4 as i32) as i16;
    AnimePhases {
        temp,
        temp2,
        temp3,
        temp4,
        temp5,
    }
}

fn anime_piece_blend(hio: &MeterMsgHio, param_3: i16, param_4: i16) -> Option<[f32; 4]> {
    let p = anime_phases(hio, param_4);
    if param_3 < p.temp2 {
        Some(BLEND_FULL)
    } else if param_3 < p.temp3 {
        let t =
            fop_msgm_value_increase(p.temp as i32, param_3 as i32 - p.temp2 as i32, 0).unwrap_or_default();
        Some([t, 1.0 - t, 1.0, 1.0])
    } else if param_3 < p.temp4 {
        Some([1.0, 0.0, 1.0, 1.0])
    } else if param_3 < p.temp5 {
        let t =
            fop_msgm_value_increase(p.temp as i32, param_3 as i32 - p.temp4 as i32, 0).unwrap_or_default();
        Some([1.0 - t, t, 1.0, 1.0])
    } else {
        None
    }
}

pub fn fop_msgm_out_font_stick_anime_piece(
    hio: &MeterMsgHio,
    i_icon_pic: &mut MsgIconPic,
    i_sdw_pic: &mut MsgIconPic,
    param_3: i16,
    param_4: i16,
) {
    if let Some(b) = anime_piece_blend(hio, param_3, param_4) {
        i_icon_pic.set_blend_ratio(b[0], b[1], b[2], b[3]);
        i_sdw_pic.set_blend_ratio(b[0], b[1], b[2], b[3]);
    }
}

pub fn fop_msgm_out_font_stick_anime_piece_offsets(
    hio: &MeterMsgHio,
    i_icon_pic: &mut MsgIconPic,
    param_2: i16,
    param_3: i16,
) {
    if let Some(b) = anime_piece_blend(hio, param_2, param_3) {
        i_icon_pic.set_blend_ratio(b[0], b[1], b[2], b[3]);
    }
}

fn rotate_steps(timer: i16, temp: i16) -> Option<f32> {
    if temp == 0 || timer % temp != 0 {
        return None;
    }
    Some((timer as i32 / temp as i32) as f32 * ROTATE_STEP)
}

fn swap_i32(a: &mut i32, b: &mut i32) {
    let t = *a;
    *a = *b;
    *b = t;
}

pub fn fop_msgm_out_font_stick_anime(
    hio: &MeterMsgHio,
    i_icon_pic: &mut MsgIconPic,
    i_sdw_pic: &mut MsgIconPic,
    param_3: &mut i32,
    param_4: &mut i32,
    param_5: i32,
    i_timer: &mut i16,
) {
    let temp = anime_phases(hio, 0).temp;

    if *i_timer < temp {
        fop_msgm_out_font_stick_anime_piece(hio, i_icon_pic, i_sdw_pic, *i_timer, 0);
    } else if i32::from(*i_timer) < i32::from(temp) * 2 {
        fop_msgm_out_font_stick_anime_piece(hio, i_icon_pic, i_sdw_pic, *i_timer, 1);
        *param_3 += param_5;
    } else if i32::from(*i_timer) < i32::from(temp) * 3 {
        fop_msgm_out_font_stick_anime_piece(hio, i_icon_pic, i_sdw_pic, *i_timer, 2);
        *param_3 += param_5;
        *param_4 += param_5;
    } else if i32::from(*i_timer) < i32::from(temp) * 4 {
        fop_msgm_out_font_stick_anime_piece(hio, i_icon_pic, i_sdw_pic, *i_timer, 3);
        *param_4 += param_5;
    }

    if let Some(angle) = rotate_steps(*i_timer, temp) {
        i_icon_pic.rotate(angle);
        i_sdw_pic.rotate(angle);
    }

    *i_timer += 1;
    if i32::from(*i_timer) >= i32::from(temp) * 4 {
        *i_timer = 0;
    }
}

pub fn fop_msgm_out_font_stick_anime_offsets(
    hio: &MeterMsgHio,
    i_icon_pic: &mut MsgIconPic,
    param_2: &mut i32,
    param_3: &mut i32,
    param_4: &mut i32,
    param_5: &mut i32,
    i_timer: &mut i16,
) {
    let temp = anime_phases(hio, 0).temp;

    if *i_timer < temp {
        fop_msgm_out_font_stick_anime_piece_offsets(hio, i_icon_pic, *i_timer, 0);
    } else if i32::from(*i_timer) < i32::from(temp) * 2 {
        fop_msgm_out_font_stick_anime_piece_offsets(hio, i_icon_pic, *i_timer, 1);
        swap_i32(param_4, param_5);
        *param_2 += *param_5;
    } else if i32::from(*i_timer) < i32::from(temp) * 3 {
        fop_msgm_out_font_stick_anime_piece_offsets(hio, i_icon_pic, *i_timer, 2);
        *param_2 += *param_4;
        *param_3 += *param_5;
    } else if i32::from(*i_timer) < i32::from(temp) * 4 {
        fop_msgm_out_font_stick_anime_piece_offsets(hio, i_icon_pic, *i_timer, 3);
        swap_i32(param_4, param_5);
        *param_3 += *param_4;
    }

    if let Some(angle) = rotate_steps(*i_timer, temp) {
        i_icon_pic.rotate(angle);
    }

    *i_timer += 1;
    if i32::from(*i_timer) >= i32::from(temp) * 4 {
        *i_timer = 0;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn fop_msgm_out_font_stick_anime_dir(
    hio: &MeterMsgHio,
    i_icon_pic: &mut MsgIconPic,
    i_sdw_pic: &mut MsgIconPic,
    param_3: &mut i32,
    param_4: &mut i32,
    param_5: i32,
    i_timer: &mut i16,
    param_7: u8,
) {
    let temp = anime_phases(hio, 0).temp;

    if *i_timer < temp {
        fop_msgm_out_font_stick_anime_piece(hio, i_icon_pic, i_sdw_pic, *i_timer, 0);
        match param_7 {
            1 => *param_3 += param_5,
            2 => {
                *param_3 += param_5;
                *param_4 += param_5;
            }
            3 => *param_4 += param_5,
            _ => {}
        }
    }

    let angle = param_7 as f32 * ROTATE_STEP;
    i_icon_pic.rotate(angle);
    i_sdw_pic.rotate(angle);

    *i_timer += 1;
    if *i_timer >= temp {
        *i_timer = 0;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn fop_msgm_out_font_stick_anime_dir_offsets(
    hio: &MeterMsgHio,
    i_icon_pic: &mut MsgIconPic,
    param_2: &mut i32,
    param_3: &mut i32,
    param_4: &mut i32,
    param_5: &mut i32,
    i_timer: &mut i16,
    param_7: u8,
) {
    let temp = anime_phases(hio, 0).temp;

    if *i_timer < temp {
        fop_msgm_out_font_stick_anime_piece_offsets(hio, i_icon_pic, *i_timer, 0);
        match param_7 {
            1 => {
                swap_i32(param_4, param_5);
                *param_2 += *param_5;
            }
            2 => {
                *param_2 += *param_4;
                *param_3 += *param_5;
            }
            3 => {
                swap_i32(param_4, param_5);
                *param_3 += *param_4;
            }
            _ => {}
        }
    }

    i_icon_pic.rotate(param_7 as f32 * ROTATE_STEP);

    *i_timer += 1;
    if *i_timer >= temp {
        *i_timer = 0;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn fop_msgm_out_font_stick_anime2(
    hio: &MeterMsgHio,
    i_icon_pic: &mut MsgIconPic,
    i_sdw_pic: &mut MsgIconPic,
    param_3: &mut i32,
    param_4: &mut i32,
    param_5: i32,
    i_timer: &mut i16,
    param_7: u8,
) {
    let temp = anime_phases(hio, 0).temp;

    let rot;
    if param_7 == 0 {
        if *i_timer < temp {
            fop_msgm_out_font_stick_anime_piece(hio, i_icon_pic, i_sdw_pic, *i_timer, 0);
            *param_3 += param_5;
            rot = 1;
        } else if i32::from(*i_timer) < i32::from(temp) * 2 {
            fop_msgm_out_font_stick_anime_piece(hio, i_icon_pic, i_sdw_pic, *i_timer, 1);
            *param_4 += param_5;
            rot = 3;
        } else {
            rot = 0;
        }
    } else if *i_timer < temp {
        fop_msgm_out_font_stick_anime_piece(hio, i_icon_pic, i_sdw_pic, *i_timer, 0);
        rot = 0;
    } else if i32::from(*i_timer) < i32::from(temp) * 2 {
        fop_msgm_out_font_stick_anime_piece(hio, i_icon_pic, i_sdw_pic, *i_timer, 1);
        *param_3 += param_5;
        *param_4 += param_5;
        rot = 2;
    } else {
        rot = 0;
    }

    let angle = rot as f32 * ROTATE_STEP;
    i_icon_pic.rotate(angle);
    i_sdw_pic.rotate(angle);

    *i_timer += 1;
    if *i_timer >= temp as i32 * 2 {
        *i_timer = 0;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn fop_msgm_out_font_stick_anime2_offsets(
    hio: &MeterMsgHio,
    i_icon_pic: &mut MsgIconPic,
    param_2: &mut i32,
    param_3: &mut i32,
    param_4: &mut i32,
    param_5: &mut i32,
    i_timer: &mut i16,
    param_7: u8,
) {
    let temp = anime_phases(hio, 0).temp;

    let rot;
    if param_7 == 0 {
        if *i_timer < temp {
            fop_msgm_out_font_stick_anime_piece_offsets(hio, i_icon_pic, *i_timer, 0);
            swap_i32(param_4, param_5);
            *param_2 += *param_5;
            rot = 1;
        } else if i32::from(*i_timer) < i32::from(temp) * 2 {
            fop_msgm_out_font_stick_anime_piece_offsets(hio, i_icon_pic, *i_timer, 1);
            swap_i32(param_4, param_5);
            *param_3 += *param_4;
            rot = 3;
        } else {
            rot = 0;
        }
    } else if *i_timer < temp {
        fop_msgm_out_font_stick_anime_piece_offsets(hio, i_icon_pic, *i_timer, 0);
        rot = 0;
    } else if i32::from(*i_timer) < i32::from(temp) * 2 {
        fop_msgm_out_font_stick_anime_piece_offsets(hio, i_icon_pic, *i_timer, 1);
        *param_2 += *param_4;
        *param_3 += *param_5;
        rot = 2;
    } else {
        rot = 0;
    }

    i_icon_pic.rotate(rot as f32 * ROTATE_STEP);

    *i_timer += 1;
    if *i_timer >= temp as i32 * 2 {
        *i_timer = 0;
    }
}

pub fn fop_msgm_out_font_arrow(
    i_icon_pic: &mut MsgIconPic,
    i_sdw_pic: &mut MsgIconPic,
    param_3: &mut i32,
    param_4: &mut i32,
    param_5: i32,
    i_icon_no: u8,
) {
    if i_icon_no == ICON_ARROW_LEFT {
        *param_3 += param_5;
        i_icon_pic.rotate(ARROW_LEFT_ANGLE);
        i_sdw_pic.rotate(ARROW_LEFT_ANGLE);
    } else if i_icon_no == ICON_ARROW_RIGHT {
        *param_4 += param_5;
        i_icon_pic.rotate(ARROW_RIGHT_ANGLE);
        i_sdw_pic.rotate(ARROW_RIGHT_ANGLE);
    } else if i_icon_no == ICON_ARROW_UP {
        *param_3 += param_5;
        *param_4 += param_5;
        i_icon_pic.rotate(ARROW_UP_ANGLE);
        i_sdw_pic.rotate(ARROW_UP_ANGLE);
    }
}

pub fn fop_msgm_out_font_arrow_offsets(
    i_icon_pic: &mut MsgIconPic,
    param_2: &mut i32,
    param_3: &mut i32,
    param_4: &mut i32,
    param_5: &mut i32,
    i_icon_no: u8,
) {
    if i_icon_no == ICON_ARROW_LEFT {
        swap_i32(param_4, param_5);
        *param_2 += *param_5;
        i_icon_pic.rotate(ARROW_LEFT_ANGLE);
    } else if i_icon_no == ICON_ARROW_RIGHT {
        swap_i32(param_4, param_5);
        *param_3 += *param_4;
        i_icon_pic.rotate(ARROW_RIGHT_ANGLE);
    } else if i_icon_no == ICON_ARROW_UP {
        *param_2 += *param_4;
        *param_3 += *param_5;
        i_icon_pic.rotate(ARROW_UP_ANGLE);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn fop_msgm_out_font_draw(
    hio: &MeterMsgHio,
    i_icon_pic: &mut MsgIconPic,
    i_sdw_pic: &mut MsgIconPic,
    mut param_3: i32,
    mut param_4: i32,
    param_5: i32,
    i_timer: &mut i16,
    i_alpha: u8,
    i_icon_no: u8,
) {
    match i_icon_no {
        ICON_MAIN_STICK => {
            fop_msgm_out_font_stick_anime(
                hio,
                i_icon_pic,
                i_sdw_pic,
                &mut param_3,
                &mut param_4,
                param_5,
                i_timer,
            );
        }
        ICON_MAIN_STICK_UP => {
            fop_msgm_out_font_stick_anime_dir(
                hio,
                i_icon_pic,
                i_sdw_pic,
                &mut param_3,
                &mut param_4,
                param_5,
                i_timer,
                1,
            );
        }
        ICON_MAIN_STICK_DOWN => {
            fop_msgm_out_font_stick_anime_dir(
                hio,
                i_icon_pic,
                i_sdw_pic,
                &mut param_3,
                &mut param_4,
                param_5,
                i_timer,
                3,
            );
        }
        ICON_MAIN_STICK_LEFT => {
            fop_msgm_out_font_stick_anime_dir(
                hio,
                i_icon_pic,
                i_sdw_pic,
                &mut param_3,
                &mut param_4,
                param_5,
                i_timer,
                0,
            );
        }
        ICON_MAIN_STICK_RIGHT => {
            fop_msgm_out_font_stick_anime_dir(
                hio,
                i_icon_pic,
                i_sdw_pic,
                &mut param_3,
                &mut param_4,
                param_5,
                i_timer,
                2,
            );
        }
        ICON_MAIN_STICK_UP_DOWN => {
            fop_msgm_out_font_stick_anime2(
                hio,
                i_icon_pic,
                i_sdw_pic,
                &mut param_3,
                &mut param_4,
                param_5,
                i_timer,
                0,
            );
        }
        ICON_MAIN_STICK_LEFT_RIGHT => {
            fop_msgm_out_font_stick_anime2(
                hio,
                i_icon_pic,
                i_sdw_pic,
                &mut param_3,
                &mut param_4,
                param_5,
                i_timer,
                1,
            );
        }
        ICON_ARROW_LEFT | ICON_ARROW_RIGHT | ICON_ARROW_UP | ICON_ARROW_DOWN => {
            fop_msgm_out_font_arrow(
                i_icon_pic,
                i_sdw_pic,
                &mut param_3,
                &mut param_4,
                param_5,
                i_icon_no,
            );
        }
        _ => {}
    }

    let sdw_y = param_4 + DRAW_SDW_OFFSET_Y + hio.field_0x6a as i32;
    let icon_y = param_4 + DRAW_ICON_OFFSET_Y + hio.field_0x6a as i32;
    i_sdw_pic.draw(
        (param_3 + DRAW_SDW_OFFSET_X) as f32,
        sdw_y as f32,
        param_5 as f32,
        param_5 as f32,
    );
    i_icon_pic.draw(param_3 as f32, icon_y as f32, param_5 as f32, param_5 as f32);

    i_icon_pic.set_alpha(i_alpha);
    i_sdw_pic.set_alpha(i_alpha);
}

#[allow(clippy::too_many_arguments)]
pub fn fop_msgm_out_font_draw2(
    hio: &MeterMsgHio,
    i_icon_pic: &mut MsgIconPic,
    param_2: &mut MsgIconPic,
    param_3: i32,
    param_4: i32,
    mut i_width: i32,
    mut i_height: i32,
    i_timer: &mut i16,
    i_alpha: u8,
    i_icon_no: u8,
) {
    let mut temp1 = 0i32;
    let mut temp2 = 0i32;

    match i_icon_no {
        ICON_MAIN_STICK => {
            fop_msgm_out_font_stick_anime_offsets(
                hio,
                i_icon_pic,
                &mut temp1,
                &mut temp2,
                &mut i_width,
                &mut i_height,
                i_timer,
            );
        }
        ICON_MAIN_STICK_UP => {
            fop_msgm_out_font_stick_anime_dir_offsets(
                hio,
                i_icon_pic,
                &mut temp1,
                &mut temp2,
                &mut i_width,
                &mut i_height,
                i_timer,
                1,
            );
        }
        ICON_MAIN_STICK_DOWN => {
            fop_msgm_out_font_stick_anime_dir_offsets(
                hio,
                i_icon_pic,
                &mut temp1,
                &mut temp2,
                &mut i_width,
                &mut i_height,
                i_timer,
                3,
            );
        }
        ICON_MAIN_STICK_LEFT => {
            fop_msgm_out_font_stick_anime_dir_offsets(
                hio,
                i_icon_pic,
                &mut temp1,
                &mut temp2,
                &mut i_width,
                &mut i_height,
                i_timer,
                0,
            );
        }
        ICON_MAIN_STICK_RIGHT => {
            fop_msgm_out_font_stick_anime_dir_offsets(
                hio,
                i_icon_pic,
                &mut temp1,
                &mut temp2,
                &mut i_width,
                &mut i_height,
                i_timer,
                2,
            );
        }
        ICON_MAIN_STICK_UP_DOWN => {
            fop_msgm_out_font_stick_anime2_offsets(
                hio,
                i_icon_pic,
                &mut temp1,
                &mut temp2,
                &mut i_width,
                &mut i_height,
                i_timer,
                0,
            );
        }
        ICON_MAIN_STICK_LEFT_RIGHT => {
            fop_msgm_out_font_stick_anime2_offsets(
                hio,
                i_icon_pic,
                &mut temp1,
                &mut temp2,
                &mut i_width,
                &mut i_height,
                i_timer,
                1,
            );
        }
        ICON_ARROW_LEFT | ICON_ARROW_RIGHT | ICON_ARROW_UP | ICON_ARROW_DOWN => {
            fop_msgm_out_font_arrow_offsets(
                i_icon_pic,
                &mut temp1,
                &mut temp2,
                &mut i_width,
                &mut i_height,
                i_icon_no,
            );
        }
        _ => {}
    }

    param_2.move_to(param_3 as f32, param_4 as f32);
    i_icon_pic.move_to(temp1 as f32, temp2 as f32);
    i_icon_pic.resize(i_width as f32, i_height as f32);

    i_icon_pic.set_alpha(i_alpha);
}