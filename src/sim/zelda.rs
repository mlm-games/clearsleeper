//! `daNpc_Zl1_c` — Princess Zelda (`Zl1`, `fpcNm_NPC_ZL1_e`).

use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::area::StageName;
use super::event::{ActiveEvent, EventList};
use super::flags::{Collect, EventFlags};
use super::intent::{DoIntent, MoveIntent, PadButtons};
use super::player::Player;
use super::walk;
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs,
    fop_msgm_message_set_actor,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS, STATUS_TYPING,
};

/// `OBJNAME("Zl1", fpcNm_NPC_ZL1_e, 255, 60)` — `d_stage.cpp:725`.
pub const ZELDA_ACTOR: &str = "Zl1";

/// `l_evn_tbl[4]` — `:127-132`, indexed by `field_0x84A - 3`.
pub const EVENT_NAMES: [&str; 4] = [
    "yuukaigo", // <- :128
    "ooi", // <- :129
    "majyuu_shinnyuu", // <- :130
    "nakaniwa", // <- :131
];

/// `decideType` writes `mArcName` — `:1330-1337`. Types 0, 6 and 7 use `"Zl2"`,
/// every other accepted type `"Zl"`.
const ARC_SLEEPING: &str = "Zl2"; // <- :1332
const ARC_AWAKE: &str = "Zl"; // <- :1335

/// `decideType(fopAcM_GetParam(this) & 0xFF)` — `:2883`.
const PARAM_BITS: u32 = 0xFF;
/// The last `switch (param_1)` case `decideType` accepts — `:1321`.
const TYPE_MAX: i8 = 7;
/// `field_0x7BC = -1` — `:406`.
const TURN_STEP_DEFAULT: i8 = -1;
/// `field_0x849 = 0x11` — `:409`.
const MODE_MAIN: i8 = 0x11;
/// `gravity = 0.0f` — `:408`; `gravity = -4.5f` in `init_ZL1_5` — `:372`.
const GRAVITY_DEFAULT: f32 = 0.0;
const GRAVITY_TYPE5: f32 = -4.5;

/// `d_sv_event_flag_c` ids — `d_save_event_flag.inc`.
const BIT_0001: u32 = 0x0001; // <- UNK_0001, :289
const BIT_0101: u32 = 0x0101; // <- UNK_0101, :302
const BIT_0801: u32 = 0x0801; // <- UNK_0801, :322
const BIT_0802: u32 = 0x0802; // <- UNK_0802, :2402
const BIT_0804: u32 = 0x0804; // <- UNK_0804, :2464
const BIT_0810: u32 = 0x0810; // <- UNK_0810, :1022
const BIT_0E20: u32 = 0x0E20; // <- UNK_0E20, :313
const BIT_2401: u32 = 0x2401; // <- UNK_2401, :313
const BIT_2908: u32 = 0x2908; // <- UNK_2908, :1013
/// `dSv_event_flag_c::HYRULE_COURTYARD_CUTSCENE` — `:348`.
const BIT_COURTYARD: u32 = 0x2E04;
/// `dSv_event_flag_c::ZELDA_AWAKENED` — `:344`; set by `awake_zelda.stb`.
const BIT_AWAKENED: u32 = 0x2D02;

/// `field_0x84A`, the event order — `:1053-1064`.
const ORDER_NONE: i8 = 0;
const ORDER_SPEAK: i8 = 1; // <- :1054
const ORDER_TALK: i8 = 2; // <- :1054
const ORDER_FIRST_EVENT: i8 = 3; // <- `field_0x84A >= 3`, :1060
/// `field_0x7A8 = field_0x84A - 3` — `:1060`.
const EVENT_BASE: i8 = 3;

/// `dComIfGp_getStartStageName()` — `:357`, `:360`.
const STAGE_HYRULE: &str = "Hyrule"; // <- :357
const STAGE_HYROOM: &str = "Hyroom"; // <- :360

/// `cLib_getRndValue(min, range) = min + cM_rndF(range)` — `c_lib.h:80`.
/// `cLib_getRndValue(0x5A, 0xB4)` — `:1918`, `:2309`.
const RND_MIN_SWAY: i32 = 0x5A;
const RND_RANGE_SWAY: f32 = 180.0; // <- 0xB4
/// `cLib_getRndValue(0xf, 0x1e)` — `:2173`, `:2213`.
const RND_MIN_TIMER: i32 = 0x0F;
const RND_RANGE_TIMER: f32 = 30.0; // <- 0x1E
/// `cM_initRnd(100, 100, 100)` — `m_Do_machine.cpp:617`.
const RND_SEED: i32 = 100;
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA]; // <- cM_rnd, c_math.cpp:181
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673]; // <- c_math.cpp:181
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0]; // <- c_math.cpp:182

