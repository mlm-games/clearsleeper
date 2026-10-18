use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::area::StageName;
use super::collision::Collision;
use super::event::{ActiveEvent, EventList};
use super::flags::{EventFlags, StoryBits, Switches};
use super::ground;
use super::intent::DoIntent;
use super::npc_ah::JntCtrl;
use super::npc_mt::{FIGURE_ROOM_NAME, PHOTO_NONE, TOTAL_FIGURE_COUNT, fig_room_id, is_figure_get};
use super::ohatch::OHATCH_ACTORS;
use super::player::Player;
use super::stream::add_calc_angle_s2;
use super::transition::NextStage;
use super::walk::to_s16;
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    DA_EVT_MODE_NONE, DA_PY_STTS0_TELESCOPE_LOOK, FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2,
    MsgGameInfo, MsgProcs, fop_msgm_message_set, fop_msgm_message_set_actor,
    fop_msgm_scope_message_set,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_BOX_CLOSING, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED,
    STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS, STATUS_SCOPE_ACTIVE, STATUS_TYPING,
};

/// `OBJNAME("Mn", fpcNm_NPC_MN_e, 255, DEMO_SELECT(0, 59))` (`d_stage.cpp:1019`) is the authority
/// for the stage string; `daNpcMn_c::m_arcname[] = "Mn"` (`d_a_npc_mn.cpp:17`) agrees.
pub const MARLON_ACTOR: &str = "Mn";

/// `static char* l_npc_staff_id[] = { "Mn" };` (`:19-21`) - the staff
/// `dComIfGp_evmng_getMyStaffId` is asked for at `:869`.
const STAFF_MN: &str = "Mn"; // <- d_a_npc_mn.cpp:20
/// `mHatchEventIdx = dComIfGp_evmng_getEventIdx("FIGURE_HATCH_OPEN");` (`:442`), re-tested by
/// `checkOrder` (`:828`) and `chkEndEvent` (`:1610`).
const EVENT_HATCH_OPEN: &str = "FIGURE_HATCH_OPEN"; // <- d_a_npc_mn.cpp:442

/// `dSv_event_flag_c::UNK_870F = 0x870F` (`d_save_event_flag.inc:512`) - the room register
/// `phase_1` writes at `:295`, `:308` and reads at `:301`.
const FLAG_870F: u16 = 0x870F; // <- d_a_npc_mn.cpp:295
/// `dSv_event_flag_c::UNK_2F08 = 0x2F08` (`d_save_event_flag.inc:359`) - the Outset first talk.
const FLAG_2F08: u16 = 0x2F08; // <- d_a_npc_mn.cpp:290
/// `dSv_event_flag_c::UNK_2F04 = 0x2F04` (`d_save_event_flag.inc:360`).
const FLAG_2F04: u16 = 0x2F04; // <- d_a_npc_mn.cpp:1297
/// `dSv_event_flag_c::UNK_3120 = 0x3120` (`d_save_event_flag.inc:373`) - the companion-room talk.
const FLAG_3120: u16 = 0x3120; // <- d_a_npc_mn.cpp:1298
/// `dSv_event_flag_c::UNK_3A01 = 0x3A01` (`d_save_event_flag.inc:448`) - "a figure was collected".
const FLAG_3A01: u16 = 0x3A01; // <- d_a_npc_mn.cpp:1311
/// `dSv_event_flag_c::UNK_3D08 = 0x3D08` (`d_save_event_flag.inc:469`) - the companion room.
const FLAG_3D08: u16 = 0x3D08; // <- d_a_npc_mn.cpp:1296

/// `static u32 l_msg_mn_1st_talk[] = { 0x35E9, 0x35EA, 0x35EB, 0x0000 };` (`:125-130`). The
/// `0x0000` sentinel is real data: it is what `next_msgStatus`'s `case 0` (`:1269-1272`) reads.
const MSG_1ST_TALK: [u32; 4] = [0x35E9, 0x35EA, 0x35EB, 0x0000]; // <- d_a_npc_mn.cpp:126-129
/// `static u32 l_msg_mn_2nd_talk[] = { 0x35EC, 0x35ED, 0x35EE, 0x0000 };` (`:132-137`).
const MSG_2ND_TALK: [u32; 4] = [0x35EC, 0x35ED, 0x35EE, 0x0000]; // <- d_a_npc_mn.cpp:133-136
/// `static u32 l_msg_mn_1st_talk_in[] = { 0x35F1, 0x0000 };` (`:139-142`).
const MSG_1ST_TALK_IN: [u32; 2] = [0x35F1, 0x0000]; // <- d_a_npc_mn.cpp:140-141
/// `static u32 l_msg_mn_2nd_talk_in[] = { 0x35F2, 0x0000 };` (`:144-147`).
const MSG_2ND_TALK_IN: [u32; 2] = [0x35F2, 0x0000]; // <- d_a_npc_mn.cpp:145-146
/// `static u32 l_msg_mn_3rd_talk_in[] = { 0x35F0, 0x0000 };` (`:149-152`).
const MSG_3RD_TALK_IN: [u32; 2] = [0x35F0, 0x0000]; // <- d_a_npc_mn.cpp:150-151
/// `static u32 l_msg_mn_comp_1st[] = { 0x35F3, 0x35F4, 0x35F5, 0x0000 };` (`:154-159`).
const MSG_COMP_1ST: [u32; 4] = [0x35F3, 0x35F4, 0x35F5, 0x0000]; // <- d_a_npc_mn.cpp:155-158
/// `static u32 l_msg_mn_comp_2nd[] = { 0x35F6, 0x0000 };` (`:161-164`).
const MSG_COMP_2ND: [u32; 2] = [0x35F6, 0x0000]; // <- d_a_npc_mn.cpp:162-163
/// `static u32 l_msg_mn_figure[] = { 0x35F7, ..., 0x3600 };` (`:166-177`) - NO `0x0000`
/// terminator, and never walked: `getMsg` reads it as a SCALAR
/// (`msg = l_msg_mn_figure[mFigureMsgIdx]`, `:1309`) and leaves `mpMsgNo` NULL.
const MSG_FIGURE: [u32; 10] = [
    0x35F7, 0x35F8, 0x35F9, 0x35FA, 0x35FB, 0x35FC, 0x35FD, 0x35FE, 0x35FF, 0x3600,
]; // <- d_a_npc_mn.cpp:167-176
/// `u32 daNpcMn_c::getMsg3() { mpMsgNo = 0; return 0x35EF; }` (`:1326-1329`).
const MSG_TALK3: u32 = 0x35EF; // <- d_a_npc_mn.cpp:1328

/// `enum Prm_e { PRM_SWITCH_W = 0x8, PRM_SWITCH_S = 0x8, PRM_SWITCH2_W = 0x8,
/// PRM_SWITCH2_S = 0x10, PRM_RAIL_W = 0x8, PRM_RAIL_S = 0x18 }` (`d_a_npc_mn.h:34-40`), read
/// through `daObj::PrmAbstract(this, W, S)` (`d_a_npc_mn.cpp:1373`, `:1378`). `PRM_RAIL` feeds only
/// the PATH arm of `createInit` (`:408-432`), so it is not decoded.
const PRM_SWITCH_WIDTH: u32 = 0x08; // <- d_a_npc_mn.h:35 (PRM_SWITCH_W)
const PRM_SWITCH_SHIFT: u32 = 0x08; // <- d_a_npc_mn.h:36 (PRM_SWITCH_S)
const PRM_SWITCH2_WIDTH: u32 = 0x08; // <- d_a_npc_mn.h:37 (PRM_SWITCH2_W)
const PRM_SWITCH2_SHIFT: u32 = 0x10; // <- d_a_npc_mn.h:38 (PRM_SWITCH2_S)

/// `static char* l_room_name[] = { "sea", "Pfigure", "figureA", "figureB", "figureC", "figureD",
/// "figureE", "figureF", "figureG", "K_Testb" };` (`:209-220`). Only the first entry is a
/// literal here: `:211-218` are the eight figure rooms `npc_mt::FIGURE_ROOM_NAME` already owns.
const ROOM_SEA_NAME: &str = "sea"; // <- d_a_npc_mn.cpp:210 (l_room_name[0])
const ROOM_K_TESTB_NAME: &str = "K_Testb"; // <- d_a_npc_mn.cpp:219 (l_room_name[9])
/// `ARRAY_SIZE(l_room_name)` in `chkPosNo`'s `while` bound and its fall-out return (`:1621-1627`),
/// so 10 is a real value, not a failure marker.
const ROOM_COUNT: u8 = 10; // <- d_a_npc_mn.cpp:1621
const ROOM_SEA: u8 = 0; // <- d_a_npc_mn.cpp:210
const ROOM_PFIGURE: u8 = 1; // <- d_a_npc_mn.cpp:211

/// `NpcDatStruct l_npc_dat[0]` (`:104-111`), the row `mNpcNo == 0` indexes, in the field order of
/// `d_a_npc_mn.h:9-40`. Only the fields this module reads are transcribed; the omitted ones and
/// their lines are in the `Marlon` doc comment.
const DAT0_MAX_HEAD_X: i16 = 0x0FA0; // <- d_a_npc_mn.cpp:104 (mMax_head_x)
const DAT0_MAX_HEAD_Y: i16 = 0x2710; // <- d_a_npc_mn.cpp:104 (mMax_head_y)
const DAT0_MAX_BACKBONE_X: i16 = 0x0000; // <- d_a_npc_mn.cpp:104 (mMax_backbone_x)
const DAT0_MAX_BACKBONE_Y: i16 = 0x1770; // <- d_a_npc_mn.cpp:104 (mMax_backbone_y)
const DAT0_MIN_HEAD_X: i16 = -0x07D0; // <- d_a_npc_mn.cpp:105 (mMin_head_x = 0xF830)
const DAT0_MIN_HEAD_Y: i16 = -0x2710; // <- d_a_npc_mn.cpp:105 (mMin_head_y = 0xD8F0)
const DAT0_MIN_BACKBONE_X: i16 = -0x1770; // <- d_a_npc_mn.cpp:105 (mMin_backbone_x = 0xE890)
const DAT0_MIN_BACKBONE_Y: i16 = -0x0FA0; // <- d_a_npc_mn.cpp:105 (mMin_backbone_y = 0xF060)
const DAT0_MAX_TURN_STEP: i16 = 0x05DC; // <- d_a_npc_mn.cpp:106 (mMax_turn_step)
const DAT0_PLAYER_EYE_OFS_Y: f32 = 0.0; // <- d_a_npc_mn.cpp:106 (mPlayerEyeOfsY)
const DAT0_EYE_POS_OFS_Y: f32 = 120.0; // <- d_a_npc_mn.cpp:107 (mEyePosOfsY)
const DAT0_ATTN_DIST: f32 = 250.0; // <- d_a_npc_mn.cpp:107 (mAttnDist)
const DAT0_NEAR_DIST: f32 = 0.0; // <- d_a_npc_mn.cpp:107 (mNearDist)
const DAT0_ATTN_ANGLE: i16 = 0x7FFF; // <- d_a_npc_mn.cpp:107 (mAttnAngle)
const DAT0_LOOK_AT_MAX_VEL: i16 = 0x0514; // <- d_a_npc_mn.cpp:108 (mLookAtMaxVel)
const DAT0_LOOK_RESET_TIMER: i16 = 0x001E; // <- d_a_npc_mn.cpp:111 (mLookResetTimer)
const DAT0_ALLOW_BODY_TURN: u8 = 0x01; // <- d_a_npc_mn.cpp:111 (mbAllowBodyTurn)
const DAT0_LOOK_ONLY: u8 = 0x01; // <- d_a_npc_mn.cpp:111 (mbLookOnly)
/// `l_npc_dat[1]` (`:113-122`) is byte-identical to `l_npc_dat[0]` in every field this module
/// reads EXCEPT `mWalkSpeed` (3.0 -> 1.5 at `:119`), which only the omitted PATH walk consumes.
/// `mEyePosOfsY` is still indexed through `mNpcNo` because `_execute` re-reads it every tick
/// (`:600`), so its second-row literal is transcribed too.
const DAT1_EYE_POS_OFS_Y: f32 = 120.0; // <- d_a_npc_mn.cpp:117 (mEyePosOfsY)

/// `chkAttention`: `maxDist += 40.0f;` (`:1414`) once the player already holds his attention.
const ATTN_DIST_EXTEND: f32 = 40.0; // <- d_a_npc_mn.cpp:1414
/// `maxAngle += 0x71C;` (`:1415`).
const ATTN_ANGLE_EXTEND: i32 = 0x71C; // <- d_a_npc_mn.cpp:1415
/// `dNpc_playerEyePos` (`d_npc.cpp:610-623`) returns the player's head-top point with x and z
/// forced back to the player's own position, so only the height matters, and it comes from
/// `cMtx_multVec(mpCLModel->getAnmMatrix(CL_JNT_HEAD_JNT_e), &head_offset, &mHeadTopPos)` with
/// `head_offset = {40.0f, 0.0f, 0.0f}` (`d_a_player_main.cpp:11560`, `:11563`). The world-space
/// height depends on the animated head joint, which RULE 3b classes as absent, so the joint's own
/// local x offset stands in.
const PLAYER_HEAD_TOP: f32 = 40.0; // <- d_a_player_main.cpp:11560 (head_offset.x)
/// `lookBack`: `cLib_addCalcAngleS2(&mTurnVel, maxVel, 4, 0x800);` (`:1505`). That is the FOUR-
/// argument overload (`c_lib.h:23`), transcribed by `stream::add_calc_angle_s2` - NOT the five-
/// argument `cLib_addCalcAngleS` (`c_lib.h:22`), which this file never calls.
const TURN_VEL_SCALE: i16 = 4; // <- d_a_npc_mn.cpp:1505
const TURN_VEL_MAX_STEP: i16 = 0x800; // <- d_a_npc_mn.cpp:1505

/// `enum MoveProcIdx { MOVE_PROC_WAIT = 0, MOVE_PROC_TALK = 1, MOVE_PROC_WALK = 2,
/// MOVE_PROC_TURN = 3, MOVE_PROC_TALK3 = 4 }` (`d_a_npc_mn.h:44-50`). WALK and TURN are omitted:
/// the only entries into them are `executeWalk` and `executeTurnInit`, and both need room PATH
/// data (RULE 3b).
const MOVE_WAIT: u8 = 0; // <- d_a_npc_mn.h:45
const MOVE_TALK: u8 = 1; // <- d_a_npc_mn.h:46
const MOVE_TALK3: u8 = 4; // <- d_a_npc_mn.h:49

