use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, MemBits, StoryBits};
use super::player::Player;
use super::stream::{add_calc_angle_s2, cos_s, sin_s};
use super::walk::to_s16;
use crate::assets::MsgArchives;
use crate::text::driver::{Counters, TextFlags};
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs, fop_msgm_message_set,
    fop_msgm_message_set_actor,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
};

pub const NPC_BMSW_ACTORS: [&str; 1] = ["Bmsw"];

pub fn is_npc_bmsw(name: &str) -> bool {
    NPC_BMSW_ACTORS.contains(&name)
}

pub const STAFF_BMSW: &str = "Bmsw";
pub const EVENT_SHIWAKEGAME: &str = "SHIWAKEGAME";
pub const EVENT_SHIWAKEGAME2: &str = "SHIWAKEGAME2";

const FLAG_2701: u16 = 0x2701;
const FLAG_1A01: u16 = 0x1A01;
const FLAG_1A02: u16 = 0x1A02;
const REG_C203: u16 = 0xC203;
const REG_8AFF: u16 = 0x8AFF;
const TMP_FLAG_0320: u32 = 0x0320;

const HIO_M04: f32 = -20.0;
const HIO_MAX_HEAD_X: i16 = 0x1FFE;
const HIO_MAX_HEAD_Y: i16 = 3000;
const HIO_MAX_BACKBONE_X: i16 = 0;
const HIO_MAX_BACKBONE_Y: i16 = 0x1C70;
const HIO_MIN_HEAD_X: i16 = -0x9C4;
const HIO_MIN_HEAD_Y: i16 = -3000;
const HIO_MIN_BACKBONE_X: i16 = 0;
const HIO_MIN_BACKBONE_Y: i16 = -0x1C70;
const HIO_MAX_TURN_STEP: i16 = 0x1000;
const HIO_MAX_HEAD_TURN_VEL: i16 = 0x800;
const HIO_ATTN_Y_OFFSET: f32 = 50.0;
const HIO_MAX_ATTN_ANGLE_Y: i16 = 0x2000;
const HIO_MAX_ATTN_DIST_XZ: f32 = 300.0;

const HIO_32: i16 = 0xF;
const HIO_34: i16 = 0x3C;
const HIO_44: f32 = 40.0;
const HIO_50: f32 = 0.8;
const HIO_54: f32 = 0.75;

const ATTENTION_EXTEND_DIST: f32 = 40.0;
const ATTENTION_EXTEND_S: i16 = 0xC4E;
const PLAYER_HEAD_TOP: f32 = 40.0;
const BTP_FRAME_MAX: u8 = 8;
const BTP_TIMER_MIN: f32 = 30.0;
const BTP_TIMER_RANGE: f32 = 100.0;
const SHIWAKE_TIMER_START: i32 = 20000;
const SHIWAKE_TIMER_BEAT: i32 = 1000;
const SHIWAKE_TIMER_BEAT_BIG: i32 = 10000;
const SHIWAKE_TIMER_GATE: i32 = 10000;
const SHIWAKE_LIMIT_MS: i32 = HIO_32 as i32 * 1000;
const TIMER_DELETE_FRAMES: i32 = 3;
const MS_PER_FRAME: i32 = 1000 / 60;
const MINI_GAME_LETTER: u8 = 7;
const MAIL_SLOTS: usize = 3;

const SE_LETTER_GAME_TIMER_0: u32 = 0x08AF;
const SE_LETTER_GAME_CURSOR: u32 = 0x08B0;
const SE_LETTER_GAME_OK: u32 = 0x08B1;
const SE_LETTER_GAME_NEW: u32 = 0x69A5;
const SE_LETTER_IN_BOX: u32 = 0x69A6;
const SE_MINIGAME_TIMER_30: u32 = 0x08AD;
const SE_MINIGAME_TIMER_10: u32 = 0x08AE;
const SE_MINIGAME_RIGHT: u32 = 0x08A5;
const SE_MINIGAME_WRONG: u32 = 0x08A6;
const SE_LK_SW_KAZEKIRI_S: u32 = 0x2800;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const CULL_MIN: [f32; 3] = [-120.0, 0.0, -120.0];
const CULL_MAX: [f32; 3] = [120.0, 300.0, 120.0];

const ACTION_STARTING: i8 = 0;
const ACTION_ONGOING: i8 = 1;
const ACTION_UNK_2: i8 = 2;
const ACTION_ENDING: i8 = -1;

const ACTION_WAIT: u8 = 0;
const ACTION_SHIWAKE: u8 = 1;

const MODE_WAIT: i8 = 1;
const MODE_TALK: i8 = 2;
const MODE_SHIWAKE: i8 = 3;

const ORDER_NONE: i8 = 0;
const ORDER_SPEAK: i8 = 1;
const ORDER_CAN_TALK: i8 = 2;

const ANM_WAIT_01: i8 = 0;
const ANM_WAIT_02: i8 = 1;
const ANM_TALK_01: i8 = 2;
const ANM_TALK_02: i8 = 3;
const ANM_TALK_03: i8 = 4;
const ANM_SHIWAKE_01: i8 = 5;
const ANM_SHIWAKE_02: i8 = 6;

const ANM_FRAME_MAX: [u16; 7] = [39, 39, 39, 57, 40, 40, 70];

const REG0_S0: i16 = 0;
const REG0_S1: i16 = 0;
const REG0_S2: i16 = 0;
const REG0_S3: i16 = 0;
const REG0_F0: f32 = 0.0;
const REG0_F2: f32 = 0.0;
const REG0_F3: f32 = 0.0;
const REG0_F4: f32 = 0.0;
const REG0_F10: f32 = 0.0;
const REG0_F11: f32 = 0.0;

const NO_STATUS: u8 = 0xFF;
const MSG_CONTINUES: u8 = 0x0F;
const MSG_ENDS: u8 = 0x10;
const NO_EVENT: i16 = -1;
const NEAR_ZERO_SQ: f32 = 1e-8;

const AIM_POS: [[Vec3; 3]; 2] = [
    [
        Vec3::new(-40.0, 772.0, 1035.0),
        Vec3::new(-139.0, 772.0, 1035.0),
        Vec3::new(-240.0, 772.0, 1035.0),
    ],
    [
        Vec3::new(-40.0, 855.0, 1035.0),
        Vec3::new(-139.0, 855.0, 1035.0),
        Vec3::new(-240.0, 855.0, 1035.0),
    ],
];

const CURSOR_POS: [[Vec3; 3]; 2] = [
    [
        Vec3::new(-52.0, 804.0, 996.0),
        Vec3::new(-140.0, 804.0, 996.0),
        Vec3::new(-227.0, 804.0, 996.0),
    ],
    [
        Vec3::new(-52.0, 889.0, 996.0),
        Vec3::new(-140.0, 889.0, 996.0),
        Vec3::new(-227.0, 889.0, 996.0),
    ],
];

const CAMERA_CENTER: [[Vec3; 3]; 2] = [
    [
        Vec3::new(-104.0, 805.0, 875.0),
        Vec3::new(-139.0, 805.0, 882.0),
        Vec3::new(-173.0, 805.0, 875.0),
    ],
    [
        Vec3::new(-104.0, 825.0, 868.0),
        Vec3::new(-139.0, 825.0, 875.0),
        Vec3::new(-173.0, 825.0, 868.0),
    ],
];

const CAMERA_EYE: Vec3 = Vec3::new(-139.0, 815.0, 662.0);
const CAMERA_FOV_Y: f32 = 58.0;