/// `cLib_addCalcAngleS(&current.angle.y, field_0x738.y, 4, 0x800, 0x80)` — `:2159`.
/// Five-argument overload (`c_lib.h:22`), transcribed by `walk::add_calc_angle_s`.
const TURN_SCALE: i16 = 4;
const TURN_MAX: i16 = 0x800;
const TURN_MIN: i16 = 0x80;

/// `field_0x84B`, the `setStt` state — `:1893-1922`.
const STT_NONE: i8 = 0; // <- case 0, :1897
const STT_IDLE: i8 = 1; // <- case 1, :1901
const STT_TALK: i8 = 2; // <- case 2, :1906
const STT_APPROACH: i8 = 3; // <- case 3, :1915
const STT_TURN: i8 = 4; // <- case 4, :1898
const STT_SEE: i8 = 5; // <- case 5, :1920
const STT_DEMO_A: i8 = 6; // <- case 6, :1902
const STT_DEMO_B: i8 = 7; // <- case 7, :1902
const STT_DEMO_C: i8 = 8; // <- case 8, :1902
const STT_HOLD: i8 = 9; // <- case 9, :1902

/// `set_action`'s `field_0x850` — `:1881-1890`.
const PHASE_STARTING: i8 = 0;
const PHASE_ONGOING: i8 = 1;
const PHASE_ENDING: i8 = 9;

/// `dComIfGp_evmng_getEventIdx` returning -1 — `:394`.
const NO_EVENT: usize = usize::MAX;

/// `fopNpc_npc_c::talk`'s no-status return — `d_npc.cpp:665`.
const NO_STATUS: u8 = 0xFF;

/// `event_proc`'s hard move for event slot 1 — `:1851`, `:2465`.
const EVENT_MOVE_POS: Vec3 = Vec3::new(-37692.0, 2200.0, 8016.0); // <- :1851
const EVENT_MOVE_YAW: i16 = -0x5C72; // <- :1852

/// `a_chk_playerPos[2][3]` — `:234-243`.
const CHK_PLAYER_POS: [[[f32; 3]; 3]; 2] = [
    [
        [0.0, -1145.0, -4005.0],
        [0.0, -2099.0, 3913.0],
        [0.0, -1145.0, -3542.0],
    ],
    [
        [-4.0, 0.0, 2695.0],
        [0.0, 149.0, -955.0],
        [0.0, -1997.0, -7758.0],
    ],
];
/// `a_set_tetoraPos[2][3]` — `:245-254`.
const SET_TETORA_POS: [[[f32; 3]; 3]; 2] = [
    [
        [-75.0, -1146.0, -4160.0],
        [80.0, -2100.0, 4193.0],
        [-75.0, -1146.0, -3700.0],
    ],
    [
        [92.0, 0.0, 2842.0],
        [-129.0, 149.0, -1132.0],
        [107.0, -1978.0, -7559.0],
    ],
];
/// `a_set_tetoraAng[2][3]`, in degrees — `:256-259`.
const SET_TETORA_ANG: [[f32; 3]; 2] = [
    [16.0, -171.0, 16.0],
    [-159.0, 15.0, -153.0],
];

/// The action functions `set_action` selects between — `d_a_npc_zl1.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Wait,
    Demo1,
    Demo2,
    Optn,
}

/// The two outcomes `next_msgStatus` can return.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Next(u32),
    Ends,
}

impl Step {
    fn status(self) -> u8 {
        match self {
            Step::Next(_) => STATUS_MSG_CONTINUES,
            Step::Ends => STATUS_MSG_ENDS,
        }
    }
}

/// `daNpc_Zl1_c` — `d_a_npc_zl1.h`. The fields are public because the readers
/// this port omits — `lookBack` for `look`, `optn_1` for `sway`,
/// `_execute`'s joint-limit pick for `turn_step`, `play_animation` for `mode` —
/// are all outside this file; see the report.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Zelda {
    pub kind: i8,
    pub arc: &'static str,
    pub action: Action,
    pub phase: i8,
    pub state: i8,
    pub state_prev: i8,
    pub look: i8,
    pub order: i8,
    pub event_slot: usize,
    pub event_idx: [usize; 4],
    pub talk_cmd: bool,
    pub look_lock: bool,
    pub has_attention: bool,
    pub seeded: bool,
    /// `field_0x7D2`, set by `init_ZL1_0` — `:293`; `_draw` returns before any
    /// draw call when it is set — `:2654`.
    pub hidden_demo: bool,
    /// `field_0x7D4`, written only by `cut_init_DRW_ONOFF` — `:1570`.
    pub draw_off: bool,
    /// `_draw`'s `if (field_0x7D2 || field_0x7D4) return TRUE;` — `:2654`.
    pub drawn: bool,
    /// `field_0x7D3`, the Outset-Island demonstration flag.
    pub demo_wait: bool,
    pub in_event: bool,
    pub msg_id: u32,
    pub msg_no: u32,
    pub item_no: u8,
    pub timer: i32,
    pub sway: i32,
    pub turn_step: i8,
    pub mode: i8,
    pub home_pos: Vec3,
    pub home_angle: i16,
    pub attn_pos: Vec3,
    pub anchor: Vec3,
    pub seed: [i32; 3],
    pub prev_do: bool,
    pub entered: bool,
    pub refused: bool,
}