/// `enum Talk3State { TALK3_INIT = 0, TALK3_ORDER = 1, TALK3_TALK = 2 }` (`d_a_npc_mn.h:57-61`).
const TALK3_INIT: u8 = 0; // <- d_a_npc_mn.h:58
const TALK3_ORDER: u8 = 1; // <- d_a_npc_mn.h:59
const TALK3_TALK: u8 = 2; // <- d_a_npc_mn.h:60

/// `enum LookMode { LOOK_MODE_NONE = 0, LOOK_MODE_ATTN = 1, LOOK_MODE_TURN = 2 }`
/// (`d_a_npc_mn.h:52-56`).
const LOOK_NONE: i8 = 0; // <- d_a_npc_mn.h:53
const LOOK_ATTN: i8 = 1; // <- d_a_npc_mn.h:54
const LOOK_TURN: i8 = 2; // <- d_a_npc_mn.h:55

/// `enum BckIdx { BCK_WAIT01 = 0, ..., BCK_NULL = 0xFF }` (`d_a_npc_mn.h:62-71`).
const BCK_WAIT01: u8 = 0; // <- d_a_npc_mn.h:63
const BCK_WAIT02: u8 = 1; // <- d_a_npc_mn.h:64
const BCK_TALK01: u8 = 2; // <- d_a_npc_mn.h:65
const BCK_TALK02: u8 = 3; // <- d_a_npc_mn.h:66
const BCK_WALK: u8 = 4; // <- d_a_npc_mn.h:67
const BCK_BIKKURI: u8 = 5; // <- d_a_npc_mn.h:68
const BCK_NULL: u8 = 0xFF; // <- d_a_npc_mn.h:71

/// `static char* cut_name_tbl[] = { "MES_SET", "GET_ITEM", "WAIT", "HATCH", "BIKKURI", "TURN",
/// "WALK", "LOOK", "JUMP", "SWON" };` (`:866-868`) - the argument of
/// `dComIfGp_evmng_getMyActIdx(staffIdx, cut_name_tbl, 10, TRUE, 0)` (`:871`).
const CUT_NAMES: [&str; 10] = [
    "MES_SET", "GET_ITEM", "WAIT", "HATCH", "BIKKURI", "TURN", "WALK", "LOOK", "JUMP", "SWON",
]; // <- d_a_npc_mn.cpp:867
const CUT_MES_SET: i8 = 0; // <- d_a_npc_mn.cpp:867
const CUT_WAIT: i8 = 2; // <- d_a_npc_mn.cpp:867
const CUT_HATCH: i8 = 3; // <- d_a_npc_mn.cpp:867
const CUT_BIKKURI: i8 = 4; // <- d_a_npc_mn.cpp:867
const CUT_TURN: i8 = 5; // <- d_a_npc_mn.cpp:867
const CUT_WALK: i8 = 6; // <- d_a_npc_mn.cpp:867
const CUT_JUMP: i8 = 8; // <- d_a_npc_mn.cpp:867
const CUT_SWON: i8 = 9; // <- d_a_npc_mn.cpp:867

/// The staff-record keys the cut initialisers read. Distinct from the `CUT_NAMES` ACTION strings
/// even where the text matches ("SwOn" is both an action name at `:867` and a `WAIT` key at
/// `:1002`), so they carry their own prefix.
const KEY_MSG_NO: &str = "MsgNo"; // <- d_a_npc_mn.cpp:951
const KEY_TIMER: &str = "Timer"; // <- d_a_npc_mn.cpp:989
const KEY_SWON: &str = "SwOn"; // <- d_a_npc_mn.cpp:1002
const KEY_TURN_MODE: &str = "TurnMode"; // <- d_a_npc_mn.cpp:1080

/// `eventWaitInit` / `eventSwOnInit`: `mEventTimer = 0;` then `*timer` if the record has one
/// (`:990-993`, `:1011-1014`).
const CUT_TIMER_DEFAULT: i16 = 0; // <- d_a_npc_mn.cpp:990
/// `eventBikkuriInit`: `mEventTimer = 1; ... if (mEventTimer <= 0) mEventTimer = 1;` (`:1052-1058`).
const BIKKURI_TIMER_MIN: i16 = 1; // <- d_a_npc_mn.cpp:1057

/// `eventJumpInit` (`:1153-1175`). The three staff floats `SpeedX` / `SpeedY` / `Gravity` are read
/// at `:1158-1171` and then IMMEDIATELY overwritten by the unconditional tail `:1172-1174`, so in
/// retail only the tail survives and the staff parameters are dead.
const JUMP_TARGET_SPEED_F: f32 = 40.0; // <- d_a_npc_mn.cpp:1172
const JUMP_SPEED_Y: f32 = 25.0; // <- d_a_npc_mn.cpp:1173
const JUMP_GRAVITY: f32 = -1.0; // <- d_a_npc_mn.cpp:1174

/// `createInit`: `fopAcM_SetGravity(this, -9.0f);` (`:440`).
const CREATE_GRAVITY: f32 = -9.0; // <- d_a_npc_mn.cpp:440
/// `createInit`: `int weight = 0xFF;` (`:406`), lowered to `0xFE` only on the PATH rail arm
/// (`:431`). It is `cCcD_Stts::Init`'s first argument, the collision weight byte.
const CYL_STTS_WEIGHT: u8 = 0xFF; // <- d_a_npc_mn.cpp:406

/// `u16 status = 0xFF` - `talk2`'s (`:1191`) and `talk3`'s (`:1228`) "no status yet" value.
const NO_STATUS: u8 = 0xFF; // <- d_a_npc_mn.cpp:1191
/// `mLastMsgStatus = -1;` (`:1198`, `:1235`) into the `u16` field at `d_a_npc_mn.h:102`.
const PREV_STATUS_NONE: u16 = 0xFFFF; // <- d_a_npc_mn.cpp:1198

/// `mEvtOrderType` is `1`/`0` from `executeCommon` (`mEvtOrderType = true / false`, `:609-613`),
/// `3` from `executeWait` (`:664`), and `2` only from the base-class event path - which is why
/// `checkOrder` and `eventOrder` still test for it.
const ORDER_NONE: u8 = 0; // <- d_a_npc_mn.cpp:612
const ORDER_ATTENTION: u8 = 1; // <- d_a_npc_mn.cpp:610
const ORDER_TALK: u8 = 2; // <- d_a_npc_mn.cpp:831
const ORDER_HATCH: u8 = 3; // <- d_a_npc_mn.cpp:664

/// `dSv_info_c::isSwitch` returns FALSE for `i_no == 255` (`d_save.cpp:1591-1593`), so a
/// `getPrmSwitchBit()` of 0xFF never fires. The index is carried through unclamped; RULE 11b's
/// zone-bit gap (`d_save.h:983-984`) is a save-model limitation, not this actor's.
const NO_SWITCH: u8 = 0xFF; // <- d_save.cpp:1591

/// `cM_initRnd(100, 100, 100)` (`m_Do_machine.cpp:617`) is the boot seed of the single global
/// three-register generator `cM_rnd` (`c_math.cpp:170-182`). This sim has no global RNG channel,
/// so the generator is carried per actor from that exact seed and that exact algorithm: it is not
/// a fixed point (a `[0; 3]` seed would be) and no other channel is substituted for it.
const RND_SEED: i32 = 100; // <- m_Do_machine.cpp:617
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA]; // <- c_math.cpp:178-180
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673]; // <- c_math.cpp:178-180
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0]; // <- c_math.cpp:181

/// `float cM_rnd() { r0 = (r0 * 0xAB) % 0x763D; r1 = (r1 * 0xAC) % 0x7663;
/// r2 = (r2 * 0xAA) % 0x7673; return fabsf(fmodf(r0 / 30269.0f + r1 / 30307.0f +
/// r2 / 30323.0f, 1.0)); }` (`c_math.cpp:177-182`).
fn c_m_rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

/// `float cM_rndF(float f) { return cM_rnd() * f; }` (`c_math.cpp:185-187`).
fn c_m_rnd_f(seed: &mut [i32; 3], max: f32) -> f32 {
    c_m_rnd(seed) * max
}

/// `int daNpcMn_c::getRand(int i_max) { int rnd = cM_rndF(i_max); if (rnd == i_max) rnd = 0;
/// return rnd; }` (`:1592-1598`). `cM_rndF` returns `f32`, so the `int` truncates.
fn get_rand(seed: &mut [i32; 3], max: i32) -> i32 {
    let rolled = c_m_rnd_f(seed, max as f32) as i32;
    if rolled == max { 0 } else { rolled }
}

/// `daObj::PrmAbstract(this, W, S)` is `((1 << W) - 1) & (param >> S)` (`d_a_obj.h:14`).
fn prm(parameters: u32, width: u32, shift: u32) -> u8 {
    ((parameters >> shift) & ((1 << width) - 1)) as u8
}

/// `static char* l_room_name[]` (`:209-220`): `"sea"`, the eight `FIGURE_ROOM_NAME` entries
/// (`:211-218`), then `"K_Testb"`. `FIGURE_ROOM_NAME` is borrowed from `npc_mt.rs` because
/// `d_a_npc_mn.cpp:211-218` and `d_a_npc_mt.cpp`'s table are the same eight names.
fn room_names() -> [&'static str; 10] {
    [
        ROOM_SEA_NAME,
        FIGURE_ROOM_NAME[0],
        FIGURE_ROOM_NAME[1],
        FIGURE_ROOM_NAME[2],
        FIGURE_ROOM_NAME[3],
        FIGURE_ROOM_NAME[4],
        FIGURE_ROOM_NAME[5],
        FIGURE_ROOM_NAME[6],
        FIGURE_ROOM_NAME[7],
        ROOM_K_TESTB_NAME,
    ]
}

/// `u8 daNpcMn_c::chkPosNo()` (`:1619-1629`): the index of the stage name in `l_room_name`, or
/// `ARRAY_SIZE` when it matches nothing. `strcmp(dComIfGp_getNextStageName(), l_room_name[posNo])`
/// is the only test; `dComIfGp_isEnableNextStage()` guards only the `_delete` copy at `:505`.
fn chk_pos_no(stage: Option<&str>) -> u8 {
    let Some(stage) = stage else {
        return ROOM_COUNT;
    };
    for (index, name) in room_names().iter().enumerate() {
        if *name == stage {
            return index as u8;
        }
    }
    ROOM_COUNT
}

/// `dNpc_calc_DisXZ_AngY(a, b, &dist, &angle)` (`d_npc.cpp:626-639`): the XZ distance
/// (`sqrtf(SQUARE(diff.x) + SQUARE(diff.z))`, `:632`) and `cM_atan2s(diff.x, diff.z)` (`:637`).
fn dist_and_yaw(from: Vec3, to: Vec3) -> (f32, i16) {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    let dist = (dx * dx + dz * dz).sqrt();
    let angle = to_s16(dx.atan2(dz));
    (dist, angle)
}

/// `dNpc_calc_DisXZ_AngY(current.pos, target, NULL, &angle)` - the yaw-only form, called at
/// `:732`, `:784`, `:798`, `:813`, `:1032`, `:1100`.
fn face_yaw(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

/// `cXyz dNpc_playerEyePos(f32 offsetY)` (`d_npc.cpp:610-623`): the head-top point with x and z
/// written back to the player's own position (`:619-620`).
fn player_eye(player: Player, offset_y: f32) -> Vec3 {
    Vec3::new(
        player.pos.x,
        player.pos.y + PLAYER_HEAD_TOP + offset_y,
        player.pos.z,
    )
}

/// `dComIfGp_evmng_getMyActIdx`-equivalent: the index of the playing cut's name in `CUT_NAMES`, or
/// `None` for `mActIdx == -1` (`:871-872`).
fn cut_name_index(list: &EventList, cut: usize) -> Option<i8> {
    let name = list.cut_name(cut)?;
    let index = CUT_NAMES.iter().position(|entry| *entry == name)?;
    i8::try_from(index).ok()
}

/// `struct sMnAnmDat { u8 mBckIdx; u8 mMorf; s8 mLoopCount; }` (`d_a_npc_mn.h:7-12`), a by-value
/// copy of the `l_npc_anm_*` entries. Only `mBckIdx` (the morph row) and the `mLoopCount > 0`
/// test that picks `EMode_NONE` over `EMode_LOOP` (`:1576-1579`) are modelled; the BCK archive
/// itself is RULE 3b.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AnmTable {
    /// `mBckIdx` (`d_a_npc_mn.h:8`).
    bck: u8,
    /// `mLoopCount` (`d_a_npc_mn.h:10`), an `s8` - `0xFF` is `-1`, i.e. loop forever.
    loop_count: i8,
}

/// `static sMnAnmDat l_npc_anm_wait = { daNpcMn_c::BCK_WAIT01, 0x08, 0xFF };` (`:48`).
const ANM_WAIT: AnmTable = AnmTable {
    bck: BCK_WAIT01,
    loop_count: -1,
}; // <- d_a_npc_mn.cpp:48
/// `static sMnAnmDat l_npc_anm_wait2 = { daNpcMn_c::BCK_WAIT02, 0x08, 0xFF };` (`:50`).
const ANM_WAIT2: AnmTable = AnmTable {
    bck: BCK_WAIT02,
    loop_count: -1,
}; // <- d_a_npc_mn.cpp:50
/// `static sMnAnmDat l_npc_anm_talk = { BCK_TALK01, 0x08, 0xFF };` (`:52-56`).
const ANM_TALK: AnmTable = AnmTable {
    bck: BCK_TALK01,
    loop_count: -1,
}; // <- d_a_npc_mn.cpp:53-55
/// `static sMnAnmDat l_npc_anm_talk2 = { BCK_TALK02, 0x08, 0xFF };` (`:58-62`).
const ANM_TALK2: AnmTable = AnmTable {
    bck: BCK_TALK02,
    loop_count: -1,
}; // <- d_a_npc_mn.cpp:59-61
/// `static sMnAnmDat l_npc_anm_walk = { BCK_WALK, 0x08, 0xFF };` (`:64-68`).
const ANM_WALK: AnmTable = AnmTable {
    bck: BCK_WALK,
    loop_count: -1,
}; // <- d_a_npc_mn.cpp:65-67
/// `static sMnAnmDat l_npc_anm_bikkuri[] = { { BCK_BIKKURI, 0x08, 0x01 },
/// { BCK_WAIT01, 0x08, 0xFF } };` (`:70-81`). `eventBikkuri` hands `setAnmTbl` the ARRAY, i.e.
/// row 0 (`:1066`); row 1 is only reached by `playAnm`'s chain advance (`:1545`), which needs
/// BCK frame data.
const ANM_BIKKURI: AnmTable = AnmTable {
    bck: BCK_BIKKURI,
    loop_count: 1,
}; // <- d_a_npc_mn.cpp:72-75

