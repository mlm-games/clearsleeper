use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::flags::{EventFlags, Switches};
use super::intent::DoIntent;
use super::player::Player;
use super::stream::add_calc_angle_s2;
use super::walk::{from_s16, to_s16};
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs, fop_msgm_message_set,
    fop_msgm_message_set_actor,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS, STATUS_TYPING,
};

pub const NPC_AH_ACTORS: [&str; 1] = ["Ah"];

pub fn is_npc_ah(name: &str) -> bool {
    NPC_AH_ACTORS.contains(&name)
}

const STAFF_AH: &str = "Ah"; // <- l_npc_staff_id[0], :29

// NpcDatStruct l_npc_dat - d_a_npc_ah.cpp:47-56, field order per d_a_npc_ah.h:15-49
const DAT_MAX_HEAD_X: i16 = 0x1000; // <- mMax_head_x, :48
const DAT_MAX_HEAD_Y: i16 = 0x2000; // <- mMax_head_y, :48
const DAT_MAX_BACKBONE_X: i16 = 0x0000; // <- mMax_backbone_x, :48
const DAT_MAX_BACKBONE_Y: i16 = 0x1000; // <- mMax_backbone_y, :48
const DAT_MIN_HEAD_X: i16 = -0x1000; // <- mMin_head_x 0xF000, :49
const DAT_MIN_HEAD_Y: i16 = -0x2000; // <- mMin_head_y 0xE000, :49
const DAT_MIN_BACKBONE_X: i16 = 0x0000; // <- mMin_backbone_x, :49
const DAT_MIN_BACKBONE_Y: i16 = -0x1000; // <- mMin_backbone_y 0xF000, :49
const DAT_MAX_TURN_STEP: i16 = 0x0800; // <- mMax_turn_step, :50
const DAT_PLAYER_EYE_Y: f32 = 0.0; // <- field_0x14, :50
const DAT_ATTN_X: f32 = 0.0; // <- field_0x18, :50
const DAT_ATTN_Y: f32 = 190.0; // <- field_0x1C, :51
const DAT_ATTN_Z: f32 = -20.0; // <- field_0x20, :51
const DAT_EYE_Y: f32 = 150.0; // <- field_0x24, :52
const DAT_ATTN_DIST: f32 = 200.0; // <- field_0x28, :52
const DAT_ATTN_DIST_NEAR: f32 = 0.0; // <- field_0x2C, :53
const DAT_ATTN_ANGLE: i16 = 0x7FFF; // <- field_0x30, :53
const DAT_LOOK_MAX_VEL: i16 = 0x0800; // <- field_0x32, :53
const DAT_RETURN_TIMER: i16 = 0x001E; // <- field_0x50, :55
const DAT_FIELD_52: u8 = 0x00; // <- field_0x52, :55
const DAT_FIELD_53: u8 = 0x00; // <- field_0x53, :55

// l_msg_ah_0..9 - :58-111; each entry is a two-slot chain terminated by 0x0000.
const MSG_AH: [u32; 10] = [
    0x36B1, 0x36B2, 0x36B3, 0x36B4, 0x36B5, 0x36B6, 0x36B7, 0x36B8, 0x36B9, 0x36BA,
];

const ATTN_DIST_EXTEND: f32 = 40.0; // <- :712
const ATTN_ANGLE_EXTEND: i32 = 0x71C; // <- :713
const LOOK_MAX_VEL_STEP: i16 = 0x800; // <- :795
const LOOK_MAX_VEL_SCALE: i16 = 4; // <- :795
const JNT_SCALE: i16 = 4; // <- d_npc.cpp dNpc_JntCtrl_c::follow
const JNT_MIN_STEP: i32 = 1; // <- d_npc.cpp dNpc_JntCtrl_c::move
const PLAYER_HEAD_TOP: f32 = 40.0; // <- d_a_player_main.cpp:11575

const CULL_MIN: [f32; 3] = [-70.0, 0.0, -70.0]; // <- fopAcM_setCullSizeBox, :281
const CULL_MAX: [f32; 3] = [70.0, 200.0, 70.0]; // <- :281