impl Zelda {
    /// `daNpc_Zl1_c::_create` (`:2875-2915`) plus `decideType` (`:1288-1345`).
    pub fn spawn(parameters: u32) -> Self {
        let kind = (parameters & PARAM_BITS) as u8 as i8; // <- :2883
        let arc = match kind {
            0 | STT_DEMO_A | STT_DEMO_B => ARC_SLEEPING, // <- :1332
            _ => ARC_AWAKE,                               // <- :1335
        };
        Self {
            kind,
            arc,
            action: Action::Wait,
            phase: PHASE_STARTING,
            state: STT_NONE,
            state_prev: STT_NONE,
            look: 0,
            order: ORDER_NONE,
            event_slot: 0,
            event_idx: [NO_EVENT; 4],
            talk_cmd: false,
            look_lock: false,
            has_attention: false,
            seeded: false,
            hidden_demo: false,
            draw_off: false,
            drawn: true,
            demo_wait: false,
            in_event: false,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            msg_no: 0,
            item_no: 0xFF,
            timer: 0,
            sway: 0,
            turn_step: TURN_STEP_DEFAULT, // <- :406
            mode: MODE_MAIN,              // <- :409
            home_pos: Vec3::ZERO,
            home_angle: 0,
            attn_pos: Vec3::ZERO,
            anchor: Vec3::ZERO,
            seed: [RND_SEED; 3],
            prev_do: false,
            entered: false,
            // `decideType` returns false for a parameter outside 0..=7 and
            // `_create` then returns `cPhs_ERROR_e` — :2884-2886 — so the process
            // is cancelled during Create and never executes (RULE 6b).
            refused: kind < 0 || kind > TYPE_MAX,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -50.0f, -20.0f, -50.0f, 50.0f, 140.0f, 50.0f)`
    /// — `:2905`. `g_profile_NPC_ZL1` is `fopAc_CULLBOX_CUSTOM_e` (`:3061`) and
    /// that call is the only box the translation unit makes.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (
            [-50.0, -20.0, -50.0], // <- fopAcM_setCullSizeBox, :2905
            [50.0, 140.0, 50.0],   // <- :2905
        )
    }

    /// `daNpc_Zl1_c::mArcName` — `:1330-1337`.
    pub fn arc_name(&self) -> &'static str {
        self.arc
    }

    /// `daNpc_Zl1_c::setAttention` — `:1277-1286`; `l_HIO.mPrmTbl.field_1C` is
    /// `140.0f` — `:101`.
    fn set_attention(&mut self, actor: &Actor) {
        self.attn_pos = Vec3::new(actor.pos.x, actor.pos.y + 140.0, actor.pos.z);
    }

    /// `daNpc_Zl1_c::setStt` — `:1893-1922`.
    fn set_stt(&mut self, next: i8, actor: &mut Actor) {
        let previous = self.state;
        self.state = next;
        match next {
            STT_NONE | STT_TURN => {}
            STT_IDLE | STT_DEMO_A | STT_DEMO_B | STT_DEMO_C | STT_HOLD => {
                self.order = ORDER_NONE; // <- :1903
            }
            STT_TALK => {
                self.order = ORDER_NONE; // <- :1906
                self.state_prev = previous; // <- field_0x84C = temp, :1909
            }
            STT_APPROACH => {
                actor.speed_f = 0.0; // <- :1916
                actor.speed.y = 0.0; // <- :1917
                let roll = rnd(&mut self.seed) * RND_RANGE_SWAY; // <- :1918
                self.sway = RND_MIN_SWAY + roll as i32;
            }
            STT_SEE => {
                self.demo_wait = true; // <- field_0x7D3 = true, :1921
            }
            _ => {}
        }
    }

    /// `daNpc_Zl1_c::createInit` (`:391-458`) folded with `init_ZL1_0` ..
    /// `init_ZL1_7` (`:288-390`).
    fn create_init(&mut self, actor: &mut Actor, list: &EventList, ctx: &mut TalkCtx<'_>) {
        for (i, name) in EVENT_NAMES.iter().enumerate() {
            self.event_idx[i] = list.event_index(name).unwrap_or(NO_EVENT); // <- :394-396
        }
        self.turn_step = TURN_STEP_DEFAULT; // <- :406
        self.mode = MODE_MAIN; // <- :409
        self.init_zl1(actor, ctx);
        actor.shape_angle = actor.angle; // <- shape_angle = field_0x73E, :445
        actor.gravity = if self.kind == 5 {
            GRAVITY_TYPE5 // <- :372
        } else {
            GRAVITY_DEFAULT // <: :408
        };
        self.home_pos = actor.pos; // <- field_0x72C, :2786
        self.home_angle = actor.angle[1]; // <- field_0x738, :2787
        self.set_attention(actor);
        self.entered = true;
    }

    /// `init_ZL1_0` .. `init_ZL1_7` — `:288-390`.
    fn init_zl1(&mut self, actor: &mut Actor, ctx: &mut TalkCtx<'_>) {
        match self.kind {
            0 => {
                if !ctx.flags.test(BIT_0001) {
                    self.action = Action::Demo1; // <- :290
                    self.hidden_demo = true; // <- field_0x7D2 = true, :293
                }
            }
            1 => {
                if !ctx.flags.test(BIT_0101) {
                    self.action = Action::Demo1; // <- :303
                }
            }
            2 => {
                if ctx.flags.test(BIT_0E20) && !ctx.flags.test(BIT_2401) {
                    self.action = Action::Wait; // <- :314
                }
            }
            3 => {
                if !ctx.flags.test(BIT_0801) {
                    self.anchor = anchor(actor); // <- :323-326
                    self.action = Action::Demo2; // <- :327
                }
            }
            STT_TURN => {
                self.action = Action::Wait; // <- :337
            }
            5 => {
                if !ctx.flags.test(BIT_AWAKENED) {
                    self.action = Action::Optn; // <- :345
                    if !ctx.flags.test(BIT_COURTYARD) {
                        self.order = 6; // <- field_0x84A = 6, :350
                        self.event_slot = 3; // <- field_0x7A8 = 6 - 3, :351
                    } else if ctx.stage == STAGE_HYRULE {
                        self.set_start_pos(1, ctx.player, actor); // <- :357
                    } else if ctx.stage == STAGE_HYROOM {
                        self.set_start_pos(0, ctx.player, actor); // <- :360
                    }
                }
            }
            STT_DEMO_A | STT_DEMO_B => {
                self.action = Action::Demo1; // <- :374, :383
            }
            _ => {}
        }
    }

    /// `daNpc_Zl1_c::set_startPos` — `:232-286`. Picks the nearest of the three
    /// `a_chk_playerPos` anchors for the stage the player is standing in.
    fn set_start_pos(&mut self, group: usize, player: Option<&Player>, actor: &mut Actor) {
        let Some(player) = player else {
            return;
        };
        if group >= 2 {
            return; // <- `if (param_1 == 0 || param_1 == 1)`, :265
        }
        let mut best = f32::INFINITY;
        let mut at = 3usize;
        for i in 0..3 {
            let anchor = Vec3::from_array(CHK_PLAYER_POS[group][i]);
            let diff = anchor - player.pos;
            let spread = Vec3::new(diff.x, 0.0, diff.z).length(); // <- :271
            if best > spread {
                best = spread;
                at = i;
            }
        }
        if at < 3 {
            actor.pos = Vec3::from_array(SET_TETORA_POS[group][at]); // <- :274
            actor.angle[1] = deg2s(SET_TETORA_ANG[group][at]); // <- :275
        }
    }

    /// `daNpc_Zl1_c::chk_talk` — `:1101-1113`.
    fn chk_talk(&mut self) -> bool {
        // `dComIfGp_event_chkTalkXY()` (:1103) and
        // `dComIfGp_evmng_ChkPresentEnd()` (:1104) both read the presenting-event
        // channel, which `ActiveEvent` does not expose. `mItemNo` is pinned to
        // the `0xFF` both of its arms write — :1106, :1110.
        self.item_no = 0xFF;
        true
    }

    /// `daNpc_Zl1_c::getMsg` — `:1031-1051`.
    fn get_msg(&self, collect: &Collect, flags: &EventFlags) -> u32 {
        match self.kind {
            2 => get_msg_zl1_2(collect, flags), // <- :1034
            STT_TURN => get_msg_zl1_4(flags),    // <- :1037
            5 => 0x15EE,                         // <- getMsg_ZL1_5, :1028
            1 | 3 | STT_DEMO_A | STT_DEMO_B => 0, // <- :1043-1047
            _ => 0, // no case for type 0, :1048
        }
    }

    /// `daNpc_Zl1_c::next_msgStatus` — `:965-1008`.
    fn next_msg_status(&mut self, msg: u32, select: u8) -> Step {
        match msg {
            0xCA2 | 0xCA3 => Step::Next(0xC90), // <- :970-973
            0xC90 => match select {
                0 => Step::Ends,        // <- :978
                1 => Step::Next(0xC91), // <- :980
                _ => Step::Ends,        // <- the inner switch's fall-out, :983
            },
            0x15E1 => Step::Next(0x15E2), // <- :986
            0x15E2 => Step::Next(0x15E3), // <- :990
            0x15E3 => Step::Next(0x15E4), // <- :994
            0x15E7 => Step::Next(0x15E8), // <- :998
            _ => Step::Ends,             // <- :1001-1004
        }
    }

    /// `fopNpc_npc_c::talk` (`d_npc.cpp:664-697`) with `param_1 == 1`, as
    /// `talk_1` calls it — `:2187`.
    fn talk(&mut self, ctx: &mut TalkCtx<'_>, first: bool) -> u8 {
        if self.msg_id == FPC_M_ERROR_PROCESS_ID {
            if first {
                self.msg_no = self.get_msg(ctx.collect, ctx.flags); // <- :669
            }
            self.msg_id = ctx.text.message_set_actor(self.msg_no, self.attn_pos); // <- :672
            ctx.bound = false; // <- mpCurrMsg = NULL, :673
            return NO_STATUS;
        }
        if !ctx.bound {
            ctx.bound = ctx.text.bound(); // <- fopMsgM_SearchByID, :692
            return NO_STATUS;
        }
        let status = ctx.text.status();
        if status == STATUS_MSG_DISPLAYED {
            let step = self.next_msg_status(self.msg_no, ctx.select_num);
            if let Step::Next(msg) = step {
                self.msg_no = msg;
            }
            ctx.text.set_status(step.status()); // <- :679
            if step.status() == STATUS_MSG_CONTINUES {
                let no = self.msg_no;
                ctx.text.message_set_actor(no, self.attn_pos); // <- :682
            }
        } else if status == STATUS_BOX_CLOSED {
            ctx.text.set_status(STATUS_MSG_DESTROYED); // <- :686
            self.msg_id = FPC_M_ERROR_PROCESS_ID; // <- :687
        }
        status
    }

    /// `daNpc_Zl1_c::eventOrder` — `:1053-1064`.
    fn event_order(&mut self) -> Option<usize> {
        if self.order >= ORDER_FIRST_EVENT {
            self.event_slot = (self.order - EVENT_BASE) as usize; // <- :1060
            return self.event_idx.get(self.event_slot).copied(); // <- :1061
        }
        None
    }

    /// `daNpc_Zl1_c::checkOrder` — `:1066-1099`.
    fn check_order(&mut self, talk_command: bool, demo_accept: bool) -> bool {
        if demo_accept {
            if self.order < ORDER_FIRST_EVENT {
                return false; // <- :1075
            }
            match self.event_slot {
                0 => self.demo_wait = true,  // <- field_0x7D3 = true, :1077
                2 => self.in_event = true,   // <- field_0x7C9 = true, :1081
                3 => self.look_lock = true,  // <- field_0x7D8 = true, :1086
                _ => {}
            }
            self.order = ORDER_NONE; // <- :1090
            return true;
        }
        if talk_command && (self.order == ORDER_SPEAK || self.order == ORDER_TALK) {
            self.order = ORDER_NONE; // <- :1095
            self.talk_cmd = true; // <- field_0x7D7 = true, :1096
            return true;
        }
        false
    }

    /// `daNpc_Zl1_c::event_proc` — `:1839-1877`.
    fn event_proc(&mut self, actor: &mut Actor, ctx: &mut TalkCtx<'_>) {
        let Some(index) = self.event_idx.get(self.event_slot).copied() else {
            return;
        };
        if index == NO_EVENT || !ctx.active.end_check(index) {
            return;
        }
        match self.event_slot {
            0 => {
                self.demo_wait = false; // <- field_0x7D3 = false, :1844
                ctx.flags.set(BIT_0802); // <- :1845
                self.home_pos = actor.pos; // <- field_0x72C, :1846
                self.home_angle = actor.angle[1]; // <- field_0x738, :1847
                self.set_stt(STT_IDLE, actor); // <- :1848
            }
            1 => {
                ctx.flags.set(BIT_0804); // <- :1850
                actor.pos = EVENT_MOVE_POS; // <- :1851
                actor.angle[1] = EVENT_MOVE_YAW; // <- :1852
                self.anchor.y = actor.pos.y; // <- field_0x764.y, :1853
                self.home_pos = actor.pos; // <- :1854
                self.home_angle = actor.angle[1]; // <- :1855
                self.look = 0; // <- :1856
                self.set_stt(STT_DEMO_C, actor); // <- :1857
            }
            2 => {
                ctx.flags.set(BIT_0801); // <- :1860
                self.in_event = false; // <- field_0x7C9 = false, :1861
            }
            _ => {
                ctx.flags.set(BIT_COURTYARD); // <- :1864
                self.set_stt(STT_APPROACH, actor); // <- :1865
            }
        }
        self.end_event(); // <- :1868
    }

    /// `daNpc_Zl1_c::endEvent` — `:1827-1832`.
    fn end_event(&mut self) {
        self.order = ORDER_NONE; // <- :1831
    }

    /// `daNpc_Zl1_c::wait_action1` — `:2399-2432`.
    fn wait_action(&mut self, actor: &mut Actor, ctx: &mut TalkCtx<'_>) {
        match self.phase {
            PHASE_STARTING => {
                if self.kind == 2 && !ctx.flags.test(BIT_0802) {
                    self.set_stt(STT_HOLD, actor); // <- :2403
                } else {
                    self.set_stt(STT_IDLE, actor); // <- :2405
                }
                self.phase = PHASE_ONGOING; // <- :2404
            }
            PHASE_ENDING => {}
            _ => match self.state {
                STT_HOLD => {
                    if self.order < ORDER_FIRST_EVENT {
                        self.order = ORDER_FIRST_EVENT; // <- demo_4, :2264
                    }
                }
                STT_IDLE => self.wait_1(actor),
                STT_TALK => {}
                _ => {}
            },
        }
    }

    /// `daNpc_Zl1_c::wait_1` — `:2157-2182`.
    fn wait_1(&mut self, actor: &mut Actor) {
        if self.kind == STT_TURN {
            actor.angle[1] = walk::add_calc_angle_s(
                actor.angle[1],
                self.home_angle,
                TURN_SCALE,
                TURN_MAX,
                TURN_MIN,
            ); // <- :2159
        }
        if self.talk_cmd {
            if self.chk_talk() {
                self.set_stt(STT_TALK, actor); // <- :2165
                self.look = 1; // <- :2166
                self.look_lock = false; // <- field_0x7D8 = false, :2167
            }
            return;
        }
        self.order = ORDER_TALK; // <- :2172
        self.look_lock = true; // <- field_0x7D8 = true, :2173
        if self.has_attention {
            let roll = rnd(&mut self.seed) * RND_RANGE_TIMER; // <- :2174
            self.timer = RND_MIN_TIMER + roll as i32;
        }
        if calc_timer(&mut self.timer) != 0 {
            self.look = 1; // <- :2177
            return;
        }
        self.look = 0; // <- :2180
    }

    /// `daNpc_Zl1_c::talk_1` — `:2184-2221`, minus `chk_parts_notMov` (`:1115`).
    fn talk_1(&mut self, actor: &mut Actor, ctx: &mut TalkCtx<'_>) {
        if !self.talk_cmd {
            return;
        }
        let first = self.msg_id == FPC_M_ERROR_PROCESS_ID;
        let _ = self.talk(ctx, first); // <- talk(1), :2187
        if !ctx.bound {
            return; // <- `if (mpCurrMsg == NULL)`, :2188
        }
        let status = ctx.text.status();
        if status != crate::text::STATUS_BOX_OPENING && status != STATUS_TYPING {
            if status != crate::text::STATUS_MSG_DESTROYED {
                return; // <- the `default` arm, :2217
            }
            match self.msg_no {
                0xC91 => ctx.flags.set(BIT_2908),  // <- :2200
                0x1005 => ctx.flags.set(BIT_0810), // <- :2203
                0xC90 => {
                    // `dComIfGp_setNextStage("sea", 0xcd,
                    // dIsleRoom_OutsetIsland_e, 10, 0.0f, 0, 1, 0)` — :2206 —
                    // is a stage transition with no sim counterpart.
                    self.talk_cmd = false; // <- :2207
                    return;
                }
                _ => {}
            }
            self.item_no = 0xFF; // <- :2210
            self.talk_cmd = false; // <- :2211
            let previous = self.state_prev;
            self.set_stt(previous, actor); // <- :2212
            let roll = rnd(&mut self.seed) * RND_RANGE_TIMER; // <- :2213
            self.timer = RND_MIN_TIMER + roll as i32;
            self.end_event(); // <- :2214
        }
    }

    /// `daNpc_Zl1_c::demo_action1` — `:2434-2459`.
    fn demo1_action(&mut self, actor: &mut Actor) {
        match self.phase {
            PHASE_STARTING => {
                self.set_stt(STT_DEMO_A, actor); // <- :2438
                self.phase = PHASE_ONGOING; // <- :2443
            }
            PHASE_ENDING => {}
            _ => {}
        }
    }

    /// `daNpc_Zl1_c::demo_action2` — `:2461-2492`.
    fn demo2_action(&mut self, actor: &mut Actor, ctx: &mut TalkCtx<'_>) {
        match self.phase {
            PHASE_STARTING => {
                if ctx.flags.test(BIT_0804) {
                    self.set_stt(STT_DEMO_C, actor); // <- :2464
                    actor.pos = EVENT_MOVE_POS; // <- :2465
                    actor.angle[1] = EVENT_MOVE_YAW; // <- :2466
                    self.anchor.y = actor.pos.y; // <- :2467
                } else {
                    self.set_stt(STT_DEMO_B, actor); // <- :2469
                }
                self.phase = PHASE_ONGOING; // <- :2471
            }
            PHASE_ENDING => {}
            _ => {}
        }
    }

    /// `daNpc_Zl1_c::optn_action1` — `:2494-2531`.
    fn optn_action(&mut self, actor: &mut Actor, ctx: &mut TalkCtx<'_>) {
        match self.phase {
            PHASE_STARTING => {
                if !ctx.flags.test(BIT_COURTYARD) {
                    self.set_stt(STT_SEE, actor); // <- :2497
                } else {
                    self.set_stt(STT_APPROACH, actor); // <- :2500
                }
                self.phase = PHASE_ONGOING; // <- :2499
            }
            PHASE_ENDING => {}
            _ => {}
        }
    }

    /// `daNpc_Zl1_c::_execute` (`:2784-2846`) reduced to the order, event and
    /// action dispatch.
    fn execute(&mut self, actor: &mut Actor, ctx: &mut TalkCtx<'_>) {
        if !self.seeded {
            self.home_pos = actor.pos; // <- field_0x72C, :2786
            self.home_angle = actor.angle[1]; // <- field_0x738, :2787
            self.seeded = true; // <- field_0x7D5 = true, :2788
        }
        self.drawn = !(self.hidden_demo || self.draw_off); // <- _draw, :2654
        if self.check_order(ctx.talk_command, ctx.demo_accept) {
            return;
        }
        if self.in_event {
            self.event_proc(actor, ctx); // <- :2804
        } else {
            match self.action {
                Action::Wait => {
                    self.wait_action(actor, ctx);
                    self.talk_1(actor, ctx); // <- :2810
                }
                Action::Demo1 => self.demo1_action(actor),
                Action::Demo2 => self.demo2_action(actor, ctx),
                Action::Optn => self.optn_action(actor, ctx),
            }
        }
        self.set_attention(actor); // <- :2812
    }
}

