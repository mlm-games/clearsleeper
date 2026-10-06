//! `daNpc_kam_c` — the Hyoi Seagull.
//!
//! Transcribed from `tww/src/d/actor/d_a_npc_kamome.cpp` (`Nonmatching == 0`, so every
//! mode value, timer, table and branch is real) plus its own header
//! `tww/include/d/actor/d_a_npc_kamome.h`. Retail NTSC-U, so the
//! `VERSION > VERSION_JPN` and `> VERSION_DEMO` blocks are live and the two
//! `VERSION_DEMO`-only blocks are not.
//!
//! ## What is NOT here, and why (every omission is also marked at its call site)
//!
//! * `create()`'s single-instance latch `if (l_act != NULL && l_act != this) return
//!   cPhs_ERROR_e;` (:316-318) and `~daNpc_kam_c`'s `l_act` reset (:204-206) live in
//!   file-scope statics. `IsDelete` (:1470-1473) is the bare-`TRUE` "accepts a delete
//!   request" form, so this actor never self-destructs and `spawn` returns `Self` (RULE 6b).
//! * BCK/BAS archive contents: `setAnm`'s `l_anmTbl` (:1186-1190), the
//!   `J3DAnmTransform::getFrameMax()` end frames `mDoExt_McaMorf::setAnm` installs
//!   (`m_Do_ext.cpp:1421`), the `mMorf = 8.0f` blend (:1195, :1202, :1209) and
//!   `l_anmTbl_c::mSoundFileIdx`.
//! * `dComIfGp_getSelectItem(i_itemBtn)` (:125) — the X/Y/Z button→item map
//!   (`d_com_inf_game.h:2882`) — and the `eventInfo` give-item channel that would call
//!   `XyCheckCB`/`XyEventCB` (:488-489). `xy_check_cb` and `call_demo_start_check` are
//!   transcribed in the same shape aryll.rs and grammarian.rs ship them, but nothing in
//!   this crate can fire them, so `mDescendStartPos`/`mDescendStartAngle` stay zero.
//! * `dBgS_ObjAcch::ChkRoofHit` (:638), `dBgS_AcchCir::ChkWallHit` (:651-653), the
//!   `mStts` character-collision move `fopAcM_posMove(this, mStts.GetCCMoveP())` (:1357)
//!   and the `cCcD` registry itself — RULE 3b "per-poly-group ground queries" and
//!   "`cCcD` actor-collision registry". `l_sph_src`/`l_tg_sph_src`/`l_kam_at_cps_src`
//!   (:350-436) register into that registry, so no `HitPoints`/`Hurt` is attached.
//! * `attention_info.distances[TALK] = 38` / `[SPEAK] = 38` (:481-482) — `dAttention_c`.
//! * `getHeadTopPos()` (:239, :1103) is a joint-matrix query; see `set_attention`.
//! * Particle emitters (:1375-1384), `GetMtrlSndId` (:1306), `dComIfGp_getReverb`
//!   (:1087, :1309), the zel-audio flags (:520, :958, :1089), the per-actor status bits
//!   (:1330, :1402, :540-542), the shadow (:1449-1452) and `dSnap_RegistFig` (:1454).
//! * `fopAcM_SetRoomNo(this, fopAcM_GetHomeRoomNo(this))` (:1068),
//!   `dComIfGs_setBaitItemEmpty` (:1071), `fopAcM_orderSpeakEvent` (:910),
//!   `dComIfGp_event_setTalkPartner` (:989), `dComIfGp_event_reset` (:1034) and
//!   `CPad_CHECK_TRIG_START` (:556) have no channel in this sim.

use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Vec2, Vec3};

use super::actor::Actor;
use super::area::StageName;
use super::collision::Collision;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, OrderFlags};
use super::intent::{CameraYaw, DoIntent, MoveIntent, TargetIntent};
use super::player::Player;
use super::sea::Sea;
use super::stream;
use super::vibration::Vibration;
use super::water;
use super::walk;

// OBJNAME("HyoiKam", fpcNm_NPC_KAM_e, 255, 0) - tww/src/d/d_stage.cpp:1170. (The
// "Kamome" name at d_stage.cpp:646 is fpcNm_KAMOME_e, the unrelated d_a_kamome.cpp.)
pub const KAMOME_ACTORS: [&str; 1] = ["HyoiKam"];

pub fn is_kamome(name: &str) -> bool {
    KAMOME_ACTORS.contains(&name)
}

/// `static const char* l_staff_name = "HyoiKam";` :70
pub const STAFF_NAME: &str = "HyoiKam";

/// `event_name_tbl` :438-442
const EVENT_OPTION_CHAR_END: &str = "OPTION_CHAR_END"; // <- :439
const EVENT_KAMOME_CALL: &str = "kamome_call"; // <- :440
const EVENT_KAMOME_OUT_TURN: &str = "kamome_out_turn"; // <- :441
const EVENT_NAMES: [&str; 3] = [
    EVENT_OPTION_CHAR_END,
    EVENT_KAMOME_CALL,
    EVENT_KAMOME_OUT_TURN,
];

/// `cut_name_tbl` :976-981 — the argument of `dComIfGp_evmng_getMyActIdx` :1006
const CUT_NAMES: [&str; 4] = ["wait", "change", "descend", "out_turn"]; // <- :977-980
const CUT_WAIT: i32 = 0; // <- event_init_tbl[0] &initialWaitEvent, :963
const CUT_CHANGE: i32 = 1; // <- event_init_tbl[1] &initialChangeEvent, :964
const CUT_DESCEND: i32 = 2; // <- event_init_tbl[2] &initialDescendEvent, :965
const CUT_OUT_TURN: i32 = 3; // <- event_init_tbl[3] &initialAreaOutTurn, :966
const CUT_NONE: i32 = -1; // <- `if (actIdx == -1)`, :1007

/// `dItemButton_e` - tww/include/d/d_com_inf_game.h:1100-1105; `dItemNo_HYOI_PEAR_e` -
/// tww/include/d/d_item_data.h:138. `XyCheckCB` :125 indexes the button array with the
/// button the player pressed.
pub const ITEM_BUTTON_COUNT: usize = 3; // <- dItemBtn_COUNT_e = 3, d_com_inf_game.h:1105
pub const ITEM_HYOI_PEAR: u8 = 0x83; // <- dItemNo_HYOI_PEAR_e, d_item_data.h:138

/// `mEventState` s8, written and compared in `waitNpcAction`, `waitPlayerAction`,
/// `eventOrder`, `checkOrder`, `checkCommandTalk` and `execute`.
const STATE_NONE: i8 = -1; // <- mEventState = -1, :448, :449, :697, :913, :920, :936, :993, :998
const EVENT_STATE_OPTION_END: i8 = 0; // <- mEventState = 0, :987, :1405
const EVENT_STATE_CALL: i8 = 2; // <- mEventState = 2, :829
const EVENT_STATE_SPEAK: i8 = 4; // <- mEventState == 4, :906, :933
const EVENT_STATE_SPEAK_NPC: i8 = 5; // <- mEventState == 5, :906, :909, :933
const EVENT_STATE_TALK_ITEM: i8 = 6; // <- mEventState = 6, :694, :915, :933, :943

/// `enum ActionStatus` - tww/include/d/actor/d_a_npc_kamome.h:13-19
const ACTION_STARTING: i8 = 0; // <- h:14
const ACTION_ONGOING_1: i8 = 1; // <- h:15
const ACTION_ONGOING_2: i8 = 2; // <- h:16
const ACTION_ONGOING_3: i8 = 3; // <- h:17
const ACTION_ENDING: i8 = -1; // <- h:18

/// `enum Animation` - d_a_npc_kamome.h:21-25
const ANM_WAIT1: i32 = 0; // <- h:22
const ANM_WAIT2: i32 = 1; // <- h:23
const ANM_SING: i32 = 2; // <- h:24
const ANM_COUNT: usize = 3; // <- l_anmPrm entry count, :1191-1213

/// `mAnmTblIdx`'s value before any `setAnm` has run: `l_anmTbl_c::mSoundFileIdx` is -1
/// for ANM_SING (:1189). Nothing else in `setAnm` reads the table.
const ANM_TBL_NONE: i8 = -1;

/// `l_anmPrm` :1191-1213 — mAnmTblIdx, mLoopMode, mPlaySpeed.
const ANM_TBL_IDX: [i8; ANM_COUNT] = [0, 1, 2]; // <- :1193, :1200, :1207
const ANM_LOOP: [u8; ANM_COUNT] = [
    J3D_MODE_LOOP, // <- J3DFrameCtrl::EMode_LOOP, :1194
    J3D_MODE_LOOP, // <- :1201
    J3D_MODE_NONE, // <- J3DFrameCtrl::EMode_NONE, :1208
];
const ANM_PLAY_SPEED: [f32; ANM_COUNT] = [1.0, 1.0, 1.0]; // <- :1196, :1203, :1210

/// `J3DFrameCtrl::Attribute_e` / `State_e` -
/// tww/include/JSystem/J3DGraphAnimator/J3DAnimation.h:830-842
const J3D_MODE_NONE: u8 = 0; // <- EMode_NONE, J3DAnimation.h:832
const J3D_MODE_LOOP: u8 = 2; // <- EMode_LOOP, J3DAnimation.h:834
const J3D_STATE_STOP: u8 = 1; // <- STATE_STOP_E, J3DAnimation.h:840
const J3D_STATE_LOOP: u8 = 2; // <- STATE_LOOP_E, J3DAnimation.h:841