const BTP_TIMER_MIN: f32 = 30.0; // <- :823
const BTP_TIMER_RANGE: f32 = 100.0; // <- :823
const BTP_FRAME_MAX: u8 = 0xFF; // <- mBtpFrame >= getFrameMax() stand-in, :822

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const ITEM_BOMB: u32 = 7; // <- eventMesSet mItemNo = 7, :550
const ITEM_ARROW: u32 = 5; // <- eventMesSet mItemNo = 5, :556
const GIVE_FLAG_BOMB: u8 = 0x1; // <- field_0x748 & 1, :548
const GIVE_FLAG_ARROW: u8 = 0x2; // <- field_0x748 & 2, :554

const SWITCH_PR2: u8 = 0x6C; // <- phase_1 case 2, :167
const SWITCH_PR5: u8 = 0x10; // <- phase_1 case 5, :175
const FLAG_0520: u32 = 0x0520; // <- dSv_event_flag_c::UNK_0520, :170

const PRM_ARG0_MAX: u8 = 0xA; // <- getPrmArg0 clamp, :669
const PRM_SWSAVE_S: u32 = 0x08; // <- daNpcAh_c::Prm_e::PRM_SWSAVE_S, d_a_npc_ah.h:54
const PRM_SWSAVE_W: u32 = 0x00; // <- :55
const PRM_SWSAVE2_S: u32 = 0x08; // <- :57
const PRM_SWSAVE2_W: u32 = 0x08; // <- :56

const NO_STATUS: u8 = 0xFF; // <- talk2, :574
const PREV_STATUS_NONE: u16 = 0xFFFF; // <- talk2 field_0x734 = -1, :579
const ATTN_NONE: i8 = 0; // <- chkAttention, :749
const ATTN_PLAYER: i8 = 1; // <- :694
const ATTN_HOME: i8 = 2; // <- :725

const MODE_WAIT: u8 = 0; // <- moveProc[0], :355
const MODE_TALK: u8 = 1; // <- moveProc[1], :356

const ORDER_TALK: u8 = 1; // <- checkOrder field_0x744 == 1, :451
const ORDER_ATTENTION: u8 = 2; // <- checkOrder field_0x744 == 2, :451
const ORDER_ITEM: u8 = 3; // <- eventMesSet field_0x744 = 3, :551

const BCK_WAIT01: u8 = 0x00; // <- l_npc_anm_wait.mBckIdx, :36
const BCK_WAIT02: u8 = 0x01; // <- l_npc_anm_wait2.mBckIdx, :42
const BCK_NONE: u8 = 0xFF; // <- setAnmTbl terminator, :867
const ANM_LOOP: u8 = 2; // <- J3DFrameCtrl::EMode_LOOP, :873
const ANM_NONE: u8 = 1; // <- J3DFrameCtrl::EMode_NONE, :874
const MORF_OVERRIDE_NONE: f32 = -1.0; // <- daNpcAh_c::field_0x71C, :119

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn calc_timer(value: &mut i16) -> i16 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