/// `getMsg_ZL1_2` — `:1010-1019`.
fn get_msg_zl1_2(collect: &Collect, flags: &EventFlags) -> u32 {
    if collect.check(1) != 0 {
        // <- dComIfGs_checkCollect(1), :1012
        0xCA2 + u32::from(flags.test(BIT_2908)) // <- :1013
    } else {
        0xCA1 // <- :1015
    }
}

/// `getMsg_ZL1_4` — `:1021-1024`.
fn get_msg_zl1_4(flags: &EventFlags) -> u32 {
    0x1005 + u32::from(flags.test(BIT_0810)) // <- :1022
}

/// `cM_rnd` — `c_math.cpp:180-186`. Retail's lanes are file-static and shared
/// process-wide; one set per Zelda, seeded at the retail boot value.
fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

/// `cLib_calcTimer` — `c_lib.cpp:47-54`: read, then decrement if non-zero.
fn calc_timer(value: &mut i32) -> i32 {
    let before = *value;
    if before != 0 {
        *value -= 1;
    }
    before
}

/// `cM_deg2s` — `c_math.cpp:157`: degrees onto the `0x10000` turn.
fn deg2s(deg: f32) -> i16 {
    walk::to_s16(deg * core::f32::consts::PI / 180.0)
}

/// `init_ZL1_3`'s `mDoMtx_stack_c::transS(current.pos)`,
/// `YrotM(current.angle.y)` and `multVec(&(-123, 0, -100), &field_0x764)` with
/// `field_0x764.y = 400.0f` — `:323-326`.
fn anchor(actor: &Actor) -> Vec3 {
    let yaw = walk::from_s16(actor.angle[1]);
    Vec3::new(
        actor.pos.x - 123.0 * yaw.cos() - 100.0 * yaw.sin(),
        400.0, // <- field_0x764.y = 400.0f, :326
        actor.pos.z + 123.0 * yaw.sin() - 100.0 * yaw.cos(),
    )
}