const MAIL_EYE_DROP: f32 = 200.0;
const MAIL_SIDE_DROP: f32 = 150.0;
const MAIL_APPEAR_DIST: f32 = 120.0;
const MAIL_APPEAR_LIFT: f32 = 40.0;
const MAIL_THROW_LIFT: f32 = 40.0;
const MAIL_THROW_PUSH: f32 = 50.0;
const MAIL_THROW_FWD: f32 = 100.0;
const MAIL_SAME_SPAN: f32 = 1.25;
const MAIL_TILT_PIVOT_Y: f32 = 720.0;
const MAIL_FALL_GATE_Z: f32 = 995.0;
const MAIL_CLAMP_Y: f32 = 700.0;
const MAIL_FLOOR_Y: f32 = 700.0;
const MAIL_POP_STEP_X: f32 = 0.9;
const MAIL_POP_STEP_Y: f32 = 0.6;
const MAIL_POP_STEP: f32 = 0.9;
const MAIL_POP_LIFT: f32 = 3.0;
const MAIL_TILT_STEP: i16 = 0x800;
const MAIL_POP_ANGLE: i16 = -0x4000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailProc {
    Dummy,
    Appear,
    Wait,
    Throw,
    End,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MailState {
    pub seed: [i32; 3],
    pub same_count: u8,
    pub no_buff: u8,
}

impl MailState {
    pub fn create() -> Self {
        Self {
            seed: [RND_SEED; 3],
            same_count: 0,
            no_buff: 0,
        }
    }

    pub fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }
}

fn get_next_no(state: &mut MailState, previous: u8) -> u8 {
    let random = state.rnd_f(10.0);
    let mut fvar = 2.5 - f32::from(state.same_count) * MAIL_SAME_SPAN;
    let mut box_y = i32::from(previous) % 3;
    let mut box_x = i32::from(previous) / 3;

    if random < fvar {
        state.same_count += 1;
    } else {
        let new_threshold = fvar + 2.5;
        if random < new_threshold {
            if state.rnd_f(1.0) > 0.5 {
                if box_y == 0 || box_y == 2 {
                    box_y = 1;
                } else if state.rnd_f(1.0) > 0.5 {
                    box_y = 0;
                } else {
                    box_y = 2;
                }
            } else {
                box_x = i32::from(box_x == 0);
            }
            state.same_count = 0;
        } else if random >= new_threshold {
            fvar = state.rnd_f(1.0);
            if fvar < 0.3333 {
                if box_y == 0 || box_y == 2 {
                    box_y = 1;
                } else if state.rnd_f(1.0) > 0.5 {
                    box_y = 0;
                } else {
                    box_y = 2;
                }
                box_x = i32::from(box_x == 0);
            } else if fvar < 0.6666 {
                if box_y == 0 {
                    box_y = 2;
                } else if box_y == 2 {
                    box_y = 0;
                } else if state.rnd_f(1.0) > 0.5 {
                    box_y = 0;
                } else {
                    box_y = 2;
                }
            } else if box_y == 0 {
                box_y = 2;
            } else if box_y == 2 {
                box_y = 0;
            } else if state.rnd_f(1.0) > 0.5 {
                box_y = 0;
            } else {
                box_y = 2;
            }
            if fvar >= 0.6666 {
                box_x = i32::from(box_x == 0);
            }
            state.same_count = 0;
        }
    }

    let res = (box_y + box_x * 3) as u8;
    state.no_buff = res;
    res
}

pub struct MailCtx<'a> {
    pub center: Vec3,
    pub eye: Vec3,
    pub events: &'a mut SimEvents,
    pub mini_rupee: &'a mut i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwMail {
    pub func: MailProc,
    pub pos: Vec3,
    pub target: Vec3,
    pub offset: Vec3,
    pub rot: [i16; 3],
    pub rot_target: [i16; 3],
    pub no: u8,
    pub want_no: u8,
    pub phase: u8,
}

impl SwMail {
    pub fn create() -> Self {
        Self {
            func: MailProc::Dummy,
            pos: Vec3::ZERO,
            target: Vec3::ZERO,
            offset: Vec3::ZERO,
            rot: [0; 3],
            rot_target: [0; 3],
            no: 0,
            want_no: 0,
            phase: 0,
        }
    }

    pub fn check_proc(&self, func: MailProc) -> bool {
        self.func == func
    }

    pub fn set_proc(&mut self, func: MailProc) {
        self.func = func;
    }

