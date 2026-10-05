use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Vec2, Vec3};

use super::actor::Actor;
use super::area::StageName;
use super::collision::Collision;
use super::event::{ActiveEvent, EventList, ItemPartner};
use super::events::{SimEvent, SimEvents};
use super::flags::EventFlags;
use super::intent::{CameraYaw, DoIntent, MoveIntent, TargetIntent};
use super::player::Player;
use super::sea::Sea;
use super::stream;
use super::talk::Talk;
use super::water;
use super::walk;

pub const KAMOME_ACTORS: [&str; 1] = ["HyoiKam"];

pub const STAFF_NAME: &str = "HyoiKam";

pub const EVENT_OPTION_CHAR_END: &str = "OPTION_CHAR_END";
pub const EVENT_KAMOME_CALL: &str = "kamome_call";
pub const EVENT_KAMOME_OUT_TURN: &str = "kamome_out_turn";

const EVENT_NAMES: [&str; 3] = [
    EVENT_OPTION_CHAR_END,
    EVENT_KAMOME_CALL,
    EVENT_KAMOME_OUT_TURN,
];

pub const CUT_NAMES: [&str; 4] = ["wait", "change", "descend", "out_turn"];

const CUT_WAIT: i32 = 0;
const CUT_CHANGE: i32 = 1;
const CUT_DESCEND: i32 = 2;
const CUT_OUT_TURN: i32 = 3;
const CUT_NONE: i32 = -1;

pub const ITEM_BUTTON_COUNT: usize = 4;
pub const ITEM_HYOI_PEAR: u8 = 0x83;

const HIO_SPEED_F: f32 = 17.0;
const HIO_ACCEL_F: f32 = 0.7;
const HIO_FLAPPING_SPEED_F: f32 = 23.0;
const HIO_GLIDING_ANG_VEL_Y: i16 = 0x2000;
const HIO_GLIDING_ANG_VEL_X: i16 = 0x0AAA;
const HIO_FLAPPING_ANG_VEL_Y: i16 = 0x2710;
const HIO_FLAPPING_ANG_VEL_X: i16 = 0x1555;
const HIO_MAX_ANGLE_Z: i16 = 0x2000;
const HIO_ANG_VEL_STEP_SCALE: i16 = 5;
const HIO_ANG_VEL_MAX_STEP: i16 = 0x1000;
const HIO_ANG_VEL_MIN_STEP: i16 = 0x0400;
const HIO_FLAP_DURATION: i16 = 30;
const HIO_FLAP_EXHAUSTED_DURATION: i16 = 150;
const HIO_FLAP_ENERGY_DURATION: i16 = 150;

pub const ACTION_STARTING: i8 = 0;
pub const ACTION_ONGOING_1: i8 = 1;
pub const ACTION_ONGOING_2: i8 = 2;
pub const ACTION_ONGOING_3: i8 = 3;
pub const ACTION_ENDING: i8 = -1;

pub const ANM_WAIT1: i32 = 0;
pub const ANM_WAIT2: i32 = 1;
pub const ANM_SING: i32 = 2;

const ANM_COUNT: usize = 3;
const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 2;
const STATE_STOP: u8 = 1;
const STATE_LOOP: u8 = 2;
const ANM_TBL_NONE: i8 = -1;

const ANM_TBL_IDX: [i8; ANM_COUNT] = [0, 1, 2];
const ANM_LOOP: [u8; ANM_COUNT] = [EMODE_LOOP, EMODE_LOOP, EMODE_NONE];
const ANM_PLAY_SPEED: [f32; ANM_COUNT] = [1.0, 1.0, 1.0];
const ANM_TAIL: [i32; ANM_COUNT] = [0, 0, 0];

pub const FRAME_WAIT1: i16 = 60;
pub const FRAME_WAIT2: i16 = 34;
pub const FRAME_SING1: i16 = 31;

const MS_AT_LOCAL_START: [f32; 3] = [100.0, 20.0, 0.0];
const MS_AT_LOCAL_END: [f32; 3] = [-100.0, 20.0, 0.0];
const LINE_BG_LOCAL_END: [f32; 3] = [0.0, 0.0, 500.0];
const CALL_LOCAL_KYORI: [f32; 3] = [0.0, 0.0, 500.0];
const MODEL_BASE_Y: f32 = 30.0;

const FOREST_HAVEN_CENTER: [f32; 3] = [211555.0, 1000.0, 191870.0];
const FOREST_HAVEN_RANGE: f32 = 1300.0;
const SEA_ROOM_HAVEN: i8 = 41;
const SEA_STAGE: &str = "sea";

const ATTN_SPEAK: u32 = 0x0000_0008;
const ATTN_TALKFLAG_NOTALK: u32 = 0x0200_0000;
const ATTN_SPEAK_NOTALK: u32 = ATTN_SPEAK | ATTN_TALKFLAG_NOTALK;

const COND_CANTALK: u32 = 0x0001;
const COND_CANTALKITEM: u32 = 0x0002;

const DESCEND_RAYS: usize = 16;
const DESCEND_STEP_Y: i16 = 0x2000;
const DESCEND_ANGLE_X: i16 = -0x1555;
const DESCEND_YAW: i16 = -0x8000;
const HEAD_LIFT: f32 = 20.0;
const DESCEND_CATCH_Y: f32 = 50.0;
const DESCEND_CATCH_RANGE: f32 = 100.0;
const DESCEND_WAIT: i16 = 30 * 30;

const AREA_OUT_WAIT: i16 = 10;
const AREA_OUT_TIMER: i16 = 180 * 30;

const TURN_Y_ANGLE: [i16; 6] = [0x1000, -0x0C00, -0x1000, 0x0800, 0x0C00, -0x0800];
const ANM_TYPE: [i32; 16] = [
    ANM_WAIT2, ANM_WAIT2, ANM_WAIT2, ANM_WAIT2, ANM_WAIT2, ANM_WAIT2, ANM_WAIT2, ANM_WAIT2,
    ANM_WAIT2, ANM_WAIT2, ANM_WAIT1, ANM_WAIT1, ANM_SING, ANM_WAIT1, ANM_SING, ANM_WAIT1,
];
const TURN_LAST_PICK: i32 = 4;
const ANM_LAST_PICK: i32 = 14;

const LOCK_ANGLE_X_TIMER: i16 = 30;
const NPC_TURN_STEP: i16 = 0x2000;
const WALL_TURN_STEP: i16 = 0x4000;
const LINE_YAW: i16 = 0x2000;

const TURN_SCALE_NPC_Y: i16 = 30;
const TURN_CAP_NPC_Y: i16 = 0x2000;
const TURN_MIN_NPC_Y: i16 = 0x100;
const TURN_SCALE: i16 = 8;
const TURN_CAP: i16 = 0x2000;
const TURN_MIN: i16 = 0x400;
const TURN_SCALE_DESCEND: i16 = 30;
const TURN_SCALE_OUT_TURN: i16 = 16;
const TURN_GATE: i32 = 0x200;

const TG_SPH_RADIUS: f32 = 30.0;
const DAMAGE_FOG: u8 = 150;

const CULL_MIN: [f32; 3] = [-100.0, -300.0, -100.0];
const CULL_MAX: [f32; 3] = [100.0, 500.0, 100.0];

const SE_CV_KAMOME: u32 = 0x4805;
const SE_CTRL_NPC_TO_LINK: u32 = 0x0886;
const SE_HYOI_USE_DEMO: u32 = 0x08EF;