/// `fopAc_CULLBOX_CUSTOM_e` (`d_a_npc_mn.cpp:1728`) plus the actor's real
/// `fopAcM_setCullSizeBox(this, -70.0f, 0.0f, -70.0f, 70.0f, 200.0f, 70.0f)` (`:450`) - unlike most
/// NPCs this one is not boxless.
const CULL_MIN: [f32; 3] = [-70.0, 0.0, -70.0]; // <- d_a_npc_mn.cpp:450
const CULL_MAX: [f32; 3] = [70.0, 200.0, 70.0]; // <- d_a_npc_mn.cpp:450

/// The `l_msg_mn_*` table `mpMsgNo` currently walks, or `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chain {
    /// `mpMsgNo = NULL` (`:1286`, `:1327`, `:953`).
    None,
    /// `l_msg_mn_1st_talk` (`:292`).
    FirstTalk,
    /// `l_msg_mn_2nd_talk` (`:1294`).
    SecondTalk,
    /// `l_msg_mn_1st_talk_in` (`:1307`).
    FirstTalkIn,
    /// `l_msg_mn_2nd_talk_in` (`:1312`).
    SecondTalkIn,
    /// `l_msg_mn_3rd_talk_in` (`:1314`).
    ThirdTalkIn,
    /// `l_msg_mn_comp_1st` (`:1300`).
    CompFirst,
    /// `l_msg_mn_comp_2nd` (`:1302`).
    CompSecond,
}

impl Chain {
    /// The chain's `u32` row, terminating `0x0000` included where the decomp has one.
    fn row(self) -> &'static [u32] {
        match self {
            Chain::None => &[],
            Chain::FirstTalk => &MSG_1ST_TALK,
            Chain::SecondTalk => &MSG_2ND_TALK,
            Chain::FirstTalkIn => &MSG_1ST_TALK_IN,
            Chain::SecondTalkIn => &MSG_2ND_TALK_IN,
            Chain::ThirdTalkIn => &MSG_3RD_TALK_IN,
            Chain::CompFirst => &MSG_COMP_1ST,
            Chain::CompSecond => &MSG_COMP_2ND,
        }
    }
}

/// `daNpcMn_c` (`d_a_npc_mn.h:31-118`) - the Outset Island quest-giver, the first NPC the player
/// meets and the one who hands over the ocarina, the bombs and the telescope.
///
/// ## Damage
/// (a) inbound: NONE. `daNpcMn_c` carries no hit-point field (`d_a_npc_mn.h:71-118`), has no
/// damage reaction, and no `ChkTgHit` arm exists anywhere in `d_a_npc_mn.cpp`.
/// (b) outbound: NONE. One `dCcD_Cyl` IS registered -
/// `mStts.Init(weight, 0xFF, this); mCyl.Set(dNpc_cyl_src); mCyl.SetStts(&mStts);
/// setCollision(&mCyl, current.pos, l_npc_dat[mNpcNo].mCylRadius, 150.0f);` (`:481-484`, and again
/// every frame at `:598`, `:1605`'s `dComIfG_Ccsp()->Set`) - but its `SrcObjAt Type` is `0`
/// (`d_a_npc_mn.cpp:183`), so it deals no damage. Neither `combat::HitPoints` nor `enemy::Hurt` is
/// attached: attaching them would let `combat::control` despawn a NPC retail cannot damage.
/// `cCcD_Stts::Init`'s weight byte does land, on `Actor::weight_uc`.
///
/// ## What survives into the sim
/// `daNpcMn_c` plays no sound and no rumble, so `SimEvents` is untouched. Everything audible is
/// the opening conversation: `getMsg` / `getMsg3`'s `fopMsgM_messageSet` and
/// `fopMsgM_scopeMessageSet` chains, the `dSv_event_c` bits they set, `phase_1`'s `cPhs_STOP_e`
/// refusal, `executeWait`'s telescope-handoff switch, `executeTalk3`'s scope conversation, and the
/// `"FIGURE_HATCH_OPEN"` cut `chkEndEvent` waits on.
///
/// ## Omitted, and why (RULE 1 / RULE 3b) - exact decomp lines in the report
/// * BCK / BTP animation: `playTexPatternAnm` (`:1529-1538`), `playAnm` (`:1541-1553`),
///   `eventLookInit` / `eventLook` (`:1143-1150`, whose only test is
///   `mpMorf->getFrame() >= 72.0f`) and `setAnm`'s morph table (`:1556-1564`). Frame counts and
///   end frames are archive data this checkout does not ship; no stand-in is invented. The
///   consequence is disclosed: `mAnmFlag & 1` never becomes set, so `eventBikkuri` (`:1070`) cannot
///   self-terminate.
/// * Everything `mPathRun` drives: `createInit`'s rail arm (`:408-432`, also the only reader of
///   `PRM_RAIL`), `executeWalkInit` / `executeWalk` (`:739-792`), `executeTurnInit` /
///   `executeTurn` (`:795-823`), `eventWalkInit` / `eventWalk` (`:1112-1140`), and the
///   `mbLookFigure` arms of `executeWait` (`:666-688`). Room `PATH` blocks are a confirmed-absent
///   input. Because `mbLookFigure = 1` is assigned ONLY at `:754` inside `executeWalk`, it is
///   permanently false here, which also makes `getMsg`'s `l_msg_mn_figure` arm (`:1308-1309`)
///   unreachable and `mFigureMsgIdx` (`getRand(5)` at `:753`) permanently 0. `MSG_FIGURE` is
///   transcribed anyway so the arm exists rather than being imagined.
/// * `mWaitTimer` - its only readers are `:667` and `:681-686`, both inside that PATH branch, so
///   the three `cM_rndF` draws at `:633`, `:636` and `:801` have no observable reader. They are
///   not carried rather than folded to a constant, which is the `kamome.rs` failure mode.
/// * `l_npc_dat` fields only the omitted code reads: `mMax_turn_step` is carried (the joint
///   controller uses it) but `mWalkSpeed` 3.0 / 1.5 (`:109`, `:119`), `mWalkAnmRate` 0.5 (`:108`),
///   `mCylRadius` 60.0 (`:108`), `mAttnPosOfsY` 150.0 (`:106`), `mWalkLookAtMaxVel` 0x0190 (`:108`),
///   `mWaitTimerMin` 0x003C / `mWaitTimerMax` 0x005A (`:110`), `mTurnWaitMin` 0x00C8 /
///   `mTurnWaitMax` 0x012C (`:110`) and `field_0x12` / `field_0x2E` / `field_0x3C` / `field_0x3E`
///   (`:105`, `:108`, `:109`) have no reader left and are not declared.
/// * `eventGetItemInit` (`:980-985`): `mItemNo` (`d_a_npc_mn.h:80`) is never written by this actor,
///   and `fopAcM_createItemForPresentDemo` is item-table work. The cut still ends the way retail
///   does, because `privateCut`'s second switch has no `case 1` and falls to `default:
///   shouldEnd = true` (`:938-940`).
/// * `eventOrder` (`:838-847`) and the three ordering calls behind it:
///   `fopAcM_orderPotentialEvent` (`:723`), `fopAcM_orderSpeakEvent` (`:842`) and
///   `fopAcM_orderChangeEventId` (`:845`), plus `eventInfo.onCondition(dEvtCnd_CANTALK_e)`
///   (`:840`) / `(dEvtCnd_UNK2_e)` (`:724`). The order queue and mode those ride have
///   no counterpart in this port's event manager: `dEvt_control_c::order` queues the request
///   (`d_event.cpp:37-79`), `changeProc` consumes it once and drops the rest (`:439-471`), and
///   `demoCheck` (`:295-326`) is what turns it into `dComIfGp_evmng_order` plus
///   `dEvtCmd_INDEMO_e` on both actors (`:312-315`) and `mMode = dEvtMode_DEMO_e` (`:317`). That
///   mode is the port's `MsgGameInfo::event_mode`, which nothing raises off `DA_EVT_MODE_NONE`
///   (`warpfo.rs:154-157` records the same absence), so `checkCommandDemoAccrpt()`
///   (`f_op_actor.h:177`) never answers and `checkOrder`'s `mEvtOrderType = 0` clear (`:829-830`)
///   is unreachable. Without it an order the actor must keep issuing until the event starts would
///   land on [`ActiveEvent::order`], which re-seats a running event at `Stage::Order` and drops
///   its staff runs: the open cut would restart every tick instead of running. The
///   `mEvtOrderType == 2` half of the first arm is dead in the target regardless - the field is
///   written only `0`, `1` and `3` (`:611`, `:613`, `:665`, `:830`) - so the live arm is the
///   `orderChangeEventId` one, and the port cannot reach it: `FIGURE_HATCH_OPEN` has no other
///   orderer anywhere in the checkout (`d_a_obj_ohatch.cpp` only tests its existence at `:201`),
///   so `ohatch.rs`'s OPEN cut never runs today. Flagged in the report.
///   `eventInfo.onCondition` itself IS modelled (`OrderFlags`, the shape `ba1.rs` transcribes at
///   `d_a_npc_ba1.cpp:541-555`), but it is downstream of the same missing gate.
/// * `dComIfGp_setScopeType(dScpTyp_UNK1_e)` (`:652`): the camera's scope type has no counterpart
///   in this port's `Camera`, so the write is dropped; the gate around it is transcribed.
/// * `dComIfGp_event_chkTalkXY()` (`:1288`) is an inline in `d_com_inf_game.h:3505` with no body
///   anywhere in the decomp; it is fed `false`, the value `npc_ah.rs` already feeds it.
/// * `dEventCut_c::getAttnFlag` / `getAttnPos` / `getTurnSpeed` / `cutProc` (`:1393`, `:852`,
///   `:1502`, `:853`) have no counterpart, so `eventMove` always takes the `privateCut` arm - the
///   same gap `ohatch.rs` records for `getMyActIdx`.
/// * `attention_info` (`:451-453`, `:599`), `dSnap_RegistFig` (`:546`), `_draw`'s
///   `dSv_event_tmp_flag_c::UNK_0408` test (`:513`) and the whole draw pass: the attention /
///   catch-request and snapshot subsystems are RULE 3b absences and the rest is renderer work.
/// * `setCollision` (`:1601-1606`) and `dComIfG_Ccsp()->Set`: this sim has no `cCcD` actor
///   collision registry (RULE 3b), so the live cylinder registration has nowhere to go.
/// * `createInit`'s ground snap (`:475-477`, `mObjAcch.GetGroundH()`): a per-poly-group ground
///   query, and `Collision` is deliberately not inserted - a snap would teleport the NPC to y = 0.
/// * `fopAcM_posMoveF` (`:594`), `mObjAcch.CrrPos` (`:596`) and the `mTargetSpeedF` / `speedF`
///   chase (`:583-592`): every writer of a non-zero `mTargetSpeedF` is PATH-gated, so the chase
///   is a move toward zero with no position integration behind it.
/// * `_delete` (`:490-509`), including its `dComIfGp_isEnableNextStage() && strcmp(...,
///   "sea") == 0` reset of `UNK_870F` (`:505-507`) - delete-path work; the same reset runs on
///   every Outset `phase_1` (`:295`), which IS ported.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Marlon {
    /// `mMoveState` (`d_a_npc_mn.h:102`) - the `moveProc[]` dispatch index (`:560-566`).
    pub move_state: u8,
    /// `mTalk3State` (`d_a_npc_mn.h:114`) - arms the telescope scope conversation.
    pub talk3_state: u8,
    /// `mEtcFlag` (`d_a_npc_mn.h:103`) - bit 0 latches "the scope conversation is under way".
    pub etc_flag: u16,
    /// `mTalkOrder` (`d_a_npc_mn.h:100`) - `1` while an ordered talk is live.
    pub talk_order: u8,
    /// `mEvtOrderType` (`d_a_npc_mn.h:102`) - see the `ORDER_*` constants.
    pub evt_order_type: u8,
    /// `mSwitchFlag` (`d_a_npc_mn.h:106`) - bit 0 latches the `getPrmSwitchBit()` set at `:660`.
    pub switch_flag: u8,
    /// `mPosFlag` (`d_a_npc_mn.h:112`) - `chkPosNo()`'s index into `l_room_name`.
    pub pos_flag: u8,
    /// `mNpcNo` (`d_a_npc_mn.h:105`) - `getPrmNpcNo()`'s `mPosFlag != false`, the `l_npc_dat` row.
    pub npc_no: u8,
    /// `mbAllowBodyTurn` (`d_a_npc_mn.h:111`) - `l_npc_dat[mNpcNo].mbAllowBodyTurn`.
    pub allow_body_turn: bool,
    /// `mbLookOnly` (`d_a_npc_mn.h:112`) - `l_npc_dat[mNpcNo].mbLookOnly`.
    pub look_only: bool,
    /// `mbPlayerAttention` (`d_a_npc_mn.h:101`) - `chkAttention`'s within-cone latch.
    pub player_attention: bool,
    /// `mbNearPlayer` (`d_a_npc_mn.h:111`) - set only when `mNearDist > dist` (`:1442`), and
    /// `mNearDist` is `0.0f` in both rows, so retail never sets it either.
    pub near_player: bool,
    /// `mbLookFigure` (`d_a_npc_mn.h:115`) - set ONLY at `:754`, behind PATH data, so it stays
    /// false here. Kept because `getMsg` branches on it (`:1308`).
    pub look_figure: bool,
    /// `mFigureMsgIdx` (`d_a_npc_mn.h:116`) - `getRand(5)` at `:753`.
    pub figure_msg_idx: u8,
    /// `mLookMode` (`d_a_npc_mn.h:112`).
    pub look_mode: i8,
    /// `mHeadOnlyFollow` (`d_a_npc_mn.h:93`).
    pub head_only_follow: bool,
    /// `mHomeYRot` (`d_a_npc_mn.h:100`) - `home.angle.y` at `:251`, then the `HATCH` / `TURN`
    /// cut targets at `:1033` / `:785`.
    pub home_y_rot: i16,
    /// `mTargetYRot` (`d_a_npc_mn.h:109`).
    pub target_y_rot: i16,
    /// `mTurnVel` (`d_a_npc_mn.h:109`) - the head/backbone turn rate (`:1505`).
    pub turn_vel: i16,
    /// `mLookAtMaxVel` (`d_a_npc_mn.h:109`) - reloaded from `l_npc_dat` at the end of every
    /// `chkAttention` (`:1473`).
    pub look_at_max_vel: i16,
    /// `mLookResetTimer` (`d_a_npc_mn.h:101`) - `l_npc_dat[mNpcNo].mLookResetTimer`.
    pub look_reset_timer: i16,
    /// `mAttnDist` (`d_a_npc_mn.h:78`) - the live copy, doubled while the player looks through
    /// the telescope (`:646`).
    pub attn_dist: f32,
    /// `mAttnAngle` (`d_a_npc_mn.h:100`) - `l_npc_dat[mNpcNo].mAttnAngle`.
    pub attn_angle: i16,
    /// `mLookAtPos` (`d_a_npc_mn.h:83`).
    pub look_at_pos: Vec3,
    /// `eyePos` (`d_a_npc_mn.h:74`) - `current.pos.y + mEyePosOfsY` (`:600`).
    pub eye_pos: Vec3,
    /// `mJumpSpeedY` (`d_a_npc_mn.h:79`) - the `JUMP` cut's vertical integrator.
    pub jump_speed_y: f32,
    /// `mEventTimer` (`d_a_npc_mn.h:100`) - the `WAIT` / `SWON` / `BIKKURI` cut timer.
    pub event_timer: i16,
    /// `mCurrMsgNo` (`d_a_npc_mn.h:95`) - the number `fopMsgM_messageSet` was handed.
    pub msg_id: u32,
    /// `mCurrMsgBsPcId` (`d_a_npc_mn.h:95`) - `fpcM_ERROR_PROCESS_ID_e` when nothing is live.
    pub msg_bs_id: u32,
    /// `mpCurrMsg` (`d_a_npc_mn.h:75`) - whether `fopMsgM_SearchByID` has resolved yet.
    pub msg_bound: bool,
    /// `mLastMsgStatus` (`d_a_npc_mn.h:102`).
    pub prev_msg_status: u16,
    /// Which `l_msg_mn_*` table `mpMsgNo` walks, or `Chain::None` for `mpMsgNo == NULL`.
    pub msg_chain: Chain,
    /// `mpMsgNo` (`d_a_npc_mn.h:77`) - `Some(index)` into `msg_chain`'s row while walking it.
    pub msg_cursor: Option<usize>,
    /// `mAnmFlag` (`d_a_npc_mn.h:108`) - bit 0 is `setAnmTbl`'s chain-finished latch, read by
    /// `eventBikkuri` (`:1070`).
    pub anm_flag: u8,
    /// `mBckIdx` (`d_a_npc_mn.h:107`) - the morph row `setAnm` selected (`:1563`).
    pub bck_idx: u8,
    /// `getPrmSwitchBit()` (`d_a_npc_mn.cpp:1372-1374`) - the ocarina / bomb / telescope handoff.
    pub prm_switch: u8,
    /// `getPrmSwitchBit2()` (`d_a_npc_mn.cpp:1377-1379`) - the `WAIT` / `SWON` cut's switch.
    pub prm_switch2: u8,
    /// `m_jnt` (`d_a_npc_mn.h:90`) - the shared `dNpc_JntCtrl_c` head/backbone controller, the
    /// same base NPC type `npc_ah.rs` already transcribes; `lookAtTarget` writes
    /// `current.angle.y`, so it is gameplay, not decoration.
    pub jnt: JntCtrl,
    /// The `cM_rnd` lane set, seeded exactly as retail boots it.
    pub seed: [i32; 3],
    /// `entered` - one-shot guard so the ctor's world reads and `phase_1` run once, on the first
    /// tick, which is where `StageName` and `EventList` first become readable (RULE 6b).
    pub entered: bool,
    /// `deleted` - `chkEndEvent`'s `fopAcM_delete(this)` (`:1612`).
    pub deleted: bool,
    /// `prev_do` - the crate's `DoIntent` edge latch, the same shape `npc_ah.rs` keeps so a held
    /// A button reads as one `eventInfo.checkCommandTalk()`.
    pub prev_do: bool,
}