    pub fn r#move(&mut self, ctx: &mut MailCtx<'_>) {
        match self.func {
            MailProc::Dummy => {}
            MailProc::Appear => self.appear(ctx),
            MailProc::Wait => self.wait(ctx),
            MailProc::Throw => self.throw_mail(ctx),
            MailProc::End => self.end(),
        }
    }

    pub fn init(&mut self, eye: Vec3) {
        self.no = 0;
        self.phase = 0;
        self.target = Vec3::ZERO;
        self.offset = Vec3::ZERO;
        self.want_no = 0;
        self.pos = Vec3::new(eye.x, eye.y - MAIL_EYE_DROP, eye.z);
    }

    pub fn dummy_init(&mut self, ctx: &MailCtx<'_>, state: &mut MailState) {
        self.rot = [0, -0x4000, 0];
        self.pos = Vec3::new(
            ctx.eye.x - MAIL_SIDE_DROP,
            ctx.eye.y - MAIL_EYE_DROP,
            ctx.eye.z,
        );
        self.no = get_next_no(state, self.no);
        self.rot_target = [0, -0x4000, -0x4000];
        self.offset = Vec3::ZERO;
        self.set_proc(MailProc::Dummy);
    }

    pub fn appear_init(&mut self, ctx: &mut MailCtx<'_>, state: &mut MailState) {
        self.rot = [0, -0x4000, 0];
        self.pos = Vec3::new(ctx.eye.x, ctx.eye.y - MAIL_EYE_DROP, ctx.eye.z);
        self.no = get_next_no(state, self.no);
        self.rot_target = [0, -0x4000, -0x4000];
        self.offset = Vec3::ZERO;
        self.phase = 0;
        ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_NEW));
        self.set_proc(MailProc::Appear);
    }

    fn appear(&mut self, ctx: &MailCtx<'_>) {
        let new_y = target_angle_y(ctx.center, ctx.eye);
        let mut diff = ctx.center - ctx.eye;
        if !normalize_rs(&mut diff) {
            diff = Vec3::new(0.0, 0.0, 1.0);
        }
        let mut vec = ctx.eye;
        diff *= MAIL_APPEAR_DIST + REG0_F0;
        vec += diff;
        vec.y -= MAIL_APPEAR_LIFT + REG0_F11;
        self.rot[0] = self.rot_target[0];
        self.rot[1] = self.rot_target[1].wrapping_add(new_y).wrapping_add(REG0_S1);
        add_calc_angle_s2(&mut self.rot[2], self.rot_target[2], 4, 0x1000);
        if add_calc_pos(&mut self.pos, vec, 0.25, 30.0, 2.5).abs() < 2.5 {
            self.set_proc(MailProc::Wait);
        }
    }

    fn wait(&mut self, ctx: &MailCtx<'_>) {
        let new_y = target_angle_y(ctx.center, ctx.eye);
        let mut diff = ctx.center - ctx.eye;
        if !normalize_rs(&mut diff) {
            diff = Vec3::new(0.0, 0.0, 1.0);
        }
        self.pos = ctx.eye;
        diff *= MAIL_APPEAR_DIST + REG0_F0;
        self.pos += diff;
        self.pos.y -= MAIL_APPEAR_LIFT + REG0_F11;
        self.rot = [
            self.rot_target[0],
            self.rot_target[1].wrapping_add(new_y).wrapping_add(REG0_S1),
            self.rot_target[2].wrapping_add(REG0_S2),
        ];
    }

    pub fn throw_init(&mut self, target: Vec3, want_no: u8) {
        self.target = target;
        self.phase = 0;
        self.want_no = want_no;
        self.set_proc(MailProc::Throw);
    }

    fn throw_mail(&mut self, ctx: &mut MailCtx<'_>) {
        if self.phase == 0 {
            let y_angle = target_angle_y(ctx.center, ctx.eye);
            let vec = Vec3::new(
                REG0_F2 - MAIL_THROW_LIFT,
                REG0_F3 - MAIL_THROW_PUSH,
                MAIL_THROW_FWD * REG0_F4,
            );
            let mut mult = Vec3::new(
                vec.x * cos_s(y_angle) + vec.z * sin_s(y_angle),
                vec.y,
                vec.z * cos_s(y_angle) - vec.x * sin_s(y_angle),
            );
            mult += ctx.eye;

            if add_calc_pos(&mut self.pos, mult, 0.5, 30.0, 2.0) < 1.0 {
                self.rot[1] = 0;
                self.rot_target[0] = 0;
                ctx.events.list.push(SimEvent::Se(SE_LK_SW_KAZEKIRI_S));
                self.phase += 1;
            }
        } else {
            let x_angle = target_angle_x(self.target, ctx.eye);
            self.rot[0] = -x_angle;
            self.rot[1] = self.rot[1].wrapping_add((i32::from(self.no) * 0x80 + 0x1000) as i16);
            self.rot[2] = 0;
            let pos_step = add_calc_pos(&mut self.pos, self.target, 0.5, HIO_44, 1.0);
            self.rot_target[0] += 4000 + REG0_S0;
            self.offset.y = (1.0 - cos_s(self.rot_target[0])) * (20.0 + REG0_F10);
            if pos_step < 1.0 {
                ctx.events.list.push(SimEvent::Se(SE_LETTER_IN_BOX));
                if self.want_no == self.no {
                    *ctx.mini_rupee += 1;
                    ctx.events.list.push(SimEvent::Se(SE_MINIGAME_RIGHT));
                } else {
                    ctx.events.list.push(SimEvent::Se(SE_MINIGAME_WRONG));
                }
                self.end_init();
            }
        }
    }

    fn end_init(&mut self) {
        self.target = Vec3::new(0.0, 0.0, -10.0);
        self.set_proc(MailProc::End);
    }

    fn end(&mut self) {
        if self.want_no == self.no {
            add_calc_angle_s2(&mut self.rot[1], 0, 4, 0x1000);
            add_calc_pos2(&mut self.offset, Vec3::ZERO, 0.25, 5.0);
        } else {
            add_calc_angle_s2(&mut self.rot[1], -0x8000, 4, 0x1000);
            add_calc_pos2(&mut self.offset, Vec3::ZERO, 0.25, 5.0);
            if self.pos.z < MAIL_FALL_GATE_Z {
                let target = if self.pos.y > MAIL_TILT_PIVOT_Y {
                    self.target.y -= MAIL_POP_LIFT + REG0_F11;
                    MAIL_POP_ANGLE
                } else {
                    self.target.y *= MAIL_POP_STEP_Y;
                    self.target.x *= MAIL_POP_STEP_X;
                    self.target.z *= MAIL_POP_STEP_X;
                    0
                };
                if self.pos.y <= MAIL_CLAMP_Y {
                    self.target.y = 0.0;
                    self.pos.y = MAIL_FLOOR_Y;
                }
                add_calc_angle_s2(&mut self.rot[0], target, 2, MAIL_TILT_STEP + REG0_S3);
            }
            self.target *= MAIL_POP_STEP;
            self.pos += self.target;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwCam {
    pub center: Vec3,
    pub eye: Vec3,
    pub fov_y: f32,
    pub aim_row: i8,
    pub aim_col: i8,
    pub active: bool,
}

impl SwCam {
    pub fn create() -> Self {
        Self {
            center: CAMERA_CENTER[0][1],
            eye: CAMERA_EYE,
            fov_y: CAMERA_FOV_Y,
            aim_row: 0,
            aim_col: 0,
            active: false,
        }
    }

    pub fn active_off(&mut self) {
        self.active = false;
    }

    pub fn active_on(&mut self) {
        self.active = true;
        self.center = CAMERA_CENTER[0][1];
        self.eye = CAMERA_EYE;
        self.fov_y = CAMERA_FOV_Y;
        self.set_aim_idx(0, 0);
    }

    pub fn set_aim_idx(&mut self, row: i8, col: i8) {
        self.aim_row = row;
        self.aim_col = col;
    }

    pub fn center_pos(&self) -> Vec3 {
        CAMERA_CENTER
            .get(usize::from(self.aim_col.max(0) as u8))
            .and_then(|row| row.get(usize::from((self.aim_row + 1).max(0) as u8)))
            .copied()
            .unwrap_or(CAMERA_CENTER[0][1])
    }

    pub fn r#move(&mut self) {
        if !self.active {
            return;
        }
        let center = self.center_pos();
        add_calc_pos(&mut self.center, center, 0.25, 10.0, 1.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StControl {
    pub max: i16,
    pub min: i16,
    pub step: i16,
    pub stop: i16,
    pub hi: f32,
    pub lo: f32,
    pub gap: i16,
    pub limit: i16,
    pub state: u8,
    pub prev: u8,
    pub hold_x: i16,
    pub hold_y: i16,
    pub wait_x: i16,
    pub wait_y: i16,
    pub rest_x: i16,
    pub rest_y: i16,
    pub base: i16,
}

impl StControl {
    pub fn create() -> Self {
        Self {
            max: 0xF,
            min: 0xF,
            step: 0,
            stop: 0,
            hi: 0.9,
            lo: 0.5,
            gap: 0,
            limit: 0x2000,
            state: 0,
            prev: 0,
            hold_x: 0,
            hold_y: 0,
            wait_x: 0xF,
            wait_y: 0xF,
            rest_x: 0,
            rest_y: 0,
            base: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_wait_parm(
        &mut self,
        max: i16,
        min: i16,
        step: i16,
        stop: i16,
        hi: f32,
        lo: f32,
        gap: i16,
        limit: i16,
    ) {
        self.max = max;
        self.min = min;
        self.step = step;
        self.stop = stop;
        self.hi = hi;
        self.lo = lo;
        self.gap = gap;
        self.limit = limit;
    }

    pub fn init(&mut self) {
        self.hold_x = 0;
        self.hold_y = 0;
        self.state = 0;
        self.prev = 0;
        self.base = 0;
        self.wait_x = self.max;
        self.wait_y = self.max;
        self.rest_x = self.stop;
        self.rest_y = self.stop;
    }

    fn xinit(&mut self) {
        self.hold_x = 0;
        self.state &= !0xFC;
        self.wait_x = self.max;
        self.rest_x = self.stop;
    }

    fn yinit(&mut self) {
        self.hold_y = 0;
        self.state &= !0x0C;
        self.wait_y = self.max;
        self.rest_y = self.stop;
    }

    pub fn check_trigger(&mut self, value: f32, angle: i16) -> u8 {
        self.prev = self.state;
        let mut bits = 0u8;
        let half = (0x2000 - self.limit) >> 1;
        if value != 0.0 {
            if angle < self.base + -0x7000 + half {
                bits |= 4;
            } else if angle < self.base + -0x5000 - half {
                bits |= 4 | 1;
            } else if angle < self.base + -0x3000 + half {
                bits |= 1;
            } else if angle < self.base + -0x1000 - half {
                bits |= 1 | 8;
            } else if angle < self.base + 0x1000 + half {
                bits |= 8;
            } else if angle < self.base + 0x3000 - half {
                bits |= 8 | 2;
            } else if angle < self.base + 0x5000 + half {
                bits |= 2;
            } else if angle < self.base + 0x7000 - half {
                bits |= 2 | 4;
            } else {
                bits |= 4;
            }

            if value >= self.hi {
                self.state = bits;
            } else if value < self.lo {
                self.state = 0;
            } else {
                self.state &= !bits;
            }

            if self.state != self.prev {
                if self.state == 0 {
                    self.base = 0;
                } else if (angle & 0x1FFF) > 0x1000 {
                    self.base = self.gap;
                } else {
                    self.base = -self.gap;
                }
            }

            if self.state & 0x03 == 0 {
                self.xinit();
            }
            if self.state & 0x0C == 0 {
                self.yinit();
            }
        } else {
            self.state = 0;
            self.xinit();
            self.yinit();
        }

        if self.prev & self.state & 0x03 != 0 && self.hold_x > 0 {
            self.hold_x -= 1;
        }
        if self.prev & self.state & 0x0C != 0 && self.hold_y > 0 {
            self.hold_y -= 1;
        }
        self.state
    }

    pub fn check_left_trigger(&mut self) -> bool {
        self.horizontal(1)
    }

    pub fn check_right_trigger(&mut self) -> bool {
        self.horizontal(2)
    }

    fn horizontal(&mut self, bit: u8) -> bool {
        if self.hold_x == 0 && self.state & bit != 0 {
            self.hold_x = self.wait_x;
            if self.rest_x == 0 {
                self.wait_x = self.wait_x.wrapping_sub(self.step);
                if self.wait_x < self.min {
                    self.wait_x = self.min;
                }
            } else {
                self.rest_x -= 1;
            }
            return true;
        }
        false
    }

    pub fn check_up_trigger(&mut self) -> bool {
        self.vertical(4)
    }

    pub fn check_down_trigger(&mut self) -> bool {
        self.vertical(8)
    }

    fn vertical(&mut self, bit: u8) -> bool {
        if self.hold_y == 0 && self.state & bit != 0 {
            self.hold_y = self.wait_y;
            if self.rest_y == 0 {
                self.wait_y = self.wait_y.wrapping_sub(self.step);
                if self.wait_y < self.min {
                    self.wait_y = self.min;
                }
            } else {
                self.rest_y -= 1;
            }
            return true;
        }
        false
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
    pub backbone_lock: bool,
}

impl JntCtrl {
    pub fn create() -> Self {
        Self {
            angles: [[0; 2]; 2],
            min_angles: [[0; 2]; 2],
            max_angles: [[0; 2]; 2],
            max_turn_step: [[0; 2]; 2],
            trn: false,
            head_lock: false,
            backbone_lock: false,
        }
    }

    pub fn head_y(&self) -> i16 {
        self.angles[0][1]
    }

    pub fn backbone_y(&self) -> i16 {
        self.angles[1][1]
    }

    pub fn on_head_lock(&mut self) {
        self.head_lock = true;
    }

    pub fn off_head_lock(&mut self) {
        self.head_lock = false;
    }

    pub fn on_backbone_lock(&mut self) {
        self.backbone_lock = true;
    }

    pub fn off_backbone_lock(&mut self) {
        self.backbone_lock = false;
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
        self.max_angles[1][0] = max_backbone_x;
        self.max_angles[1][1] = max_backbone_y;
        self.min_angles[1][0] = min_backbone_x;
        self.min_angles[1][1] = min_backbone_y;
        self.max_angles[0][0] = max_head_x;
        self.max_angles[0][1] = max_head_y;
        self.min_angles[0][0] = min_head_x;
        self.min_angles[0][1] = min_head_y;
        self.max_turn_step[1][0] = max_turn_step;
        self.max_turn_step[1][1] = max_turn_step;
        self.max_turn_step[0][0] = max_turn_step;
        self.max_turn_step[0][1] = max_turn_step;
    }

    fn ang_calc_s(out: &mut i16, target: i16, speed: i16, max_vel: i16) -> bool {
        let orig = out.wrapping_sub(target);
        let mut diff = orig;
        add_calc_angle_s2(&mut diff, 0, speed, max_vel);
        if i32::from(diff).abs() > i32::from(speed) {
            *out = out.wrapping_add(diff.wrapping_sub(orig));
        } else {
            *out = target;
        }
        target.wrapping_sub(*out) == 0
    }

    fn follow(&mut self, out: &mut i16, target: i16, max_vel: i16, axis: usize) -> bool {
        let orig = *out;
        Self::ang_calc_s(out, target, 4, max_vel);
        let delta = i32::from(out.wrapping_sub(orig));
        let mut prev = 0i16;
        let mut acc = 0i16;
        let want = target.wrapping_sub(*out);
        for row in (0..2).rev() {
            acc = acc.wrapping_add(self.angles[row][axis]);
            if (delta > 0 && i32::from(acc) > i32::from(want))
                || (delta < 0 && i32::from(acc) < i32::from(want))
            {
                self.angles[row][axis] = want.wrapping_sub(prev);
                limitter(
                    &mut self.angles[row][axis],
                    self.min_angles[row][axis],
                    self.max_angles[row][axis],
                );
            }
            prev = prev.wrapping_add(self.angles[row][axis]);
        }
        target != *out
    }

    fn r#move(&mut self, target: i16, axis: usize) -> bool {
        let mut rest = target;
        let mut reached = [0i16; 2];
        for row in 0..2 {
            let locked = (row == 0 && self.head_lock) || (row == 1 && self.backbone_lock);
            let mut angle = if locked { 0 } else { rest };
            limitter(
                &mut angle,
                self.min_angles[row][axis],
                self.max_angles[row][axis],
            );
            rest = rest.wrapping_sub(angle);
            reached[row] = angle;
        }
        for row in 0..2 {
            let current = i32::from(self.angles[row][axis]);
            let next = add_calc_angle_l(
                current,
                i32::from(reached[row]),
                4,
                i32::from(self.max_turn_step[row][axis]),
                1,
            );
            self.angles[row][axis] = next as i16;
        }
        let mut done = 0;
        for row in 0..2 {
            if i32::from(reached[row].wrapping_sub(self.angles[row][axis])).abs()
                <= i32::from(self.max_turn_step[row][axis] / 2)
            {
                done += 1;
            }
        }
        done == 2
    }

    #[allow(clippy::too_many_arguments)]
    pub fn look_at_target(
        &mut self,
        out_y: &mut i16,
        dst: Option<Vec3>,
        src: Vec3,
        default_y: i16,
        max_vel: i16,
        free: bool,
    ) {
        let (target_y, pitch) = match dst {
            Some(point) => {
                let delta = point - src;
                let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
                (to_s16(delta.x.atan2(delta.z)), to_s16(delta.y.atan2(flat)))
            }
            None => (default_y, 0),
        };
        let delta_y = i32::from(target_y) - i32::from(*out_y);
        let max_y = i32::from(self.max_angles[1][1]) + i32::from(self.max_angles[0][1]);
        let min_y = i32::from(self.min_angles[1][1]) + i32::from(self.min_angles[0][1]);
        let settled = self.r#move(delta_y as i16, 1);
        if (self.trn || (delta_y >= max_y || delta_y <= min_y) && settled) && !free {
            self.trn = self.follow(out_y, target_y, max_vel, 1);
        }
        self.r#move(pitch, 0);
    }
}

fn limitter(value: &mut i16, min_diff: i16, max_diff: i16) {
    if *value > max_diff {
        *value = max_diff;
    }
    if *value < min_diff {
        *value = min_diff;
    }
}

fn add_calc_angle_l(value: i32, target: i32, scale: i32, max_step: i32, min_step: i32) -> i32 {
    let diff = target - value;
    if value == target {
        return value;
    }
    let mut step = diff / scale;
    if step > min_step || step < -min_step {
        if step > max_step {
            step = max_step;
        }
        if step < -max_step {
            step = -max_step;
        }
        value + step
    } else if diff >= 0 {
        let next = value + min_step;
        if target - next <= 0 { target } else { next }
    } else {
        let next = value - min_step;
        if target - next >= 0 { target } else { next }
    }
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
            if step != 0.0 {
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

fn add_calc_pos2(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32) {
    if *value != target {
        let mut diff = (*value - target) * scale;
        if diff.length() > max_step {
            diff = norm_zp(diff) * max_step;
        }
        *value -= diff;
    }
}

fn norm_zp(value: Vec3) -> Vec3 {
    if value.length_squared() > NEAR_ZERO_SQ {
        value.normalize()
    } else {
        Vec3::ZERO
    }
}

fn normalize_rs(value: &mut Vec3) -> bool {
    if value.length_squared() <= NEAR_ZERO_SQ {
        return false;
    }
    *value = value.normalize();
    true
}

fn target_angle_y(lhs: Vec3, rhs: Vec3) -> i16 {
    to_s16((rhs.x - lhs.x).atan2(rhs.z - lhs.z))
}

fn target_angle_x(lhs: Vec3, rhs: Vec3) -> i16 {
    let delta = rhs - lhs;
    to_s16(
        delta
            .y
            .atan2((delta.x * delta.x + delta.z * delta.z).sqrt()),
    )
}

fn calc_timer(value: &mut i16) -> i16 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

fn event_index(list: &EventList, name: &str) -> i16 {
    list.event_index(name)
        .and_then(|index| i16::try_from(index).ok())
        .unwrap_or(NO_EVENT)
}

fn get_game_end_msg(rupees: i16, stories: &mut StoryBits) -> u32 {
    match stories.reg(REG_C203) {
        0 => {
            if rupees == 0 {
                0x1A3A
            } else if rupees == 1 {
                0x1A3B
            } else if rupees < 10 {
                0x1A3C
            } else {
                0x1A37
            }
        }
        1 => {
            if rupees == 0 {
                0x1A4F
            } else if rupees == 1 {
                0x1A50
            } else if rupees < 20 {
                0x1A52
            } else {
                0x1A4C
            }
        }
        2 => {
            if rupees == 0 {
                0x1A5C
            } else if rupees == 1 {
                0x1A5D
            } else if rupees < 25 {
                0x1A60
            } else {
                0x1A57
            }
        }
        _ => {
            if stories.test(FLAG_2701) {
                if rupees == 0 {
                    0x1A69
                } else if rupees == 1 {
                    0x1A6A
                } else if rupees <= i16::from(stories.reg(REG_8AFF)) {
                    0x1A6B
                } else {
                    stories.set_reg(REG_8AFF, rupees as u8);
                    0x1A6D
                }
            } else {
                0x1A62
            }
        }
    }
}

fn aim_pos(aim_y: i8, row: i32) -> Vec3 {
    AIM_POS
        .get(usize::from(aim_y.max(0) as u8))
        .and_then(|line| line.get(usize::try_from(row).unwrap_or(1)))
        .copied()
        .unwrap_or(AIM_POS[0][1])
}

fn cursor_pos(aim_y: i8, aim_x: i8) -> Vec3 {
    CURSOR_POS
        .get(usize::from(aim_y.max(0) as u8))
        .and_then(|row| row.get(usize::from((aim_x + 1).max(0) as u8)))
        .copied()
        .unwrap_or(CURSOR_POS[0][1])
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct NpcBmsw {
    pub entered: bool,
    pub att_pos: Vec3,
    pub eye_pos: Vec3,
    pub init_angle: [i16; 3],
    pub look_yaw: i16,
    pub head_turn: i16,
    pub anm_idx: i8,
    pub anm_frame: u16,
    pub wait_count: i8,
    pub order: i8,
    pub event_idx: i16,
    pub mode: i8,
    pub msg_no: u32,
    pub field_9c4: i8,
    pub game_get_rupee: i16,
    pub timer_created: u8,
    pub mail_live: u8,
    pub btp_timer: i16,
    pub btp_frame: u8,
    pub mail_idx: u8,
    pub aim_x: i8,
    pub aim_y: i8,
    pub timer_rest: i32,
    pub timer_delete: i32,
    pub has_attention: u8,
    pub talk_order: u8,
    pub action_status: i8,
    pub cur_action: u8,
    pub item_rupee_count: i32,
    pub curr_msg_no: u32,
    pub curr_msg_id: u32,
    pub mail: MailState,
    pub jnt: JntCtrl,
    pub st: StControl,
    pub mails: [SwMail; MAIL_SLOTS],
    pub cam: SwCam,
}

impl NpcBmsw {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            entered: false,
            att_pos: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            init_angle: [0; 3],
            look_yaw: 0,
            head_turn: 0,
            anm_idx: 0,
            anm_frame: 0,
            wait_count: 1,
            order: ORDER_NONE,
            event_idx: NO_EVENT,
            mode: MODE_WAIT,
            msg_no: FPC_M_ERROR_PROCESS_ID,
            field_9c4: 0,
            game_get_rupee: 0,
            timer_created: 0,
            mail_live: 0,
            btp_timer: 0,
            btp_frame: 0,
            mail_idx: 0,
            aim_x: 0,
            aim_y: 0,
            timer_rest: 0,
            timer_delete: 0,
            has_attention: 0,
            talk_order: 0,
            action_status: ACTION_STARTING,
            cur_action: ACTION_WAIT,
            item_rupee_count: 0,
            curr_msg_no: 0,
            curr_msg_id: FPC_M_ERROR_PROCESS_ID,
            mail: MailState::create(),
            jnt: JntCtrl::create(),
            st: StControl::create(),
            mails: [SwMail::create(); MAIL_SLOTS],
            cam: SwCam::create(),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn create_init(&mut self, actor: &mut Actor) {
        self.init_angle = actor.angle;
        self.att_pos = actor.pos;
        self.eye_pos = actor.pos;
        self.field_9c4 = 0;
        self.msg_no = FPC_M_ERROR_PROCESS_ID;
        self.timer_created = 0;
        self.mail_live = 0;
        self.btp_frame = 0;
        self.btp_timer = 0;
        self.mail_idx = 0;
        self.cam = SwCam::create();
        self.st.set_wait_parm(5, 2, 3, 2, 1.0, 0.9, 0, 0x2000);
        self.st.init();
        self.mail = MailState::create();
        for mail in &mut self.mails {
            *mail = SwMail::create();
        }
        self.set_action(ACTION_WAIT);
        self.entered = true;
    }

    fn set_action(&mut self, action: u8) {
        if self.cur_action == action {
            return;
        }
        if self.cur_action == ACTION_SHIWAKE {
            self.action_status = ACTION_ENDING;
        }
        self.cur_action = action;
        self.action_status = ACTION_STARTING;
    }

    pub fn set_anm(&mut self, idx: i8) {
        if idx != self.anm_idx {
            self.anm_idx = idx;
            self.anm_frame = 0;
        }
    }

    fn anm_frame_max(&self) -> u16 {
        ANM_FRAME_MAX
            .get(usize::from(self.anm_idx.max(0) as u8))
            .copied()
            .unwrap_or(ANM_FRAME_MAX[0])
    }

    fn anm_advance(&mut self) {
        let max = self.anm_frame_max();
        if max == 0 {
            return;
        }
        self.anm_frame = if self.anm_frame >= max {
            0
        } else {
            self.anm_frame + 1
        };
    }

    fn anm_frame_done(&self) -> bool {
        let max = self.anm_frame_max();
        self.anm_frame + 1 == max
    }

    pub fn play_tex_pattern_anm(&mut self) {
        if calc_timer(&mut self.btp_timer) == 0 {
            if self.btp_frame >= BTP_FRAME_MAX {
                self.btp_frame -= BTP_FRAME_MAX;
                self.btp_timer = (self.mail.rnd_f(BTP_TIMER_RANGE) + BTP_TIMER_MIN) as i16;
            } else {
                self.btp_frame += 1;
            }
        }
    }

    pub fn chk_attention(&self, pos: Vec3, angle: i16, player: Option<Vec3>) -> bool {
        let mut max_dist = HIO_MAX_ATTN_DIST_XZ;
        let mut max_angle = HIO_MAX_ATTN_ANGLE_Y;
        let Some(player) = player else {
            return false;
        };
        let delta = Vec3::new(player.x - pos.x, 0.0, player.z - pos.z);
        let dist_xz = (delta.x * delta.x + delta.z * delta.z).sqrt();
        let mut target_angle = to_s16(delta.x.atan2(delta.z));
        if self.has_attention != 0 {
            max_dist += ATTENTION_EXTEND_DIST;
            max_angle = max_angle.wrapping_add(ATTENTION_EXTEND_S);
        }
        target_angle = target_angle.wrapping_sub(angle);
        i32::from(max_angle) > i32::from(target_angle).abs() && max_dist > dist_xz
    }

    pub fn check_order(&mut self, talk_command: bool) {
        if !talk_command {
            return;
        }
        if self.order != ORDER_SPEAK && self.order != ORDER_CAN_TALK {
            return;
        }
        self.order = ORDER_NONE;
        self.talk_order = 1;
    }

    pub fn event_order(&mut self) {
        if self.order == ORDER_SPEAK || self.order == ORDER_CAN_TALK {
            self.order = ORDER_NONE;
        }
    }

    pub fn get_msg(&mut self, stories: &mut StoryBits, tmp: &mut EventFlags) -> u32 {
        if self.msg_no != FPC_M_ERROR_PROCESS_ID {
            let msg = self.msg_no;
            self.msg_no = FPC_M_ERROR_PROCESS_ID;
            return msg;
        }
        if stories.test(FLAG_2701) {
            if !tmp.test(TMP_FLAG_0320) {
                tmp.set(TMP_FLAG_0320);
                0x1A63
            } else {
                0x1A66
            }
        } else if !stories.test(FLAG_1A02) {
            stories.set(FLAG_1A02);
            0x1A2D
        } else if !stories.test(FLAG_1A01) {
            0x1A3D
        } else {
            match stories.reg(REG_C203) {
                0 => 0x1A3F,
                1 => 0x1A44,
                2 => 0x1A54,
                _ => 0x1A62,
            }
        }
    }

    pub fn next_msg_status(
        &mut self,
        curr: &mut u32,
        select: u8,
        stories: &mut StoryBits,
        mem: &MemBits,
        mini_rupee: i32,
    ) -> u8 {
        let mut ret = MSG_CONTINUES;
        match *curr {
            0x1A2D | 0x1A31 | 0x1A32 | 0x1A35 | 0x1A37 | 0x1A41 | 0x1A42 | 0x1A47 | 0x1A4A
            | 0x1A4C | 0x1A50 | 0x1A55 | 0x1A57 | 0x1A59 | 0x1A5A | 0x1A5D | 0x1A67 | 0x1A6D
            | 0x1A6E => *curr += 1,
            0x1A63 | 0x1A66 => *curr = 0x1A64,
            0x1A6A | 0x1A6B => *curr = 0x1A6C,
            0x1A52 => *curr = if mini_rupee < 15 { 0x1A51 } else { 0x1A53 },
            0x1A60 => *curr = if mini_rupee < 20 { 0x1A5E } else { 0x1A61 },
            0x1A2E => {
                if select == 0 {
                    stories.set(FLAG_1A01);
                    *curr = 0x1A31;
                } else if select == 1 {
                    *curr = if mem.is_stage_boss_enemy() {
                        0x1A2F
                    } else {
                        0x1A30
                    };
                }
            }
            0x1A33 => {
                if select == 0 {
                    *curr = 0x1A35;
                } else if select == 1 {
                    *curr = 0x1A34;
                }
            }
            0x1A34 => {
                *curr = if !stories.test(FLAG_1A01) {
                    0x1A32
                } else {
                    0x1A42
                }
            }
            0x1A38 => {
                *curr += 1;
                stories.set_reg(REG_C203, 1);
            }
            0x1A3D => {
                if select == 0 {
                    stories.set(FLAG_1A01);
                    *curr = 0x1A31;
                } else {
                    *curr = 0x1A3E;
                }
            }
            0x1A4D => {
                stories.set_reg(REG_C203, 2);
                *curr = 0x1A4E;
            }
            0x1A51 | 0x1A53 => *curr = 0x1A4E,
            0x1A58 => {
                stories.set_reg(REG_C203, 3);
                *curr += 1;
            }
            0x1A5E | 0x1A61 => *curr = 0x1A5F,
            0x1A3F => *curr = if select == 0 { 0x1A41 } else { 0x1A40 },
            0x1A43 => *curr = if select == 0 { 0x1A35 } else { 0x1A34 },
            0x1A44 => *curr = if select == 0 { 0x1A46 } else { 0x1A45 },
            0x1A46 => *curr = if select == 0 { 0x1A4A } else { 0x1A47 },
            0x1A48 => *curr = if select == 0 { 0x1A4A } else { 0x1A49 },
            0x1A49 => *curr = 0x1A47,
            0x1A54 => *curr = if select == 0 { 0x1A55 } else { 0x1A45 },
            0x1A64 => *curr = if select == 0 { 0x1A67 } else { 0x1A65 },
            _ => ret = MSG_ENDS,
        }
        ret
    }

    pub fn anm_atr(&mut self, attr: u8) {
        match attr {
            5 => self.set_anm(ANM_WAIT_02),
            6 => self.set_anm(ANM_TALK_01),
            9 => self.set_anm(ANM_TALK_02),
            14 => self.set_anm(ANM_TALK_03),
            _ => {}
        }
    }

    pub fn set_attention(&self) -> Vec3 {
        Vec3::new(
            self.att_pos.x,
            self.att_pos.y + HIO_ATTN_Y_OFFSET,
            self.att_pos.z,
        )
    }

    pub fn look_back(&mut self, actor: &mut Actor, dst: Option<Vec3>, src: Vec3) {
        let desired = actor.angle[1];
        if self.jnt.trn {
            add_calc_angle_s2(&mut self.head_turn, HIO_MAX_HEAD_TURN_VEL, 4, 0x800);
        } else {
            self.head_turn = 0;
        }
        self.jnt
            .look_at_target(&mut actor.angle[1], dst, src, desired, self.head_turn, true);
        self.look_yaw = actor.angle[1];
    }

    pub fn wait01(&mut self, stories: &StoryBits) {
        if self.talk_order != 0 {
            self.mode = MODE_TALK;
            self.jnt.off_head_lock();
            self.jnt.off_backbone_lock();
            return;
        }
        if self.has_attention != 0 {
            self.order = ORDER_CAN_TALK;
        }
        if !stories.test(FLAG_2701) {
            if self.anm_frame_done() {
                self.wait_count -= 1;
                if self.wait_count <= 0 {
                    if self.anm_idx == ANM_SHIWAKE_01 {
                        self.wait_count = 1;
                        self.set_anm(ANM_SHIWAKE_02);
                    } else {
                        self.wait_count = (self.mail.rnd_f(4.0) + 2.0) as i8;
                        self.set_anm(ANM_SHIWAKE_01);
                    }
                }
            }
            if self.anm_idx == ANM_SHIWAKE_01 || self.anm_idx == ANM_SHIWAKE_02 {
                self.jnt.on_head_lock();
                self.jnt.on_backbone_lock();
            } else {
                self.jnt.off_head_lock();
                self.jnt.off_backbone_lock();
            }
        }
    }

    fn talk(&mut self, ctx: &mut Ctx<'_>) -> u8 {
        let mut status = NO_STATUS;
        if self.curr_msg_id == FPC_M_ERROR_PROCESS_ID {
            self.curr_msg_no = self.get_msg(ctx.stories, ctx.ev_flags);
            self.curr_msg_id = ctx.text.message_set_actor(self.curr_msg_no, self.eye_pos);
        } else if ctx.text.msg_live(self.curr_msg_id) {
            status = ctx.text.status().unwrap_or(NO_STATUS);
            if status == STATUS_MSG_DISPLAYED {
                let mut curr = self.curr_msg_no;
                let next = self.next_msg_status(
                    &mut curr,
                    ctx.select,
                    ctx.stories,
                    ctx.mem,
                    ctx.mini_rupee,
                );
                self.curr_msg_no = curr;
                ctx.text.set_status(next);
                if next == STATUS_MSG_CONTINUES {
                    ctx.text.message_set(self.curr_msg_no);
                }
            } else if status == STATUS_BOX_CLOSED {
                ctx.text.set_status(STATUS_MSG_DESTROYED);
                self.curr_msg_id = FPC_M_ERROR_PROCESS_ID;
            }
            self.anm_atr(ctx.text.mesg_anime());
            ctx.text.set_mesg_anime(0xFF);
        }
        status
    }

    pub fn talk01(&mut self, ctx: &mut Ctx<'_>) -> u8 {
        let talk_res = self.talk(ctx);
        if ctx.mesg_send_button == 1 {
            match self.curr_msg_no {
                0x1A32 | 0x1A42 | 0x1A47 | 0x1A4A | 0x1A55 | 0x1A67 => self.cam.active_on(),
                0x1A39 | 0x1A4E | 0x1A5F | 0x1A59 | 0x1A6F => {
                    self.item_rupee_count += i32::from(self.game_get_rupee);
                }
                _ => {}
            }
        }
        if talk_res == STATUS_BOX_CLOSED {
            self.mode = MODE_WAIT;
            ctx.event_reset(self.event_idx);
            if !ctx.stories.test(FLAG_2701) {
                self.set_anm(ANM_WAIT_01);
            } else {
                self.set_anm(ANM_WAIT_02);
            }
            self.wait_count = 1;
            self.talk_order = 0;
            if matches!(self.curr_msg_no, 0x1A36 | 0x1A4B | 0x1A56 | 0x1A68) {
                self.set_action(ACTION_SHIWAKE);
            }
        }
        talk_res
    }

    pub fn check_next_mail_throw_ok(&self) -> bool {
        let idx = if self.mail_idx < 2 {
            self.mail_idx + 1
        } else {
            0
        };
        self.mails
            .get(usize::from(idx))
            .is_some_and(|mail| mail.check_proc(MailProc::Dummy) || mail.check_proc(MailProc::End))
    }

    pub fn set_game_get_rupee(&mut self, rupees: i16, stories: &StoryBits) -> i16 {
        let value = match stories.reg(REG_C203) {
            0 => {
                if rupees < 10 {
                    0
                } else {
                    rupees >> 1
                }
            }
            1 => {
                if rupees < 15 {
                    0
                } else if rupees < 20 {
                    rupees >> 1
                } else {
                    rupees
                }
            }
            2 => {
                if rupees < 20 {
                    0
                } else if rupees < 25 {
                    rupees
                } else {
                    rupees * 3
                }
            }
            _ => {
                if stories.test(FLAG_2701) {
                    rupees
                } else {
                    rupees * 3
                }
            }
        };
        self.game_get_rupee = value;
        value
    }

    pub fn timer_count_down(&mut self, events: &mut SimEvents) {
        if self.timer_rest > SHIWAKE_TIMER_START {
            if self.timer_rest <= SHIWAKE_TIMER_GATE {
                events.list.push(SimEvent::Se(SE_MINIGAME_TIMER_30));
                self.timer_rest -= SHIWAKE_TIMER_BEAT;
            } else {
                events.list.push(SimEvent::Se(SE_MINIGAME_TIMER_10));
                self.timer_rest -= SHIWAKE_TIMER_BEAT_BIG;
            }
        }
    }

    pub fn shiwake_game_action(&mut self, ctx: &mut Ctx<'_>) {
        if self.action_status == ACTION_STARTING {
            self.mode = MODE_SHIWAKE;
            self.aim_x = 0;
            self.aim_y = 0;

            if !ctx.demo_accepted {
                self.event_idx = if !ctx.stories.test(FLAG_2701) {
                    event_index(ctx.list, EVENT_SHIWAKEGAME)
                } else {
                    event_index(ctx.list, EVENT_SHIWAKEGAME2)
                };
                ctx.order_event(self.event_idx, ctx.entity);
                return;
            }

            if self.timer_created == 0 {
                self.timer_created = 1;
                ctx.mini_rupee = 0;
                ctx.mini_game_type = MINI_GAME_LETTER;
            }

            self.mail_live = 1;
            self.mail_start(ctx);
            self.mail_idx = 0;
            self.action_status += 1;
            self.timer_rest = SHIWAKE_LIMIT_MS;
            self.timer_delete = 0;
        } else if self.action_status == ACTION_ENDING {
            self.timer_created = 0;
            self.mail_live = 0;
        } else if self.timer_rest > 0 {
            self.timer_rest = (self.timer_rest - MS_PER_FRAME).max(0);
            self.timer_count_down(ctx.events);

            self.st.check_trigger(ctx.stick_value, ctx.stick_angle);
            if self.aim_x > -1 && self.st.check_left_trigger() {
                ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_CURSOR));
                self.aim_x -= 1;
            } else if self.aim_x < 1 && self.st.check_right_trigger() {
                ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_CURSOR));
                self.aim_x += 1;
            }

            if self.aim_y < 1 && self.st.check_up_trigger() {
                ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_CURSOR));
                self.aim_y += 1;
            } else if self.aim_y > 0 && self.st.check_down_trigger() {
                ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_CURSOR));
                self.aim_y -= 1;
            }

            let ready = self
                .mails
                .get(usize::from(self.mail_idx))
                .is_some_and(|mail| mail.check_proc(MailProc::Wait));
            if ready && ctx.timer_status != 1 && ctx.trig_a && self.check_next_mail_throw_ok() {
                let row = i32::from(self.aim_x) + 1;
                let col = i32::from(self.aim_y);
                let idx = (row + (1 - col) * 3) as u8;
                let target = aim_pos(self.aim_y, row);
                if let Some(mail) = self.mails.get_mut(usize::from(self.mail_idx)) {
                    mail.throw_init(target, idx);
                }
                if self.mail_idx < 2 {
                    self.mail_idx += 1;
                } else {
                    self.mail_idx = 0;
                }
                ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_OK));
                {
                    let mut state = self.mail;
                    let mut mail_ctx = ctx.mail_ctx();
                    if let Some(mail) = self.mails.get_mut(usize::from(self.mail_idx)) {
                        mail.appear_init(&mut mail_ctx, &mut state);
                    }
                    self.mail = state;
                }
            }

            self.cam.set_aim_idx(self.aim_x, self.aim_y);
            ctx.cursor_pos = cursor_pos(self.aim_y, self.aim_x);
        } else if self.action_status == ACTION_ONGOING {
            ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_TIMER_0));
            ctx.timer_end = HIO_34;
            ctx.cursor_hidden = true;
            self.action_status += 1;
        } else if self.action_status == ACTION_UNK_2 {
            self.timer_delete += 1;
            if self.timer_delete >= TIMER_DELETE_FRAMES {
                self.action_status += 1;
            }
        } else {
            ctx.end_staff_cut(STAFF_BMSW);
            if ctx.event_ended(self.event_idx) {
                ctx.event_reset(self.event_idx);
                let rupees = ctx.mini_rupee as i16;
                self.msg_no = get_game_end_msg(rupees, ctx.stories);
                let value = self.set_game_get_rupee(rupees, ctx.stories);
                ctx.message_count = i32::from(value);
                self.order = ORDER_SPEAK;
                self.set_action(ACTION_WAIT);
                self.cam.active_off();
                if ctx.mini_game_type == MINI_GAME_LETTER {
                    ctx.end_mini_game = true;
                }
            }
        }

        self.mail_step(ctx);
    }

    fn mail_start(&mut self, ctx: &mut Ctx<'_>) {
        let eye = self.cam.eye;
        let mut state = self.mail;
        for idx in 0..MAIL_SLOTS {
            self.mails[idx].init(eye);
        }
        {
            let mut mail_ctx = ctx.mail_ctx();
            self.mails[0].appear_init(&mut mail_ctx, &mut state);
            self.mails[1].dummy_init(&mail_ctx, &mut state);
            self.mails[2].dummy_init(&mail_ctx, &mut state);
        }
        self.mail = state;
    }

    fn mail_step(&mut self, ctx: &mut Ctx<'_>) {
        let mut mail_ctx = ctx.mail_ctx();
        for idx in 0..MAIL_SLOTS {
            self.mails[idx].r#move(&mut mail_ctx);
        }
    }

    pub fn wait_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        if self.action_status == ACTION_STARTING {
            self.mode = MODE_WAIT;
            self.wait_count = 1;
            if !ctx.stories.test(FLAG_2701) {
                self.set_anm(ANM_WAIT_01);
            } else {
                self.set_anm(ANM_WAIT_02);
            }
            self.action_status += 1;
        } else if self.action_status != ACTION_ENDING {
            let angle = actor.angle[1]
                .wrapping_add(self.jnt.head_y())
                .wrapping_add(self.jnt.backbone_y());
            self.has_attention = u8::from(self.chk_attention(actor.pos, angle, ctx.player_pos));
            self.order = ORDER_NONE;
            match self.mode {
                MODE_WAIT => self.wait01(ctx.stories),
                MODE_TALK => {
                    self.talk01(ctx);
                }
                _ => {}
            }
            let dst = self.look_target(ctx);
            let src = Vec3::new(actor.pos.x, self.eye_pos.y, actor.pos.z);
            self.look_back(actor, dst, src);
            self.att_pos = actor.pos;
        }
    }

    fn look_target(&self, ctx: &Ctx<'_>) -> Option<Vec3> {
        if self.mode != MODE_WAIT && self.mode != MODE_TALK {
            return None;
        }
        if self.has_attention != 0 {
            return ctx.player_eye_pos;
        }
        if ctx.stories.test(FLAG_2701) {
            return ctx.btsw_eye;
        }
        None
    }

    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.jnt.set_param(
            HIO_MAX_BACKBONE_X,
            HIO_MAX_BACKBONE_Y,
            HIO_MIN_BACKBONE_X,
            HIO_MIN_BACKBONE_Y,
            HIO_MAX_HEAD_X,
            HIO_MAX_HEAD_Y,
            HIO_MIN_HEAD_X,
            HIO_MIN_HEAD_Y,
            HIO_MAX_TURN_STEP,
        );
        self.st.set_wait_parm(5, 2, 3, 2, HIO_50, HIO_54, 0, 0x2000);
        self.play_tex_pattern_anm();
        self.anm_advance();
        self.check_order(ctx.talk_command);
        if !ctx.cut_proc {
            match self.cur_action {
                ACTION_WAIT => self.wait_action(actor, ctx),
                _ => self.shiwake_game_action(ctx),
            }
        }
        self.cam.r#move();
        self.event_order();
    }
}