const MORF_CV_FRAME: f32 = 8.0;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const HIT_FRONT: u32 = 0x01;
const HIT_LEFT: u32 = 0x02;
const HIT_RIGHT: u32 = 0x04;
const HIT_WATER: u32 = 0x08;
const HIT_NO_BG: u32 = 0x10;
const HIT_LINE_ALL: u32 = 0x07;

const STATE_NONE: i8 = -1;
const EVENT_STATE_OPTION_END: i8 = 0;
const EVENT_STATE_CALL: i8 = 2;
const EVENT_STATE_SPEAK: i8 = 4;
const EVENT_STATE_SPEAK_NPC: i8 = 5;
const EVENT_STATE_TALK_ITEM: i8 = 6;

pub fn is_kamome(name: &str) -> bool {
    KAMOME_ACTORS.contains(&name)
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

fn calc_timer(value: &mut i16) -> bool {
    if *value != 0 {
        *value = value.wrapping_sub(1);
    }
    *value == 0
}

fn calc_timer_u8(value: &mut u8) -> bool {
    if *value != 0 {
        *value = value.wrapping_sub(1);
    }
    *value == 0
}

fn frame_max(anm: i32) -> i16 {
    match anm {
        ANM_WAIT1 => FRAME_WAIT1,
        ANM_WAIT2 => FRAME_WAIT2,
        ANM_SING => FRAME_SING1,
        _ => FRAME_WAIT1,
    }
}

fn abs_xz(delta: Vec3) -> f32 {
    (delta.x * delta.x + delta.z * delta.z).sqrt()
}

fn abs2_xz(delta: Vec3) -> f32 {
    delta.x * delta.x + delta.z * delta.z
}

fn mult_vec(angle: [i16; 3], local: [f32; 3]) -> Vec3 {
    let (sin_x, cos_x) = walk::from_s16(angle[0]).sin_cos();
    let (sin_y, cos_y) = walk::from_s16(angle[1]).sin_cos();
    let (sin_z, cos_z) = walk::from_s16(angle[2]).sin_cos();
    let z1 = local[2] * cos_x - local[1] * sin_x;
    let y1 = local[2] * sin_x + local[1] * cos_x;
    let x2 = local[0] * cos_y + z1 * sin_y;
    let z2 = z1 * cos_y - local[0] * sin_y;
    let y2 = y1 * cos_z - x2 * sin_z;
    let x3 = x2 * cos_z + y1 * sin_z;
    Vec3::new(x3, y2, z2)
}

fn mult_vec_zxy(angle: [i16; 3], local: [f32; 3]) -> Vec3 {
    mult_vec([angle[2], angle[0], angle[1]], local)
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn line_crossed(collision: Option<&Collision>, from: Vec3, to: Vec3) -> bool {
    match collision {
        Some(collision) => collision.slide(from, to) != to,
        None => false,
    }
}

fn slide(from: Vec3, to: Vec3, collision: Option<&Collision>) -> Vec3 {
    match collision {
        Some(collision) => collision.slide(from, to),
        None => to,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcAct {
    None,
    Wait,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAct {
    None,
    Wait,
    Damage,
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
}

impl Frame {
    fn idle() -> Self {
        Self {
            attribute: EMODE_LOOP,
            state: 0,
            start: 0.0,
            end: 0.0,
            loop_point: 0.0,
            rate: 1.0,
            frame: 0.0,
        }
    }

    fn init(&mut self, end: f32) {
        self.attribute = EMODE_LOOP;
        self.state = 0;
        self.start = 0.0;
        self.end = end;
        self.loop_point = 0.0;
        self.rate = 1.0;
        self.frame = 0.0;
    }

    fn update(&mut self) {
        self.state = 0;
        self.frame += self.rate;
        match self.attribute {
            EMODE_NONE => {
                if self.frame < self.start {
                    self.frame = self.start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
                if self.frame >= self.end {
                    self.frame = self.end - 0.001;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            EMODE_LOOP => {
                while self.frame < self.start {
                    self.state |= STATE_LOOP;
                    if self.loop_point - self.start <= 0.0 {
                        break;
                    }
                    self.frame += self.loop_point - self.start;
                }
                while self.frame >= self.end {
                    self.state |= STATE_LOOP;
                    if self.end - self.loop_point <= 0.0 {
                        break;
                    }
                    self.frame -= self.end - self.loop_point;
                }
            }
            _ => {}
        }
    }

    fn check_pass(&self, pass_frame: f32) -> bool {
        let cur = self.frame;
        let next = cur + self.rate;
        match self.attribute {
            EMODE_NONE => {
                let mut next = next;
                if next < self.start {
                    next = self.start;
                }
                if next >= self.end {
                    next = self.end - 0.001;
                }
                if cur <= next {
                    return cur <= pass_frame && pass_frame < next;
                }
                next <= pass_frame && pass_frame < cur
            }
            EMODE_LOOP => {
                let mut next = next;
                if cur < self.start {
                    while next < self.start {
                        if self.loop_point - self.start <= 0.0 {
                            break;
                        }
                        next += self.loop_point - self.start;
                    }
                    next <= pass_frame && pass_frame < self.loop_point
                } else if self.end <= cur {
                    while next >= self.end {
                        if self.end - self.loop_point <= 0.0 {
                            break;
                        }
                        next -= self.end - self.loop_point;
                    }
                    self.loop_point <= pass_frame && pass_frame < next
                } else if next < self.start {
                    while next < self.start {
                        if self.loop_point - self.start <= 0.0 {
                            break;
                        }
                        next += self.loop_point - self.start;
                    }
                    (self.start <= pass_frame && pass_frame < cur)
                        || (next <= pass_frame && pass_frame < self.loop_point)
                } else if self.end <= next {
                    while next >= self.end {
                        if self.end - self.loop_point <= 0.0 {
                            break;
                        }
                        next -= self.end - self.loop_point;
                    }
                    (cur <= pass_frame && pass_frame < self.end)
                        || (self.loop_point <= pass_frame && pass_frame < next)
                } else if cur <= next {
                    cur <= pass_frame && pass_frame < next
                } else {
                    next <= pass_frame && pass_frame < cur
                }
            }
            _ => false,
        }
    }

    fn play(&mut self) -> bool {
        self.update();
        self.state & STATE_STOP != 0 || self.rate == 0.0
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Kamome {
    pub entered: bool,
    pub possessed: bool,
    pub npc_action: NpcAct,
    pub player_action: PlayerAct,
    pub action_status: i8,
    pub c08: i16,
    pub c0a: i16,
    pub c0c: i16,
    pub wait_timer: i16,
    pub area_out_timer: i16,
    pub damage_fog: u8,
    pub lock_angle_x_timer: i16,
    pub flap_timer: i16,
    pub flap_exhausted_timer: i16,
    pub flap_energy_timer: i16,
    pub event_state: i8,
    pub curr_event_idx_idx: i8,
    pub event_idxs: [i16; 3],
    pub event_conditions: u32,
    pub attn_flags: u32,
    pub event_accept: bool,
    pub return_link: bool,
    pub npc_not_change: bool,
    pub hyoi_kamome: bool,
    pub hit_flags: u32,
    pub anm_idx: i32,
    pub anm_tbl_idx: i8,
    pub reached_anim_end: bool,
    pub morph: Frame,
    pub prev_morf_frame: f32,
    pub shape_angle: [i16; 3],
    pub speed: Vec3,
    pub speed_f: f32,
    pub target_speed_f: f32,
    pub accel_f: f32,
    pub ang_vel_y: i16,
    pub ang_vel_x: i16,
    pub target_ang_vel_y: i16,
    pub target_ang_vel_x: i16,
    pub target_angle_x: i16,
    pub max_y: f32,
    pub min_y: f32,
    pub scale: Vec3,
    pub home_pos: Vec3,
    pub attn_pos: Vec3,
    pub eye_pos: Vec3,
    pub descend_start_pos: Vec3,
    pub descend_start_angle: [i16; 3],
    pub sea_in: bool,
    pub ground_hit: bool,
    pub wall_hit: i32,
    pub tg_hit: bool,
    pub demo_chk_cnt: i16,
    pub demo_chk_flag: i16,
    pub seed: [i32; 3],
}

impl Kamome {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            entered: false,
            possessed: false,
            npc_action: NpcAct::None,
            player_action: PlayerAct::None,
            action_status: ACTION_STARTING,
            c08: 0,
            c0a: 0,
            c0c: 0,
            wait_timer: 0,
            area_out_timer: 0,
            damage_fog: 0,
            lock_angle_x_timer: 0,
            flap_timer: 0,
            flap_exhausted_timer: 0,
            flap_energy_timer: 0,
            event_state: STATE_NONE,
            curr_event_idx_idx: STATE_NONE,
            event_idxs: [-1; 3],
            event_conditions: 0,
            attn_flags: 0,
            event_accept: false,
            return_link: false,
            npc_not_change: false,
            hyoi_kamome: false,
            hit_flags: 0,
            anm_idx: ANM_WAIT2,
            anm_tbl_idx: ANM_TBL_NONE,
            reached_anim_end: false,
            morph: Frame::idle(),
            prev_morf_frame: 0.0,
            shape_angle: [0; 3],
            speed: Vec3::ZERO,
            speed_f: 0.0,
            target_speed_f: 0.0,
            accel_f: HIO_ACCEL_F,
            ang_vel_y: 0,
            ang_vel_x: 0,
            target_ang_vel_y: 0,
            target_ang_vel_x: 0,
            target_angle_x: 0,
            max_y: 0.0,
            min_y: 0.0,
            scale: Vec3::ONE,
            home_pos: Vec3::ZERO,
            attn_pos: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            descend_start_pos: Vec3::ZERO,
            descend_start_angle: [0; 3],
            sea_in: false,
            ground_hit: false,
            wall_hit: -1,
            tg_hit: false,
            demo_chk_cnt: 0,
            demo_chk_flag: 0,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    fn off_line_hit(&mut self) {
        self.hit_flags &= !HIT_LINE_ALL;
    }

    fn on_front_line_hit(&mut self) {
        self.hit_flags |= HIT_FRONT;
    }

    fn is_front_line_hit(&self) -> bool {
        self.hit_flags & HIT_FRONT != 0
    }

    fn on_left_line_hit(&mut self) {
        self.hit_flags |= HIT_LEFT;
    }

    fn is_left_line_hit(&self) -> bool {
        self.hit_flags & HIT_LEFT != 0
    }

    fn on_right_line_hit(&mut self) {
        self.hit_flags |= HIT_RIGHT;
    }

    fn is_right_line_hit(&self) -> bool {
        self.hit_flags & HIT_RIGHT != 0
    }

    fn on_water_hit(&mut self) {
        self.hit_flags |= HIT_WATER;
    }

    fn off_water_hit(&mut self) {
        self.hit_flags &= !HIT_WATER;
    }

    fn is_water_hit(&self) -> bool {
        self.hit_flags & HIT_WATER != 0
    }

    fn on_no_bg_check(&mut self) {
        self.hit_flags |= HIT_NO_BG;
    }

    fn off_no_bg_check(&mut self) {
        self.hit_flags &= !HIT_NO_BG;
    }

    fn is_no_bg_check(&self) -> bool {
        self.hit_flags & HIT_NO_BG != 0
    }

    fn on_hyoi_kamome(&mut self) {
        self.hyoi_kamome = true;
    }

    fn off_hyoi_kamome(&mut self) {
        self.hyoi_kamome = false;
    }

    fn on_event_accept(&mut self) {
        self.event_accept = true;
    }

    fn off_event_accept(&mut self) {
        self.event_accept = false;
    }

    fn return_link(&mut self) {
        self.return_link = true;
    }

    fn off_return_link(&mut self) {
        self.return_link = false;
    }

    fn npc_not_change(&mut self) {
        self.npc_not_change = true;
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }

    fn get_rnd_value(&mut self, low: i32, high: i32) -> i32 {
        low + (self.rnd_f(((high - low) as f32).abs()) as i32)
    }

    pub fn begin(&mut self, actor: &mut Actor, list: &EventList) {
        self.entered = true;
        self.off_hyoi_kamome();
        self.event_state = STATE_NONE;
        self.curr_event_idx_idx = STATE_NONE;
        self.scale *= 1000.0;
        self.max_y = actor.pos.y + self.scale.y;
        self.min_y = actor.pos.y - self.scale.y;

        self.target_speed_f = HIO_SPEED_F;
        self.ang_vel_y = HIO_GLIDING_ANG_VEL_Y;
        self.ang_vel_x = HIO_GLIDING_ANG_VEL_X;
        self.target_ang_vel_y = HIO_GLIDING_ANG_VEL_Y;
        self.target_ang_vel_x = HIO_GLIDING_ANG_VEL_X;

        self.npc_action = NpcAct::Wait;
        self.set_action_reset();

        self.home_pos = actor.pos;
        self.attn_pos = actor.pos;
        self.eye_pos = actor.pos;
        self.shape_angle = actor.angle;

        for slot in 0..EVENT_NAMES.len() {
            let idx = EVENT_NAMES
                .get(slot)
                .copied()
                .and_then(|name| list.event_index(name))
                .and_then(|index| i16::try_from(index).ok())
                .unwrap_or(-1);
            if let Some(entry) = self.event_idxs.get_mut(slot) {
                *entry = idx;
            }
        }
    }

    pub fn xy_check_cb(
        &mut self,
        item_btn: u8,
        select_item: [u8; ITEM_BUTTON_COUNT],
        link_pos: Vec3,
        link_shape_angle: [i16; 3],
        collision: Option<&Collision>,
    ) -> i16 {
        let selected = select_item
            .get(usize::from(item_btn))
            .copied()
            .unwrap_or(0);
        if selected == ITEM_HYOI_PEAR {
            return self.call_demo_start_check(link_pos, link_shape_angle, collision);
        }
        0
    }

    pub fn call_demo_start_check(
        &mut self,
        link_pos: Vec3,
        link_shape_angle: [i16; 3],
        collision: Option<&Collision>,
    ) -> i16 {
        if self.demo_chk_cnt != 0 {
            return self.demo_chk_flag;
        }
        self.demo_chk_cnt = self.demo_chk_cnt.wrapping_add(1);
        self.demo_chk_flag = 0;

        let hyoi_pear_pos = link_pos + Vec3::new(0.0, HEAD_LIFT, 0.0);
        let angle_x = DESCEND_ANGLE_X;
        let mut angle_y = link_shape_angle[1].wrapping_add(DESCEND_YAW);

        for _ in 0..DESCEND_RAYS {
            let mut line_end_pos = hyoi_pear_pos;
            let mut line_start_pos = Vec3::ZERO;
            let mut descend_start_pos = Vec3::ZERO;

            line_end_pos += mult_vec(
                [0, angle_y.wrapping_add(DESCEND_YAW), 0],
                CALL_LOCAL_KYORI,
            );
            line_start_pos += mult_vec([0, angle_y, 0], CALL_LOCAL_KYORI);
            descend_start_pos += mult_vec([angle_x, 0, 0], CALL_LOCAL_KYORI);

            line_end_pos += hyoi_pear_pos;
            line_start_pos += hyoi_pear_pos;
            descend_start_pos += line_start_pos;

            if !line_crossed(collision, descend_start_pos, line_start_pos)
                && !line_crossed(collision, line_start_pos, line_end_pos)
            {
                self.descend_start_pos = descend_start_pos;
                self.descend_start_angle = [
                    angle_x.wrapping_neg(),
                    angle_y.wrapping_add(DESCEND_YAW),
                    0,
                ];
                self.demo_chk_flag = 1;
                return 1;
            }
            angle_y = angle_y.wrapping_add(DESCEND_STEP_Y);
        }
        0
    }

    pub fn xy_event_cb(&mut self) -> i16 {
        self.on_event_accept();
        self.curr_event_idx_idx = 1;
        self.event_idxs.get(1).copied().unwrap_or(-1)
    }

    pub fn order_event_id(&self) -> i16 {
        let idx = self.event_index_at(self.curr_event_idx_idx);
        idx
    }

    fn event_index_at(&self, slot: i8) -> i16 {
        if slot >= 0 {
            let slot = usize::try_from(slot).unwrap_or(0);
            return self.event_idxs.get(slot).copied().unwrap_or(-1);
        }
        if slot == -1 {
            return self.area_out_timer;
        }
        self.wait_timer
    }

    pub fn npc_not_change_flag(&self) -> bool {
        self.npc_not_change
    }

    pub fn event_conditions_of(&self) -> u32 {
        self.event_conditions
    }

    pub fn attention_flags(&self) -> u32 {
        self.attn_flags
    }

    fn set_attention(&mut self, on: bool, use_current: i32, actor: &Actor) {
        if !on {
            return;
        }
        let head_top = actor.pos + Vec3::new(0.0, MODEL_BASE_Y, 0.0);
        self.eye_pos = Vec3::new(head_top.x, head_top.y - HEAD_LIFT, head_top.z);
        self.attn_pos = if use_current != 0 {
            actor.pos
        } else {
            self.home_pos
        };
    }

    fn set_action_reset(&mut self) {
        self.action_status = ACTION_STARTING;
        self.c08 = 0;
        self.c0a = 0;
        self.c0c = 0;
    }

    fn set_npc_action(&mut self, func: NpcAct, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.player_action = PlayerAct::None;
        if self.npc_action != func {
            if self.npc_action != NpcAct::None {
                self.action_status = ACTION_ENDING;
                let old = self.npc_action;
                self.run_npc(old, actor, ctx);
            }
            self.npc_action = func;
            self.set_action_reset();
            self.run_npc(func, actor, ctx);
        }
    }

    fn set_player_action(&mut self, func: PlayerAct, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.npc_action = NpcAct::None;
        if self.player_action != func {
            if self.player_action != PlayerAct::None {
                self.action_status = ACTION_ENDING;
                let old = self.player_action;
                self.run_player(old, actor, ctx);
            }
            self.player_action = func;
            self.set_action_reset();
            self.run_player(func, actor, ctx);
        }
    }

    fn run_npc(&mut self, func: NpcAct, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if func == NpcAct::Wait {
            self.wait_npc_action(actor, ctx);
        }
    }

    fn run_player(&mut self, func: PlayerAct, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        match func {
            PlayerAct::Wait => self.wait_player_action(actor, ctx),
            PlayerAct::Damage => self.damage_player_action(actor, ctx),
            PlayerAct::None => {}
        }
    }

    fn npc_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.npc_action == NpcAct::None {
            self.speed_f = 0.0;
            self.off_hyoi_kamome();
            self.set_npc_action(NpcAct::Wait, actor, ctx);
        }
        let func = self.npc_action;
        self.run_npc(func, actor, ctx);
    }

    fn player_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.player_action == PlayerAct::None {
            self.speed_f = 0.0;
            self.set_player_action(PlayerAct::Wait, actor, ctx);
        }
        let func = self.player_action;
        self.run_player(func, actor, ctx);
    }

    fn return_link_check(&self, run_check: bool, trig_r: bool, trig_start: bool) -> bool {
        !run_check && (trig_r || trig_start)
    }

    fn change_area_check(&mut self, actor: &Actor, player_pos: Vec3, stage: &StageName) -> bool {
        let inside = if stage.0 == SEA_STAGE && actor.room == SEA_ROOM_HAVEN {
            let center = Vec3::from_array(FOREST_HAVEN_CENTER);
            if player_pos.y > center.y {
                abs_xz(player_pos - center) < FOREST_HAVEN_RANGE
            } else {
                false
            }
        } else {
            abs_xz(player_pos - self.home_pos) < self.scale.x
        };
        if inside {
            self.npc_not_change();
            true
        } else {
            self.npc_not_change = false;
            false
        }
    }

    fn area_out_check(&self, actor: &Actor) -> bool {
        abs_xz(actor.pos - self.home_pos) > self.scale.x
    }

    fn get_stick_ang_y(&self, stick: Vec2, camera_angle_y: i16) -> (bool, i16, i16) {
        let mut delta_angle_y: i16 = 0;
        let mut target_angle_z: i16 = 0;
        let mut is_turning = false;
        if stick.x != 0.0 {
            delta_angle_y = (stick.x * -f32::from(self.ang_vel_y)) as i16;
            target_angle_z = (stick.x * f32::from(HIO_MAX_ANGLE_Z)) as i16;
            is_turning = true;
        } else if self.wall_hit >= 0 {
            if self.is_left_line_hit() {
                delta_angle_y = -WALL_TURN_STEP;
                target_angle_z = HIO_MAX_ANGLE_Z;
                is_turning = true;
            } else if self.is_right_line_hit() {
                delta_angle_y = WALL_TURN_STEP;
                target_angle_z = -HIO_MAX_ANGLE_Z;
                is_turning = true;
            } else if self.is_front_line_hit() {
                delta_angle_y = -WALL_TURN_STEP;
                target_angle_z = HIO_MAX_ANGLE_Z;
                is_turning = true;
            }
        }
        (
            is_turning,
            delta_angle_y.wrapping_add(camera_angle_y),
            target_angle_z,
        )
    }

    fn get_angle_x(&mut self, actor: &Actor, stick: Vec2) -> i16 {
        if actor.pos.y < self.min_y || self.ground_hit || self.is_water_hit() {
            self.lock_angle_x_timer = LOCK_ANGLE_X_TIMER;
            self.ang_vel_x.wrapping_neg()
        } else if actor.pos.y > self.max_y {
            self.lock_angle_x_timer = LOCK_ANGLE_X_TIMER;
            self.ang_vel_x
        } else {
            (stick.y * f32::from(self.ang_vel_x)) as i16
        }
    }

    fn npc_turn_check(&self, actor: &Actor, target_angle_y: &mut i16) -> bool {
        let mut is_turning = false;
        if self.is_left_line_hit() {
            *target_angle_y = actor.angle[1].wrapping_sub(NPC_TURN_STEP);
            is_turning = true;
        } else if self.is_right_line_hit() {
            *target_angle_y = actor.angle[1].wrapping_add(NPC_TURN_STEP);
            is_turning = true;
        } else if self.is_front_line_hit() {
            *target_angle_y = actor.angle[1].wrapping_sub(NPC_TURN_STEP);
            is_turning = true;
        }
        is_turning
    }

    fn wait_npc_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.action_status == ACTION_STARTING {
            self.action_status += 1;
            self.set_anm(ANM_WAIT2);

            self.speed_f = HIO_SPEED_F;
            self.target_speed_f = HIO_SPEED_F;
            self.ang_vel_y = HIO_GLIDING_ANG_VEL_Y;
            self.ang_vel_x = HIO_GLIDING_ANG_VEL_X;
            self.target_ang_vel_y = HIO_GLIDING_ANG_VEL_Y;
            self.target_ang_vel_x = HIO_GLIDING_ANG_VEL_X;

            self.c08 = self.get_rnd_value(90, 60) as i16;
            self.c0c = self.get_rnd_value(10, 80) as i16;
        } else if self.action_status != ACTION_ENDING {
            if self.change_area_check(actor, ctx.link_pos, ctx.stage) {
                self.attn_flags |= ATTN_SPEAK_NOTALK;
                self.event_state = EVENT_STATE_TALK_ITEM;
            } else {
                self.attn_flags &= !ATTN_SPEAK_NOTALK;
                self.event_state = STATE_NONE;
            }

            let mut target_angle_x: i16 = 0;
            let mut target_angle_y = actor.angle[1];
            if self.action_status == ACTION_ONGOING_1 {
                if self.area_out_check(actor) {
                    self.action_status = ACTION_ONGOING_2;
                    self.c08 = 60;
                    self.set_anm(ANM_WAIT1);
                } else {
                    if calc_timer(&mut self.c0c) {
                        self.c0c = self.get_rnd_value(10, 80) as i16;
                        let pick = self.get_rnd_value(0, ANM_LAST_PICK).max(0);
                        let anm = ANM_TYPE.get(pick as usize).copied().unwrap_or(ANM_WAIT2);
                        self.set_anm(anm);
                    }
                    let turning = self.npc_turn_check(actor, &mut target_angle_y);
                    if !turning && calc_timer(&mut self.c08) {
                        self.action_status = ACTION_ONGOING_3;
                        self.c08 = self.get_rnd_value(60, 60) as i16;
                        let pick = self.get_rnd_value(0, TURN_LAST_PICK).max(0);
                        self.c0a = TURN_Y_ANGLE.get(pick as usize).copied().unwrap_or(0x1000);
                    }
                }
            } else if self.action_status == ACTION_ONGOING_2 {
                target_angle_y = bearing(actor.pos, self.home_pos);
                if calc_timer(&mut self.c08) && !self.area_out_check(actor) {
                    self.action_status = ACTION_ONGOING_1;
                    self.set_anm(ANM_WAIT2);
                    self.c08 = self.get_rnd_value(90, 60) as i16;
                }
            } else if self.action_status == ACTION_ONGOING_3 {
                if self.npc_turn_check(actor, &mut target_angle_y) {
                    self.action_status = ACTION_ONGOING_1;
                    self.c08 = self.get_rnd_value(90, 60) as i16;
                } else {
                    target_angle_y = self.c0a;
                    if calc_timer(&mut self.c08) {
                        self.action_status = ACTION_ONGOING_1;
                        self.c08 = self.get_rnd_value(90, 60) as i16;
                    }
                }
            }

            if self.anm_idx == ANM_SING && self.morph.check_pass(MORF_CV_FRAME) {
                ctx.se(SE_CV_KAMOME);
            }

            if actor.pos.y < self.home_pos.y {
                target_angle_x = -HIO_GLIDING_ANG_VEL_X;
            }

            let mut angle_diff_y = actor.angle[1].wrapping_sub(target_angle_y);
            if i32::from(angle_diff_y).abs() > TURN_GATE {
                actor.angle[1] = walk::add_calc_angle_s(
                    actor.angle[1],
                    target_angle_y,
                    TURN_SCALE_NPC_Y,
                    TURN_CAP_NPC_Y,
                    TURN_MIN_NPC_Y,
                );
                self.shape_angle[1] = actor.angle[1];
                if angle_diff_y > HIO_MAX_ANGLE_Z {
                    angle_diff_y = HIO_MAX_ANGLE_Z;
                } else if angle_diff_y < -HIO_MAX_ANGLE_Z {
                    angle_diff_y = -HIO_MAX_ANGLE_Z;
                }
            }

            actor.angle[0] = walk::add_calc_angle_s(
                actor.angle[0],
                target_angle_x,
                TURN_SCALE,
                TURN_CAP,
                TURN_MIN,
            );
            actor.angle[2] = walk::add_calc_angle_s(
                actor.angle[2],
                angle_diff_y,
                TURN_SCALE,
                TURN_CAP,
                TURN_MIN,
            );
            self.set_attention(true, 0, actor);
        }
    }

    fn key_proc(&mut self, trig_a: bool) {
        if calc_timer(&mut self.flap_exhausted_timer) {
            if trig_a {
                self.flap_timer = HIO_FLAP_DURATION;
            }
            if calc_timer(&mut self.flap_timer) {
                self.target_speed_f = HIO_FLAPPING_SPEED_F;
                self.target_ang_vel_y = HIO_FLAPPING_ANG_VEL_Y;
                self.target_ang_vel_x = HIO_FLAPPING_ANG_VEL_X;

                if calc_timer(&mut self.flap_energy_timer) {
                    self.flap_exhausted_timer = HIO_FLAP_EXHAUSTED_DURATION;
                }
            } else {
                self.target_speed_f = HIO_SPEED_F;
                self.target_ang_vel_y = HIO_GLIDING_ANG_VEL_Y;
                self.target_ang_vel_x = HIO_GLIDING_ANG_VEL_X;

                self.flap_energy_timer = self.flap_energy_timer.wrapping_add(1);
                if self.flap_energy_timer > HIO_FLAP_ENERGY_DURATION {
                    self.flap_energy_timer = HIO_FLAP_ENERGY_DURATION;
                }
            }
        } else {
            self.flap_timer = 0;
            self.flap_energy_timer = HIO_FLAP_ENERGY_DURATION;
        }
    }

    fn wait_player_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.action_status == ACTION_STARTING {
            self.action_status += 1;
            self.on_hyoi_kamome();
            self.set_anm(ANM_WAIT1);
            self.speed_f = HIO_SPEED_F;
            self.target_speed_f = HIO_SPEED_F;
            self.ang_vel_y = HIO_GLIDING_ANG_VEL_Y;
            self.ang_vel_x = HIO_GLIDING_ANG_VEL_X;
            self.target_ang_vel_y = HIO_GLIDING_ANG_VEL_Y;
            self.target_ang_vel_x = HIO_GLIDING_ANG_VEL_X;
            self.c08 = self.get_rnd_value(60, 90) as i16;
            self.c0a = 0;
            self.flap_timer = 0;
            self.flap_exhausted_timer = 0;
            self.flap_energy_timer = HIO_FLAP_ENERGY_DURATION;
        } else if self.action_status == ACTION_ENDING {
            self.off_hyoi_kamome();
        } else {
            self.key_proc(ctx.trig_a);
            calc_timer(&mut self.lock_angle_x_timer);
            if !ctx.run_check && self.area_out_check(actor) {
                self.event_state = EVENT_STATE_CALL;
            }

            let camera_angle_y = walk::to_s16(ctx.camera.0);
            let (turning, target_angle_y, target_angle_z) =
                self.get_stick_ang_y(ctx.stick, camera_angle_y);
            if turning {
                actor.angle[1] = walk::add_calc_angle_s(
                    actor.angle[1],
                    target_angle_y,
                    TURN_SCALE,
                    TURN_CAP,
                    TURN_MIN,
                );
            }
            self.shape_angle[1] = actor.angle[1];

            if self.lock_angle_x_timer == 0 {
                self.target_angle_x = self.get_angle_x(actor, ctx.stick);
            }

            actor.angle[0] = walk::add_calc_angle_s(
                actor.angle[0],
                self.target_angle_x,
                TURN_SCALE,
                TURN_CAP,
                TURN_MIN,
            );
            actor.angle[2] = walk::add_calc_angle_s(
                actor.angle[2],
                target_angle_z,
                TURN_SCALE,
                TURN_CAP,
                TURN_MIN,
            );
            self.set_attention(true, 1, actor);

            if self.anm_idx != ANM_WAIT2 {
                if self.flap_timer != 0 {
                    self.set_anm(ANM_WAIT2);
                } else if calc_timer(&mut self.c08) {
                    self.c08 = self.get_rnd_value(90, 90) as i16;
                    self.set_anm(ANM_SING);
                }

                if self.anm_idx == ANM_SING {
                    if self.reached_anim_end {
                        self.set_anm(ANM_WAIT1);
                    }

                    if self.anm_idx == ANM_SING && self.morph.check_pass(MORF_CV_FRAME) {
                        ctx.se(SE_CV_KAMOME);
                    }
                }
            } else if self.reached_anim_end && self.flap_timer == 0 {
                self.set_anm(ANM_SING);
            }

            if self.tg_hit {
                self.set_player_action(PlayerAct::Damage, actor, ctx);
            }
        }
    }

    fn damage_player_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.action_status == ACTION_STARTING {
            ctx.vibration();
            self.damage_fog = DAMAGE_FOG;
            self.action_status += 1;
            self.on_hyoi_kamome();
            self.set_anm(ANM_WAIT1);
            self.speed_f = 0.0;
            ctx.se(SE_CV_KAMOME);
            self.c08 = 30;
        } else if self.action_status == ACTION_ENDING {
            self.off_hyoi_kamome();
        } else {
            self.shape_angle[1] = self.shape_angle[1].wrapping_add(NPC_TURN_STEP);
            actor.angle[1] = self.shape_angle[1];
            self.set_attention(true, 1, actor);
            if calc_timer(&mut self.c08) {
                self.return_link();
            }
        }
    }

    fn event_order(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.event_accept {
            return;
        }
        if self.event_state == EVENT_STATE_SPEAK_NPC || self.event_state == EVENT_STATE_SPEAK {
            if !self.possessed {
                self.event_conditions |= COND_CANTALK;
                if self.event_state == EVENT_STATE_SPEAK_NPC {
                    ctx.order_speak();
                }
            } else {
                self.event_state = STATE_NONE;
            }
        } else if self.event_state == EVENT_STATE_TALK_ITEM {
            if !self.possessed {
                self.event_conditions |= COND_CANTALK | COND_CANTALKITEM;
            } else {
                self.event_state = STATE_NONE;
            }
        } else if self.event_state != STATE_NONE && self.event_state < 3 {
            self.curr_event_idx_idx = self.event_state;
            let idx = self.event_index_at(self.curr_event_idx_idx);
            ctx.order_event(idx);
        }
    }

    fn check_order(&mut self, talk_command: bool) {
        if !talk_command {
            return;
        }
        if self.event_state != EVENT_STATE_SPEAK_NPC
            && self.event_state != EVENT_STATE_SPEAK
            && self.event_state != EVENT_STATE_TALK_ITEM
        {
            return;
        }
        self.event_state = STATE_NONE;
    }

    fn check_command_talk(&mut self, talk_command: bool, talk_xy: bool) -> bool {
        if talk_command {
            if talk_xy {
                if self.event_state == EVENT_STATE_TALK_ITEM {
                    self.event_state = STATE_NONE;
                }
                return false;
            }
            return true;
        }
        false
    }

    fn return_link_player(&mut self) {
        self.possessed = false;
        self.off_hyoi_kamome();
    }

    fn event_proc(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) -> bool {
        if ctx.demo_accept {
            if self.event_state != STATE_NONE {
                if self.event_state == EVENT_STATE_OPTION_END {
                    let opt = ctx.list.event_index(EVENT_OPTION_CHAR_END);
                    if opt.is_some_and(|index| ctx.active.start_check(index))
                        || opt.is_some_and(|index| ctx.active.end_check(index))
                    {
                        ctx.set_talk_partner();
                        ctx.se(SE_CTRL_NPC_TO_LINK);
                    } else {
                        self.off_return_link();
                        self.event_state = STATE_NONE;
                    }
                }
                if self.event_state != STATE_NONE {
                    self.on_event_accept();
                    self.event_state = STATE_NONE;
                }
            }
        }

        if ctx.run_check && !self.check_command_talk(ctx.talk_command, ctx.talk_xy) {
            if ctx.has_staff {
                let act = ctx.act_idx;
                if act == CUT_NONE {
                    ctx.end_staff_cut();
                } else {
                    if ctx.cut_advance {
                        self.event_init(act, actor, ctx);
                    }
                    if self.event_action(act, actor, ctx) {
                        ctx.end_staff_cut();
                    }
                }
            }

            if self.event_accept {
                let idx = self.ordered_event_index();
                if let Some(index) = idx
                    && ctx.active.end_check(index)
                {
                    self.event_end(ctx);
                }
                return true;
            }
            return true;
        }
        false
    }

    fn ordered_event_index(&self) -> Option<usize> {
        let idx = self.event_index_at(self.curr_event_idx_idx);
        usize::try_from(idx).ok()
    }

    fn event_end(&mut self, _ctx: &mut Ctx<'_, '_>) {
        self.off_event_accept();
        if self.curr_event_idx_idx == 0 {
            self.return_link_player();
            self.off_return_link();
        }
        self.curr_event_idx_idx = STATE_NONE;
    }

    fn event_init(&mut self, act: i32, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        match act {
            CUT_WAIT => {
                self.wait_timer = ctx.cut_timer.unwrap_or(0) as i16;
            }
            CUT_CHANGE => self.initial_change_event(actor),
            CUT_DESCEND => self.initial_descend_event(actor, ctx),
            CUT_OUT_TURN => self.initial_area_out_event(actor),
            _ => {}
        }
    }

    fn event_action(&mut self, act: i32, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) -> bool {
        match act {
            CUT_WAIT => {
                self.set_attention(true, 1, actor);
                calc_timer(&mut self.wait_timer)
            }
            CUT_CHANGE => true,
            CUT_DESCEND => self.action_descend_event(actor, ctx),
            CUT_OUT_TURN => self.action_area_out_event(actor),
            _ => true,
        }
    }

    fn initial_change_event(&mut self, _actor: &mut Actor) {
        self.possessed = true;
        self.off_no_bg_check();
    }

    fn initial_descend_event(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        actor.pos = self.descend_start_pos;
        actor.angle = self.descend_start_angle;
        self.shape_angle = actor.angle;

        self.target_speed_f = HIO_SPEED_F;
        self.speed_f = HIO_SPEED_F;
        self.ang_vel_y = HIO_GLIDING_ANG_VEL_Y;
        self.ang_vel_x = HIO_GLIDING_ANG_VEL_X;
        self.target_ang_vel_y = HIO_GLIDING_ANG_VEL_Y;
        self.target_ang_vel_x = HIO_GLIDING_ANG_VEL_X;

        ctx.se(SE_HYOI_USE_DEMO);
        self.on_no_bg_check();
        self.wait_timer = DESCEND_WAIT;
    }

    fn action_descend_event(&mut self, actor: &mut Actor, ctx: &Ctx<'_, '_>) -> bool {
        let link_head_top = ctx.link_head_top + Vec3::new(0.0, HEAD_LIFT, 0.0);
        self.set_attention(true, 1, actor);

        if actor.pos.y < link_head_top.y {
            actor.pos.y = link_head_top.y;
        }
        if actor.pos.y < link_head_top.y + DESCEND_CATCH_Y {
            actor.angle[0] = walk::add_calc_angle_s(
                actor.angle[0],
                0,
                TURN_SCALE_DESCEND,
                TURN_CAP,
                TURN_MIN,
            );
            self.shape_angle[0] = actor.angle[0];
        }

        let delta = link_head_top - actor.pos;
        if abs2_xz(delta) < DESCEND_CATCH_RANGE * DESCEND_CATCH_RANGE {
            return true;
        }

        calc_timer(&mut self.wait_timer)
    }

    fn initial_area_out_event(&mut self, actor: &mut Actor) {
        self.wait_timer = AREA_OUT_WAIT;
        self.area_out_timer = AREA_OUT_TIMER;
        actor.angle[0] = 0;
        actor.angle[2] = 0;
    }

    fn action_area_out_event(&mut self, actor: &mut Actor) -> bool {
        if calc_timer(&mut self.wait_timer) {
            self.on_no_bg_check();
            self.speed_f = 0.0;

            let aim = bearing(actor.pos, self.home_pos);
            let angle_diff = actor.angle[1].wrapping_sub(aim);
            if i32::from(angle_diff) != 0 {
                actor.angle[1] = walk::add_calc_angle_s(
                    actor.angle[1],
                    aim,
                    TURN_SCALE_OUT_TURN,
                    TURN_CAP,
                    TURN_MIN,
                );
            } else {
                self.speed_f = HIO_SPEED_F;
            }
            self.shape_angle[1] = actor.angle[1];

            actor.angle[2] = angle_diff;
            if actor.angle[2] > HIO_MAX_ANGLE_Z {
                actor.angle[2] = HIO_MAX_ANGLE_Z;
            } else if actor.angle[2] < -HIO_MAX_ANGLE_Z {
                actor.angle[2] = -HIO_MAX_ANGLE_Z;
            }

            if !self.area_out_check(actor) {
                self.off_no_bg_check();
                return true;
            }
        }

        self.shape_angle[0] = walk::add_calc_angle_s(
            self.shape_angle[0],
            actor.angle[0],
            TURN_SCALE,
            TURN_CAP,
            TURN_MIN,
        );
        self.shape_angle[2] = walk::add_calc_angle_s(
            self.shape_angle[2],
            actor.angle[2],
            TURN_SCALE,
            TURN_CAP,
            TURN_MIN,
        );

        self.set_attention(true, 1, actor);
        calc_timer(&mut self.area_out_timer)
    }

    fn set_anm(&mut self, anm_idx: i32) {
        self.anm_idx = anm_idx;
        let slot = usize::try_from(anm_idx).ok().filter(|slot| *slot < ANM_COUNT);
        let Some(slot) = slot else {
            return;
        };
        let tbl_idx = ANM_TBL_IDX.get(slot).copied().unwrap_or(ANM_TBL_NONE);
        let play_speed = ANM_PLAY_SPEED.get(slot).copied().unwrap_or(1.0);
        if tbl_idx == self.anm_tbl_idx && play_speed == self.morph.rate {
            return;
        }
        self.anm_tbl_idx = tbl_idx;
        self.prev_morf_frame = 0.0;
        self.reached_anim_end = false;
        let loop_mode = ANM_LOOP.get(slot).copied().unwrap_or(EMODE_LOOP);
        let tail = ANM_TAIL.get(slot).copied().unwrap_or(0);
        self.morph.init(f32::from(frame_max(anm_idx)));
        self.morph.attribute = loop_mode;
        self.morph.rate = play_speed;
        self.morph.frame = 0.0;
        self.morph.loop_point = self.morph.frame;
        if tail < 0 {
            self.morph.frame = self.morph.end;
        }
    }

    fn set_collision(&mut self, actor: &Actor, ctx: &Ctx<'_, '_>) {
        let centre = actor.pos;
        let at_start = centre + mult_vec_zxy(self.shape_angle, MS_AT_LOCAL_START);
        let at_end = centre + mult_vec_zxy(self.shape_angle, MS_AT_LOCAL_END);
        self.tg_hit = walk::swept(at_start, at_end, ctx.link_pos, TG_SPH_RADIUS);
    }

    fn set_line_bg_check(&mut self, actor: &Actor, ctx: &Ctx<'_, '_>) {
        self.off_line_hit();
        let mut line_angle = self.shape_angle;

        let end = actor.pos + mult_vec_zxy(line_angle, LINE_BG_LOCAL_END);
        if line_crossed(ctx.collision, actor.pos, end) {
            self.on_front_line_hit();
        }

        line_angle[1] = self.shape_angle[1].wrapping_add(LINE_YAW);
        let end = actor.pos + mult_vec_zxy(line_angle, LINE_BG_LOCAL_END);
        if line_crossed(ctx.collision, actor.pos, end) {
            self.on_left_line_hit();
        }

        line_angle[1] = self.shape_angle[1].wrapping_sub(LINE_YAW);
        let end = actor.pos + mult_vec_zxy(line_angle, LINE_BG_LOCAL_END);
        if line_crossed(ctx.collision, actor.pos, end) {
            self.on_right_line_hit();
        }
    }

    fn animation_play(&mut self) {
        let reached = self.morph.play();
        self.reached_anim_end = reached;

        if self.morph.rate < 0.0 {
            if self.morph.frame > self.prev_morf_frame {
                self.reached_anim_end = true;
            }
        } else if self.morph.frame < self.prev_morf_frame {
            self.reached_anim_end = true;
        }

        self.prev_morf_frame = self.morph.frame;
    }

    fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.demo_chk_cnt = 0;
        self.tg_hit = false;

        calc_timer_u8(&mut self.damage_fog);

        if !self.return_link {
            if self.check_now_pos_move(ctx) {
                self.accel_f = HIO_ACCEL_F;

                if self.speed_f < self.target_speed_f {
                    self.speed_f += self.accel_f;
                    if self.speed_f > self.target_speed_f {
                        self.speed_f = self.target_speed_f;
                    }
                } else if self.speed_f > self.target_speed_f {
                    self.speed_f -= self.accel_f;
                    if self.speed_f < self.target_speed_f {
                        self.speed_f = self.target_speed_f;
                    }
                }

                self.ang_vel_y = walk::add_calc_angle_s(
                    self.ang_vel_y,
                    self.target_ang_vel_y,
                    HIO_ANG_VEL_STEP_SCALE,
                    HIO_ANG_VEL_MAX_STEP,
                    HIO_ANG_VEL_MIN_STEP,
                );
                self.ang_vel_x = walk::add_calc_angle_s(
                    self.ang_vel_x,
                    self.target_ang_vel_x,
                    HIO_ANG_VEL_STEP_SCALE,
                    HIO_ANG_VEL_MAX_STEP,
                    HIO_ANG_VEL_MIN_STEP,
                );

                let cos_x_speed = self.speed_f * stream::cos_s(actor.angle[0]);
                self.speed.y = self.speed_f * -stream::sin_s(actor.angle[0]);
                self.speed.x = cos_x_speed * stream::sin_s(actor.angle[1]);
                self.speed.z = cos_x_speed * stream::cos_s(actor.angle[1]);

                self.move_actor(actor, ctx);
            }

            if !self.is_no_bg_check() {
                self.acch_correction(actor, ctx);
                if self.sea_in {
                    if !self.is_water_hit() {
                        self.on_water_hit();
                        ctx.spark_splash();
                        ctx.spark_ripple();
                    }
                } else {
                    self.off_water_hit();
                }
            }

            self.set_line_bg_check(actor, ctx);

            if self.check_now_pos_move(ctx) {
                self.animation_play();
            }
        }

        let in_event = self.event_proc(actor, ctx);

        if !in_event {
            if self.possessed {
                if self.return_link {
                    self.event_state = EVENT_STATE_OPTION_END;
                } else {
                    self.player_action(actor, ctx);
                    if self.return_link_check(ctx.run_check, ctx.trig_r, ctx.trig_start) {
                        self.return_link();
                    }
                }
            } else {
                self.check_order(ctx.talk_command);
                self.npc_action(actor, ctx);
            }

            self.shape_angle[0] = walk::add_calc_angle_s(
                self.shape_angle[0],
                actor.angle[0],
                TURN_SCALE,
                TURN_CAP,
                TURN_MIN,
            );
            self.shape_angle[2] = walk::add_calc_angle_s(
                self.shape_angle[2],
                actor.angle[2],
                TURN_SCALE,
                TURN_CAP,
                TURN_MIN,
            );
        }

        self.event_order(ctx);

        if !in_event && self.possessed {
            self.set_collision(actor, ctx);
        }
    }

    fn check_now_pos_move(&self, ctx: &Ctx<'_, '_>) -> bool {
        if !ctx.run_check {
            return true;
        }
        if ctx.talk_command {
            return true;
        }
        if self.event_accept && !self.return_link {
            return true;
        }
        ctx.has_staff
    }

    fn move_actor(&mut self, actor: &mut Actor, ctx: &Ctx<'_, '_>) {
        actor.pos = slide(actor.pos, actor.pos + self.speed, ctx.collision);
        self.wall_hit = self.wall_hit_check(actor.pos, ctx.collision);
    }

    fn acch_correction(&mut self, actor: &mut Actor, ctx: &Ctx<'_, '_>) {
        self.sea_in = ctx.sea_in;
        if let Some(height) = super::ground::surface(ctx.collision, actor.pos) {
            self.ground_hit = actor.pos.y <= height;
            if self.ground_hit {
                actor.pos.y = height;
            }
        } else {
            self.ground_hit = false;
        }
    }

    fn wall_hit_check(&self, pos: Vec3, collision: Option<&Collision>) -> i32 {
        let Some(collision) = collision else {
            return -1;
        };
        let probe = pos + Vec3::new(0.0, HEAD_LIFT, 0.0);
        let yaw = walk::from_s16(self.shape_angle[1]);
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        for index in 0..2_usize {
            let offset = if index == 0 { ACCH_CIR_0 } else { ACCH_CIR_1 };
            let at = probe + right * offset;
            let to = at + right;
            if collision.slide(at, to) != to {
                return index as i32;
            }
        }
        -1
    }
}

const ACCH_CIR_0: f32 = 20.0;
const ACCH_CIR_1: f32 = -20.0;

struct Ctx<'a, 'b> {
    link_pos: Vec3,
    link_head_top: Vec3,
    collision: Option<&'b Collision>,
    stage: &'b StageName,
    sea_in: bool,
    stick: Vec2,
    camera: CameraYaw,
    trig_a: bool,
    trig_r: bool,
    trig_start: bool,
    talk_command: bool,
    talk_xy: bool,
    demo_accept: bool,
    run_check: bool,
    has_staff: bool,
    cut_advance: bool,
    cut_timer: Option<i32>,
    act_idx: i32,
    events: &'a mut SimEvents,
    active: &'a mut ActiveEvent,
    ev_flags: &'a mut EventFlags,
    list: &'b EventList,
    entity: Entity,
}

impl Ctx<'_, '_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn spark_splash(&mut self) {}

    fn spark_ripple(&mut self) {}

    fn vibration(&mut self) {}

    fn set_talk_partner(&mut self) {}

    fn order_speak(&mut self) {}

    fn order_event(&mut self, index: i16) {
        let Some(slot) = usize::try_from(index).ok() else {
            return;
        };
        let list = self.list;
        let entity = self.entity;
        self.active.order(list, slot, entity);
    }

    fn end_staff_cut(&mut self) {
        let list = self.list;
        self.active
            .end_staff_cut(list, self.ev_flags, STAFF_NAME);
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    stage: Res<'w, StageName>,
    sea: Option<Res<'w, Sea>>,
    camera: Res<'w, CameraYaw>,
    intent: Res<'w, MoveIntent>,
    do_intent: Res<'w, DoIntent>,
    target_intent: Res<'w, TargetIntent>,
    partner: Res<'w, ItemPartner>,
    list: Res<'w, EventList>,
    players: Query<'w, 's, (Entity, &'static Player), Without<Kamome>>,
    talks: Query<'w, 's, Entity, (With<Talk>, Without<Kamome>)>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut active: ResMut<ActiveEvent>,
    mut ev_flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    mut birds: Query<(Entity, &mut Actor, &mut Kamome), (With<Kamome>, Without<Player>)>,
) {
    let player = watch.players.iter().next();
    for (entity, mut actor, mut bird) in &mut birds {
        let actor = &mut *actor;
        let bird = &mut *bird;
        if !bird.entered {
            bird.begin(actor, &watch.list);
        }
        let link = player.map(|(_, body)| *body);
        let link_pos = link.map_or(actor.pos, |body| body.pos);
        let link_head_top = link_pos + Vec3::new(0.0, HEAD_LIFT, 0.0);
        let talk_command = watch.talks.iter().any(|talk| talk == entity);
        let staff_cut = active.staff_cut(&watch.list, STAFF_NAME);
        let has_staff = staff_cut.is_some();
        let cut = staff_cut.map(|(index, _)| index).unwrap_or(0);
        let cut_advance = staff_cut.is_some_and(|(_, advance)| advance);
        let act_idx = match watch.list.cut_name(cut) {
            Some(name) => CUT_NAMES
                .iter()
                .position(|entry| *entry == name)
                .and_then(|index| i32::try_from(index).ok())
                .unwrap_or(CUT_NONE),
            None => CUT_NONE,
        };
        let run_check = has_staff
            || [
                EVENT_OPTION_CHAR_END,
                EVENT_KAMOME_CALL,
                EVENT_KAMOME_OUT_TURN,
            ]
            .iter()
            .any(|name| {
                watch
                    .list
                    .event_index(name)
                    .is_some_and(|slot| active.stage_of(slot).is_some())
            });
        let (sea_in, _) = water::probe(watch.sea.as_deref(), actor.pos.x, actor.pos.z);
        let mut ctx = Ctx {
            link_pos,
            link_head_top,
            collision: watch.collision.as_deref(),
            stage: &watch.stage,
            sea_in,
            stick: watch.intent.axis,
            camera: *watch.camera,
            trig_a: watch.do_intent.0,
            trig_r: watch.target_intent.0,
            trig_start: false,
            talk_command,
            talk_xy: watch.partner.entity == Some(entity),
            demo_accept: false,
            run_check,
            has_staff,
            cut_advance,
            cut_timer: watch.list.cut_number(cut, "timer"),
            act_idx,
            events: &mut events,
            active: &mut active,
            ev_flags: &mut ev_flags,
            list: &watch.list,
            entity,
        };
        bird.execute(actor, &mut ctx);
    }
}