impl Marlon {
    /// `daNpcMn_c::daNpcMn_c()` (`:243-256`) minus `mResFlag` (resource bookkeeping), minus
    /// `mAnmMorfOverride` / `mBckIdx` / `mLookMode` / `mHeadOnlyFollow`, which carry their
    /// documented defaults, and minus the three initialisers that read the world -
    /// `mHomeYRot = home.angle.y` (`:251`), `mPosFlag = chkPosNo()` (`:254`) and
    /// `mNpcNo = getPrmNpcNo()` (`:255`). Those three run in `create_init` on the first tick,
    /// where `StageName` and `Actor` are available.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            move_state: MOVE_WAIT,
            talk3_state: TALK3_INIT,
            etc_flag: 0,
            talk_order: 0,
            evt_order_type: ORDER_NONE,
            switch_flag: 0,
            pos_flag: ROOM_COUNT,
            npc_no: 1,
            allow_body_turn: DAT0_ALLOW_BODY_TURN != 0,
            look_only: DAT0_LOOK_ONLY != 0,
            player_attention: false,
            near_player: false,
            look_figure: false,
            figure_msg_idx: 0,
            look_mode: LOOK_NONE,
            head_only_follow: true,
            home_y_rot: 0,
            target_y_rot: 0,
            turn_vel: 0,
            look_at_max_vel: DAT0_LOOK_AT_MAX_VEL,
            look_reset_timer: 0,
            attn_dist: DAT0_ATTN_DIST,
            attn_angle: DAT0_ATTN_ANGLE,
            look_at_pos: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            jump_speed_y: 0.0,
            event_timer: 0,
            msg_id: 0,
            msg_bs_id: FPC_M_ERROR_PROCESS_ID,
            msg_bound: false,
            prev_msg_status: PREV_STATUS_NONE,
            msg_chain: Chain::None,
            msg_cursor: None,
            anm_flag: 0,
            bck_idx: BCK_WAIT01,
            prm_switch: prm(parameters, PRM_SWITCH_WIDTH, PRM_SWITCH_SHIFT),
            prm_switch2: prm(parameters, PRM_SWITCH2_WIDTH, PRM_SWITCH2_SHIFT),
            jnt: JntCtrl::create(),
            seed: [RND_SEED; 3],
            entered: false,
            deleted: false,
            prev_do: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -70.0f, 0.0f, -70.0f, 70.0f, 200.0f, 70.0f)` (`:450`). The
    /// profile is `fopAc_CULLBOX_CUSTOM_e` (`:1728`) AND the actor really does call
    /// `fopAcM_setCullSizeBox`, so this is a transcribed box, not a table default.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daNpcMn_c::createInit` (`:405-487`), the reachable half.
    fn create_init(&mut self, actor: &mut Actor, stage: Option<&str>) {
        // `mHomeYRot = home.angle.y;` (:251) - `home` is the stage-placed spawn transform.
        self.home_y_rot = actor.angle[1];
        // `mPosFlag = chkPosNo();` (:254) - `chkPosNo` compares `dComIfGp_getNextStageName()`
        // against `l_room_name`, which is the name `dComIfGp_setNextStage` last stored
        // (`d_com_inf_game.h:2303-2305`): the field keeps that name after the mount clears only
        // the enable flag (`dStage_nextStage_c::offEnable` via `d_s_play.cpp:1344`), so an
        // unconsumed request is `transition::NextStage::target` and a mounted stage is the stage
        // itself, which is what `stage` carries.
        self.pos_flag = chk_pos_no(stage);
        // `mNpcNo = getPrmNpcNo();` (:255) - `getPrmNpcNo()` is `mPosFlag != false`
        // (`:1362-1364`). A name that matched nothing yields `ARRAY_SIZE`, still non-zero, so
        // the inside row is also the safe default.
        self.npc_no = u8::from(self.pos_flag != 0);
        // `fopAcM_SetGravity(this, -9.0f);` (:440)
        actor.gravity = CREATE_GRAVITY;
        // `setAnmTbl(&l_npc_anm_wait);` (:441)
        self.set_anm_tbl(ANM_WAIT);
        // `mHatchEventIdx = dComIfGp_evmng_getEventIdx("FIGURE_HATCH_OPEN");` (:442) - the 8-bit
        // evId form is a confirmed-absent input (port_contract.md 3b), so `EventList::event_index`
        // over the NAME is used instead, the same substitution `mbdoor.rs`, `vgnfd.rs`,
        // `zunari.rs` and `dk.rs` already make. Flagged in the report.
        // `mTurnVel = 0; mTalkOrder = 0; mbPlayerAttention = 0; mbNearPlayer = 0;` (:445-448)
        self.turn_vel = 0;
        self.talk_order = 0;
        self.player_attention = false;
        self.near_player = false;
        // `attention_info.distances[TALK] = 0xAA; distances[SPEAK] = 0xAA;
        // attention_info.flags = LOCKON_TALK | ACTION_SPEAK;` (:451-453) - omitted, the
        // attention subsystem has no counterpart.
        // `m_jnt.setParam(mMax_backbone_x, mMax_backbone_y, mMin_backbone_x, mMin_backbone_y,
        // mMax_head_x, mMax_head_y, mMin_head_x, mMin_head_y, mMax_turn_step);` (:454-464)
        self.jnt.set_param(
            DAT0_MAX_BACKBONE_X,
            DAT0_MAX_BACKBONE_Y,
            DAT0_MIN_BACKBONE_X,
            DAT0_MIN_BACKBONE_Y,
            DAT0_MAX_HEAD_X,
            DAT0_MAX_HEAD_Y,
            DAT0_MIN_HEAD_X,
            DAT0_MIN_HEAD_Y,
            DAT0_MAX_TURN_STEP,
        );
        // `mbAllowBodyTurn = l_npc_dat[mNpcNo].mbAllowBodyTurn;` (:465)
        self.allow_body_turn = DAT0_ALLOW_BODY_TURN != 0;
        // `mbLookOnly = l_npc_dat[mNpcNo].mbLookOnly;` (:466)
        self.look_only = DAT0_LOOK_ONLY != 0;
        // `mAttnDist = l_npc_dat[mNpcNo].mAttnDist;` (:467)
        self.attn_dist = DAT0_ATTN_DIST;
        // `mAttnAngle = l_npc_dat[mNpcNo].mAttnAngle;` (:468)
        self.attn_angle = DAT0_ATTN_ANGLE;
        // `mStts.Init(weight, 0xFF, this);` (:481) with `weight = 0xFF` from `:406` - the only
        // reader of `cCcD_Stts::Init`'s first argument in this crate is `Actor::weight_uc`.
        actor.weight_uc = CYL_STTS_WEIGHT;
        // `mCyl.Set(dNpc_cyl_src); mCyl.SetStts(&mStts); setCollision(&mCyl, current.pos,
        // l_npc_dat[mNpcNo].mCylRadius, 150.0f);` (:482-484) - the `dCcD` registration is
        // omitted (RULE 3b, no actor-collision registry).
    }

    /// `phase_1` (`:290-316`). `true` means one of the `cPhs_STOP_e` arms, which cancels the
    /// process during Create (`f_pc_create_req.cpp:106-108`) so the actor never executes even once:
    /// RULE 6b's "spawn it, then despawn on the FIRST TICK", never `Option<Self> -> None`.
    fn phase_1(&mut self, switches: &Switches, stories: &mut StoryBits) -> bool {
        // `u8 eventReg = dComIfGs_getEventReg(dSv_event_flag_c::UNK_870F);` (:301) - read before
        // the switch so the `default` arm sees it.
        let mut event_reg = stories.reg(FLAG_870F);
        match self.pos_flag {
            // `case 0:` (:294-299)
            ROOM_SEA => {
                // `dComIfGs_setEventReg(dSv_event_flag_c::UNK_870F, 0);` (:295)
                stories.set_reg(FLAG_870F, 0);
                // `if (fopAcM_isSwitch(this, i_this->getPrmSwitchBit())) return cPhs_STOP_e;`
                // (:296-298). `dSv_info_c::isSwitch` is FALSE for 255, so the explicit guard
                // only avoids handing 0xFF to the array.
                if self.prm_switch != NO_SWITCH && switches.test(self.prm_switch) {
                    return true;
                }
            }
            // `default:` (:300-312)
            _ => {
                // `if (i_this->isChangePos(eventReg))` (:302) - `isChangePos(i_posNo)` is
                // `i_posNo == false` (`:1677-1679`), i.e. true only while the register is clear.
                if event_reg == 0 {
                    // `if (dComIfGs_isEventBit(UNK_3A01)) eventReg = i_this->getPosNo(); else
                    // eventReg = 1;` (:303-307)
                    event_reg = if stories.test(FLAG_3A01) {
                        self.get_pos_no(stories)
                    } else {
                        ROOM_PFIGURE
                    };
                    // `dComIfGs_setEventReg(dSv_event_flag_c::UNK_870F, eventReg);` (:308)
                    stories.set_reg(FLAG_870F, event_reg);
                }
                // `if (eventReg != i_this->getPosFlag()) return cPhs_STOP_e;` (:310-312)
                if event_reg != self.pos_flag {
                    return true;
                }
            }
        }
        false
    }

    /// `u8 daNpcMn_c::getPosNo()` (`:1632-1674`) - choose which figure room this Marlon instance
    /// walks to, weighted by how many collected figures already stand in each room.
    fn get_pos_no(&mut self, stories: &StoryBits) -> u8 {
        // `if (dComIfGs_isEventBit(UNK_3D08) && !dComIfGs_isEventBit(UNK_3120)) return 1;`
        // (:1634-1636)
        if stories.test(FLAG_3D08) && !stories.test(FLAG_3120) {
            return ROOM_PFIGURE;
        }
        // `int roomCnt[8]; roomCnt[0] = 1; for (int i = 1; i < 8; i++) roomCnt[i] = 0;`
        // (:1638-1641)
        let mut room_cnt = [0u8; 8];
        room_cnt[0] = 1;
        // `for (int fig = 0; fig < 0x86; fig++) { if (fig / 8 < 0x11) { int bit = fig % 8; u8 reg
        // = dComIfGs_getEventReg(l_figure_comp[fig / 8]); if (reg & (1 << bit)) { int roomId =
        // dSnap_GetFigRoomId(fig); if (roomId != 0xFF && roomId < 8) roomCnt[roomId]++; } } }`
        // (:1643-1654). `is_figure_get` is the `fig / 8 < 0x11` + `reg & (1 << bit)` half and
        // `fig_room_id` is `dSnap_GetFigRoomId` (`d_snap.cpp`), both already transcribed by
        // `npc_mt.rs` against that same `l_figure_comp` table.
        for fig in 0..TOTAL_FIGURE_COUNT {
            if is_figure_get(stories, fig) == 0 {
                continue;
            }
            let room = fig_room_id(fig);
            if room != PHOTO_NONE && room < 8 {
                let slot = &mut room_cnt[usize::from(room)];
                *slot = slot.saturating_add(1);
            }
        }
        // `int nonZero = 0; for (int i = 0; i < 8; i++) if (roomCnt[i] != 0) nonZero++;`
        // (:1656-1661)
        let non_zero = room_cnt.iter().filter(|count| **count != 0).count();
        // `int rnd = getRand(nonZero);` (:1663)
        let mut rnd = get_rand(&mut self.seed, i32::try_from(non_zero).unwrap_or(i32::MAX));
        // `for (int i = 0; i < 8; i++) { if (roomCnt[i] != 0) { if (rnd) { rnd--; } else
        // { return i + 1; } } } return 1;` (:1664-1673) - the result is an `l_room_name` index,
        // so room 0 (`"sea"`, `l_room_name[0]`) maps to 1.
        for (i, count) in room_cnt.iter().enumerate() {
            if *count != 0 {
                if rnd != 0 {
                    rnd -= 1;
                } else {
                    return i as u8 + 1;
                }
            }
        }
        ROOM_PFIGURE
    }

    /// `bool daNpcMn_c::_execute()` (`:569-605`).
    fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        // `chkAttention();` (:570)
        self.chk_attention(ctx);
        // `checkOrder();` (:571)
        self.check_order(actor, ctx);
        // `if (!dComIfGp_event_runCheck() || eventInfo.checkCommandTalk() || mTalk3State !=
        // TALK3_INIT) { (this->*moveProc[mMoveState])(); } else { eventMove(); }` (:573-577).
        // `moveProc[2]` (WALK) and `moveProc[3]` (TURN) are PATH-only and unreachable, so the
        // `_` arm is `MOVE_PROC_WAIT`.
        if !ctx.event_run || ctx.talk_command || self.talk3_state != TALK3_INIT {
            match self.move_state {
                MOVE_TALK => self.execute_talk(actor, ctx),
                MOVE_TALK3 => self.execute_talk3(actor, ctx),
                _ => self.execute_wait(actor, ctx),
            }
        } else {
            self.event_move(actor, ctx);
        }
        // `eventOrder();` (:579) - omitted, see the type-level doc comment.
        // `playTexPatternAnm(); playAnm();` (:580-581) - BTP / BCK frame data, RULE 3b.
        // `fopAcM_posMoveF(this, mStts.GetCCMoveP());` / `mObjAcch.CrrPos(...)` / `setCollision(
        // ...)` / the `speedF` chase (:583-598) - omitted.
        // `attention_info.position.set(current.pos.x, current.pos.y + mAttnPosOfsY, current.pos.z);`
        // (:599) - omitted, the attention request channel is a RULE 3b absence.
        // `eyePos.set(current.pos.x, current.pos.y + mEyePosOfsY, current.pos.z);` (:600).
        self.eye_pos = Vec3::new(
            actor.pos.x,
            actor.pos.y + self.dat_eye_pos_ofs_y(),
            actor.pos.z,
        );
        // `lookBack();` (:602)
        self.look_back(actor);
    }

    /// `l_npc_dat[mNpcNo].mEyePosOfsY` - the one `l_npc_dat` field `_execute` re-reads every tick.
    /// Both rows hold `120.0f` (`:107`, `:117`), so this returns the same number either way; it is
    /// written through the row index because that is how the reference reaches it.
    fn dat_eye_pos_ofs_y(&self) -> f32 {
        if self.npc_no == 0 {
            DAT0_EYE_POS_OFS_Y
        } else {
            DAT1_EYE_POS_OFS_Y
        }
    }

    /// `u8 daNpcMn_c::executeCommon()` (`:608-619`).
    fn execute_common(&mut self, actor: &mut Actor) -> u8 {
        // `if (mbPlayerAttention) mEvtOrderType = true; else mEvtOrderType = false;` (:609-613) -
        // a `bool` assigned into the `u8` field.
        self.evt_order_type = if self.player_attention {
            ORDER_ATTENTION
        } else {
            ORDER_NONE
        };
        // `if (mTalkOrder == 1 && mMoveState != MOVE_PROC_TALK)
        // executeSetMode(MOVE_PROC_TALK);` (:615-617)
        if self.talk_order == 1 && self.move_state != MOVE_TALK {
            self.execute_set_mode(MOVE_TALK, actor);
        }
        self.talk_order
    }

    /// `void daNpcMn_c::executeSetMode(u8 i_initIdx)` (`:622-626`):
    /// `mMoveState = (this->*l_execute_init[i_initIdx])();` - the argument names the INITIALISER
    /// and the initialiser returns the NEW state (`l_execute_init[]` at `:551-557`).
    fn execute_set_mode(&mut self, init: u8, actor: &mut Actor) {
        // `mTargetSpeedF = 0.0f; fopAcM_SetSpeedF(this, 0.0f);` (:623-624)
        actor.speed_f = 0.0;
        let next = match init {
            MOVE_TALK => self.execute_talk_init(),
            MOVE_TALK3 => self.execute_talk3_init(),
            _ => self.execute_wait_init(),
        };
        self.move_state = next;
    }

    /// `int daNpcMn_c::executeWaitInit()` (`:629-639`). The `mWaitTimer` draws at `:633` and
    /// `:636` reach only PATH-gated readers, so neither `cM_rndF` is carried.
    fn execute_wait_init(&mut self) -> u8 {
        // `fopAcM_SetSpeedF(this, 0.0f);` (:630) - `executeSetMode` already zeroed it.
        // `if (mbLookFigure) { setAnmTbl(&l_npc_anm_wait2); ... } else
        // { setAnmTbl(&l_npc_anm_wait); ... }` (:631-637)
        if self.look_figure {
            self.set_anm_tbl(ANM_WAIT2);
        } else {
            self.set_anm_tbl(ANM_WAIT);
        }
        // `return MOVE_PROC_WAIT;` (:638)
        MOVE_WAIT
    }

    /// `int daNpcMn_c::executeTalkInit() { return MOVE_PROC_TALK; }` (`:693-695`).
    fn execute_talk_init(&mut self) -> u8 {
        MOVE_TALK
    }

    /// `int daNpcMn_c::executeTalk3Init() { mTalk3State = TALK3_INIT; return MOVE_PROC_TALK3; }`
    /// (`:710-713`).
    fn execute_talk3_init(&mut self) -> u8 {
        self.talk3_state = TALK3_INIT;
        MOVE_TALK3
    }

    /// `void daNpcMn_c::executeWait()` (`:642-690`).
    fn execute_wait(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        // `if (!executeCommon()) {` (:643) - the return value IS `mTalkOrder`.
        if self.execute_common(actor) != 0 {
            return;
        }
        // `if (mPosFlag == 0) {` (:644)
        if self.pos_flag != ROOM_SEA {
            return;
        }
        // `if (dComIfGp_checkPlayerStatus0(0, daPyStts0_TELESCOPE_LOOK_e)) { mAttnDist =
        // (s16)(l_npc_dat[mNpcNo].mAttnDist * 2.0f); } else { mAttnDist =
        // l_npc_dat[mNpcNo].mAttnDist; mEtcFlag &= 0xFFFE; }` (:645-650)
        if ctx.telescope_look {
            self.attn_dist = DAT0_ATTN_DIST * 2.0;
        } else {
            self.attn_dist = DAT0_ATTN_DIST;
            self.etc_flag &= 0xFFFE;
        }
        // `if (mbPlayerAttention && dComIfGs_isEventBit(UNK_2F08) && dComIfGp_checkPlayerStatus0(0,
        // daPyStts0_TELESCOPE_LOOK_e)) { dComIfGp_setScopeType(dScpTyp_UNK1_e);` (:651-652) - the
        // scope-type write is a camera setting with no counterpart; the gate is transcribed.
        if self.player_attention && ctx.stories.test(FLAG_2F08) && ctx.telescope_look {
            // `if (dComIfGp_getMesgStatus() == fopMsgStts_SCOPE_ACTIVE_e && !(mEtcFlag & 1)) {
            // mEtcFlag |= 1; executeSetMode(MOVE_PROC_TALK3); }` (:653-656)
            if ctx.text.info.mesg_status == STATUS_SCOPE_ACTIVE && self.etc_flag & 1 == 0 {
                self.etc_flag |= 1;
                self.execute_set_mode(MOVE_TALK3, actor);
            }
        }
        // `if (!(mSwitchFlag & 1) && fopAcM_isSwitch(this, getPrmSwitchBit())) {
        // fopAcM_onSwitch(this, getPrmSwitchBit()); mSwitchFlag |= 1; mbAllowBodyTurn = 0;
        // mbLookOnly = 0; mEvtOrderType = 3; }` (:659-665) - the ocarina / bomb / telescope
        // handoff, the switch the player sets to open the game. RULE 11b: the real index passes
        // through unclamped; an index >= 0x80 reads false here and that is the save model's gap.
        if self.switch_flag & 1 == 0
            && self.prm_switch != NO_SWITCH
            && ctx.switches.test(self.prm_switch)
        {
            ctx.switches.set(self.prm_switch);
            self.switch_flag |= 1;
            self.allow_body_turn = false;
            self.look_only = false;
            self.evt_order_type = ORDER_HATCH;
        }
        // The `else if (mbLookFigure)` (`:666-679`) and final `else` (`:680-688`) arms are gated on
        // `mPathRun` / `mbLookFigure`; RULE 3b, omitted. See the type-level doc comment.
    }

    /// `void daNpcMn_c::executeTalk()` (`:698-707`).
    fn execute_talk(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        // `executeCommon();` (:699)
        self.execute_common(actor);
        // `if (talk2(1) == fopMsgStts_BOX_CLOSED_e) { mTalkOrder = 0;
        // executeSetMode(MOVE_PROC_WAIT); dComIfGp_event_reset(); } else
        // { setAnmFromMsgTag(); }` (:700-706)
        if self.talk2(true, ctx) == STATUS_BOX_CLOSED {
            self.talk_order = 0;
            self.execute_set_mode(MOVE_WAIT, actor);
            ctx.text.reset_event();
        } else {
            self.set_anm_from_msg_tag(ctx);
        }
    }

    /// `void daNpcMn_c::executeTalk3()` (`:716-736`) - the telescope scope conversation.
    fn execute_talk3(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        match self.talk3_state {
            // `case TALK3_INIT: case TALK3_ORDER:` (:718-727)
            TALK3_INIT | TALK3_ORDER => {
                // `if (eventInfo.checkCommandDemoAccrpt()) {` (:720)
                if ctx.demo_accept {
                    // `mTalk3State = TALK3_TALK;` (:721)
                    self.talk3_state = TALK3_TALK;
                } else {
                    // `fopAcM_orderPotentialEvent(this, dEvtFlag_STAFF_ALL_e | dEvtFlag_UNK8_e, 0,
                    // 0); eventInfo.onCondition(dEvtCnd_UNK2_e);` (:723-724) - omitted.
                    // `mTalk3State = TALK3_ORDER;` (:725)
                    self.talk3_state = TALK3_ORDER;
                }
            }
            // `case TALK3_TALK: if (talk3(1) == fopMsgStts_BOX_CLOSED_e) { mTalk3State =
            // TALK3_INIT; executeSetMode(MOVE_PROC_WAIT); dComIfGp_event_reset(); } break;`
            // (:728-734)
            TALK3_TALK => {
                if self.talk3(true, ctx) == STATUS_BOX_CLOSED {
                    self.talk3_state = TALK3_INIT;
                    self.execute_set_mode(MOVE_WAIT, actor);
                    ctx.text.reset_event();
                }
            }
            // `mTalk3State` cannot hold anything else: the three enumerators are written
            // exhaustively by `executeTalk3Init` (`:711`) and this function.
            _ => {}
        }
    }

    /// `void daNpcMn_c::checkOrder()` (`:826-835`).
    fn check_order(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        // `if (eventInfo.checkCommandDemoAccrpt()) {` (:827)
        if ctx.demo_accept {
            // `if (dComIfGp_evmng_startCheck(mHatchEventIdx) && mEvtOrderType == 3)
            // mEvtOrderType = 0;` (:828-830)
            if ctx.hatch.is_some_and(|index| ctx.active.start_check(index))
                && self.evt_order_type == ORDER_HATCH
            {
                self.evt_order_type = ORDER_NONE;
            }
        // `} else if (eventInfo.checkCommandTalk() && (mEvtOrderType == 2 ||
        // mEvtOrderType == 1)) { mTalkOrder = 1; executeSetMode(MOVE_PROC_TALK); }` (:831-834)
        } else if ctx.talk_command
            && (self.evt_order_type == ORDER_TALK || self.evt_order_type == ORDER_ATTENTION)
        {
            self.talk_order = 1;
            self.execute_set_mode(MOVE_TALK, actor);
        }
    }

    /// `void daNpcMn_c::eventMove()` (`:850-862`).
    fn event_move(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        // `if (!chkEndEvent()) {` (:851)
        if self.chk_end_event(ctx) {
            return;
        }
        // `bool attn = mEventCut.getAttnFlag(); if (mEventCut.cutProc()) { if
        // (!mEventCut.getAttnFlag()) mEventCut.setAttnFlag(attn); } else { privateCut();
        // setAnmFromMsgTag(); }` (:852-860). `dEventCut_c::getAttnFlag` / `cutProc` have no
        // counterpart, so the `else` arm is always taken.
        self.private_cut(actor, ctx);
        self.set_anm_from_msg_tag(ctx);
    }

    /// `void daNpcMn_c::privateCut()` (`:865-947`) - the ten `cut_name_tbl` actions.
    fn private_cut(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        // `int staffIdx = dComIfGp_evmng_getMyStaffId(l_npc_staff_id[0]); if (staffIdx != -1) {`
        // (:869-870). `ActiveEvent::staff_cut` returns `None` for the `-1` case.
        let Some((cut, advance)) = ctx.active.staff_cut(ctx.list, STAFF_MN) else {
            return;
        };
        // `mActIdx = dComIfGp_evmng_getMyActIdx(staffIdx, cut_name_tbl, 10, TRUE, 0);` (:871)
        let Some(act) = cut_name_index(ctx.list, cut) else {
            // `if (mActIdx == -1) { dComIfGp_evmng_cutEnd(staffIdx); }` (:872-874)
            ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_MN);
            return;
        };
        // `if (dComIfGp_evmng_getIsAddvance(staffIdx)) { switch (mActIdx) { case 0:
        // eventMesSetInit(staffIdx); case 1: eventGetItemInit(); case 2: eventWaitInit(staffIdx);
        // case 3: eventHatchInit(); case 4: eventBikkuriInit(staffIdx); case 5: eventTurnInit();
        // case 6: eventWalkInit(); case 7: eventLookInit(); case 8: eventJumpInit(staffIdx);
        // case 9: eventSwOnInit(staffIdx); } }` (:875-908). Cases 1, 6 and 7 have no ported
        // body: 1's `mItemNo` is never written by this actor and its `createItemForPresentDemo`
        // is item-table work; 6 needs a room PATH; 7 sets a two-row animation table whose only
        // reader is `eventLook`'s BCK frame test.
        if advance {
            match act {
                CUT_MES_SET => self.cut_mes_set_init(ctx),
                CUT_HATCH => self.cut_hatch_init(ctx),
                CUT_TURN => self.cut_turn_init(),
                // `eventWalkInit() { setAnmTbl(&l_npc_anm_walk); }` (:1112-1114). The initialiser
                // really does run in retail; only `eventWalk`'s per-frame PATH test is omitted, so
                // the cut then takes `default: shouldEnd = true` below.
                CUT_WALK => self.cut_walk_init(),
                CUT_JUMP => self.cut_jump_init(actor),
                // `eventWaitInit` / `eventSwOnInit`: `mEventTimer = 0; if (timer != NULL)
                // mEventTimer = *timer;` (:989-993, :1010-1014). The `int` truncates into the
                // `s16` field.
                CUT_WAIT | CUT_SWON => {
                    self.event_timer = ctx
                        .cut_number(KEY_TIMER)
                        .map_or(CUT_TIMER_DEFAULT, |value| value as i16);
                }
                // `eventBikkuriInit`: `mEventTimer = 1; if (timer != NULL) mEventTimer = *timer;
                // if (mEventTimer <= 0) mEventTimer = 1;` (:1052-1058)
                CUT_BIKKURI => {
                    let rolled = ctx
                        .cut_number(KEY_TIMER)
                        .map_or(BIKKURI_TIMER_MIN, |value| value as i16);
                    self.event_timer = if rolled <= 0 {
                        BIKKURI_TIMER_MIN
                    } else {
                        rolled
                    };
                }
                _ => {}
            }
        }
        // `bool shouldEnd; switch (mActIdx) { case 0: eventMesSet(); case 2: eventWait(staffIdx);
        // case 3: eventHatch(); case 4: eventBikkuri(); case 5: eventTurn(staffIdx); case 6:
        // eventWalk(); case 7: eventLook(); case 8: eventJump(); case 9: eventSwOn(); default:
        // shouldEnd = true; } if (shouldEnd) dComIfGp_evmng_cutEnd(staffIdx);` (:909-945).
        // `dComIfGp_evmng_cutEnd` maps to `ActiveEvent::end_staff_cut`, the substitution
        // `npc_tc.rs` and `ivan.rs` already make.
        let should_end = match act {
            // `bool daNpcMn_c::eventMesSet() { return talk2(0) == fopMsgStts_BOX_CLOSED_e; }`
            // (:975-977)
            CUT_MES_SET => self.talk2(false, ctx) == STATUS_BOX_CLOSED,
            CUT_WAIT => self.cut_wait(ctx),
            CUT_HATCH => self.cut_hatch(actor),
            CUT_BIKKURI => self.cut_bikkuri(),
            CUT_TURN => self.cut_turn(actor, ctx),
            CUT_JUMP => self.cut_jump(actor, ctx),
            CUT_SWON => self.cut_sw_on(ctx),
            // `default: shouldEnd = true; break;` (:938-940). Case 1 (`GET_ITEM`) lands here in
            // retail too, because the second switch has no `case 1` - `eventGetItemInit` runs once
            // and the cut closes. Cases 6 (`WALK`) and 7 (`LOOK`) land here as declared RULE 3b
            // omissions: WALK's only target is a room PATH, LOOK's only test is a BCK frame.
            _ => true,
        };
        if should_end {
            ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_MN);
        }
    }

    /// `bool daNpcMn_c::eventWait(int i_staffIdx)` (`:997-1006`).
    fn cut_wait(&mut self, ctx: &mut Ctx) -> bool {
        // `if (mEventTimer != 0) { mEventTimer--; return false; }` (:998-1001)
        if self.event_timer != 0 {
            self.event_timer -= 1;
            return false;
        }
        // `if (dComIfGp_evmng_getMyIntegerP(i_staffIdx, "SwOn") != NULL) fopAcM_onSwitch(this,
        // getPrmSwitchBit2()); return true;` (:1002-1005)
        if ctx.cut_has(KEY_SWON) {
            ctx.switches.set(self.prm_switch2);
        }
        true
    }

    /// `bool daNpcMn_c::eventSwOn()` (`:1018-1025`).
    fn cut_sw_on(&mut self, ctx: &mut Ctx) -> bool {
        // `if (mEventTimer) { mEventTimer--; return false; }` (:1019-1022)
        if self.event_timer != 0 {
            self.event_timer -= 1;
            return false;
        }
        // `fopAcM_onSwitch(this, getPrmSwitchBit2()); return true;` (:1023-1024)
        ctx.switches.set(self.prm_switch2);
        true
    }

    /// `void daNpcMn_c::eventMesSetInit(int i_staffIdx)` (`:950-972`).
    fn cut_mes_set_init(&mut self, ctx: &mut Ctx) {
        // `int* pData = dComIfGp_evmng_getMyIntegerP(i_staffIdx, "MsgNo");` (:951)
        let Some(data) = ctx.cut_number(KEY_MSG_NO) else {
            // `} else { mpMsgNo++; setMessage(*mpMsgNo); }` (:968-971)
            let next = self
                .msg_cursor
                .and_then(|cursor| self.chain_entry(cursor + 1));
            if let Some(no) = next {
                self.set_message(no);
            }
            return;
        };
        // `mpMsgNo = NULL;` (:953)
        self.msg_chain = Chain::None;
        self.msg_cursor = None;
        // `int msg = *pData; switch (msg) { case 1: break; case 0: msg = getMsg();
        // setMessage(msg); break; default: setMessage(msg); }` (:954-964)
        match data {
            1 => {}
            0 => {
                let msg = self.get_msg(ctx);
                self.set_message(msg);
            }
            other => self.set_message(other as u32),
        }
        // `if (mpMsgNo != NULL) { setMessage(*mpMsgNo); }` (:965-967) - `getMsg`'s `case 0` arm
        // already returned the head, so this re-writes the same number.
        if self.msg_cursor.is_some() {
            if let Some(no) = self.chain_entry(0) {
                self.set_message(no);
            }
        }
    }

    /// `void daNpcMn_c::eventHatchInit()` (`:1028-1035`) - face the Gallery hatch.
    fn cut_hatch_init(&mut self, ctx: &Ctx) {
        // `daObjOhatch_c* hatch = (daObjOhatch_c*)fopAcM_searchFromName("Ohatch", 0, 0); if (hatch
        // != NULL) { s16 angle; dNpc_calc_DisXZ_AngY(current.pos, hatch->current.pos, NULL,
        // &angle); mHomeYRot = angle; }` (:1029-1034)
        if let Some(hatch) = ctx.hatch_pos {
            self.home_y_rot = face_yaw(ctx.pos, hatch);
        }
    }

    /// `bool daNpcMn_c::eventHatch()` (`:1038-1047`).
    fn cut_hatch(&mut self, actor: &mut Actor) -> bool {
        // `mTargetYRot = mHomeYRot; mHeadOnlyFollow = false; mLookMode = LOOK_MODE_TURN;
        // m_jnt.setTrn();` (:1039-1042)
        self.turn_to(self.home_y_rot);
        // `if (mHomeYRot - current.angle.y == 0) return true; return false;` (:1043-1046)
        self.home_y_rot.wrapping_sub(actor.angle[1]) == 0
    }

    /// `void daNpcMn_c::eventTurnInit()` (`:1074-1076`).
    fn cut_turn_init(&mut self) {
        // `setAnmTbl(&l_npc_anm_wait);` (:1075)
        self.set_anm_tbl(ANM_WAIT);
    }

    /// `bool daNpcMn_c::eventTurn(int i_staffIdx)` (`:1079-1109`).
    fn cut_turn(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        // `int* pData = dComIfGp_evmng_getMyIntegerP(i_staffIdx, "TurnMode"); int turnMode = 0; if
        // (pData != NULL) turnMode = *pData;` (:1080-1084)
        // `switch (turnMode) { case 1: hatch = fopAcM_searchFromName("Ohatch", 0, 0); if (hatch !=
        // NULL) { pos = hatch->current.pos; break; } default: pos = mPathRun.getPoint(...); }`
        // (:1088-1098). Only the hatch target survives RULE 3b.
        let target = if ctx.cut_number(KEY_TURN_MODE).unwrap_or(0) == 1 {
            ctx.hatch_pos
        } else {
            None
        };
        let Some(target) = target else {
            // The `default:` arm's target is a room PATH point. With no PATH the turn cannot be
            // evaluated, so the cut ends here instead of turning toward an undefined point.
            return true;
        };
        // `s16 angle; dNpc_calc_DisXZ_AngY(current.pos, pos, NULL, &angle); mTargetYRot = angle;
        // mHeadOnlyFollow = false; mLookMode = LOOK_MODE_TURN; m_jnt.setTrn();` (:1099-1104)
        let angle = face_yaw(ctx.pos, target);
        self.turn_to(angle);
        // `if (current.angle.y == angle) return true; return false;` (:1105-1108)
        actor.angle[1] == angle
    }

    /// `void daNpcMn_c::eventWalkInit()` (`:1112-1114`).
    fn cut_walk_init(&mut self) {
        // `setAnmTbl(&l_npc_anm_walk);` (:1113)
        self.set_anm_tbl(ANM_WALK);
    }

    /// `void daNpcMn_c::eventJumpInit(int i_staffIdx)` (`:1153-1175`). Every staff float it reads
    /// is overwritten by the unconditional tail, so the tail is the whole behaviour.
    fn cut_jump_init(&mut self, actor: &mut Actor) {
        // `mTargetSpeedF = 40.0f;` (:1172) - no reader without `fopAcM_posMoveF`.
        let _ = JUMP_TARGET_SPEED_F;
        // `mJumpSpeedY = 25.0f;` (:1173)
        self.jump_speed_y = JUMP_SPEED_Y;
        // `fopAcM_SetGravity(this, -1.0f);` (:1174)
        actor.gravity = JUMP_GRAVITY;
    }

    /// `bool daNpcMn_c::eventJump()` (`:1178-1187`).
    fn cut_jump(&mut self, actor: &mut Actor, ctx: &Ctx) -> bool {
        // `current.pos.y += mJumpSpeedY; mJumpSpeedY += gravity;` (:1179-1180). `gravity` is
        // `fopAc_ac_c`'s member, written by `fopAcM_SetGravity` at `:1174` and `:440`.
        actor.pos.y += self.jump_speed_y;
        self.jump_speed_y += actor.gravity;
        // `if (mJumpSpeedY < 0.0f && mObjAcch.ChkGroundLanding()) { fopAcM_SetSpeedF(this, 0.0f);
        // mTargetSpeedF = 0.0f; return true; } return false;` (:1181-1186). `ChkGroundLanding` is
        // the per-actor ground check; the crate's sanctioned merged-DZB stand-in is
        // `ground::grounded`.
        if self.jump_speed_y < 0.0 && ground::grounded(ctx.collision, actor.pos) {
            actor.speed_f = 0.0;
            return true;
        }
        false
    }

    /// `bool daNpcMn_c::eventBikkuri()` (`:1062-1071`).
    fn cut_bikkuri(&mut self) -> bool {
        // `if (mEventTimer != 0) { mEventTimer--; if (mEventTimer == 0)
        // setAnmTbl(l_npc_anm_bikkuri); return false; }` (:1063-1069)
        if self.event_timer != 0 {
            self.event_timer -= 1;
            if self.event_timer == 0 {
                self.set_anm_tbl(ANM_BIKKURI);
            }
            return false;
        }
        // `return mAnmFlag & 1;` (:1070). Bit 0 is set by `playAnm` when the morph chain
        // completes (`:1547`), which needs BCK frame data this checkout does not ship - so this
        // never returns true and the cut cannot self-terminate. Declared, not invented.
        self.anm_flag & 1 != 0
    }

    /// `u16 daNpcMn_c::talk2(int i_param)` (`:1190-1224`) - the ordinary message box.
    fn talk2(&mut self, get_msg: bool, ctx: &mut Ctx) -> u8 {
        // `u16 status = 0xFF;` (:1191)
        let mut status = NO_STATUS;
        if self.msg_bs_id == FPC_M_ERROR_PROCESS_ID {
            // `if (i_param == 1) mCurrMsgNo = getMsg();` (:1193-1195)
            if get_msg {
                let msg = self.get_msg(ctx);
                self.set_message(msg);
            }
            // `mCurrMsgBsPcId = fopMsgM_messageSet(mCurrMsgNo, this); mpCurrMsg = NULL;
            // mLastMsgStatus = -1;` (:1196-1198)
            let eye = self.eye_pos;
            self.msg_bs_id = ctx.text.message_set_actor(self.msg_id, eye);
            self.msg_bound = false;
            self.prev_msg_status = PREV_STATUS_NONE;
        } else if self.msg_bound {
            // `status = mpCurrMsg->mStatus;` (:1200)
            status = ctx.text.status().unwrap_or(NO_STATUS);
            match status {
                // `case fopMsgStts_MSG_DISPLAYED_e: mpCurrMsg->mStatus = next_msgStatus(
                // &mCurrMsgNo); if (mpCurrMsg->mStatus == fopMsgStts_MSG_CONTINUES_e)
                // fopMsgM_messageSet(mCurrMsgNo);` (:1202-1207)
                STATUS_MSG_DISPLAYED => {
                    let next = self.next_msg_status();
                    ctx.text.set_status(next);
                    if next == STATUS_MSG_CONTINUES {
                        let no = self.msg_id;
                        ctx.text.message_set(no);
                    }
                }
                // `case fopMsgStts_MSG_TYPING_e: if (mLastMsgStatus == fopMsgStts_MSG_CONTINUES_e)
                // chkMsg();` (:1208-1212)
                STATUS_TYPING => {
                    if self.prev_msg_status == u16::from(STATUS_MSG_CONTINUES) {
                        self.chk_msg();
                    }
                }
                // `case fopMsgStts_BOX_CLOSED_e: mpCurrMsg->mStatus =
                // fopMsgStts_MSG_DESTROYED_e; mCurrMsgBsPcId = fpcM_ERROR_PROCESS_ID_e;`
                // (:1213-1216)
                STATUS_BOX_CLOSED => {
                    ctx.text.set_status(STATUS_MSG_DESTROYED);
                    self.msg_bs_id = FPC_M_ERROR_PROCESS_ID;
                }
                _ => {}
            }
            // `mLastMsgStatus = status; anmAtr(status);` (:1218-1219) - `anmAtr` is the base
            // class's animation hook, renderer work.
            self.prev_msg_status = u16::from(status);
        } else {
            // `mpCurrMsg = fopMsgM_SearchByID(mCurrMsgBsPcId);` (:1221)
            self.msg_bound = ctx.text.is_bound(self.msg_bs_id);
        }
        status
    }

    /// `u16 daNpcMn_c::talk3(int i_param)` (`:1227-1261`) - the SCOPE message box, which reads and
    /// writes `dComIfGp_getScopeMesgStatus` rather than the per-box status word.
    fn talk3(&mut self, get_msg: bool, ctx: &mut Ctx) -> u8 {
        // `u16 status = 0xFF;` (:1228)
        let mut status = NO_STATUS;
        if self.msg_bs_id == FPC_M_ERROR_PROCESS_ID {
            // `if (i_param == 1) mCurrMsgNo = getMsg3();` (:1230-1232); `getMsg3()` is
            // `mpMsgNo = 0; return 0x35EF;` (:1326-1329).
            if get_msg {
                self.msg_chain = Chain::None;
                self.msg_cursor = None;
                self.msg_id = MSG_TALK3;
            }
            // `mCurrMsgBsPcId = fopMsgM_scopeMessageSet(mCurrMsgNo); mpCurrMsg = NULL;
            // mLastMsgStatus = -1;` (:1233-1235)
            self.msg_bs_id = ctx.text.scope_message_set(self.msg_id);
            self.msg_bound = false;
            self.prev_msg_status = PREV_STATUS_NONE;
        } else if self.msg_bound {
            // `status = dComIfGp_getScopeMesgStatus();` (:1237)
            status = ctx.text.scope_status();
            match status {
                // `case fopMsgStts_MSG_DISPLAYED_e: dComIfGp_setScopeMesgStatus(next_msgStatus(
                // &mCurrMsgNo)); if (dComIfGp_getScopeMesgStatus() == fopMsgStts_MSG_CONTINUES_e)
                // fopMsgM_scopeMessageSet(mCurrMsgNo);` (:1239-1244)
                STATUS_MSG_DISPLAYED => {
                    let next = self.next_msg_status();
                    ctx.text.set_scope_status(next);
                    if ctx.text.scope_status() == STATUS_MSG_CONTINUES {
                        let no = self.msg_id;
                        ctx.text.scope_message_set(no);
                    }
                }
                // `case fopMsgStts_MSG_TYPING_e: if (mLastMsgStatus == fopMsgStts_MSG_CONTINUES_e)
                // chkMsg();` (:1245-1249)
                STATUS_TYPING => {
                    if self.prev_msg_status == u16::from(STATUS_MSG_CONTINUES) {
                        self.chk_msg();
                    }
                }
                // `case fopMsgStts_BOX_CLOSED_e: dComIfGp_setScopeMesgStatus(fopMsgStts_
                // BOX_CLOSING_e); mCurrMsgBsPcId = fpcM_ERROR_PROCESS_ID_e;` (:1250-1253)
                STATUS_BOX_CLOSED => {
                    ctx.text.set_scope_status(STATUS_BOX_CLOSING);
                    self.msg_bs_id = FPC_M_ERROR_PROCESS_ID;
                }
                _ => {}
            }
            // `mLastMsgStatus = status; anmAtr(status);` (:1255-1256)
            self.prev_msg_status = u16::from(status);
        } else {
            // `mpCurrMsg = fopMsgM_SearchByID(mCurrMsgBsPcId);` (:1258)
            self.msg_bound = ctx.text.is_bound(self.msg_bs_id);
        }
        status
    }

    /// `u16 daNpcMn_c::next_msgStatus(u32* i_msgNo)` (`:1264-1281`) - advance the chain cursor.
    fn next_msg_status(&mut self) -> u8 {
        // `u16 ret = fopMsgStts_MSG_CONTINUES_e;` (:1265)
        // `if (mpMsgNo != NULL) { ... } else { ret = fopNpc_npc_c::next_msgStatus(i_msgNo); }`
        // (:1266-1279) - the base is `{ return fopMsgStts_MSG_ENDS_e; }` (`d_npc.h:269`).
        let Some(cursor) = self.msg_cursor else {
            return STATUS_MSG_ENDS;
        };
        // `mpMsgNo++; switch (*mpMsgNo) { case 0: mpMsgNo = 0; ret =
        // fopNpc_npc_c::next_msgStatus(i_msgNo); break; default: *i_msgNo = *mpMsgNo; break; }`
        // (:1267-1276). `ret` stays `MSG_CONTINUES_e` in the `default` arm.
        let index = cursor + 1;
        match self.chain_entry(index) {
            // `case 0:` - the chain's `0x0000` sentinel, or a walk off the end of a row that has
            // none (`l_msg_mn_figure` is never walked, so this cannot happen for it).
            Some(0) | None => {
                self.msg_chain = Chain::None;
                self.msg_cursor = None;
                STATUS_MSG_ENDS
            }
            Some(no) => {
                self.msg_cursor = Some(index);
                self.msg_id = no;
                STATUS_MSG_CONTINUES
            }
        }
    }

    /// `*mpMsgNo` for a cursor position inside the armed chain.
    fn chain_entry(&self, index: usize) -> Option<u32> {
        self.msg_chain.row().get(index).copied()
    }

    /// `u32 daNpcMn_c::getMsg()` (`:1284-1323`) - the branch table that drives the opening.
    fn get_msg(&mut self, ctx: &mut Ctx) -> u32 {
        // `u32 msg = 0; mpMsgNo = NULL;` (:1285-1286)
        let mut msg: u32 = 0;
        self.msg_chain = Chain::None;
        self.msg_cursor = None;
        // `if (!dComIfGp_event_chkTalkXY()) {` (:1288)
        if !ctx.talk_xy {
            if self.pos_flag == ROOM_SEA {
                // `if (!mPosFlag) { if (!dComIfGs_isEventBit(UNK_2F08)) {
                // dComIfGs_onEventBit(UNK_2F08); mpMsgNo = l_msg_mn_1st_talk; } else
                // { mpMsgNo = l_msg_mn_2nd_talk; } }` (:1289-1295)
                if !ctx.stories.test(FLAG_2F08) {
                    ctx.stories.set(FLAG_2F08);
                    self.msg_chain = Chain::FirstTalk;
                } else {
                    self.msg_chain = Chain::SecondTalk;
                }
            } else if self.pos_flag == ROOM_PFIGURE && ctx.stories.test(FLAG_3D08) {
                // `} else if (mPosFlag == 1 && dComIfGs_isEventBit(UNK_3D08)) {
                // dComIfGs_onEventBit(UNK_2F04); if (!dComIfGs_isEventBit(UNK_3120)) {
                // dComIfGs_onEventBit(UNK_3120); mpMsgNo = l_msg_mn_comp_1st; } else
                // { mpMsgNo = l_msg_mn_comp_2nd; } }` (:1296-1303)
                ctx.stories.set(FLAG_2F04);
                if !ctx.stories.test(FLAG_3120) {
                    ctx.stories.set(FLAG_3120);
                    self.msg_chain = Chain::CompFirst;
                } else {
                    self.msg_chain = Chain::CompSecond;
                }
            } else if !ctx.stories.test(FLAG_2F04) {
                // `} else { if (!dComIfGs_isEventBit(UNK_2F04)) { dComIfGs_onEventBit(UNK_2F04);
                // mpMsgNo = l_msg_mn_1st_talk_in; }` (:1304-1307)
                ctx.stories.set(FLAG_2F04);
                self.msg_chain = Chain::FirstTalkIn;
            } else if self.look_figure {
                // `else if (mbLookFigure != 0) { msg = l_msg_mn_figure[mFigureMsgIdx]; }`
                // (:1308-1309) - a SCALAR read: `mpMsgNo` stays NULL. `mFigureMsgIdx` comes from
                // `getRand(5)` at `:753`, which is PATH-gated, so it is 0 here.
                msg = MSG_FIGURE
                    .get(usize::from(self.figure_msg_idx))
                    .copied()
                    .unwrap_or(0);
            } else if !ctx.stories.test(FLAG_3A01) {
                // `else { if (!dComIfGs_isEventBit(UNK_3A01)) { mpMsgNo = l_msg_mn_2nd_talk_in;
                // } else { mpMsgNo = l_msg_mn_3rd_talk_in; } }` (:1310-1316)
                self.msg_chain = Chain::SecondTalkIn;
            } else {
                self.msg_chain = Chain::ThirdTalkIn;
            }
        }
        // `if (mpMsgNo != NULL) msg = *mpMsgNo; return msg;` (:1319-1322). A non-NULL `mpMsgNo`
        // points at the chain HEAD, so the cursor starts at index 0.
        if let Some(first) = self.msg_chain.row().first() {
            msg = *first;
            self.msg_cursor = Some(0);
        }
        msg
    }

    /// `void daNpcMn_c::chkMsg() { return; }` (`:1332-1334`).
    fn chk_msg(&mut self) {}

    /// `void daNpcMn_c::setMessage(u32 i_message) { mCurrMsgNo = i_message; }` (`:1337-1340`).
    fn set_message(&mut self, message: u32) {
        self.msg_id = message;
    }

    /// `void daNpcMn_c::setAnmFromMsgTag()` (`:1343-1359`) - the message's anime attribute picks
    /// one of four rows and is consumed either way.
    fn set_anm_from_msg_tag(&mut self, ctx: &mut Ctx) {
        // `switch (dComIfGp_getMesgAnimeAttrInfo()) { case 0: setAnmTbl(&l_npc_anm_wait); case 1:
        // setAnmTbl(&l_npc_anm_talk); case 2: setAnmTbl(&l_npc_anm_talk2); case 3:
        // setAnmTbl(&l_npc_anm_wait2); }` (:1344-1357)
        match ctx.text.mesg_anime() {
            0 => {
                let _ = self.set_anm_tbl(ANM_WAIT);
            }
            1 => {
                let _ = self.set_anm_tbl(ANM_TALK);
            }
            2 => {
                let _ = self.set_anm_tbl(ANM_TALK2);
            }
            3 => {
                let _ = self.set_anm_tbl(ANM_WAIT2);
            }
            _ => {}
        }
        // `dComIfGp_setMesgAnimeAttrInfo(0xFF);` (:1358)
        ctx.text.set_mesg_anime(0xFF);
    }

    /// `bool daNpcMn_c::setAnmTbl(sMnAnmDat* i_anmDat)` (`:1567-1584`).
    fn set_anm_tbl(&mut self, dat: AnmTable) -> bool {
        // `mAnmFlag &= 0xFE;` (:1568)
        self.anm_flag &= 0xFE;
        // `if (i_anmDat->mBckIdx == BCK_NULL) { mpAnmDat = NULL; return TRUE; }` (:1569-1572)
        if dat.bck == BCK_NULL {
            return true;
        }
        // `mpAnmDat = i_anmDat; mAnmLoopCnt = mpAnmDat->mLoopCount;` (:1573-1574)
        // `int loopMode = J3DFrameCtrl::EMode_LOOP; if (mAnmLoopCnt > 0)
        // loopMode = J3DFrameCtrl::EMode_NONE;` (:1576-1579)
        // `if (mBckIdx != mpAnmDat->mBckIdx || loopMode == J3DFrameCtrl::EMode_NONE) setAnm(
        // mpAnmDat->mBckIdx, loopMode, mpAnmDat->mMorf); return FALSE;` (:1580-1583)
        if self.bck_idx != dat.bck || dat.loop_count > 0 {
            // `void daNpcMn_c::setAnm(u8, int, f32)` (`:1556-1564`) minus the
            // `dComIfG_getObjectIDRes` / `mpMorf->setAnm` pair and the `mAnmMorfOverride` latch,
            // whose only surviving effect is `mBckIdx = i_bckIdx;` (`:1563`).
            self.bck_idx = dat.bck;
        }
        false
    }

    /// `void daNpcMn_c::chkAttention()` (`:1391-1474`).
    fn chk_attention(&mut self, ctx: &Ctx) {
        // `mbNearPlayer = 0;` (:1392)
        self.near_player = false;
        // `if (mEventCut.getAttnFlag()) {` (:1393) - `dEventCut_c::getAttnFlag` / `getAttnPos` have
        // no counterpart, so the player branch below always runs. `npc_ah.rs` records the same
        // gap.
        let Some(player) = ctx.player else {
            // With no link player `dNpc_calc_DisXZ_AngY` has no second point, so neither distance
            // test can pass and `mLookMode = LOOK_MODE_NONE` (`:1459`) is the faithful result.
            self.look_mode = LOOK_NONE;
            self.look_at_max_vel = DAT0_LOOK_AT_MAX_VEL;
            return;
        };
        // `f32 maxDist = mAttnDist; int maxAngle = mAttnAngle;` (:1407-1408)
        let mut max_dist = self.attn_dist;
        let mut max_angle = i32::from(self.attn_angle);
        // `dNpc_calc_DisXZ_AngY(current.pos, player->current.pos, &dist, &angle);` (:1412)
        let (dist, angle) = dist_and_yaw(ctx.pos, player.pos);
        // `if (mbPlayerAttention != 0) { maxDist += 40.0f; maxAngle += 0x71C; }` (:1413-1416)
        if self.player_attention {
            max_dist += ATTN_DIST_EXTEND;
            max_angle += ATTN_ANGLE_EXTEND;
        }
        // `angle -= shape_angle.y;` (:1417) - `shape_angle` is written at the END of the previous
        // `lookBack` (`:1510`), so this tick still reads last tick's value.
        let angle = angle.wrapping_sub(ctx.shape_angle);
        // `if (maxDist > dist && maxAngle > abs(angle)) {` (:1419). `abs` is on an `int`, so the
        // `i16` difference widens first and RULE 10's `unsigned_abs` applies to the widened value.
        if max_dist > dist && max_angle > i32::from(angle.unsigned_abs()) {
            // `mLookAtPos = dNpc_playerEyePos(l_npc_dat[mNpcNo].mPlayerEyeOfsY);` (:1420)
            self.look_at_pos = player_eye(player, DAT0_PLAYER_EYE_OFS_Y);
            // `mLookMode = LOOK_MODE_ATTN;` (:1421)
            self.look_mode = LOOK_ATTN;
            // `if (mbAllowBodyTurn) mHeadOnlyFollow = false; else mHeadOnlyFollow = true;`
            // (:1422-1426)
            self.head_only_follow = !self.allow_body_turn;
            // `if (mbLookOnly == 0) { mTargetYRot = mHomeYRot; mHeadOnlyFollow = false;
            // mLookMode = LOOK_MODE_TURN; m_jnt.setTrn(); }` (:1428-1433)
            if !self.look_only {
                self.turn_to_home();
            }
            // `if (mbPlayerAttention == 0) mbPlayerAttention = 1;` (:1434-1436)
            self.player_attention = true;
        } else {
            // `if (mbPlayerAttention == 1) { mbPlayerAttention = 0; mLookResetTimer =
            // l_npc_dat[mNpcNo].mLookResetTimer; }` (:1438-1441)
            if self.player_attention {
                self.player_attention = false;
                self.look_reset_timer = DAT0_LOOK_RESET_TIMER;
            }
            // `if (l_npc_dat[mNpcNo].mNearDist > dist) {` (:1442) - `mNearDist` is `0.0f` in both
            // `l_npc_dat` rows, and a distance is never negative, so this arm is dead in retail
            // too and `mbNearPlayer` is never set.
            if DAT0_NEAR_DIST > dist {
                self.look_at_pos = player_eye(player, DAT0_PLAYER_EYE_OFS_Y);
                self.look_mode = LOOK_ATTN;
                self.head_only_follow = !self.allow_body_turn;
                if !self.look_only {
                    self.turn_to_home();
                }
                self.near_player = true;
            } else {
                // `mLookMode = LOOK_MODE_NONE; if (!mPathRun.isPath()) { if (mLookResetTimer != 0)
                // mLookResetTimer--; else { mTargetYRot = mHomeYRot; mHeadOnlyFollow = false;
                // mLookMode = LOOK_MODE_TURN; m_jnt.setTrn(); } }` (:1459-1469) - with no PATH the
                // `!mPathRun.isPath()` test is always true.
                self.look_mode = LOOK_NONE;
                if self.look_reset_timer != 0 {
                    self.look_reset_timer -= 1;
                } else {
                    self.turn_to_home();
                }
            }
        }
        // `mLookAtMaxVel = l_npc_dat[mNpcNo].mLookAtMaxVel;` (:1473)
        self.look_at_max_vel = DAT0_LOOK_AT_MAX_VEL;
    }

    /// `mTargetYRot = mHomeYRot; mHeadOnlyFollow = false; mLookMode = LOOK_MODE_TURN;
    /// m_jnt.setTrn();` (`:1429-1432`, `:1452-1455`, `:1464-1467`) - the block `chkAttention`
    /// repeats three times.
    fn turn_to_home(&mut self) {
        self.turn_to(self.home_y_rot);
    }

    /// `mTargetYRot = <angle>; mHeadOnlyFollow = false; mLookMode = LOOK_MODE_TURN;
    /// m_jnt.setTrn();` (`:1039-1042`, `:1101-1104`).
    fn turn_to(&mut self, angle: i16) {
        self.target_y_rot = angle;
        self.head_only_follow = false;
        self.look_mode = LOOK_TURN;
        self.jnt.set_trn();
    }

    /// `void daNpcMn_c::lookBack()` (`:1477-1511`).
    fn look_back(&mut self, actor: &mut Actor) {
        // `s16 maxVel = mLookAtMaxVel; s16 desiredYRot = current.angle.y;` (:1478-1479)
        let max_vel = self.look_at_max_vel;
        let mut desired_y_rot = actor.angle[1];
        // `cXyz* dstTemp = NULL; cXyz dstPos = eyePos; bool headOnlyFollow = mHeadOnlyFollow;`
        // (:1481-1484)
        let mut dst: Option<Vec3> = None;
        let dst_pos = self.eye_pos;
        let mut head_only_follow = self.head_only_follow;
        match self.look_mode {
            // `case LOOK_MODE_ATTN: temp2 = mLookAtPos; dstTemp = &temp2; break;` (:1486-1489)
            LOOK_ATTN => dst = Some(self.look_at_pos),
            // `case LOOK_MODE_TURN: desiredYRot = mTargetYRot; break;` (:1490-1492)
            LOOK_TURN => desired_y_rot = self.target_y_rot,
            _ => {}
        }
        // `if (mTalkOrder && mbAllowBodyTurn) { headOnlyFollow = false; m_jnt.setTrn(); }`
        // (:1497-1500)
        if self.talk_order != 0 && self.allow_body_turn {
            head_only_follow = false;
            self.jnt.set_trn();
        }
        // `if (m_jnt.trnChk()) { if (mEventCut.getTurnSpeed() != 0) maxVel =
        // mEventCut.getTurnSpeed(); cLib_addCalcAngleS2(&mTurnVel, maxVel, 4, 0x800); } else
        // { mTurnVel = 0; }` (:1501-1508). `getTurnSpeed` has no counterpart, so `maxVel` stands.
        if self.jnt.trn_chk() {
            add_calc_angle_s2(
                &mut self.turn_vel,
                max_vel,
                TURN_VEL_SCALE,
                TURN_VEL_MAX_STEP,
            );
        } else {
            self.turn_vel = 0;
        }
        let vel = self.turn_vel;
        // `m_jnt.lookAtTarget(&current.angle.y, dstTemp, dstPos, desiredYRot, mTurnVel,
        // headOnlyFollow);` (:1509) - RULE 9b: the rate is hoisted so `self` is not borrowed
        // mutably while `actor` is borrowed.
        self.jnt.look_at_target(
            &mut actor.angle[1],
            dst,
            dst_pos,
            desired_y_rot,
            vel,
            head_only_follow,
        );
        // `shape_angle = current.angle;` (:1510)
        actor.shape_angle = actor.angle;
    }

    /// `BOOL daNpcMn_c::chkEndEvent()` (`:1609-1616`).
    fn chk_end_event(&mut self, ctx: &mut Ctx) -> bool {
        // `if (dComIfGp_evmng_endCheck(mHatchEventIdx)) { dComIfGp_event_reset();
        // fopAcM_delete(this); return TRUE; } return FALSE;` (:1610-1615)
        if ctx.hatch.is_some_and(|index| ctx.active.end_check(index)) {
            ctx.text.reset_event();
            self.deleted = true;
            return true;
        }
        false
    }
}