pub struct Text<'a> {
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub clear_count: u8,
    pub one: &'a mut Msg1Box,
    pub two: &'a mut Msg2Box,
    pub archive: Option<&'a dyn MsgArchive>,
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

    pub fn msg_live(&self, id: u32) -> bool {
        id != FPC_M_ERROR_PROCESS_ID && self.procs.search_by_id(id).is_some()
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

    pub fn mesg_anime(&self) -> u8 {
        self.info.mesg_anime
    }

    pub fn set_mesg_anime(&mut self, id: u8) {
        self.info.set_mesg_anime_attr_info(id);
    }

    pub fn message_set(&mut self, msg_no: u32) -> u32 {
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
            Vec3::ZERO,
        )
    }

    pub fn message_set_actor(&mut self, msg_no: u32, eye: Vec3) -> u32 {
        let Some(archive) = self.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        fop_msgm_message_set_actor(
            self.procs,
            self.info,
            archive,
            self.clear_count,
            msg_no,
            None,
            eye,
        )
    }
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub player_pos: Option<Vec3>,
    pub player_eye_pos: Option<Vec3>,
    pub btsw_eye: Option<Vec3>,
    pub select: u8,
    pub mesg_send_button: u8,
    pub mail_center: Vec3,
    pub mail_eye: Vec3,
    pub talk_command: bool,
    pub demo_accepted: bool,
    pub cut_proc: bool,
    pub trig_a: bool,
    pub stick_value: f32,
    pub stick_angle: i16,
    pub mini_rupee: i32,
    pub message_count: i32,
    pub mini_game_type: u8,
    pub timer_status: i32,
    pub timer_end: i16,
    pub cursor_hidden: bool,
    pub cursor_pos: Vec3,
    pub end_mini_game: bool,
    pub mem: &'a MemBits,
    pub stories: &'a mut StoryBits,
    pub ev_flags: &'a mut EventFlags,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub events: &'a mut SimEvents,
    pub text: &'a mut Text<'a>,
}