/// The text-process bridge, mirroring `npc_nz.rs:1338-1399`.
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

    /// `l_msg->mStatus` — `d_npc.cpp:677`.
    fn status(&self) -> u8 {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.status,
            Some(FPC_NM_MSG2) => self.two.status,
            _ => NO_STATUS,
        }
    }

    fn set_status(&mut self, status: u8) {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.status = status,
            Some(FPC_NM_MSG2) => self.two.status = status,
            _ => {}
        }
    }

    /// `fopMsgM_SearchByID(mCurrMsgBsPcId)` — `d_npc.cpp:692`.
    fn bound(&self) -> bool {
        self.procs.search_by_name(FPC_NM_MSG).is_some()
            || self.procs.search_by_name(FPC_NM_MSG2).is_some()
    }

    /// `l_msg->mSelectNum` — `d_npc.cpp:677`, passed to `next_msgStatus`.
    fn select_num(&self) -> u8 {
        self.one.select_num
    }

    fn message_set_actor(&mut self, msg_no: u32, eye: Vec3) -> u32 {
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

/// Everything the state machine reads and writes.
pub struct TalkCtx<'a> {
    pub text: &'a mut Text<'a>,
    pub flags: &'a mut EventFlags,
    pub active: &'a ActiveEvent,
    pub collect: &'a Collect,
    pub stage: &'a str,
    pub player: Option<&'a Player>,
    pub select_num: u8,
    pub bound: bool,
    pub talk_command: bool,
    pub demo_accept: bool,
}