/// The message-box accessor set `talk2` / `talk3` drive, over the real `MsgProcs` /
/// `MsgGameInfo` / `Msg1Box` / `Msg2Box` resources.
struct Text<'a> {
    procs: &'a mut MsgProcs,
    info: &'a mut MsgGameInfo,
    clear_count: u8,
    one: &'a mut Msg1Box,
    two: &'a mut Msg2Box,
    archive: Option<&'a dyn MsgArchive>,
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

    /// `mpCurrMsg->mStatus` (`:1200`).
    fn status(&self) -> Option<u8> {
        match self.proc_name() {
            Some(FPC_NM_MSG) => Some(self.one.status),
            Some(FPC_NM_MSG2) => Some(self.two.status),
            _ => None,
        }
    }

    /// `mpCurrMsg->mStatus = ...` (`:1203`, `:1214`).
    fn set_status(&mut self, status: u8) {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.status = status,
            Some(FPC_NM_MSG2) => self.two.status = status,
            _ => {}
        }
    }

    /// `mpCurrMsg = fopMsgM_SearchByID(i_pid)` (`:1221`, `:1258`).
    fn is_bound(&self, pid: u32) -> bool {
        self.procs.search_by_id(pid).is_some()
    }

    /// `dComIfGp_getScopeMesgStatus()` (`:1237`, `:1241`).
    fn scope_status(&self) -> u8 {
        self.info.scope_mesg_status
    }

    /// `dComIfGp_setScopeMesgStatus(status)` (`:1240`, `:1251`).
    fn set_scope_status(&mut self, status: u8) {
        self.info.scope_mesg_status = status;
    }

    /// `dComIfGp_getMesgAnimeAttrInfo()` (`:1344`).
    fn mesg_anime(&self) -> u8 {
        self.info.mesg_anime
    }

    /// `dComIfGp_setMesgAnimeAttrInfo(0xFF)` (`:1358`).
    fn set_mesg_anime(&mut self, id: u8) {
        self.info.set_mesg_anime_attr_info(id);
    }

    /// `fopMsgM_messageSet(i_msgNo)` (`:1205`, `:1242`).
    fn message_set(&mut self, msg_no: u32) -> u32 {
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

    /// `fopMsgM_messageSet(mCurrMsgNo, this)` (`:1196`) - the actor-attached form, which passes
    /// the NPC's `eyePos` as the look-at point.
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

    /// `fopMsgM_scopeMessageSet(mCurrMsgNo)` (`:1233`).
    fn scope_message_set(&mut self, msg_no: u32) -> u32 {
        fop_msgm_scope_message_set(self.procs, self.info, msg_no)
    }

    /// `dComIfGp_event_reset()` (`:703`, `:732`, `:1611`).
    fn reset_event(&mut self) {
        self.info.event_mode = DA_EVT_MODE_NONE;
    }
}