fn rot_y(angle: i16, vec: Vec3) -> Vec3 {
    let rad = from_s16(angle);
    Vec3::new(
        vec.x * rad.cos() + vec.z * rad.sin(),
        vec.y,
        vec.z * rad.cos() - vec.x * rad.sin(),
    )
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

    pub fn trn_chk(&self) -> bool {
        self.trn
    }

    pub fn set_trn(&mut self) {
        self.trn = true;
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
        Self::ang_calc_s(out, target, JNT_SCALE, max_vel);
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
                JNT_SCALE as i32,
                i32::from(self.max_turn_step[row][axis]),
                JNT_MIN_STEP,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AhAnmDat {
    pub bck_idx: u8,
    pub morf: u8,
    pub field_02: i8,
}

pub const ANM_AH_WAIT: AhAnmDat = AhAnmDat {
    bck_idx: BCK_WAIT01,
    morf: 0x08,
    field_02: -1,
};
pub const ANM_AH_WAIT2: AhAnmDat = AhAnmDat {
    bck_idx: BCK_WAIT02,
    morf: 0x08,
    field_02: -1,
};

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct NpcAh {
    pub entered: bool,
    pub prev_do: bool,
    pub deleted: bool,
    pub move_state: u8,
    pub mode: u8,
    pub order: u8,
    pub give_flag: u8,
    pub sticky_attn: u8,
    pub return_yaw: u8,
    pub facing_player: u8,
    pub attn_mode: i8,
    pub head_only_follow: bool,
    pub timer_return: i16,
    pub home_angle: i16,
    pub shape_angle: i16,
    pub target_angle: i16,
    pub look_max_vel: i16,
    pub target_y_rot: i16,
    pub morf_override: f32,
    pub eye_pos: Vec3,
    pub item_no: u32,
    pub msg_id: u32,
    pub msg_cursor: bool,
    pub msg_bs_id: u32,
    pub msg_bound: bool,
    pub prev_msg_status: u16,
    pub anm_dat: Option<AhAnmDat>,
    pub bck_idx: u8,
    pub btp_frame: u8,
    pub btp_timer: i16,
    pub seed: [i32; 3],
    pub jnt: JntCtrl,
}

impl NpcAh {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            entered: false,
            prev_do: false,
            deleted: false,
            move_state: MODE_WAIT,
            mode: 0,
            order: 0,
            give_flag: 0,
            sticky_attn: 0,
            return_yaw: 0,
            facing_player: 0,
            attn_mode: ATTN_NONE,
            head_only_follow: true,
            timer_return: 0,
            home_angle: 0,
            shape_angle: 0,
            target_angle: 0,
            look_max_vel: 0,
            target_y_rot: 0,
            morf_override: MORF_OVERRIDE_NONE,
            eye_pos: Vec3::ZERO,
            item_no: 0,
            msg_id: 0,
            msg_cursor: false,
            msg_bs_id: FPC_M_ERROR_PROCESS_ID,
            msg_bound: false,
            prev_msg_status: PREV_STATUS_NONE,
            anm_dat: None,
            bck_idx: BCK_WAIT01,
            btp_frame: 0,
            btp_timer: 0,
            seed: [RND_SEED; 3],
            jnt: JntCtrl::create(),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    /// `daNpcAh_c::getPrmArg0` (`:667-671`).
    fn prm_arg0(&self, parameters: u32) -> u8 {
        let mask = (1u32 << PRM_SWSAVE_W) - 1;
        let raw = ((parameters >> PRM_SWSAVE_S) & mask) as u8;
        if raw >= PRM_ARG0_MAX { 0 } else { raw }
    }

    /// `daNpcAh_c::getSwBit` (`:674-676`).
    pub fn sw_bit(&self, parameters: u32) -> u8 {
        let mask = (1u32 << PRM_SWSAVE2_W) - 1;
        ((parameters >> PRM_SWSAVE2_S) & mask) as u8
    }

    /// `phase_1` (`:162-187`): the `cPhs_STOP_e` cases, which cancel creation entirely.
    fn create_stopped(
        &self,
        parameters: u32,
        switches: &Switches,
        flags: &EventFlags,
    ) -> bool {
        match self.prm_arg0(parameters) {
            2 => switches.test(SWITCH_PR2) || !flags.test(FLAG_0520),
            5 => switches.test(SWITCH_PR5),
            9 => switches.test(self.sw_bit(parameters)),
            _ => false,
        }
    }

    /// `_execute` (`:361-371`): the per-frame `fopAcM_delete` switches.
    fn switch_delete(&self, parameters: u32, switches: &Switches) -> bool {
        match self.prm_arg0(parameters) {
            2 => switches.test(SWITCH_PR2),
            5 => switches.test(SWITCH_PR5),
            9 => switches.test(self.sw_bit(parameters) & 0xFF),
            _ => false,
        }
    }

    /// `daNpcAh_c::createInit` (`:262-304`).
    pub fn create_init(&mut self, actor: &mut Actor) {
        self.home_angle = actor.angle[1];
        self.shape_angle = actor.angle[1];
        self.look_max_vel = 0;
        self.sticky_attn = DAT_FIELD_52;
        self.return_yaw = DAT_FIELD_53;
        self.timer_return = 0;
        self.give_flag = 0;
        self.facing_player = 0;
        self.set_anm_tbl(ANM_AH_WAIT);
        self.entered = true;
    }

    /// `daNpcAh_c::setAnm` (`:854-863`).
    fn set_anm(&mut self, bck_ix: u8, _loop_mode: u8, morf: f32) {
        let speed = if self.morf_override >= 0.0 {
            let speed = self.morf_override;
            self.morf_override = MORF_OVERRIDE_NONE;
            speed
        } else {
            morf
        };
        let _ = speed;
        self.bck_idx = bck_ix;
    }

    /// `daNpcAh_c::setAnmTbl` (`:866-880`).
    fn set_anm_tbl(&mut self, dat: AhAnmDat) -> bool {
        if dat.bck_idx == BCK_NONE {
            self.anm_dat = None;
            return true;
        }
        self.anm_dat = Some(dat);
        let loop_mode = if dat.field_02 > 0 { ANM_NONE } else { ANM_LOOP };
        if self.bck_idx != dat.bck_idx || loop_mode == ANM_NONE {
            self.set_anm(dat.bck_idx, loop_mode, f32::from(dat.morf));
        }
        false
    }

    /// `daNpcAh_c::setAnmFromMsgTag` (`:654-664`).
    fn set_anm_from_msg_tag(&mut self, attr: u8) {
        match attr {
            0 => {
                self.set_anm_tbl(ANM_AH_WAIT);
            }
            1 => {
                self.set_anm_tbl(ANM_AH_WAIT2);
            }
            _ => {}
        }
    }

    /// `daNpcAh_c::playTexPatternAnm` (`:820-830`), minus the `getFrameMax()` bound
    /// (the BTP archive does not ship).
    fn play_tex_pattern_anm(&mut self) {
        if calc_timer(&mut self.btp_timer) == 0 {
            if self.btp_frame >= BTP_FRAME_MAX {
                self.btp_frame = self.btp_frame.wrapping_sub(BTP_FRAME_MAX);
                self.btp_timer = (rnd(&mut self.seed) * BTP_TIMER_RANGE + BTP_TIMER_MIN) as i16;
            } else {
                self.btp_frame = self.btp_frame.wrapping_add(1);
            }
        }
    }

    /// `daNpcAh_c::playAnm` (`:833-851`), minus the `mpMorf->play` return (BCK frame
    /// counts are unavailable), so the animation-chain advance cannot run.
    fn play_anm(&mut self) {}

    /// `daNpcAh_c::chkAttention` (`:688-765`).
    pub fn chk_attention(&mut self, actor: &Actor, ctx: &Ctx<'_>) {
        self.facing_player = 0;
        if ctx.cut_attn {
            self.eye_pos = ctx.cut_attn_pos;
            self.attn_mode = ATTN_PLAYER;
            if self.sticky_attn != 0 {
                self.head_only_follow = false;
                self.jnt.set_trn();
            } else {
                self.head_only_follow = true;
            }
            if self.sticky_attn == 0 {
                self.sticky_attn = 1;
            }
        } else {
            let (Some(player), Some(eye)) = (ctx.player_pos, ctx.player_eye) else {
                self.target_angle = DAT_ATTN_ANGLE;
                return;
            };
            let delta = Vec3::new(player.x - actor.pos.x, 0.0, player.z - actor.pos.z);
            let dist_xz = (delta.x * delta.x + delta.z * delta.z).sqrt();
            let mut max_dist = DAT_ATTN_DIST;
            let mut max_angle = i32::from(DAT_ATTN_ANGLE);
            if self.sticky_attn != 0 {
                max_dist += ATTN_DIST_EXTEND;
                max_angle += ATTN_ANGLE_EXTEND;
            }
            let angle = to_s16(delta.x.atan2(delta.z)).wrapping_sub(self.shape_angle);
            if max_dist > dist_xz && max_angle > i32::from(angle).abs() {
                self.eye_pos = eye;
                self.attn_mode = ATTN_PLAYER;
                self.head_only_follow = self.sticky_attn == 0;
                if self.return_yaw == 0 {
                    self.target_y_rot = self.home_angle;
                    self.head_only_follow = false;
                    self.attn_mode = ATTN_HOME;
                    self.jnt.set_trn();
                }
                if self.sticky_attn == 0 {
                    self.sticky_attn = 1;
                }
            } else {
                if self.sticky_attn == 1 {
                    self.sticky_attn = 0;
                    self.timer_return = DAT_RETURN_TIMER;
                }
                if DAT_ATTN_DIST_NEAR > dist_xz {
                    self.eye_pos = eye;
                    self.attn_mode = ATTN_PLAYER;
                    self.head_only_follow = self.sticky_attn == 0;
                    if self.return_yaw == 0 {
                        self.target_y_rot = self.home_angle;
                        self.head_only_follow = false;
                        self.attn_mode = ATTN_HOME;
                        self.jnt.set_trn();
                    }
                    self.facing_player = 1;
                } else {
                    self.attn_mode = ATTN_NONE;
                    if self.timer_return != 0 {
                        self.timer_return -= 1;
                    } else {
                        self.target_y_rot = self.home_angle;
                        self.head_only_follow = false;
                        self.attn_mode = ATTN_HOME;
                        self.jnt.set_trn();
                    }
                }
            }
        }
        self.target_angle = DAT_LOOK_MAX_VEL;
    }

    /// `daNpcAh_c::lookBack` (`:768-802`).
    pub fn look_back(&mut self, actor: &mut Actor, ctx: &Ctx<'_>) {
        let mut target = self.target_angle;
        let mut desired_y_rot = actor.angle[1];
        let mut dst = None;
        let mut head_only_follow = self.head_only_follow;
        let src = self.eye_pos;
        match self.attn_mode {
            ATTN_PLAYER => dst = Some(self.eye_pos),
            ATTN_HOME => desired_y_rot = self.target_y_rot,
            _ => {}
        }
        if self.mode != 0 && self.sticky_attn != 0 {
            head_only_follow = false;
            self.jnt.set_trn();
        }
        if self.jnt.trn_chk() {
            if ctx.cut_turn_speed != 0 {
                target = ctx.cut_turn_speed;
            }
            add_calc_angle_s2(
                &mut self.look_max_vel,
                target,
                LOOK_MAX_VEL_SCALE,
                LOOK_MAX_VEL_STEP,
            );
        } else {
            self.look_max_vel = 0;
        }
        let vel = self.look_max_vel;
        self.jnt.look_at_target(
            &mut actor.angle[1],
            dst,
            src,
            desired_y_rot,
            vel,
            head_only_follow,
        );
        self.shape_angle = actor.angle[1];
    }

    /// `daNpcAh_c::getMsg` (`:630-641`).
    pub fn get_msg(&mut self, arg0: u8, ctx: &Ctx<'_>) -> u32 {
        self.msg_cursor = false;
        self.msg_id = 0;
        if !ctx.photo && !ctx.talk_xy {
            let index = usize::from(arg0) % MSG_AH.len();
            self.msg_id = MSG_AH[index];
            self.msg_cursor = true;
        }
        if self.msg_cursor {
            self.msg_id
        } else {
            0
        }
    }

    /// `daNpcAh_c::next_msgStatus` (`:608-627`). Every `l_msg_ah_*` chain is exactly
    /// two slots with `0x0000` in the second, so `mpMsgNo++` always lands on the
    /// terminator: `mpMsgNo = 0` and the base `MSG_ENDS` is returned.
    fn next_msg_status(&mut self, curr: &mut u32) -> u8 {
        let _ = curr;
        self.msg_cursor = false;
        STATUS_MSG_ENDS
    }

    /// `daNpcAh_c::chkMsg` (`:644-646`): the body is empty.
    fn chk_msg(&mut self) {}

    /// `daNpcAh_c::talk2` (`:573-605`).
    fn talk2(&mut self, get_msg: bool, arg0: u8, ctx: &mut Ctx<'_>) -> u8 {
        let mut status = NO_STATUS;
        if self.msg_bs_id == FPC_M_ERROR_PROCESS_ID {
            if get_msg {
                self.msg_id = self.get_msg(arg0, ctx);
            }
            let eye = self.eye_pos;
            self.msg_bs_id = ctx.text.message_set_actor(self.msg_id, eye);
            self.msg_bound = false;
            self.prev_msg_status = PREV_STATUS_NONE;
        } else if self.msg_bound {
            status = ctx.text.status().unwrap_or(NO_STATUS);
            match status {
                STATUS_MSG_DISPLAYED => {
                    let mut curr = self.msg_id;
                    let next = self.next_msg_status(&mut curr);
                    self.msg_id = curr;
                    ctx.text.set_status(next);
                    if next == STATUS_MSG_CONTINUES {
                        let no = self.msg_id;
                        ctx.text.message_set(no);
                    }
                }
                STATUS_TYPING => {
                    if self.prev_msg_status == STATUS_MSG_CONTINUES as u16 {
                        self.chk_msg();
                    }
                }
                STATUS_BOX_CLOSED => {
                    ctx.text.set_status(STATUS_MSG_DESTROYED);
                    self.msg_bs_id = FPC_M_ERROR_PROCESS_ID;
                }
                _ => {}
            }
            self.prev_msg_status = status as u16;
        } else {
            self.msg_bound = ctx.text.procs.search_by_id(self.msg_bs_id).is_some();
        }
        status
    }

    /// `daNpcAh_c::eventMesSetInit` (`:520-542`) and `eventMesSet` (`:545-562`). The
    /// `MsgNo` integer comes from the stage event-info table, so only the cut's own
    /// `talk2(0)` and the give-item hand-off are modelled.
    fn event_mes_set(&mut self, arg0: u8, ctx: &mut Ctx<'_>) -> bool {
        let talk_val = self.talk2(false, arg0, ctx);
        if talk_val == STATUS_BOX_CLOSED {
            if self.give_flag & GIVE_FLAG_BOMB != 0 {
                self.give_flag &= !GIVE_FLAG_BOMB;
                self.order = ORDER_ITEM;
                self.item_no = ITEM_BOMB;
            } else if self.give_flag & GIVE_FLAG_ARROW != 0 {
                self.give_flag &= !GIVE_FLAG_ARROW;
                self.order = ORDER_ITEM;
                self.item_no = ITEM_ARROW;
            }
        }
        talk_val == STATUS_BOX_CLOSED
    }

    /// `daNpcAh_c::executeCommon` (`:399-405`).
    fn execute_common(&mut self) -> u8 {
        self.order = if self.sticky_attn != 0 { 1 } else { 0 };
        if self.mode == 1 && self.move_state != MODE_TALK {
            self.execute_set_mode(MODE_TALK);
        }
        self.mode
    }

    /// `daNpcAh_c::executeSetMode` (`:409-412`).
    fn execute_set_mode(&mut self, proc: u8) {
        self.move_state = match proc {
            MODE_TALK => {
                self.execute_talk_init();
                MODE_TALK
            }
            _ => {
                self.execute_wait_init();
                MODE_WAIT
            }
        };
    }

    /// `daNpcAh_c::executeWaitInit` (`:415-424`).
    fn execute_wait_init(&mut self) {
        self.set_anm_tbl(ANM_AH_WAIT);
        self.jnt.set_param(
            DAT_MAX_BACKBONE_X,
            DAT_MAX_BACKBONE_Y,
            DAT_MIN_BACKBONE_X,
            DAT_MIN_BACKBONE_Y,
            DAT_MAX_HEAD_X,
            DAT_MAX_HEAD_Y,
            DAT_MIN_HEAD_X,
            DAT_MIN_HEAD_Y,
            DAT_MAX_TURN_STEP,
        );
    }

    /// `daNpcAh_c::executeTalkInit` (`:432-434`).
    fn execute_talk_init(&mut self) {}

    /// `daNpcAh_c::executeTalk` (`:437-447`).
    fn execute_talk(&mut self, arg0: u8, ctx: &mut Ctx<'_>) {
        self.execute_common();
        if self.talk2(true, arg0, ctx) == STATUS_BOX_CLOSED {
            self.mode = 0;
            self.execute_set_mode(MODE_WAIT);
        } else {
            let attr = ctx.text.mesg_anime();
            self.set_anm_from_msg_tag(attr);
            ctx.text.set_mesg_anime(0xFF);
        }
    }

    /// `daNpcAh_c::executeWait` (`:427-429`).
    fn execute_wait(&mut self) {
        self.execute_common();
    }

    /// `daNpcAh_c::checkOrder` (`:450-455`).
    fn check_order(&mut self, demo_accept: bool, talk_command: bool) {
        if demo_accept {
            return;
        }
        if !talk_command {
            return;
        }
        if self.order == ORDER_ATTENTION || self.order == ORDER_TALK {
            self.mode = 1;
            self.execute_set_mode(MODE_TALK);
        }
    }

    /// `daNpcAh_c::eventMove` (`:465-478`).
    fn event_move(&mut self, arg0: u8, ctx: &mut Ctx<'_>) {
        let attn_flag = ctx.cut_attn;
        if ctx.cut_proc {
            if !ctx.cut_attn {
                ctx.cut_attn = attn_flag;
            }
        } else {
            let attr = ctx.text.mesg_anime();
            self.set_anm_from_msg_tag(attr);
            ctx.text.set_mesg_anime(0xFF);
            let _ = self.event_mes_set(arg0, ctx);
        }
    }

    /// `daNpcAh_c::_execute` (`:360-396`), minus the renderer, `fopAcM_posMoveF`,
    /// `mObjAcch.CrrPos` and the `mCyl` registration.
    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        let arg0 = ctx.arg0;
        if ctx.switch_delete {
            self.deleted = true;
        }
        self.chk_attention(actor, ctx);
        let demo_accept = ctx.demo_accept;
        let talk_command = ctx.talk_command;
        self.check_order(demo_accept, talk_command);
        if !ctx.event_run || talk_command {
            match self.move_state {
                MODE_WAIT => self.execute_wait(),
                _ => self.execute_talk(arg0, ctx),
            }
        } else {
            self.event_move(arg0, ctx);
        }
        self.play_tex_pattern_anm();
        self.play_anm();
        let attn = actor.pos
            + rot_y(
                actor.angle[1],
                Vec3::new(DAT_ATTN_X, DAT_ATTN_Y, DAT_ATTN_Z),
            );
        let _ = attn;
        self.eye_pos = Vec3::new(actor.pos.x, actor.pos.y + DAT_EYE_Y, actor.pos.z);
        self.look_back(actor, ctx);
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
    pub arg0: u8,
    pub switch_delete: bool,
    pub player_pos: Option<Vec3>,
    pub player_eye: Option<Vec3>,
    pub cut_attn: bool,
    pub cut_attn_pos: Vec3,
    pub cut_turn_speed: i16,
    pub cut_proc: bool,
    pub photo: bool,
    pub talk_xy: bool,
    pub talk_command: bool,
    pub demo_accept: bool,
    pub event_run: bool,
    pub text: &'a mut Text<'a>,
}

#[derive(SystemParam)]
pub struct Env<'w> {
    pub text_flags: Res<'w, TextFlags>,
    pub switches: Res<'w, Switches>,
    pub flags: Res<'w, EventFlags>,
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
    world: Env,
    mut msg: Msg,
    list: Res<EventList>,
    active: Res<ActiveEvent>,
    do_intent: Res<DoIntent>,
    mut commands: Commands,
    mut actors: Query<(Entity, &mut Actor, &mut NpcAh)>,
) {
    let player = msg.players.iter().next().copied();
    let clear_count = world.text_flags.clear_count;
    let event_run = msg.info.event_run_check();
    let do_now = do_intent.0;
    let archive = world
        .archive
        .as_deref()
        .map(|held| held as &dyn MsgArchive);
    for (entity, mut actor, mut ah) in &mut actors {
        if !ah.entered && ah.create_stopped(actor.parameters, &world.switches, &world.flags) {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        let tap = do_now && !ah.prev_do;
        ah.prev_do = do_now;
        let cut = ActiveEvent::staff_cut(&active, &list, STAFF_AH);
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            clear_count,
            one: &mut msg.one,
            two: &mut msg.two,
            archive,
        };
        let arg0 = ah.prm_arg0(actor.parameters);
        let mut ctx = Ctx {
            arg0,
            switch_delete: ah.switch_delete(actor.parameters, &world.switches),
            player_pos: player.map(|link| link.pos),
            player_eye: player.map(|link| {
                Vec3::new(
                    link.pos.x,
                    link.pos.y + PLAYER_HEAD_TOP + DAT_PLAYER_EYE_Y,
                    link.pos.z,
                )
            }),
            cut_attn: false,
            cut_attn_pos: Vec3::ZERO,
            cut_turn_speed: 0,
            cut_proc: cut.is_some(),
            photo: false,
            talk_xy: false,
            talk_command: tap,
            demo_accept: event_run,
            event_run,
            text: &mut text,
        };
        if !ah.entered {
            ah.create_init(&mut actor);
        }
        ah.execute(&mut actor, &mut ctx);
        if ah.deleted {
            let _ = commands.entity(entity).despawn();
        }
    }
}