impl Ctx<'_> {
    pub fn mail_ctx(&mut self) -> MailCtx<'_> {
        MailCtx {
            center: self.mail_center,
            eye: self.mail_eye,
            events: &mut *self.events,
            mini_rupee: &mut self.mini_rupee,
        }
    }

    pub fn event_reset(&mut self, index: i16) {
        if let Ok(slot) = usize::try_from(index) {
            let _ = self.active.end(slot);
        }
    }

    fn order_event(&mut self, index: i16, entity: Entity) {
        if let Ok(slot) = usize::try_from(index) {
            let _ = self.active.order(self.list, slot, entity);
        }
    }

    fn event_ended(&self, index: i16) -> bool {
        usize::try_from(index).is_ok_and(|slot| self.active.end_check(slot))
    }

    fn end_staff_cut(&mut self, name: &str) {
        let mut flags = EventFlags::default();
        self.active.end_staff_cut(self.list, &mut flags, name);
    }
}

#[derive(SystemParam)]
pub struct Env<'w> {
    pub flags: Res<'w, TextFlags>,
    pub mem: Res<'w, MemBits>,
    pub list: Res<'w, EventList>,
    pub stories: ResMut<'w, StoryBits>,
    pub ev_flags: ResMut<'w, EventFlags>,
    pub active: ResMut<'w, ActiveEvent>,
    pub events: ResMut<'w, SimEvents>,
    pub counters: ResMut<'w, Counters>,
    pub archive: Option<Res<'w, MsgArchives>>,
}