/// Everything `_execute` needs beyond `self`, gathered once per tick.
struct Ctx<'a> {
    stories: &'a mut StoryBits,
    switches: &'a mut Switches,
    list: &'a EventList,
    active: &'a mut ActiveEvent,
    flags: &'a mut EventFlags,
    /// `mHatchEventIdx` (`:442`) as an `EventList` index.
    hatch: Option<usize>,
    /// `fopAcM_searchFromName("Ohatch", 0, 0)`'s result (`:1029`, `:1090`).
    hatch_pos: Option<Vec3>,
    /// `Collision` for `mObjAcch.ChkGroundLanding()` (`:1181`).
    collision: Option<&'a Collision>,
    /// `current.pos`, snapshotted before the dispatch: the only function that moves it is
    /// `eventJump` (`:1179`), and nothing in the same tick reads it afterwards.
    pos: Vec3,
    /// `shape_angle.y` as `lookBack` last wrote it (`:1510`).
    shape_angle: i16,
    /// `dComIfGp_getLinkPlayer()` (`:1406`).
    player: Option<Player>,
    /// `dComIfGp_checkPlayerStatus0(0, daPyStts0_TELESCOPE_LOOK_e)` (`:645`, `:651`).
    telescope_look: bool,
    /// `eventInfo.checkCommandTalk()` (`:573`, `:831`).
    talk_command: bool,
    /// `dComIfGp_event_chkTalkXY()` (`:1288`), fed `false` - the inline has no body in the
    /// decomp and `npc_ah.rs` already feeds it `false`.
    talk_xy: bool,
    /// `eventInfo.checkCommandDemoAccrpt()` (`:573`, `:720`, `:827`).
    demo_accept: bool,
    /// `!dComIfGp_event_runCheck()` (`:573`).
    event_run: bool,
    /// `dComIfGp_evmng_getMyStaffId` / `getIsAddvance` folded together (`:869`, `:875`).
    cut: Option<(usize, bool)>,
    text: &'a mut Text<'a>,
}