/// `daNpc_kam_HIO1_c::daNpc_kam_HIO1_c` :85-101. `mUnused08 = 0.1f` (:87) has no reader.
/// `daNpc_kam_HIO_c::hio_prm_c` (:47-56, initialised :106-114) comes from the HIO archive
/// and this actor never reads it.
const HIO_SPEED_F: f32 = 17.0; // <- mSpeedF, :86
const HIO_ACCEL_F: f32 = 0.7; // <- mAccelF, :91
const HIO_FLAPPING_SPEED_F: f32 = 23.0; // <- mFlappingSpeedF, :92
const HIO_GLIDING_ANG_VEL_Y: i16 = 0x2000; // <- mGlidingAngVelY, :88
const HIO_GLIDING_ANG_VEL_X: i16 = 0x0AAA; // <- mGlidingAngVelX, :89
const HIO_MAX_ANGLE_Z: i16 = 0x2000; // <- mMaxAngleZ, :90
const HIO_FLAPPING_ANG_VEL_Y: i16 = 0x2710; // <- mFlappingAngVelY, :93
const HIO_FLAPPING_ANG_VEL_X: i16 = 0x1555; // <- mFlappingAngVelX, :94
const HIO_ANG_VEL_STEP_SCALE: i16 = 5; // <- mAngVelStepScale, :95
const HIO_ANG_VEL_MAX_STEP: i16 = 0x1000; // <- mAngVelMaxStep, :96
const HIO_ANG_VEL_MIN_STEP: i16 = 0x0400; // <- mAngVelMinStep, :97
const HIO_FLAP_DURATION: i16 = 30; // <- mFlapDuration, :98
const HIO_FLAP_EXHAUSTED_DURATION: i16 = 150; // <- mFlapExhaustedDuration, :99
const HIO_FLAP_ENERGY_DURATION: i16 = 150; // <- mFlapEnergyDuration, :100

/// File-scope `cXyz` literals :78-82
const LINE_BG_LOCAL_END: [f32; 3] = [0.0, 0.0, 500.0]; // <- l_line_bg_local_end, :81
const CALL_LOCAL_KYORI: [f32; 3] = [0.0, 0.0, 500.0]; // <- l_call_local_kyori, :82

/// `setBaseMtx`'s `static cXyz l_offset(0.0f, 30.0f, 0.0f)` :251
const MODEL_BASE_Y: f32 = 30.0;
/// `eyePos.set(headTopPos.x, headTopPos.y - 20.0f, headTopPos.z)` :240
const EYE_DROP_Y: f32 = 20.0;

/// `changeAreaCheck` :566-589
const FOREST_HAVEN_CENTER: [f32; 3] = [211555.0, 1000.0, 191870.0]; // <- l_center, :572
const FOREST_HAVEN_RANGE: f32 = 1300.0; // <- delta.absXZ() < 1300.0f, :575
const SEA_ROOM_HAVEN: i8 = 41; // <- fopAcM_GetRoomNo(this) == 41, :568
const SEA_STAGE: &str = "sea"; // <- strcmp(dComIfGp_getStartStageName(), "sea"), :568

/// `callDemoStartCheck` :133-188
const DESCEND_RAYS: usize = 0x10; // <- for (int i = 0; i < 0x10; i++), :156
const DESCEND_STEP_Y: i16 = 0x2000; // <- angleY = angleY + 0x2000, :182
const DESCEND_ANGLE_X: i16 = -0x1555; // <- s16 angleX = -0x1555, :150
const DESCEND_YAW: i16 = -0x8000; // <- shape_angle.y + 0x8000, :149 (s16 form of 0x8000)
const HEAD_LIFT: f32 = 20.0; // <- hyoi_pear_pos.y += 20.0f, :143 and :1104

/// `eventOrder` :902-926. `dEvtCnd_CANTALKITEM_e` is 0x0020, not 0x0002.
const EVT_CND_CANTALK: u16 = 0x0001; // <- f_op_actor.h:145, used :908, :918
const EVT_CND_CANTALKITEM: u16 = 0x0020; // <- f_op_actor.h:150, used :917

/// `waitNpcAction`'s random tables :701-705. The last entry of each is never selected
/// (`cLib_getRndValue(0, ARRAY_SSIZE(tbl)-2)`, :716, :721).
const TURN_ANGLE_Y: [i16; 6] = [
    0x1000, -0x0C00, -0x1000, 0x0800, 0x0C00, -0x0800,
]; // <- l_turn_angleY, :701 (0xF400 / 0xF000 / 0xF800 in s16 form)
const ANM_TYPE: [i32; 16] = [
    ANM_WAIT2, ANM_WAIT2, ANM_WAIT2, ANM_WAIT2, ANM_WAIT2, ANM_WAIT2, ANM_WAIT2, ANM_WAIT2,
    ANM_WAIT2, ANM_WAIT2, ANM_WAIT1, ANM_WAIT1, ANM_SING, ANM_WAIT1, ANM_SING, ANM_WAIT1,
]; // <- l_anm_type, :702-705
const TURN_LAST_PICK: i32 = 4; // <- cLib_getRndValue(0, ARRAY_SSIZE(l_turn_angleY)-2), :721
const ANM_LAST_PICK: i32 = 14; // <- cLib_getRndValue(0, ARRAY_SSIZE(l_anm_type)-2), :716

/// `getStickAngY` :599-629 and `npcTurnCheck` :661-674. `deltaAngleY = -/+0x4000` with
/// `targetAngleZ = -/+mMaxAngleZ` (:611-621) belongs to the omitted acch-circle arm.
const NPC_TURN_STEP: i16 = 0x2000; // <- current.angle.y -/+ 0x2000, :664, :667, :670
const LOCK_ANGLE_X_TIMER: i16 = 30; // <- mLockAngleXTimer = 30, :637, :641

/// `cLib_addCalcAngleS` arguments, one per call site.
const TURN_GATE: i32 = 0x200; // <- abs(angleDiffY) > 0x200, :753
const TURN_SCALE_NPC_Y: i16 = 30; // <- waitNpcAction :754
const TURN_MAX_NPC_Y: i16 = 0x2000; // <- waitNpcAction :754
const TURN_MIN_NPC_Y: i16 = 0x100; // <- waitNpcAction :754
const TURN_SCALE: i16 = 8; // <- :763, :764, :835, :843, :844, :1164, :1165, :1417, :1418
const TURN_MAX: i16 = 0x2000; // <- the 0x2000 argument in those same nine calls
const TURN_MIN: i16 = 0x400; // <- the 0x400 argument in those same nine calls
const TURN_SCALE_DESCEND: i16 = 30; // <- actionDescendEvent :1112
const TURN_SCALE_OUT_TURN: i16 = 16; // <- actionAreaOutTurn :1145

/// Event-cut timers
const DESCEND_WAIT: i16 = 30 * 30; // <- mWaitTimer = 30*30, :1094
const DESCEND_CATCH_Y: f32 = 50.0; // <- linkHeadTopPos.y + 50.0f, :1111
const DESCEND_CATCH_RANGE: f32 = 100.0; // <- SQUARE(100.0f), :1117
const AREA_OUT_WAIT: i16 = 10; // <- mWaitTimer = 10, :1130
const AREA_OUT_TIMER: i16 = 180 * 30; // <- mAreaOutTimer = 180*30, :1131

/// `mpMorf->checkFrame(8.0f)` :744, :862 — the same 8.0f as `l_anmPrm::mMorf` :1195
const MORF_CV_FRAME: f32 = 8.0;

/// `damagePlayerAction` :878-899
const SHOCK_PATTERN: i32 = 5; // <- StartShock(5, ...), :880
const SHOCK_FLAGS: i32 = -0x21; // <- StartShock(5, -0x21, ...), :880
const SHOCK_COORD: [f32; 3] = [0.0, 1.0, 0.0]; // <- cXyz(0.0f, 1.0f, 0.0f), :880
const DAMAGE_FOG: u8 = 150; // <- setDamageFogTimer(150), :881
const DAMAGE_RETURN: i16 = 30; // <- mC08 = 30, :887
const DAMAGE_SPIN_Y: i16 = 0x2000; // <- shape_angle.y += 0x2000, :891

/// `l_tg_sph_src` :379-406 — `SrcObjTg SPrm = cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsPlayer_e`
/// (:387) and `Radius 30.0f` (:404).
const TG_SPH_RADIUS: f32 = 30.0;

/// `fopAc_CULLSPHERE_2_e` (:1508) indexes `l_cullSizeSphere` =
/// `fopAc_cullSizeSphere(cXyz(0.0f, 0.0f, 0.0f), 100.0f)` -
/// tww/src/f_op/f_op_actor_mng.cpp:576. `cull()` hands that sphere back as its box.
const CULL_MIN: [f32; 3] = [-100.0, -100.0, -100.0];
const CULL_MAX: [f32; 3] = [100.0, 100.0, 100.0];

/// JA_SE ids — tww/include/JAZelAudio/JAZelAudio_SE.h
const SE_CV_KAMOME: u32 = 0x4805; // <- JA_SE_CV_KAMOME, :685; played :745, :863, :886
const SE_CTRL_NPC_TO_LINK: u32 = 0x0886; // <- JA_SE_CTRL_NPC_TO_LINK, :143; played :990
const SE_HYOI_USE_DEMO: u32 = 0x08EF; // <- JA_SE_HYOI_USE_DEMO, :244; played :1087

/// `fopAc_Attn_ACTION_SPEAK_e | fopAc_Attn_TALKFLAG_NOTALK_e` :693, :696
const ATTN_ACTION_SPEAK: u32 = 0x0000_0008; // <- f_op_actor.h:77
const ATTN_TALKFLAG_NOTALK: u32 = 0x0200_0000; // <- f_op_actor.h:83
const ATTN_SPEAK_NOTALK: u32 = ATTN_ACTION_SPEAK | ATTN_TALKFLAG_NOTALK;