#[derive(SystemParam)]
pub struct Msg<'w, 's> {
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub players: Query<'w, 's, &'static Player>,
}

pub fn control(
    mut world: Env,
    mut msg: Msg,
    mut actors: Query<(Entity, &mut Actor, &mut NpcBmsw)>,
) {
    let player = msg.players.iter().next().copied();
    let clear_count = world.flags.clear_count;
    let select = msg.one.select_num;
    let send_button = msg.one.m116a;
    for (entity, mut actor, mut bmsw) in &mut actors {
        let cut_proc = usize::try_from(bmsw.event_idx)
            .ok()
            .is_some_and(|slot| world.active.stage_of(slot).is_some());
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            clear_count,
            one: &mut msg.one,
            two: &mut msg.two,
            archive: world
                .archive
                .as_deref()
                .map(|archive| archive as &dyn MsgArchive),
        };
        if !bmsw.entered {
            bmsw.create_init(&mut actor);
        }
        let mut ctx = Ctx {
            entity,
            player_pos: player.map(|link| link.pos),
            player_eye_pos: player.map(|link| {
                Vec3::new(
                    link.pos.x,
                    link.pos.y + PLAYER_HEAD_TOP + HIO_M04,
                    link.pos.z,
                )
            }),
            btsw_eye: None,
            select,
            mesg_send_button: send_button,
            mail_center: bmsw.cam.center,
            mail_eye: bmsw.cam.eye,
            talk_command: false,
            demo_accepted: false,
            cut_proc,
            trig_a: false,
            stick_value: 0.0,
            stick_angle: 0,
            mini_rupee: world.counters.mini_game_rupee,
            message_count: world.counters.message_count,
            mini_game_type: 0,
            timer_status: 0,
            timer_end: 0,
            cursor_hidden: false,
            cursor_pos: CURSOR_POS[0][1],
            end_mini_game: false,
            mem: &world.mem,
            stories: &mut world.stories,
            ev_flags: &mut world.ev_flags,
            list: &world.list,
            active: &mut world.active,
            events: &mut world.events,
            text: &mut text,
        };
        bmsw.execute(&mut actor, &mut ctx);
        world.counters.message_count = ctx.message_count;
    }
}