impl Ctx<'_> {
    /// `dComIfGp_evmng_getMyIntegerP(i_staffIdx, key)` (`:951`, `:989`, `:1010`, `:1051`,
    /// `:1080`) - the cut record's own integer, `None` for an absent key.
    fn cut_number(&self, key: &str) -> Option<i32> {
        self.cut.and_then(|(cut, _)| self.list.cut_number(cut, key))
    }

    /// `dComIfGp_evmng_getMyIntegerP(i_staffIdx, key) != NULL` (`:1002`).
    fn cut_has(&self, key: &str) -> bool {
        self.cut_number(key).is_some()
    }
}

#[derive(SystemParam)]
pub struct World<'w, 's> {
    pub stories: ResMut<'w, StoryBits>,
    pub switches: ResMut<'w, Switches>,
    pub flags: ResMut<'w, EventFlags>,
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub stage: Res<'w, StageName>,
    /// `dComIfGp_getNextStageName()`'s store: `dStage_nextStage_c`'s name field, which
    /// `dComIfGp_setNextStage` writes and the mount only disables (`d_stage.cpp:27-33`).
    pub next: Res<'w, NextStage>,
    pub do_intent: Res<'w, DoIntent>,
    pub text_flags: Res<'w, TextFlags>,
    pub archive: Option<Res<'w, MsgArchives>>,
    /// `Without<Marlon>` proves disjointness against `Msg::actors`'s `&mut Actor`.
    pub hatches: Query<'w, 's, &'static Actor, Without<Marlon>>,
    pub players: Query<'w, 's, &'static Player>,
}