/// `mHitFlags` accessors — d_a_npc_kamome.h:31-43
const HIT_FRONT: u32 = 0x01; // <- onFrontLineHit, h:32
const HIT_LEFT: u32 = 0x02; // <- onLeftLineHit, h:34
const HIT_RIGHT: u32 = 0x04; // <- onRightLineHit, h:36
const HIT_WATER: u32 = 0x08; // <- onWaterHit, h:38
const HIT_NO_BG: u32 = 0x10; // <- onNoBgCheck, h:41
const HIT_LINE_ALL: u32 = 0x07; // <- offLineHit, h:31

/// `m4E4` bit masks — tww/include/d/actor/d_a_player_npc.h:19-26
const M4E4_RETURN_LINK: u32 = 0x01; // <- h:20
const M4E4_EVENT_ACCEPT: u32 = 0x02; // <- h:22

/// `cM_initRnd(100, 100, 100)` — tww/src/m_Do/m_Do_machine.cpp:617; `cM_rnd` —
/// tww/src/SSystem/SComponent/c_math.cpp:177-181
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA]; // <- c_math.cpp:178-180
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673]; // <- c_math.cpp:178-180
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0]; // <- c_math.cpp:181
const RND_SEED: i32 = 100; // <- m_Do_machine.cpp:617

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0]) as f32 / RND_SCALE[0]
        + (seed[1]) as f32 / RND_SCALE[1]
        + (seed[2]) as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

/// `cLib_calcTimer` — tww/include/SSystem/SComponent/c_lib.h:85-90
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

fn abs_xz(delta: Vec3) -> f32 {
    (delta.x * delta.x + delta.z * delta.z).sqrt()
}

fn abs2_xz(delta: Vec3) -> f32 {
    delta.x * delta.x + delta.z * delta.z
}

/// `mDoMtx_stack_c::multVec` :157-162, :1258-1260, :1276-1294, after the single-axis
/// `YrotS` / `XrotM` each call site leaves on the stack.
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

/// `mDoMtx_stack_c::ZXYrotS` :1254, :1275 — Y first, then X, then Z.
fn mult_vec_zxy(angle: [i16; 3], local: [f32; 3]) -> Vec3 {
    mult_vec([angle[2], angle[0], angle[1]], local)
}