#[derive(SystemParam)]
pub struct World<'w> {
    pub text_flags: Res<'w, TextFlags>,
    pub flags: ResMut<'w, EventFlags>,
    pub active: ResMut<'w, ActiveEvent>,
    pub list: Res<'w, EventList>,
    pub stage: Res<'w, StageName>,
    pub collect: Res<'w, Collect>,
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub archive: Option<Res<'w, MsgArchives>>,
}

#[derive(SystemParam)]
pub struct Cast<'w, 's> {
    pub players: Query<'w, 's, &'static Player>,
}

pub fn control(
    mut world: World,
    cast: Cast,
    _intent: Res<MoveIntent>,
    do_intent: Res<DoIntent>,
    _pad: Option<Res<PadButtons>>,
    mut zeldas: Query<(Entity, &mut Actor, &mut Zelda)>,
    mut commands: Commands,
) {
    let clear_count = world.text_flags.clear_count;
    let archive = world
        .archive
        .as_deref()
        .map(|held| held as &dyn MsgArchive);
    let player = cast.players.iter().next().copied();
    let do_now = do_intent.0;
    for (entity, mut actor, mut zelda) in &mut zeldas {
        if zelda.refused {
            // `decideType` returned false, so `_create` returned `cPhs_ERROR_e`
            // — :2884-2886 — and the process was cancelled during Create. The
            // entity is dropped on its first tick (RULE 6b).
            let _ = commands.entity(entity).despawn();
            continue;
        }
        let mut text = Text {
            procs: &mut world.procs,
            info: &mut world.info,
            clear_count,
            one: &mut world.one,
            two: &mut world.two,
            archive,
        };
        let select_num = text.select_num();
        let talk_command = do_now && !zelda.prev_do;
        zelda.prev_do = do_now;
        let mut ctx = TalkCtx {
            text: &mut text,
            flags: &mut world.flags,
            active: &world.active,
            collect: &world.collect,
            stage: &world.stage.0,
            player: player.as_ref(),
            select_num,
            bound: false,
            talk_command,
            // `eventInfo.checkCommandDemoAccrpt()` (:1063) reads a per-actor
            // stage-event condition word the sim has no model of (RULE 3b), and
            // the Zl1 staff cut is driven from `event::control`'s `zl1_cut`
            // instead. The gate is transcribed and fed false.
            demo_accept: false,
        };
        let actor = &mut *actor;
        if !zelda.entered {
            zelda.create_init(actor, &world.list, &mut ctx);
        }
        zelda.execute(actor, &mut ctx);
        if let Some(index) = zelda.event_order()
            && index != NO_EVENT
            && world.active.stage_of(index).is_none()
        {
            world.active.order(&world.list, index, entity);
        }
    }
}