#[derive(SystemParam)]
pub struct Msg<'w, 's> {
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub actors: Query<'w, 's, (Entity, &'static mut Actor, &'static mut Marlon)>,
}

pub fn is_marlon(name: &str) -> bool {
    name == MARLON_ACTOR
}

pub fn control(
    mut world: World<'_, '_>,
    mut msg: Msg<'_, '_>,
    mut commands: Commands,
    collision: Option<Res<Collision>>,
) {
    let player = world.players.iter().next().copied();
    let clear_count = world.text_flags.clear_count;
    let event_run = msg.info.event_run_check();
    let telescope_look = msg.info.check_player_status0(DA_PY_STTS0_TELESCOPE_LOOK);
    let archive = world.archive.as_deref().map(|held| held as &dyn MsgArchive);
    // `fopAcM_searchFromName("Ohatch", 0, 0)` (`:1029`, `:1090`), resolved through
    // `ohatch::OHATCH_ACTORS` because `d_stage.cpp`'s OBJNAME table is the stage-string authority.
    let hatch_pos = world
        .hatches
        .iter()
        .find(|actor| OHATCH_ACTORS.contains(&actor.name.as_str()))
        .map(|actor| actor.pos);
    let hatch = world.list.event_index(EVENT_HATCH_OPEN);
    let stage = world
        .next
        .target
        .as_ref()
        .map_or(world.stage.0.as_str(), |target| target.stage.as_str());
    let ground = collision.as_deref();
    let do_now = world.do_intent.0;
    let stories = &mut *world.stories;
    let switches = &mut *world.switches;
    let flags = &mut *world.flags;
    let list = &*world.list;
    let active = &mut *world.active;
    for (entity, mut actor, mut marlon) in &mut msg.actors {
        if !marlon.entered {
            marlon.create_init(&mut actor, Some(stage));
            // `phase_1`'s `cPhs_STOP_e` cancels the process during Create
            // (`f_pc_create_req.cpp:106-108`): spawn it, then despawn on the FIRST TICK (RULE 6b).
            if marlon.phase_1(&*switches, &mut *stories) {
                let _ = commands.entity(entity).despawn();
                continue;
            }
            marlon.entered = true;
        }
        let talk_command = do_now && !marlon.prev_do;
        marlon.prev_do = do_now;
        let pos = actor.pos;
        let shape_angle = actor.shape_angle[1];
        let cut = active.staff_cut(list, STAFF_MN);
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            clear_count,
            one: &mut msg.one,
            two: &mut msg.two,
            archive,
        };
        let mut ctx = Ctx {
            stories: &mut *stories,
            switches: &mut *switches,
            list,
            active: &mut *active,
            flags: &mut *flags,
            hatch,
            hatch_pos,
            collision: ground,
            pos,
            shape_angle,
            player,
            telescope_look,
            talk_command,
            talk_xy: false,
            demo_accept: event_run,
            event_run,
            cut,
            text: &mut text,
        };
        marlon.execute(&mut actor, &mut ctx);
        if marlon.deleted {
            let _ = commands.entity(entity).despawn();
        }
    }
}