/// `cLib_targetAngleY` :725, :1142 — tww/src/SSystem/SComponent/c_lib.cpp:343-345
fn bearing(from: Vec3, to: Vec3) -> i16 {
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

/// `dComIfG_Bgsp()->LineCross(&mLinChk)` :169, :171, :1279, :1288, :1297
fn line_crossed(collision: Option<&Collision>, from: Vec3, to: Vec3) -> bool {
    match collision {
        Some(collision) => collision.slide(from, to) != to,
        None => false,
    }
}

fn anm_type_at(pick: i16) -> i32 {
    usize::try_from(pick)
        .ok()
        .and_then(|index| ANM_TYPE.get(index).copied())
        .unwrap_or(ANM_WAIT2)
}

fn turn_angle_at(pick: i16) -> i16 {
    usize::try_from(pick)
        .ok()
        .and_then(|index| TURN_ANGLE_Y.get(index).copied())
        .unwrap_or(TURN_ANGLE_Y[0])
}

/// `J3DFrameCtrl::init` / `::update` / `::checkPass` —
/// tww/src/JSystem/J3DGraphAnimator/J3DAnimation.cpp:143-210 and :24-115 — plus
/// `mDoExt_baseAnm::isStop` (tww/include/m_Do/m_Do_ext.h:40-42). Only `EMode_NONE` and
/// `EMode_LOOP` are reachable from `l_anmPrm`, so the other three arms are dead.
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
    /// `J3DFrameCtrl::init(0)` — J3DAnimation.h:844.
    fn idle() -> Self {
        Self {
            attribute: J3D_MODE_LOOP,
            state: 0,
            start: 0.0,
            end: 0.0,
            loop_point: 0.0,
            rate: 1.0,
            frame: 0.0,
        }
    }

    /// `mDoExt_McaMorf::setAnm` :1421 — `mFrameCtrl.init(mpAnm->getFrameMax())`. The BCK
    /// archive is not shipped, so the end frame is unknown and callers pass 0.0; the
    /// archive-driven value is recorded in the module header.
    fn init(&mut self, end: f32) {
        self.attribute = J3D_MODE_LOOP;
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
            J3D_MODE_NONE => {
                if self.frame < self.start {
                    self.frame = self.start;
                    self.rate = 0.0;
                    self.state |= J3D_STATE_STOP;
                }
                if self.frame >= self.end {
                    self.frame = self.end - 0.001;
                    self.rate = 0.0;
                    self.state |= J3D_STATE_STOP;
                }
            }
            J3D_MODE_LOOP => {
                while self.frame < self.start {
                    self.state |= J3D_STATE_LOOP;
                    if self.loop_point - self.start <= 0.0 {
                        break;
                    }
                    self.frame += self.loop_point - self.start;
                }
                while self.frame >= self.end {
                    self.state |= J3D_STATE_LOOP;
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
            J3D_MODE_NONE => {
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
            J3D_MODE_LOOP => {
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

    /// `mDoExt_McaMorf::play` :1463-1473 returns `isStop()`.
    fn play(&mut self) -> bool {
        self.update();
        self.state & J3D_STATE_STOP != 0 || self.rate == 0.0
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
    /// `attention_info.flags` :693, :696 — written only; `dAttention_c` reads it.
    pub attn_flags: u32,
    /// `onNpcNotChange` / `offNpcNotChange` :576, :584, :588 — written only; `daPy_lk_c`
    /// reads it.
    pub npc_not_change: bool,
    /// `static bool daNpc_kam_c::m_hyoi_kamome` h:104 — written only; `d_camera.cpp:1287`
    /// and `d_meter.cpp:799` read it.
    pub hyoi_kamome: bool,
    /// `m4E4` :19-26 — the return-link and event-accept latches.
    pub m4e4: u32,
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
    pub demo_chk_cnt: i16,
    pub demo_chk_flag: i16,
    pub seed: [i32; 3],
    pub tg_hit: bool,
    pub ground_hit: bool,
    pub sea_in: bool,
}

impl Kamome {
    /// `create()` :313-348 fails only through the single-instance latch (:316-318) or a
    /// failed `fopAcM_entrySolidHeap` (:328-333); neither applies to a stage-placed spawn,
    /// and `IsDelete` :1470-1473 is the "accepts a delete request" form, so this actor
    /// never despawns on its own (RULE 6b). `g_profile_NPC_KAM::Parameters` is 0 (:1502),
    /// so `source.parameters` is unused (RULE 7).
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
            attn_flags: 0,
            npc_not_change: false,
            hyoi_kamome: false,
            m4e4: 0,
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
            demo_chk_cnt: 0,
            demo_chk_flag: 0,
            seed: [RND_SEED; 3],
            tg_hit: false,
            ground_hit: false,
            sea_in: false,
        }
    }

    /// `fopAc_CULLSPHERE_2_e` (:1508) -> `l_cullSizeSphere[2]`, i.e.
    /// `fopAc_cullSizeSphere(cXyz(0,0,0), 100.0f)` — f_op_actor_mng.cpp:576.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `draw()` :1437-1457 always reaches `mpMorf->entryDL()`.
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
        self.m4e4 |= M4E4_EVENT_ACCEPT;
    }

    fn off_event_accept(&mut self) {
        self.m4e4 &= !M4E4_EVENT_ACCEPT;
    }

    fn is_event_accept(&self) -> bool {
        self.m4e4 & M4E4_EVENT_ACCEPT != 0
    }

    fn return_link(&mut self) {
        self.m4e4 |= M4E4_RETURN_LINK;
    }

    fn off_return_link(&mut self) {
        self.m4e4 &= !M4E4_RETURN_LINK;
    }

    fn is_return_link(&self) -> bool {
        self.m4e4 & M4E4_RETURN_LINK != 0
    }

    fn on_npc_not_change(&mut self) {
        self.npc_not_change = true;
    }

    fn off_npc_not_change(&mut self) {
        self.npc_not_change = false;
    }

    /// `daNpc_kam_c::init` :445-492, minus the `dBgS_ObjAcch` / `dCcD_Stts` setup
    /// (:462-476), the attention distances (:481-482) and `setBaseMtx` (:478).
    pub fn begin(&mut self, actor: &mut Actor, list: &EventList) {
        self.entered = true;
        self.off_hyoi_kamome(); // <- :446
        self.event_state = STATE_NONE; // <- mEventState = -1, :448
        self.curr_event_idx_idx = STATE_NONE; // <- mCurrEventIdxIdx = -1, :449
        self.scale *= 1000.0; // <- scale *= 1000.0f, :450
        self.home_pos = actor.pos;
        self.max_y = self.home_pos.y + self.scale.y; // <- :451
        self.min_y = self.home_pos.y - self.scale.y; // <- :452

        self.target_speed_f = HIO_SPEED_F; // <- :454
        self.ang_vel_y = HIO_GLIDING_ANG_VEL_Y; // <- :455
        self.ang_vel_x = HIO_GLIDING_ANG_VEL_X; // <- :456
        self.target_ang_vel_y = HIO_GLIDING_ANG_VEL_Y; // <- :457
        self.target_ang_vel_x = HIO_GLIDING_ANG_VEL_X; // <- :458

        self.npc_action = NpcAct::Wait; // <- setNpcAction(&waitNpcAction, NULL), :460
        self.set_action_reset();

        self.attn_flags = 0; // <- attention_info.flags = 0, :480
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

    /// `XyCheckCB` :124-130.
    pub fn xy_check_cb(
        &mut self,
        item_btn: u8,
        select_item: [u8; ITEM_BUTTON_COUNT],
        link_pos: Vec3,
        link_shape_angle: [i16; 3],
        collision: Option<&Collision>,
    ) -> i16 {
        let selected = select_item.get(usize::from(item_btn)).copied();
        if selected == Some(ITEM_HYOI_PEAR) {
            self.call_demo_start_check(link_pos, link_shape_angle, collision)
        } else {
            0 // <- return FALSE, :129
        }
    }

    /// `XyEventCB` :196-200.
    pub fn xy_event_cb(&mut self) -> i16 {
        self.on_event_accept(); // <- :197
        self.curr_event_idx_idx = 1; // <- mCurrEventIdxIdx = 1 ("kamome_call"), :198
        self.event_index_at(1)
    }

    /// `callDemoStartCheck` :133-188. `hyoi_pear_pos` is
    /// `daPy_getPlayerLinkActorClass()->getHeadTopPos()` raised by 20.0f (:142-143); the
    /// head-top joint matrix is not available here, so `link_pos` stands in — the same
    /// substitution `action_descend_event` makes.
    pub fn call_demo_start_check(
        &mut self,
        link_pos: Vec3,
        link_shape_angle: [i16; 3],
        collision: Option<&Collision>,
    ) -> i16 {
        if self.demo_chk_cnt != 0 {
            return self.demo_chk_flag; // <- :134-136
        }
        self.demo_chk_cnt = self.demo_chk_cnt.wrapping_add(1); // <- :138
        self.demo_chk_flag = 0; // <- :139

        let hyoi_pear_pos = link_pos + Vec3::new(0.0, HEAD_LIFT, 0.0);
        let angle_x = DESCEND_ANGLE_X;
        let mut angle_y = link_shape_angle[1].wrapping_add(DESCEND_YAW); // <- :149

        for _ in 0..DESCEND_RAYS {
            let mut line_end_pos = Vec3::ZERO;
            let mut line_start_pos = Vec3::ZERO;
            let mut descend_start_pos = Vec3::ZERO;

            line_end_pos += mult_vec([0, angle_y.wrapping_add(DESCEND_YAW), 0], CALL_LOCAL_KYORI);
            line_start_pos += mult_vec([0, angle_y, 0], CALL_LOCAL_KYORI);
            descend_start_pos += mult_vec([angle_x, 0, 0], CALL_LOCAL_KYORI);

            line_end_pos += hyoi_pear_pos;
            line_start_pos += hyoi_pear_pos;
            descend_start_pos += line_start_pos;

            if !line_crossed(collision, descend_start_pos, line_start_pos)
                && !line_crossed(collision, line_start_pos, line_end_pos)
            {
                self.descend_start_pos = descend_start_pos; // <- :173
                self.descend_start_angle = [
                    angle_x.wrapping_neg(), // <- mDescendStartAngle.set(-angleX, angleY + 0x8000, 0), :175
                    angle_y.wrapping_add(DESCEND_YAW),
                    0,
                ];
                self.demo_chk_flag = 1; // <- :177
                return 1; // <- return TRUE, :178
            }
            angle_y = angle_y.wrapping_add(DESCEND_STEP_Y); // <- :182
        }
        0 // <- return FALSE, :187
    }

    /// `mEventIdxs[]` filled by `dComIfGp_evmng_getEventIdx` :485. `mCurrEventIdxIdx`
    /// is -1 until `XyEventCB` or `eventOrder` sets it, which is an out-of-range index in
    /// C; the crate returns the -1 sentinel instead of reading past the array.
    fn event_index_at(&self, slot: i8) -> i16 {
        if slot < 0 {
            return -1;
        }
        self.event_idxs
            .get(usize::try_from(slot).unwrap_or(0))
            .copied()
            .unwrap_or(-1)
    }

    /// `setAttention` :237-247
    fn set_attention(&mut self, on: bool, use_current: i32, actor: &Actor) {
        if !on {
            return;
        }
        // OMITTED: getHeadTopPos() :239 is the j_ka_head1 joint matrix written by
        // headNodeCallBack :261-274 (d_a_npc_kamome.h:267 l_offsetAttPos(0,20,0)). The
        // model base offset l_offset(0,30,0) :251 stands in for the head joint.
        let head_top = actor.pos + Vec3::new(0.0, MODEL_BASE_Y, 0.0);
        self.eye_pos = Vec3::new(head_top.x, head_top.y - EYE_DROP_Y, head_top.z);
        self.attn_pos = if use_current != 0 {
            actor.pos // <- :242
        } else {
            self.home_pos // <- :244
        };
    }

    /// `setAction` :495-511
    fn set_action_reset(&mut self) {
        self.action_status = ACTION_STARTING; // <- :502
        self.c08 = 0; // <- mC08 = 0, :503
        self.c0a = 0; // <- mC0A = 0, :504
        self.c0c = 0; // <- mC0C = 0, :505
    }

    /// `setNpcAction` :528-531
    fn set_npc_action(&mut self, func: NpcAct, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.player_action = PlayerAct::None; // <- mCurrPlayerActionFunc = NULL, :529
        if self.npc_action != func {
            if self.npc_action != NpcAct::None {
                self.action_status = ACTION_ENDING; // <- :498
                let old = self.npc_action;
                self.run_npc(old, actor, ctx); // <- (this->**pCurrActionFunc)(arg), :499
            }
            self.npc_action = func; // <- :501
            self.set_action_reset();
            self.run_npc(func, actor, ctx); // <- :508
        }
    }

    /// `setPlayerAction` :548-551
    fn set_player_action(&mut self, func: PlayerAct, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.npc_action = NpcAct::None; // <- mCurrNpcActionFunc = NULL, :549
        if self.player_action != func {
            if self.player_action != PlayerAct::None {
                self.action_status = ACTION_ENDING; // <- :498
                let old = self.player_action;
                self.run_player(old, actor, ctx); // <- :499
            }
            self.player_action = func; // <- :501
            self.set_action_reset();
            self.run_player(func, actor, ctx); // <- :508
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

    /// `npcAction` :514-525
    fn npc_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.npc_action == NpcAct::None {
            self.speed_f = 0.0; // <- :516
            self.off_hyoi_kamome(); // <- :517
            self.set_npc_action(NpcAct::Wait, actor, ctx); // <- :518
        }
        let func = self.npc_action;
        self.run_npc(func, actor, ctx); // <- :524
    }

    /// `playerAction` :534-545 (the three `dComIfGp_set*Status` calls :540-542 are
    /// per-actor status bits, RULE 3b).
    fn player_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.player_action == PlayerAct::None {
            self.speed_f = 0.0; // <- :536
            self.set_player_action(PlayerAct::Wait, actor, ctx); // <- :537
        }
        let func = self.player_action;
        self.run_player(func, actor, ctx); // <- :544
    }

    /// `returnLinkCheck` :554-561
    fn return_link_check(&self, run_check: bool, trig_r: bool) -> bool {
        // OMITTED: CPad_CHECK_TRIG_START(0) :556 — the crate has Do/Target/Item/Use
        // intents but no START resource, so that half of the disjunction cannot fire.
        !run_check && trig_r
    }

    /// `changeAreaCheck` :564-590
    fn change_area_check(&mut self, actor: &Actor, player_pos: Vec3, stage: &StageName) -> bool {
        let inside = if stage.0 == SEA_STAGE && actor.room == SEA_ROOM_HAVEN {
            let center = Vec3::from_array(FOREST_HAVEN_CENTER);
            if player_pos.y > center.y && abs_xz(player_pos - center) < FOREST_HAVEN_RANGE {
                true
            } else {
                false
            }
        } else {
            abs_xz(player_pos - self.home_pos) < self.scale.x // <- :583
        };
        if inside {
            self.on_npc_not_change(); // <- :576 / :584
            true
        } else {
            self.off_npc_not_change(); // <- :588
            false
        }
    }

    /// `areaOutCheck` :593-596
    fn area_out_check(&self, actor: &Actor) -> bool {
        abs_xz(actor.pos - self.home_pos) > self.scale.x
    }

    /// `getStickAngY` :599-629. The `else if (wallHitCheck() >= 0)` arm :609-623 needs
    /// `dBgS_AcchCir::ChkWallHit` (:651-653), a per-poly-group query (RULE 3b), so only
    /// the stick-driven arm is transcribed.
    fn get_stick_ang_y(&self, stick: Vec2, camera_angle_y: i16) -> (bool, i16, i16) {
        let mut delta_angle_y: i16 = 0;
        let mut target_angle_z: i16 = 0;
        let mut is_turning = false;
        if stick.x != 0.0 {
            delta_angle_y = (stick.x * -f32::from(self.ang_vel_y)) as i16; // <- :606
            target_angle_z = (stick.x * f32::from(HIO_MAX_ANGLE_Z)) as i16; // <- :607
            is_turning = true;
        }
        (
            is_turning,
            delta_angle_y.wrapping_add(camera_angle_y), // <- dCam_getControledAngleY, :625-626
            target_angle_z,
        )
    }

    /// `getAngleX` :632-646
    fn get_angle_x(&mut self, actor: &Actor, stick: Vec2) -> i16 {
        if actor.pos.y < self.min_y || self.ground_hit || self.is_water_hit() {
            // OMITTED: the `mAcch.ChkRoofHit()` term of the second test :638 — Collision
            // has no upward face query.
            self.lock_angle_x_timer = LOCK_ANGLE_X_TIMER; // <- :637
            self.ang_vel_x.wrapping_neg() // <- :636
        } else if actor.pos.y > self.max_y {
            self.lock_angle_x_timer = LOCK_ANGLE_X_TIMER; // <- :641
            self.ang_vel_x // <- :640
        } else {
            (stick.y * f32::from(self.ang_vel_x)) as i16 // <- :643
        }
    }

    /// `npcTurnCheck` :661-674
    fn npc_turn_check(&self, actor: &Actor, target_angle_y: &mut i16) -> bool {
        let mut is_turning = false;
        if self.is_left_line_hit() {
            *target_angle_y = actor.angle[1].wrapping_sub(NPC_TURN_STEP); // <- :664
            is_turning = true;
        } else if self.is_right_line_hit() {
            *target_angle_y = actor.angle[1].wrapping_add(NPC_TURN_STEP); // <- :667
            is_turning = true;
        } else if self.is_front_line_hit() {
            *target_angle_y = actor.angle[1].wrapping_sub(NPC_TURN_STEP); // <- :670
            is_turning = true;
        }
        is_turning
    }

    /// `waitNpcAction` :677-769
    fn wait_npc_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.action_status == ACTION_STARTING {
            self.action_status = ACTION_ONGOING_1; // <- mActionStatus++, :679
            self.set_anm(ANM_WAIT2); // <- :680

            self.speed_f = HIO_SPEED_F; // <- :682
            self.target_speed_f = HIO_SPEED_F; // <- :683
            self.ang_vel_y = HIO_GLIDING_ANG_VEL_Y; // <- :684
            self.ang_vel_x = HIO_GLIDING_ANG_VEL_X; // <- :685
            self.target_ang_vel_y = HIO_GLIDING_ANG_VEL_Y; // <- :686
            self.target_ang_vel_x = HIO_GLIDING_ANG_VEL_X; // <- :687

            self.c08 = self.get_rnd_value(90, 60); // <- :689
            self.c0c = self.get_rnd_value(10, 80); // <- :690
        } else if self.action_status != ACTION_ENDING {
            if self.change_area_check(actor, ctx.link_pos, ctx.stage) {
                self.attn_flags |= ATTN_SPEAK_NOTALK; // <- :693
                self.event_state = EVENT_STATE_TALK_ITEM; // <- mEventState = 6, :694
            } else {
                self.attn_flags &= !ATTN_SPEAK_NOTALK; // <- :696
                self.event_state = STATE_NONE; // <- :697
            }

            let mut target_angle_x: i16 = 0; // <- :706
            let mut target_angle_y = actor.angle[1]; // <- :707
            if self.action_status == ACTION_ONGOING_1 {
                if self.area_out_check(actor) {
                    self.action_status = ACTION_ONGOING_2; // <- :710
                    self.c08 = 60; // <- :711
                    self.set_anm(ANM_WAIT1); // <- :712
                } else {
                    if calc_timer(&mut self.c0c) {
                        self.c0c = self.get_rnd_value(10, 80); // <- :715
                        let pick = self.get_rnd_value(0, ANM_LAST_PICK);
                        let anm = anm_type_at(pick);
                        self.set_anm(anm); // <- setAnm(l_anm_type[...]), :716
                    }
                    let turning = self.npc_turn_check(actor, &mut target_angle_y);
                    if !turning && calc_timer(&mut self.c08) {
                        self.action_status = ACTION_ONGOING_3; // <- :719
                        self.c08 = self.get_rnd_value(60, 60); // <- :720
                        let pick = self.get_rnd_value(0, TURN_LAST_PICK);
                        self.c0a = turn_angle_at(pick); // <- :721
                    }
                }
            } else if self.action_status == ACTION_ONGOING_2 {
                target_angle_y = bearing(actor.pos, self.home_pos); // <- :725
                if calc_timer(&mut self.c08) && !self.area_out_check(actor) {
                    self.action_status = ACTION_ONGOING_1; // <- :727
                    self.set_anm(ANM_WAIT2); // <- :728
                    self.c08 = self.get_rnd_value(90, 60); // <- :729
                }
            } else if self.action_status == ACTION_ONGOING_3 {
                if self.npc_turn_check(actor, &mut target_angle_y) {
                    self.action_status = ACTION_ONGOING_1; // <- :733
                    self.c08 = self.get_rnd_value(90, 60); // <- :734
                } else {
                    target_angle_y = self.c0a; // <- :736
                    if calc_timer(&mut self.c08) {
                        self.action_status = ACTION_ONGOING_1; // <- :738
                        self.c08 = self.get_rnd_value(90, 60); // <- :739
                    }
                }
            }

            if self.anm_idx == ANM_SING && self.morph.check_pass(MORF_CV_FRAME) {
                ctx.se(SE_CV_KAMOME); // <- fopAcM_seStart(this, JA_SE_CV_KAMOME, 0), :744-745
            }

            if actor.pos.y < self.home_pos.y {
                target_angle_x = HIO_GLIDING_ANG_VEL_X.wrapping_neg(); // <- :749
            }

            let mut angle_diff_y = actor.angle[1].wrapping_sub(target_angle_y); // <- :752
            if i32::from(angle_diff_y).abs() > TURN_GATE {
                actor.angle[1] = walk::add_calc_angle_s(
                    actor.angle[1],
                    target_angle_y,
                    TURN_SCALE_NPC_Y,
                    TURN_MAX_NPC_Y,
                    TURN_MIN_NPC_Y,
                ); // <- :754
                self.shape_angle[1] = actor.angle[1]; // <- :755
                if angle_diff_y > HIO_MAX_ANGLE_Z {
                    angle_diff_y = HIO_MAX_ANGLE_Z; // <- :757
                } else if angle_diff_y < HIO_MAX_ANGLE_Z.wrapping_neg() {
                    angle_diff_y = HIO_MAX_ANGLE_Z.wrapping_neg(); // <- :759
                }
            }

            actor.angle[0] = walk::add_calc_angle_s(
                actor.angle[0],
                target_angle_x,
                TURN_SCALE,
                TURN_MAX,
                TURN_MIN,
            ); // <- :763
            actor.angle[2] = walk::add_calc_angle_s(
                actor.angle[2],
                angle_diff_y,
                TURN_SCALE,
                TURN_MAX,
                TURN_MIN,
            ); // <- :764
            self.set_attention(true, 0, actor); // <- :765
        }
    }

    /// `keyProc` :772-804
    fn key_proc(&mut self, trig_a: bool) {
        if calc_timer(&mut self.flap_exhausted_timer) {
            if trig_a {
                self.flap_timer = HIO_FLAP_DURATION; // <- :775
            }
            if calc_timer(&mut self.flap_timer) {
                self.target_speed_f = HIO_FLAPPING_SPEED_F; // <- :779
                self.target_ang_vel_y = HIO_FLAPPING_ANG_VEL_Y; // <- :780
                self.target_ang_vel_x = HIO_FLAPPING_ANG_VEL_X; // <- :781
                if calc_timer(&mut self.flap_energy_timer) {
                    self.flap_exhausted_timer = HIO_FLAP_EXHAUSTED_DURATION; // <- :785
                }
            } else {
                self.target_speed_f = HIO_SPEED_F; // <- :789
                self.target_ang_vel_y = HIO_GLIDING_ANG_VEL_Y; // <- :790
                self.target_ang_vel_x = HIO_GLIDING_ANG_VEL_X; // <- :791
                self.flap_energy_timer = self.flap_energy_timer.wrapping_add(1); // <- :794
                if self.flap_energy_timer > HIO_FLAP_ENERGY_DURATION {
                    self.flap_energy_timer = HIO_FLAP_ENERGY_DURATION; // <- :796
                }
            }
        } else {
            self.flap_timer = 0; // <- :800
            self.flap_energy_timer = HIO_FLAP_ENERGY_DURATION; // <- :801
        }
    }

    /// `waitPlayerAction` :807-875
    fn wait_player_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.action_status == ACTION_STARTING {
            self.action_status = ACTION_ONGOING_1; // <- :809
            self.on_hyoi_kamome(); // <- :810
            self.set_anm(ANM_WAIT1); // <- :811
            self.speed_f = HIO_SPEED_F; // <- :812
            self.target_speed_f = HIO_SPEED_F; // <- :813
            self.ang_vel_y = HIO_GLIDING_ANG_VEL_Y; // <- :814
            self.ang_vel_x = HIO_GLIDING_ANG_VEL_X; // <- :815
            self.target_ang_vel_y = HIO_GLIDING_ANG_VEL_Y; // <- :816
            self.target_ang_vel_x = HIO_GLIDING_ANG_VEL_X; // <- :817
            self.c08 = self.get_rnd_value(60, 90); // <- :818
            self.c0a = 0; // <- :819
            self.flap_timer = 0; // <- :820
            self.flap_exhausted_timer = 0; // <- :821
            self.flap_energy_timer = HIO_FLAP_ENERGY_DURATION; // <- :822
        } else if self.action_status == ACTION_ENDING {
            self.off_hyoi_kamome(); // <- :824
        } else {
            self.key_proc(ctx.trig_a); // <- :826
            calc_timer(&mut self.lock_angle_x_timer); // <- :827
            if !ctx.run_check && self.area_out_check(actor) {
                self.event_state = EVENT_STATE_CALL; // <- mEventState = 2, :829
            }

            let camera_angle_y = walk::to_s16(ctx.camera.0); // <- dCam_getControledAngleY, :625
            let (turning, target_angle_y, target_angle_z) =
                self.get_stick_ang_y(ctx.stick, camera_angle_y);
            if turning {
                actor.angle[1] = walk::add_calc_angle_s(
                    actor.angle[1],
                    target_angle_y,
                    TURN_SCALE,
                    TURN_MAX,
                    TURN_MIN,
                ); // <- :835
            }
            self.shape_angle[1] = actor.angle[1]; // <- :837

            if self.lock_angle_x_timer == 0 {
                self.target_angle_x = self.get_angle_x(actor, ctx.stick); // <- :840
            }

            actor.angle[0] = walk::add_calc_angle_s(
                actor.angle[0],
                self.target_angle_x,
                TURN_SCALE,
                TURN_MAX,
                TURN_MIN,
            ); // <- :843
            actor.angle[2] = walk::add_calc_angle_s(
                actor.angle[2],
                target_angle_z,
                TURN_SCALE,
                TURN_MAX,
                TURN_MIN,
            ); // <- :844
            self.set_attention(true, 1, actor); // <- :845

            if self.anm_idx != ANM_WAIT2 {
                if self.flap_timer != 0 {
                    self.set_anm(ANM_WAIT2); // <- :849
                } else if calc_timer(&mut self.c08) {
                    self.c08 = self.get_rnd_value(90, 90); // <- :852
                    self.set_anm(ANM_SING); // <- :853
                }

                if self.anm_idx == ANM_SING {
                    if self.reached_anim_end {
                        self.set_anm(ANM_WAIT1); // <- :859
                    }
                    if self.anm_idx == ANM_SING && self.morph.check_pass(MORF_CV_FRAME) {
                        ctx.se(SE_CV_KAMOME); // <- :862-863
                    }
                }
            } else if self.reached_anim_end && self.flap_timer == 0 {
                self.set_anm(ANM_SING); // <- :867
            }

            if self.tg_hit {
                self.set_player_action(PlayerAct::Damage, actor, ctx); // <- :870-871
            }
        }
    }

    /// `damagePlayerAction` :878-899
    fn damage_player_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.action_status == ACTION_STARTING {
            ctx.shock(SHOCK_PATTERN, SHOCK_FLAGS, Vec3::from_array(SHOCK_COORD)); // <- :880
            self.damage_fog = DAMAGE_FOG; // <- :881
            self.action_status = ACTION_ONGOING_1; // <- :882
            self.on_hyoi_kamome(); // <- :883
            self.set_anm(ANM_WAIT1); // <- :884
            self.speed_f = 0.0; // <- :885
            ctx.se(SE_CV_KAMOME); // <- :886
            self.c08 = DAMAGE_RETURN; // <- :887
        } else if self.action_status == ACTION_ENDING {
            self.off_hyoi_kamome(); // <- :889
        } else {
            self.shape_angle[1] = self.shape_angle[1].wrapping_add(DAMAGE_SPIN_Y); // <- :891
            actor.angle[1] = self.shape_angle[1]; // <- :892
            self.set_attention(true, 1, actor); // <- :893
            if calc_timer(&mut self.c08) {
                self.return_link(); // <- :895
            }
        }
    }

    /// `eventOrder` :902-926
    fn event_order(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.is_event_accept() {
            return; // <- :903
        }
        if self.event_state == EVENT_STATE_SPEAK_NPC || self.event_state == EVENT_STATE_SPEAK {
            if !self.possessed {
                ctx.order_flags.set(EVT_CND_CANTALK); // <- eventInfo.onCondition, :908
                if self.event_state == EVENT_STATE_SPEAK_NPC {
                    // OMITTED: fopAcM_orderSpeakEvent(this) :910 — it is
                    // dComIfGp_event_order(dEvtType_TALK_e, 0x1EA, 0, 0x14F, player, this)
                    // (f_op_actor_mng.cpp:676-679), an UNNAMED talk event ordered by
                    // priority. This crate's EventList holds named events only.
                }
            } else {
                self.event_state = STATE_NONE; // <- :913
            }
        } else if self.event_state == EVENT_STATE_TALK_ITEM {
            if !self.possessed {
                ctx.order_flags.set(EVT_CND_CANTALKITEM); // <- :917
                ctx.order_flags.set(EVT_CND_CANTALK); // <- :918
            } else {
                self.event_state = STATE_NONE; // <- :920
            }
        } else if self.event_state != STATE_NONE && self.event_state < 3 {
            self.curr_event_idx_idx = self.event_state; // <- :923
            let idx = self.event_index_at(self.curr_event_idx_idx);
            ctx.order_event(idx); // <- fopAcM_orderOtherEventId(this, mEventIdxs[...]), :924
        }
    }

    /// `checkOrder` :929-937
    fn check_order(&mut self, talk_command: bool) {
        if !talk_command {
            return; // <- :930-932
        }
        if self.event_state != EVENT_STATE_SPEAK_NPC
            && self.event_state != EVENT_STATE_SPEAK
            && self.event_state != EVENT_STATE_TALK_ITEM
        {
            return; // <- :933-935
        }
        self.event_state = STATE_NONE; // <- :936
    }

    /// `checkCommandTalk` :940-951
    fn check_command_talk(&mut self, talk_command: bool, talk_xy: bool) -> bool {
        if talk_command {
            if talk_xy {
                if self.event_state == EVENT_STATE_TALK_ITEM {
                    self.event_state = STATE_NONE; // <- :944
                }
                return false; // <- :946
            }
            return true; // <- :948
        }
        false // <- :950
    }

    /// `returnLinkPlayer` :954-960
    fn return_link_player(&mut self) {
        self.possessed = false; // <- changePlayer(dComIfGp_getLinkPlayer()), :955
        self.off_hyoi_kamome(); // <- :956
    }

    /// `eventProc` :984-1030
    fn event_proc(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) -> bool {
        if ctx.demo_accept {
            if self.event_state != STATE_NONE {
                if self.event_state == EVENT_STATE_OPTION_END {
                    let opt = ctx.list.event_index(EVENT_OPTION_CHAR_END);
                    if opt.is_some_and(|index| ctx.active.start_check(index))
                        || opt.is_some_and(|index| ctx.active.end_check(index))
                    {
                        // OMITTED: dComIfGp_event_setTalkPartner(dComIfGp_getLinkPlayer())
                        // :989 — no actor talk-partner slot exists here.
                        ctx.se(SE_CTRL_NPC_TO_LINK); // <- :990
                    } else {
                        self.off_return_link(); // <- :992
                        self.event_state = STATE_NONE; // <- :993
                    }
                }
                if self.event_state != STATE_NONE {
                    self.on_event_accept(); // <- :997
                    self.event_state = STATE_NONE; // <- :998
                }
            }
        }

        if ctx.run_check && !self.check_command_talk(ctx.talk_command, ctx.talk_xy) {
            if ctx.has_staff {
                let act = ctx.act_idx;
                if act == CUT_NONE {
                    ctx.end_staff_cut(); // <- :1008
                } else {
                    if ctx.cut_advance {
                        self.event_init(act, actor, ctx); // <- :1011
                    }
                    if self.event_action(act, actor, ctx) {
                        ctx.end_staff_cut(); // <- :1014
                    }
                }
            }

            if self.is_event_accept() {
                let idx = self.event_index_at(self.curr_event_idx_idx);
                if let Ok(index) = usize::try_from(idx)
                    && ctx.active.end_check(index)
                {
                    self.event_end(); // <- :1021
                }
                return true; // <- :1023
            }
            return true; // <- :1025
        }

        false // <- :1029
    }

    /// `eventEnd` :1033-1042
    fn event_end(&mut self) {
        // OMITTED: dComIfGp_event_reset() :1034 — the crate's event manager has no reset
        // hook.
        self.off_event_accept(); // <- :1035
        if self.curr_event_idx_idx == CUT_WAIT as i8 {
            // <- `if (mCurrEventIdxIdx == 0)` i.e. "OPTION_CHAR_END", :1037
            self.return_link_player(); // <- :1038
            self.off_return_link(); // <- :1039
        }
        self.curr_event_idx_idx = STATE_NONE; // <- :1041
    }

    /// `event_init_tbl` :962-967
    fn event_init(&mut self, act: i32, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        match act {
            CUT_WAIT => {
                // initialWaitEvent :1050-1057 — dComIfGp_evmng_getMyIntegerP(.., "timer")
                self.wait_timer = ctx.cut_timer.unwrap_or(0) as i16;
            }
            CUT_CHANGE => self.initial_change_event(),
            CUT_DESCEND => self.initial_descend_event(actor, ctx),
            CUT_OUT_TURN => self.initial_area_out_turn(actor),
            _ => {}
        }
    }

    /// `event_action_tbl` :969-974
    fn event_action(&mut self, act: i32, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) -> bool {
        match act {
            CUT_WAIT => {
                // actionWaitEvent :1060-1063
                self.set_attention(true, 1, actor);
                calc_timer(&mut self.wait_timer)
            }
            CUT_CHANGE => true, // <- actionDefault :1045-1047
            CUT_DESCEND => self.action_descend_event(actor, ctx),
            CUT_OUT_TURN => self.action_area_out_turn(actor),
            _ => true,
        }
    }

    /// `initialChangeEvent` :1066-1073. OMITTED:
    /// `fopAcM_SetRoomNo(this, fopAcM_GetHomeRoomNo(this))` :1068 — `Actor` carries
    /// `room` but no home room — and `dComIfGs_setBaitItemEmpty()` :1071, which needs a
    /// bait channel this crate has no resource for.
    fn initial_change_event(&mut self) {
        self.possessed = true; // <- changePlayer(this), :1070
        self.off_no_bg_check(); // <- :1072
    }

    /// `initialDescendEvent` :1076-1095
    fn initial_descend_event(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        // `mDescendStartPos` :1077 and `mDescendStartAngle` :1078 are applied, but both
        // are written only by `callDemoStartCheck` :173-175, which runs behind
        // `XyCheckCB` :124-130 whose input, `dComIfGp_getSelectItem(i_itemBtn)`
        // (d_com_inf_game.h:2882), is the X/Y/Z button->item map. `xy_check_cb` and
        // `call_demo_start_check` are transcribed in the same shape aryll.rs:264 and
        // grammarian.rs:1146 ship them, but no crate channel fires them.
        actor.pos = self.descend_start_pos; // <- :1077
        actor.angle = self.descend_start_angle; // <- :1078
        self.shape_angle = actor.angle; // <- :1079

        self.target_speed_f = HIO_SPEED_F; // <- :1081
        self.speed_f = HIO_SPEED_F; // <- :1081
        self.ang_vel_y = HIO_GLIDING_ANG_VEL_Y; // <- :1082
        self.ang_vel_x = HIO_GLIDING_ANG_VEL_X; // <- :1083
        self.target_ang_vel_y = HIO_GLIDING_ANG_VEL_Y; // <- :1084
        self.target_ang_vel_x = HIO_GLIDING_ANG_VEL_X; // <- :1085

        // OMITTED: the reverb argument dComIfGp_getReverb(fopAcM_GetRoomNo(this)) :1087.
        ctx.se(SE_HYOI_USE_DEMO);
        // OMITTED: mDoAud_zelAudio_c::getInterface()->field_0x0062 = 1, :1089.

        self.on_no_bg_check(); // <- :1092
        self.wait_timer = DESCEND_WAIT; // <- :1094
    }

    /// `actionDescendEvent` :1101-1126
    fn action_descend_event(&mut self, actor: &mut Actor, ctx: &Ctx<'_, '_>) -> bool {
        // OMITTED: link->getHeadTopPos() :1103 is a joint-matrix query; `link_pos` plus
        // the same +20.0f lift stands in (see set_attention).
        let link_head_top = ctx.link_pos + Vec3::new(0.0, HEAD_LIFT, 0.0);
        self.set_attention(true, 1, actor); // <- :1106

        if actor.pos.y < link_head_top.y {
            actor.pos.y = link_head_top.y; // <- :1109
        }
        if actor.pos.y < link_head_top.y + DESCEND_CATCH_Y {
            actor.angle[0] = walk::add_calc_angle_s(
                actor.angle[0],
                0,
                TURN_SCALE_DESCEND,
                TURN_MAX,
                TURN_MIN,
            ); // <- :1112
            self.shape_angle[0] = actor.angle[0]; // <- :1113
        }

        let delta = link_head_top - actor.pos; // <- :1116
        if abs2_xz(delta) < DESCEND_CATCH_RANGE * DESCEND_CATCH_RANGE {
            return true; // <- :1118
        }

        calc_timer(&mut self.wait_timer) // <- :1121
    }

    /// `initialAreaOutTurn` :1129-1134
    fn initial_area_out_turn(&mut self, actor: &mut Actor) {
        self.wait_timer = AREA_OUT_WAIT; // <- :1130
        self.area_out_timer = AREA_OUT_TIMER; // <- :1131
        actor.angle[0] = 0; // <- :1132
        actor.angle[2] = 0; // <- :1133
    }

    /// `actionAreaOutTurn` :1137-1170
    fn action_area_out_turn(&mut self, actor: &mut Actor) -> bool {
        if calc_timer(&mut self.wait_timer) {
            self.on_no_bg_check(); // <- :1139
            self.speed_f = 0.0; // <- :1140

            let aim = bearing(actor.pos, self.home_pos); // <- :1142
            let angle_diff = actor.angle[1].wrapping_sub(aim); // <- :1143
            if i32::from(angle_diff).abs() != 0 {
                actor.angle[1] = walk::add_calc_angle_s(
                    actor.angle[1],
                    aim,
                    TURN_SCALE_OUT_TURN,
                    TURN_MAX,
                    TURN_MIN,
                ); // <- :1145
            } else {
                self.speed_f = HIO_SPEED_F; // <- :1147
            }
            self.shape_angle[1] = actor.angle[1]; // <- :1149

            actor.angle[2] = angle_diff; // <- :1151
            if actor.angle[2] > HIO_MAX_ANGLE_Z {
                actor.angle[2] = HIO_MAX_ANGLE_Z; // <- :1153
            } else if actor.angle[2] < HIO_MAX_ANGLE_Z.wrapping_neg() {
                actor.angle[2] = HIO_MAX_ANGLE_Z.wrapping_neg(); // <- :1155
            }

            if !self.area_out_check(actor) {
                self.off_no_bg_check(); // <- :1159
                return true; // <- :1160
            }
        }

        self.shape_angle[0] = walk::add_calc_angle_s(
            self.shape_angle[0],
            actor.angle[0],
            TURN_SCALE,
            TURN_MAX,
            TURN_MIN,
        ); // <- :1164
        self.shape_angle[2] = walk::add_calc_angle_s(
            self.shape_angle[2],
            actor.angle[2],
            TURN_SCALE,
            TURN_MAX,
            TURN_MIN,
        ); // <- :1165

        self.set_attention(true, 1, actor); // <- :1167

        calc_timer(&mut self.area_out_timer) // <- :1169
    }

    /// `setAnm` :1185-1239
    fn set_anm(&mut self, anm_idx: i32) {
        self.anm_idx = anm_idx; // <- :1215
        let slot = usize::try_from(anm_idx).ok().filter(|slot| *slot < ANM_COUNT);
        let Some(slot) = slot else {
            return;
        };
        let tbl_idx = ANM_TBL_IDX.get(slot).copied().unwrap_or(ANM_TBL_NONE); // <- :1216
        let play_speed = ANM_PLAY_SPEED.get(slot).copied().unwrap_or(1.0); // <- :1218
        if tbl_idx == self.anm_tbl_idx && play_speed == self.morph.rate {
            return; // <- :1220-1222
        }
        self.anm_tbl_idx = tbl_idx; // <- :1224
        self.prev_morf_frame = 0.0; // <- :1227
        self.reached_anim_end = false; // <- :1228

        let loop_mode = ANM_LOOP.get(slot).copied().unwrap_or(J3D_MODE_LOOP);
        // `dNpc_setAnm` :1229-1234 -> `mDoExt_McaMorf::setAnm`
        // (m_Do_ext.cpp:1414-1437): the end frame is `mpAnm->getFrameMax()` :1421, which
        // lives in the unshipped BCK archive, so 0.0 stands in. playSpeed >= 0.0f, so the
        // frame is set to startFrame = 0.0 :1432 and setLoopFrame(getFrame()) makes the
        // loop point 0.0 :1436. `mMorf = 8.0f` :1437 is the blend amount and is not
        // modelled. `if (anmPrm->m10 < 0)` :1236 never fires — m10 is 0 (:1197, :1204,
        // :1211).
        self.morph.init(0.0);
        self.morph.attribute = loop_mode;
        self.morph.rate = play_speed;
        self.morph.frame = 0.0;
        self.morph.loop_point = 0.0;
    }

    /// `setCollision` :1242-1267. The collider centre is
    /// `model->getAnmMtx(m_jnt_body)` :1243-1246 — the j_ka_spin1 joint matrix
    /// (h:138 m_jnt_body) — and the registry it feeds is the `cCcD` character-collision
    /// list behind `dComIfG_Ccsp()` (d_cc_s.h:11), which RULE 3b lists as absent.
    /// `mAtSph` (:350-377, r = 30.0f, AT_TYPE_SWORD) and `mCps` (:408-436, r = 20.0f,
    /// AT_TYPE_SWORD, start/end `l_ms_at_local_start`/`l_ms_at_local_end` :79-80,
    /// `mCps.SetAtVec(l_ms_at_local_vec)` :1265 with :78) all register into it, so no
    /// `HitPoints`/`Hurt` is attached (RULE 8). What survives is the Tg sphere test:
    /// `l_tg_sph_src` :379-406 is r = 30.0f (:404) with
    /// `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsPlayer_e` (:387), moved to `current.pos`.
    fn set_collision(&mut self, actor: &Actor, ctx: &Ctx<'_, '_>) {
        self.tg_hit = walk::within(actor.pos, ctx.link_pos, TG_SPH_RADIUS);
    }

    /// `setLineBgCheck` :1270-1300
    fn set_line_bg_check(&mut self, actor: &Actor, ctx: &Ctx<'_, '_>) {
        self.off_line_hit(); // <- :1271
        let mut line_angle = self.shape_angle; // <- :1272

        let end = actor.pos + mult_vec_zxy(line_angle, LINE_BG_LOCAL_END); // <- :1276-1277
        if line_crossed(ctx.collision, actor.pos, end) {
            self.on_front_line_hit(); // <- :1280
        }

        line_angle[1] = self.shape_angle[1].wrapping_add(NPC_TURN_STEP); // <- :1283
        let end = actor.pos + mult_vec_zxy(line_angle, LINE_BG_LOCAL_END);
        if line_crossed(ctx.collision, actor.pos, end) {
            self.on_left_line_hit(); // <- :1289
        }

        line_angle[1] = self.shape_angle[1].wrapping_sub(NPC_TURN_STEP); // <- :1292
        let end = actor.pos + mult_vec_zxy(line_angle, LINE_BG_LOCAL_END);
        if line_crossed(ctx.collision, actor.pos, end) {
            self.on_right_line_hit(); // <- :1298
        }
    }

    /// `animationPlay` :1303-1321
    fn animation_play(&mut self) {
        // OMITTED: the material-sound id from dComIfG_Bgsp()->GetMtrlSndId(mAcch.m_gnd)
        // :1305-1307 and the reverb argument :1309.
        self.reached_anim_end = self.morph.play(); // <- :1309

        if self.morph.rate < 0.0 {
            if self.morph.frame > self.prev_morf_frame {
                self.reached_anim_end = true; // <- :1314
            }
        } else if self.morph.frame < self.prev_morf_frame {
            self.reached_anim_end = true; // <- :1317
        }

        self.prev_morf_frame = self.morph.frame; // <- :1320
    }

    /// `execute` :1324-1434
    fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.demo_chk_cnt = 0; // <- l_demo_start_chk_cnt = 0, :1325
        // OMITTED: fopAcM_OffStatus(this, fopAcStts_SHOWMAP_e) :1330 — per-actor status
        // bits (RULE 3b).
        calc_timer_u8(&mut self.damage_fog); // <- executeDamageFog(), :1331

        if !self.is_return_link() {
            if self.check_now_pos_move(ctx) {
                self.accel_f = HIO_ACCEL_F; // <- :1335

                if self.speed_f < self.target_speed_f {
                    self.speed_f += self.accel_f; // <- :1338
                    if self.speed_f > self.target_speed_f {
                        self.speed_f = self.target_speed_f; // <- :1340
                    }
                } else if self.speed_f > self.target_speed_f {
                    self.speed_f -= self.accel_f; // <- :1343
                    if self.speed_f < self.target_speed_f {
                        self.speed_f = self.target_speed_f; // <- :1345
                    }
                }

                self.ang_vel_y = walk::add_calc_angle_s(
                    self.ang_vel_y,
                    self.target_ang_vel_y,
                    HIO_ANG_VEL_STEP_SCALE,
                    HIO_ANG_VEL_MAX_STEP,
                    HIO_ANG_VEL_MIN_STEP,
                ); // <- :1349
                self.ang_vel_x = walk::add_calc_angle_s(
                    self.ang_vel_x,
                    self.target_ang_vel_x,
                    HIO_ANG_VEL_STEP_SCALE,
                    HIO_ANG_VEL_MAX_STEP,
                    HIO_ANG_VEL_MIN_STEP,
                ); // <- :1350

                let cos_x_speed = self.speed_f * stream::cos_s(actor.angle[0]); // <- :1352
                self.speed.y = self.speed_f * -stream::sin_s(actor.angle[0]); // <- :1353
                self.speed.x = cos_x_speed * stream::sin_s(actor.angle[1]); // <- :1354
                self.speed.z = cos_x_speed * stream::cos_s(actor.angle[1]); // <- :1355

                self.move_actor(actor); // <- fopAcM_posMove(this, ...), :1357
            }

            if !self.is_no_bg_check() {
                self.acch_correction(actor, ctx); // <- mAcch.CrrPos(*dComIfG_Bgsp()), :1361
                // OMITTED: fopAcM_SetRoomNo / tevStr / mStts.SetRoomId / mPolyInfo from the
                // ground poly :1362-1369 — per-poly-group query.
                if self.sea_in {
                    if !self.is_water_hit() {
                        self.on_water_hit(); // <- :1373
                        // OMITTED: the two emitters :1375-1384 —
                        // ID_IT_JN_WP_SHIBUKI setRate(15.0f) scale (0.6,0.6,0.6) and
                        // ID_IT_JN_WP_HAMON01 scale (1.0,1.0,1.0). Both splash_scale and
                        // ripple_scale :1327-1328 belong to them.
                    }
                } else {
                    self.off_water_hit(); // <- :1387
                }
            }

            self.set_line_bg_check(actor, ctx); // <- :1391

            if self.check_now_pos_move(ctx) {
                self.animation_play(); // <- :1394
            }
        }

        let in_event = self.event_proc(actor, ctx); // <- :1398

        if !in_event {
            if self.possessed {
                // OMITTED: fopAcM_SetStatusMap(this, 0x14) :1402 — map-visibility bit.
                if self.is_return_link() {
                    self.event_state = EVENT_STATE_OPTION_END; // <- mEventState = 0, :1405
                } else {
                    self.player_action(actor, ctx); // <- :1407
                    if self.return_link_check(ctx.run_check, ctx.trig_r) {
                        self.return_link(); // <- :1409
                    }
                }
            } else {
                self.check_order(ctx.talk_command); // <- :1413
                self.npc_action(actor, ctx); // <- :1414
            }

            self.shape_angle[0] = walk::add_calc_angle_s(
                self.shape_angle[0],
                actor.angle[0],
                TURN_SCALE,
                TURN_MAX,
                TURN_MIN,
            ); // <- :1417
            self.shape_angle[2] = walk::add_calc_angle_s(
                self.shape_angle[2],
                actor.angle[2],
                TURN_SCALE,
                TURN_MAX,
                TURN_MIN,
            ); // <- :1418
        }

        self.event_order(ctx); // <- :1421
        // OMITTED: setBaseMtx() :1422 — renderer only.

        if !in_event && self.possessed {
            self.set_collision(actor, ctx); // <- :1425
        }
        // OMITTED: mAtSph.ClrCoHit() / mTgSph.ClrTgHit() / mCps.ClrAtHit() /
        // mStts.ClrCcMove() :1428-1431 — cCcD.
    }

    /// `daPy_npc_c::checkNowPosMove` — tww/src/d/actor/d_a_player_npc.cpp:149-170
    fn check_now_pos_move(&self, ctx: &Ctx<'_, '_>) -> bool {
        if !ctx.run_check {
            return true; // <- dComIfGp_event_runCheck() == false, :150-152
        }
        if ctx.talk_command {
            return true; // <- eventInfo.checkCommandTalk(), :153-155
        }
        if self.is_event_accept() && !self.is_return_link() {
            return true; // <- :156-158
        }
        // `dComIfGp_evmng_getMyStaffId(pName) != -1` :160-163
        ctx.has_staff
    }

    /// `fopAcM_posMove(this, mStts.GetCCMoveP())` :1357. The character-collision
    /// correction vector is absent (RULE 3b); the translation is not.
    fn move_actor(&mut self, actor: &mut Actor) {
        let step = self.speed;
        actor.pos += step;
    }

    /// `mAcch.CrrPos(*dComIfG_Bgsp())` :1361 plus `mAcch.ChkSeaIn()` :1371. `Collision`
    /// is merged-DZB, so `ChkGroundHit` and the sea test are read from the merged surface
    /// and the sea charts rather than the per-poly-group acch circles (RULE 3b).
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

    /// `cLib_getRndValue(min, range)` is `min + cM_rndF((f32)range)` -
    /// tww/include/SSystem/SComponent/c_lib.h:80-82, and `cM_rndF(f)` is `cM_rnd() * f` -
    /// tww/src/SSystem/SComponent/c_math.cpp:185-187. The second argument is a RANGE,
    /// not an upper bound, so `cLib_getRndValue(60, 60)` yields 60..120 and not a
    /// constant 60.
    fn get_rnd_value(&mut self, min: i32, range: i32) -> i16 {
        (min + (rnd(&mut self.seed) * range as f32) as i32) as i16
    }
}

struct Ctx<'a, 'b> {
    link_pos: Vec3,
    collision: Option<&'b Collision>,
    stage: &'b StageName,
    sea_in: bool,
    stick: Vec2,
    camera: CameraYaw,
    trig_a: bool,
    trig_r: bool,
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
    order_flags: &'a mut OrderFlags,
    ev_flags: &'a mut EventFlags,
    vibration: &'a mut Vibration,
    list: &'b EventList,
    entity: Entity,
}

impl Ctx<'_, '_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    /// `dComIfGp_getVibration().StartShock(5, -0x21, cXyz(0.0f, 1.0f, 0.0f))` :880 —
    /// tww/src/d/d_vibration.cpp:211-238. `-0x21 & MASK_CAMERA_SHAKE` (0x3E) and
    /// `-0x21 & FLAG_MOTOR_SHAKE` (0x1) are both non-zero, so both channels fire.
    fn shock(&mut self, pattern: i32, flags: i32, coord: Vec3) {
        self.vibration.start_shock(pattern, flags, coord);
    }

    /// `fopAcM_orderOtherEventId(this, mEventIdxs[mCurrEventIdxIdx])` :924
    fn order_event(&mut self, index: i16) {
        let Some(slot) = usize::try_from(index).ok() else {
            return;
        };
        if self.active.stage_of(slot).is_some() {
            return;
        }
        let list = self.list;
        let entity = self.entity;
        self.active.order(list, slot, entity);
    }

    /// `dComIfGp_evmng_cutEnd(staffId)` :1008, :1014
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
    list: Res<'w, EventList>,
    players: Query<'w, 's, (Entity, &'static Player), Without<Kamome>>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut active: ResMut<ActiveEvent>,
    mut ev_flags: ResMut<EventFlags>,
    mut order_flags: ResMut<OrderFlags>,
    mut events: ResMut<SimEvents>,
    mut vibration: ResMut<Vibration>,
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
            collision: watch.collision.as_deref(),
            stage: &watch.stage,
            sea_in,
            stick: watch.intent.axis, // <- CPad_GET_STICK_POS_X/Y(0), :604, :643
            camera: *watch.camera,     // <- dCam_getControledAngleY, :625
            trig_a: watch.do_intent.0, // <- CPad_CHECK_TRIG_A(0), :774
            trig_r: watch.target_intent.0, // <- CPad_CHECK_TRIG_R(0), :556
            // OMITTED: `talk_command` (:153, :930), `talk_xy`
            // (dComIfGp_event_chkTalkXY, :942) and `demo_accept`
            // (eventInfo.checkCommandDemoAccrpt, :985) all read the per-actor event
            // command / talk button, which live in the event-and-attention model this
            // crate does not carry, so all three read false — the same state aryll.rs:2877,
            // grammarian.rs:2465 and medli.rs:5751 ship in. They stay as Ctx fields so the
            // branches are transcribed and start working the moment that channel exists.
            talk_command: false,
            talk_xy: false,
            demo_accept: false,
            run_check, // <- dComIfGp_event_runCheck(), :555, :828, :1004
            has_staff,
            cut_advance, // <- dComIfGp_evmng_getIsAddvance(staffId), :1010
            // <- dComIfGp_evmng_getMyIntegerP(evtStaffId, "timer"), :1051
            cut_timer: watch.list.cut_number(cut, "timer"),
            act_idx,
            events: &mut *events,
            active: &mut *active,
            order_flags: &mut *order_flags,
            ev_flags: &mut *ev_flags,
            vibration: &mut *vibration,
            list: &watch.list,
            entity,
        };
        bird.execute(actor, &mut ctx);
    }
}