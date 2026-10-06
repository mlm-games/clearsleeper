use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::daynight::DayNight;
use super::event::{ActiveEvent, EventList, Stage};
use super::flags::{Charts, EventFlags, Switches};
use super::inventory::Equipped;
use super::player::Player;
use super::walk;

/// `OBJNAME("TagMk", fpcNm_TAG_MK_e, 0, 0)`, `OBJNAME("TagWp", fpcNm_TAG_MK_e, 1, 0)`,
/// `OBJNAME("TagDM", fpcNm_TAG_MK_e, 2, 0)` and `OBJNAME("TagTy", fpcNm_TAG_MK_e, 3, 0)`
/// (`d_stage.cpp:584-587`) are the only four rows bound to `fpcNm_TAG_MK_e` (`d_a_tag_mk.cpp:553`).
/// The third column is the OBJNAME `sub`; the fourth is `gba`, which `dStage_actorCreate` copies
/// into `fopAcM_prm_class::argument` (`d_stage.cpp:1306-1311`). All four `gba` values are `0`, so
/// `src/dzs.rs`'s `StageActor` - which has NO `argument` field - loses nothing here and no
/// `argument_of` recovery is needed (the shape `Servant::argument_of` uses at
/// `src/sim/servant.rs:433`).
///
/// DEPENDENCY CLOSED: `src/sim/knob00.rs` reported `fpcNm_TAG_MK_e` as a NEEDS INSERT because
/// `daKnob00_c::CreateInit` calls `fopAcM_create(fpcNm_TAG_MK_e, 0xffffff03, ...)`
/// (`d_a_knob00.cpp:357`). This module is that actor.
pub const TAGMK_ACTORS: [&str; 4] = [
    "TagMk", // <- d_stage.cpp:584 (OBJNAME sub 0)
    "TagWp", // <- d_stage.cpp:585 (OBJNAME sub 1)
    "TagDM", // <- d_stage.cpp:586 (OBJNAME sub 2)
    "TagTy", // <- d_stage.cpp:587 (OBJNAME sub 3)
];

/// `u8 daTag_Mk_c::getType() { return (fopAcM_GetParam(this) >> 0) & 0xFF; }` (`:25-27`) - the
/// `create` switch subject (`:478`).
pub const TYPE_MASK: u32 = 0xFF; // <- d_a_tag_mk.cpp:26
/// `u8 daTag_Mk_c::getSwbit() { return (fopAcM_GetParam(this) >> 8) & 0xFF; }` (`:15-17`).
pub const SWBIT_SHIFT: u32 = 8; // <- d_a_tag_mk.cpp:16
/// `u8 daTag_Mk_c::getSwbit2() { return (fopAcM_GetParam(this) >> 16) & 0xFF; }` (`:20-22`).
pub const SWBIT2_SHIFT: u32 = 16; // <- d_a_tag_mk.cpp:21

/// `daTag_Mk_c::Act_e` (`d_a_tag_mk.h:12-24`), one constant per label, in declaration order. The ten
/// labels are the ten arms of `execute`'s switch (`:403-443`).
pub const ACT_WAIT: u8 = 0; // <- d_a_tag_mk.h:13
pub const ACT_ARRIVAL: u8 = 1; // <- d_a_tag_mk.h:14
pub const ACT_HUNT: u8 = 2; // <- d_a_tag_mk.h:15
pub const ACT_READY: u8 = 3; // <- d_a_tag_mk.h:16
pub const ACT_EVENT: u8 = 4; // <- d_a_tag_mk.h:17
pub const ACT_VILLA: u8 = 5; // <- d_a_tag_mk.h:18
pub const ACT_VILLATALK: u8 = 6; // <- d_a_tag_mk.h:19
pub const ACT_TAGWP: u8 = 7; // <- d_a_tag_mk.h:20
pub const ACT_TAGWP2: u8 = 8; // <- d_a_tag_mk.h:21
pub const ACT_DAICHI: u8 = 9; // <- d_a_tag_mk.h:22

/// `dSv_event_flag_c::UNK_2D80 = 0x2D80` (`d_save_event_flag.inc:339`), read at `:314` and `:480`.
/// `src/sim/knob00.rs:93` already holds the same id as `EVENT_UNK_2D80`; one flag id in two
/// consumers is not two actor claims.
pub const EVENT_UNK_2D80: u32 = 0x2D80; // <- d_a_tag_mk.cpp:480
/// `dSv_event_flag_c::UNK_3380 = 0x3380` (`d_save_event_flag.inc:385`), read at `:495`.
pub const EVENT_UNK_3380: u32 = 0x3380; // <- d_a_tag_mk.cpp:495
/// `dSv_event_flag_c::UNK_2D08 = 0x2D08` (`d_save_event_flag.inc:343`), read at `:497`.
pub const EVENT_UNK_2D08: u32 = 0x2D08; // <- d_a_tag_mk.cpp:497
/// `dSv_event_flag_c::ZELDA_AWAKENED = 0x2D02` (`d_save_event_flag.inc:345`), read at `:499`.
pub const EVENT_ZELDA_AWAKENED: u32 = 0x2D02; // <- d_a_tag_mk.cpp:499
/// `dSv_event_flag_c::UNK_1E02 = 0x1E02` (`d_save_event_flag.inc:231`), read at `:524` and written
/// at `:273`.
pub const EVENT_UNK_1E02: u32 = 0x1E02; // <- d_a_tag_mk.cpp:524
/// `dSv_event_flag_c::UNK_1E04 = 0x1E04` (`d_save_event_flag.inc:230`), read at `:296`.
pub const EVENT_UNK_1E04: u32 = 0x1E04; // <- d_a_tag_mk.cpp:296
/// `dItemNo_MASTER_SWORD_3_e = 0x3E` (`d_item_data.h:69`), compared against
/// `dComIfGs_getSelectEquip(0)` at `:135`.
pub const ITEM_MASTER_SWORD_3: u8 = 0x3E; // <- d_a_tag_mk.cpp:135
/// `dComIfGs_getTriforceNum() < 8` (`:137`) - the full Triforce count. `flags::Charts::total()`
/// returns exactly that (`src/sim/flags.rs:505`).
pub const TRIFORCE_FULL: usize = 8; // <- d_a_tag_mk.cpp:137
/// `m296 = 0x14;` (`:513`), `m296 = 10; mCutEndTimer = 60;` (`:515-516`) and
/// `mCutEndTimer = 180; m296 = 0;` (`:518-519`) - the three create-time entry states of
/// `actionDaichi`. The literal `0` is the ctor zero-fill and is not named.
pub const STAGE_LEAD: i16 = 0x14; // <- d_a_tag_mk.cpp:513 (20)
pub const STAGE_TIMED: i16 = 10; // <- d_a_tag_mk.cpp:515
pub const TIMED_HOLD: i16 = 60; // <- d_a_tag_mk.cpp:516
pub const TIMED_WAIT: i16 = 180; // <- d_a_tag_mk.cpp:518
/// ```
/// case 0: case 10: if (isTmpBit(UNK_0440)) { m296++; mDoAud_bgmAllMute(30); } break;
/// case 1: if (isSwitch(getSwbit(), roomNo)) { m296++; } break;
/// case 2: case 11: if (mCutEndTimer > 0) { mCutEndTimer--; }
///                else { m296++; if (isTmpBit(UNK_0440)) { subBgmStart(JA_BGM_MBOSS); onTmpBit(UNK_0420); } } break;
/// case 3: case 12: if (isSwitch(getSwbit2(), roomNo)) { m296++;
///                if (isTmpBit(UNK_0420)) { subBgmStop(); onTmpBit(UNK_0410); } } break;
/// ```
/// (`actionDaichi`, `:359-397`). The two switch tests and the timer drain survive; every
/// `dSv_event_tmp_flag_c` bit does NOT, because there is no temp-flag channel in this sim - see the
/// report. The four temp ids are `UNK_0440 = 0x0440` (`:362`), `UNK_0420 = 0x0420` (`:382`) and
/// `UNK_0410 = 0x0410` (`:393`), and the BGM they arm is `JA_BGM_MBOSS = 0x80000019`
/// (`JAZelAudio_BGM.h:31`); none of the five is carried as a constant because none has a reader.
/// `eventInfo.checkArea(SQUARE(80.0f), SQUARE(110.0f), SQUARE(250.0f))` (`:329`) - the three
/// already-squared arguments of `checkArea(f32 arg1, f32 arg2, f32 arg3)` (`:30-57`).
pub const AREA_NEAR: f32 = 80.0 * 80.0; // <- d_a_tag_mk.cpp:329
pub const AREA_FLANK: f32 = 110.0 * 110.0; // <- d_a_tag_mk.cpp:329
pub const AREA_FAR: f32 = 250.0 * 250.0; // <- d_a_tag_mk.cpp:329
/// `if (abs((s16)tmp) < 0x5000) { return FALSE; }` (`:53`) - the closing facing test of
/// `checkArea`, so the player must be facing AWAY from the sign. RULE 10: the `s16` difference goes
/// through `unsigned_abs`.
pub const AREA_FACING: u16 = 0x5000; // <- d_a_tag_mk.cpp:53
/// `fopAcM_searchActorDistanceXZ(this, player) < 150.0f` (`:286`). `fopAcM_searchActorDistanceXZ`
/// is `(*other_pos - *this_pos).absXZ()` (`f_op_actor_mng.cpp:511-515`) - a raw XZ distance, so 150
/// is 1.5 m.
pub const HUNT_DIST: f32 = 150.0; // <- d_a_tag_mk.cpp:286
/// `fopAcM_searchActorDistanceXZ(this, player) < 5000.0` (`:351`) - the same raw distance, 50 m.
pub const TAGWP_DIST: f32 = 5000.0; // <- d_a_tag_mk.cpp:351
/// `m29C.set(cM_ssin(current.angle.y), 0.0f, cM_scos(current.angle.y));` followed by
/// `attention_info.position.y += 210.0f; eyePos.y += 210.0f;` (`:486-488`) - the `checkArea` forward
/// axis, built from the sign's own yaw, and the type-3 talk anchor. Only the axis is stored: the
/// two lifts feed the per-actor attention word and the message driver's eye position, and the
/// driver (`talk`, `:96-131`) is omitted - see the report.
/// `static cXyz local_20(-3310.0f, 1463.0f, -198484.0f);` and `local_28.set(0, 0x4000, 0);`
/// (`demoInitMake`, `:164-166`) - the JOY PENDANT's absolute spawn point and yaw.
pub const PENDANT_POS: Vec3 = Vec3::new(-3310.0, 1463.0, -198484.0); // <- d_a_tag_mk.cpp:164
pub const PENDANT_YAW: i16 = 0x4000; // <- d_a_tag_mk.cpp:166
/// `dItemNo_JOY_PENDANT_e = 0x1F` (`d_item_data.h:38`), the item `demoInitMake` creates.
pub const ITEM_JOY_PENDANT: u8 = 0x1F; // <- d_a_tag_mk.cpp:168
/// `i_this->m2B8 = "TagMk";` (`:536`) - the staff name `dComIfGp_evmng_getMyStaffId(m2B8)` (`:274`)
/// resolves, and the key the whole `daTag_Mk` / `knob00` pair shares.
pub const STAFF_TAGMK: &str = "TagMk"; // <- d_a_tag_mk.cpp:536
/// `mEventIdx = dComIfGp_evmng_getEventIdx("tagwp2")` / `"tagwp"` / `"tagwp3"` (`setTagWpEvId`,
/// `:134-142`) - the three Table-of-Writings cuts.
pub const EVENT_TAGWP2: &str = "tagwp2"; // <- d_a_tag_mk.cpp:136
pub const EVENT_TAGWP: &str = "tagwp"; // <- d_a_tag_mk.cpp:138
pub const EVENT_TAGWP3: &str = "tagwp3"; // <- d_a_tag_mk.cpp:140
/// `mEventIdx = dComIfGp_evmng_getEventIdx("MK_PENDANT");` (`actionHunt`, `:287`).
pub const EVENT_MK_PENDANT: &str = "MK_PENDANT"; // <- d_a_tag_mk.cpp:287
/// `static char* action_table[] = {"WAIT", "MAKE", "DELETE", "SETGOAL"};` (`getNowEventAction`,
/// `:211`) - the four names `dComIfGp_evmng_getMyActIdx(mStaffIdx, ..., FALSE, 1)` matches against.
/// Index 0 is `WAIT`, whose `demoInit` / `demoProc` pair is the timer.
pub const DEMO_MAKE: usize = 1; // <- d_a_tag_mk.cpp:211
pub const DEMO_DELETE: usize = 2; // <- d_a_tag_mk.cpp:211
pub const DEMO_SETGOAL: usize = 3; // <- d_a_tag_mk.cpp:211

/// The Windfall / Outset / Dragon Roost signpost family: the Joy Pendant hunt, the Cabana deed, the
/// Table of Writings and the Wind's Requiem cue.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daTag_Mk_c` registers no collider of any
/// kind - the class is a bare `fopAc_ac_c` (`d_a_tag_mk.h:8`) with no `dCcD` member, no `mStts` and
/// no entry collider - and its profile `Group` is `fopAc_ACTOR_e` (`:559`). `daTag_Mk_Draw`
/// (`:448-450`) returns `TRUE` with NO draw call at all, so there is also no `actor_views` branch;
/// see the report. Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `daTag_Mk_IsDelete` (`:458-461`) is the bare "accepts a delete request" form of RULE 6b. The one
/// real destruction in the family is `demoInitDelete`'s `fopAcM_delete(pfVar1)` (`:158`), which
/// deletes the JOY PENDANT the demo made and not the signpost. `create` always returns
/// `cPhs_COMPLEATE_e` (`:537`), so `spawn` never refuses and needs no `refused` flag.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Tagmk {
    /// `mActionTypes` (`d_a_tag_mk.h:46`, a `u8`) - the `execute` switch subject (`:403`).
    pub action: u8,
    /// `m296` (`d_a_tag_mk.h:42`, an `s16`) - the `actionDaichi` stage counter.
    pub stage: i16,
    /// `mCutEndTimer` (`d_a_tag_mk.h:50`, an `s16`) - the `actionDaichi` hold timer (`:376-379`) and
    /// the demo `WAIT` timer (`:185`, `:193`).
    pub timer: i16,
    /// `mEventIdx` (`d_a_tag_mk.h:44`, an `s16`) - `dComIfGp_evmng_getEventIdx(...)` resolved into
    /// an `EventList` index. `None` stands for a failed lookup, retail's `0xFFFF` fallback.
    pub event_idx: Option<usize>,
    /// `m2B4` (`d_a_tag_mk.h:54`) - the `fpc_ProcID` of the JOY PENDANT `demoInitMake` created
    /// (`:171`), modelled as an `Option<Entity>`; `demoInitDelete` / `demoInitSetgoal` resolve it
    /// with `fopAcM_SearchByID`.
    pub pendant: Option<Entity>,
    /// `m29C` (`d_a_tag_mk.h:44`) - the `checkArea` forward axis, `(sin(yaw), 0, cos(yaw))` (`:486`);
    /// only the type-3 Villa sign writes it.
    pub facing: Vec3,
    /// `getSwbit()` (`:15-17`).
    pub swbit: u8,
    /// `getSwbit2()` (`:20-22`).
    pub swbit2: u8,
    /// `getType()` (`:25-27`) - the `create` switch subject.
    pub kind: u8,
    /// One-shot guard so the create-time work (`:486-488` and `setTagWpEvId` at `:503`) happens
    /// exactly once, where a mutable actor and the full resource set are both available.
    pub entered: bool,
}

/// The per-tick inputs `execute` reads, bundled so the ten action arms stay readable. Private:
/// `control` is the only entry point.
struct Ctx<'a, 'w, 's> {
    list: &'a EventList,
    active: &'a mut ActiveEvent,
    switches: &'a Switches,
    flags: &'a mut EventFlags,
    day: &'a DayNight,
    player: Option<Player>,
    commands: &'a mut Commands<'w, 's>,
}

impl Tagmk {
    /// `daTag_Mk_c::create` (`:475-538`), minus the `shape_angle` / `current.angle` zeroing
    /// (`:532-535`, already zero in the `Actor` default), the `m2B8` assignment (`:536`, a
    /// compile-time constant consumed directly as [`STAFF_TAGMK`]), the two `offTmpBit` calls
    /// (`:509-510`, an absent channel) and `setTagWpEvId()` (`:503`, which needs the equipped item
    /// and the Triforce count and therefore belongs in [`Tagmk::create_init`]).
    ///
    /// ```
    /// switch (getType()) {
    /// case 3: if (!isEventBit(UNK_2D80)) { setAction(ACT_VILLA_e); [attention + m29C + lifts] }
    ///         else { setAction(ACT_WAIT_e); } break;
    /// case 2: if (!isEventBit(UNK_3380)) { setAction(ACT_WAIT_e); }
    ///         else if (isEventBit(UNK_2D08)) { setAction(ACT_WAIT_e); }
    ///         else if (!isEventBit(ZELDA_AWAKENED)) { setAction(ACT_WAIT_e); }
    ///         else { setAction(ACT_TAGWP_e); setTagWpEvId(); } break;
    /// case 4: setAction(ACT_DAICHI_e);
    ///         if (isSwitch(getSwbit2(), roomNo)) { m296 = 0x14; }
    ///         else if (isSwitch(getSwbit(), roomNo)) { m296 = 10; mCutEndTimer = 60; }
    ///         else { mCutEndTimer = 180; m296 = 0; } break;
    /// default: if (!isEventBit(UNK_1E02)) { setAction(ACT_ARRIVAL_e); } else { setAction(ACT_WAIT_e); }
    /// }
    /// ```
    /// (`:478-530`). The case-2 chain is flattened into one conjunction because each of its three
    /// wait arms is the same `setAction(ACT_WAIT_e)`.
    pub fn spawn(parameters: u32, flags: &EventFlags, switches: &Switches) -> Self {
        let swbit = ((parameters >> SWBIT_SHIFT) & TYPE_MASK) as u8;
        let swbit2 = ((parameters >> SWBIT2_SHIFT) & TYPE_MASK) as u8;
        let kind = (parameters & TYPE_MASK) as u8;
        let (action, stage, timer) = match kind {
            3 => {
                if flags.test(EVENT_UNK_2D80) {
                    (ACT_WAIT, 0, 0)
                } else {
                    (ACT_VILLA, 0, 0)
                }
            }
            2 => {
                if flags.test(EVENT_UNK_3380)
                    && !flags.test(EVENT_UNK_2D08)
                    && flags.test(EVENT_ZELDA_AWAKENED)
                {
                    (ACT_TAGWP, 0, 0)
                } else {
                    (ACT_WAIT, 0, 0)
                }
            }
            4 => {
                if switches.test(swbit2) {
                    (ACT_DAICHI, STAGE_LEAD, 0)
                } else if switches.test(swbit) {
                    (ACT_DAICHI, STAGE_TIMED, TIMED_HOLD)
                } else {
                    (ACT_DAICHI, 0, TIMED_WAIT)
                }
            }
            _ => {
                if flags.test(EVENT_UNK_1E02) {
                    (ACT_WAIT, 0, 0)
                } else {
                    (ACT_ARRIVAL, 0, 0)
                }
            }
        };
        Self {
            action,
            stage,
            timer,
            event_idx: None,
            // `m2B4` (`d_a_tag_mk.h:54`) has no create-time initialiser, so it takes the ctor
            // zero-fill; retail treats `0` as "no process", which `Option::None` states directly.
            pendant: None,
            facing: Vec3::ZERO,
            swbit,
            swbit2,
            kind,
            entered: false,
        }
    }

    /// The create-time work `create` cannot finish on its own: the type-3 talk anchor and
    /// `setTagWpEvId`.
    pub fn create_init(
        &mut self,
        actor: &Actor,
        equipped: &Equipped,
        triforce: usize,
        list: &EventList,
    ) {
        // ```
        // case 3: ...
        //     m29C.set(cM_ssin(current.angle.y), 0.0f, cM_scos(current.angle.y));
        //     attention_info.position.y += 210.0f;
        //     eyePos.y += 210.0f;
        // ```
        // (`:486-488`) - only the `!isEventBit(UNK_2D80)` arm reaches these lines, which is exactly
        // the arm that selected `ACT_VILLA` in `spawn`.
        //
        // `attention_info.flags = fopAc_Attn_TALKFLAG_NOTALK_e | fopAc_Attn_ACTION_SPEAK_e;` and
        // `attention_info.distances[fopAc_Attn_TYPE_SPEAK_e] = 0xab;` (`:483-484`) are the per-actor
        // attention word - one of RULE 3b's absent "attention/catch requests" - and are omitted.
        if self.kind == 3 && self.action == ACT_VILLA {
            let yaw = walk::from_s16(actor.angle[1]);
            let (sin_yaw, cos_yaw) = yaw.sin_cos();
            self.facing = Vec3::new(sin_yaw, 0.0, cos_yaw);
        }
        self.set_tag_wp_event(equipped, triforce, list);
        self.entered = true;
    }

    /// `setTagWpEvId` (`:134-142`), reached only from `create`'s `case 2` else chain (`:503`).
    ///
    /// ```
    /// if (dComIfGs_getSelectEquip(0) != dItemNo_MASTER_SWORD_3_e) { mEventIdx = getEventIdx("tagwp2"); }
    /// else if (dComIfGs_getTriforceNum() < 8) { mEventIdx = getEventIdx("tagwp"); }
    /// else { mEventIdx = getEventIdx("tagwp3"); }
    /// ```
    ///
    /// `dComIfGs_getTriforceNum()` is `flags::Charts::total()` (`src/sim/flags.rs:505`), and
    /// `dComIfGp_evmng_getEventIdx(name)` is reduced to `EventList::event_index(name)` - the same
    /// reduction `src/sim/mbdoor.rs:80` and `src/sim/vgnfd.rs` use.
    fn set_tag_wp_event(&mut self, equipped: &Equipped, triforce: usize, list: &EventList) {
        if self.action != ACT_TAGWP {
            return;
        }
        let name = if equipped.item(0) != ITEM_MASTER_SWORD_3 {
            EVENT_TAGWP2
        } else if triforce < TRIFORCE_FULL {
            EVENT_TAGWP
        } else {
            EVENT_TAGWP3
        };
        self.event_idx = list.event_index(name);
    }

    /// `checkArea(f32 arg1, f32 arg2, f32 arg3)` (`:30-57`), called from `actionVilla` (`:329`).
    /// The `arg2` term is a squared dot product against [`Tagmk::facing`], so it is a corridor
    /// half-width rather than a radius, and the closing test demands the player be facing AWAY.
    fn check_area(&self, actor: &Actor, player: &Player, arg1: f32, arg2: f32, arg3: f32) -> bool {
        // ```
        // cXyz local_54 = player->current.pos - current.pos;
        // f32 abs1 = local_54.abs2XZ();
        // if (abs1 > arg3) { return FALSE; }
        // ```
        // (`:33-37`) - the comparison is against the already-SQUARED `arg3`.
        let delta = player.pos - actor.pos;
        let abs_xz = delta.x * delta.x + delta.z * delta.z;
        if abs_xz > arg3 {
            return false;
        }
        // ```
        // local_54.normalize();
        // f32 fVar1 = local_54.x * m29C.x + local_54.z * m29C.z;
        // f32 fVar2 = abs1 * fVar1 * fVar1;
        // if (fVar2 > arg2) { return FALSE; }
        // if (abs1 - fVar2 > arg1) { return FALSE; }
        // ```
        // (`:39-49`) - `normalize()` is the full 3-component normalisation while `abs1` stays the
        // SQUARED XZ length, so `fVar2` is a squared along-axis distance.
        let length = delta.length();
        if length == 0.0 {
            return false;
        }
        let unit = delta / length;
        let along = unit.x * self.facing.x + unit.z * self.facing.z;
        let along_sq = abs_xz * along * along;
        if along_sq > arg2 {
            return false;
        }
        if abs_xz - along_sq > arg1 {
            return false;
        }
        // ```
        // s16 tmp = (s16)current.angle.y; tmp -= player->current.angle.y;
        // if (abs((s16)tmp) < 0x5000) { return FALSE; }
        // ```
        // (`:51-55`) - RULE 10: the `s16` difference goes through `unsigned_abs`, because `abs` on a
        // signed type that can reach MIN is banned.
        let facing = walk::to_s16(player.yaw);
        actor
            .angle[1]
            .wrapping_sub(facing)
            .unsigned_abs()
            >= AREA_FACING
    }

    /// `execute` (`:402-445`) - the ten-arm dispatch plus the `default` arm's `actionWait`.
    fn execute(&mut self, ctx: &mut Ctx<'_, '_, '_>, entity: Entity, actor: &Actor) {
        match self.action {
            // ```
            // if (dComIfGs_isEventBit(dSv_event_flag_c::UNK_1E04) && !dKy_daynight_check()) {
            //     setAction(ACT_HUNT_e);
            // }
            // ```
            // `actionArrival` (`:296-298`). `dKy_daynight_check()` is `TIME_DAY` for `hour` in
            // `6..18` and `TIME_NIGHT` otherwise (`d_kankyo.cpp:625-632`), which is byte-for-byte
            // `super::daynight::DayNight::daynight()` (`src/sim/daynight.rs:48-54`). `!daynight_check()`
            // is therefore NIGHT.
            ACT_ARRIVAL => {
                if ctx.flags.test(EVENT_UNK_1E04) && !ctx.day.is_day() {
                    self.action = ACT_HUNT;
                }
            }
            // ```
            // daPy_py_c* player = getLinkPlayer();
            // if (player->checkFrontRollCrash() && fopAcM_searchActorDistanceXZ(this, player) < 150.0f) {
            //     mEventIdx = getEventIdx("MK_PENDANT");
            //     fopAcM_orderOtherEventId(this, mEventIdx);
            //     setAction(ACT_READY_e);
            // }
            // ```
            // `actionHunt` (`:286-290`). `checkFrontRollCrash()` is
            // `checkResetFlg0(daPyRFlg0_FRONT_ROLL_CRASH)` (`d_a_player.h:584`) - a per-actor status
            // bit, and RULE 3b lists per-actor status bits as confirmed-absent - so the gate is
            // transcribed and fed `false` and `ACT_HUNT` can never leave. The distance test is kept
            // so the arm is correct the moment such a bit is modelled.
            ACT_HUNT => {
                let front_roll_crash = false;
                if front_roll_crash
                    && ctx
                        .player
                        .is_some_and(|player| distance_xz(actor, player.pos) < HUNT_DIST)
                {
                    self.event_idx = ctx.list.event_index(EVENT_MK_PENDANT);
                    self.order(ctx, entity);
                    self.action = ACT_READY;
                }
            }
            // ```
            // if (eventInfo.checkCommandDemoAccrpt()) {
            //     setAction(ACT_EVENT_e);
            //     dComIfGs_onEventBit(dSv_event_flag_c::UNK_1E02);
            //     mStaffIdx = dComIfGp_evmng_getMyStaffId(m2B8);
            //     demoProc();
            // } else { fopAcM_orderOtherEventId(this, mEventIdx); }
            // ```
            // `actionReady` (`:271-279`). `m2B8` is the compile-time `"TagMk"`, so the staff is
            // [`STAFF_TAGMK`] directly rather than a stored field.
            ACT_READY => {
                if ctx.demo_accepted(self.event_idx) {
                    self.action = ACT_EVENT;
                    ctx.flags.set(EVENT_UNK_1E02);
                    self.demo_proc(ctx, actor);
                } else {
                    self.order(ctx, entity);
                }
            }
            // ```
            // if (dComIfGp_evmng_endCheck(mEventIdx)) { setAction(ACT_WAIT_e); dComIfGp_event_reset(); }
            // else { demoProc(); }
            // ```
            // `actionEvent` (`:260-266`). `dComIfGp_event_reset()` resets the global event-manager
            // state and has no reader in this sim, so it is omitted.
            ACT_EVENT => {
                if ctx.ended(self.event_idx) {
                    self.action = ACT_WAIT;
                } else {
                    self.demo_proc(ctx, actor);
                }
            }
            // ```
            // if (eventInfo.checkCommandTalk()) { setAction(ACT_VILLATALK_e); talkInit(); }
            // else if (checkArea(SQUARE(80.0f), SQUARE(110.0f), SQUARE(250.0f))) {
            //     eventInfo.onCondition(dEvtCnd_CANTALK_e);
            //     eventInfo.onCondition(dEvtCnd_CANTALKITEM_e);
            // }
            // ```
            // `actionVilla` (`:326-333`). `checkCommandTalk` is the per-actor talk-command word,
            // another of RULE 3b's absent attention/catch requests, so it is fed `false` and
            // `ACT_VILLATALK` is unreachable. `checkArea` is therefore reached every tick, but its
            // two `onCondition` writes are the same absent channel and are omitted - which leaves
            // the test with no modelled effect. It is transcribed anyway so the arm is correct the
            // moment that channel exists; see the report.
            ACT_VILLA => {
                let talk_command = false;
                if talk_command {
                    self.action = ACT_VILLATALK;
                } else if let Some(player) = ctx.player
                    && self.check_area(actor, &player, AREA_NEAR, AREA_FLANK, AREA_FAR)
                {
                    // `eventInfo.onCondition(dEvtCnd_CANTALK_e); eventInfo.onCondition(dEvtCnd_CANTALKITEM_e);`
                    // (`:330-331`) - omitted with the rest of the per-actor event info.
                }
            }
            // ```
            // if (dComIfGp_evmng_endCheck(mEventIdx)) { setAction(ACT_WAIT_e); dComIfGp_event_reset(); }
            // ```
            // `actionTagWp2` (`:338-342`).
            ACT_TAGWP2 => {
                if ctx.ended(self.event_idx) {
                    self.action = ACT_WAIT;
                }
            }
            // ```
            // if (eventInfo.checkCommandDemoAccrpt()) { setAction(ACT_TAGWP2_e); }
            // else if (fopAcM_searchActorDistanceXZ(this, player) < 5000.0) {
            //     fopAcM_orderOtherEventId(this, mEventIdx);
            // }
            // ```
            // `actionTagWp` (`:349-354`). Same absent `checkCommandDemoAccrpt` gate as
            // [`ACT_READY`], so the near-player arm is what remains.
            ACT_TAGWP => {
                if ctx.demo_accepted(self.event_idx) {
                    self.action = ACT_TAGWP2;
                } else if ctx
                    .player
                    .is_some_and(|player| distance_xz(actor, player.pos) < TAGWP_DIST)
                {
                    self.order(ctx, entity);
                }
            }
            ACT_DAICHI => self.daichi(ctx),
            // `default: actionWait();` (:440-442) - `actionWait` (`:303-305`) is `return TRUE;`.
            // `ACT_VILLATALK` (6) is dispatched by the decomp but its body is unreachable here.
            _ => {}
        }
    }

    /// `actionDaichi` (`:358-399`) - the two-step switch-cascade that gates the Wind's Requiem BGM.
    /// Only the two `dComIfGs_isSwitch` arms and the timer drain survive; the four
    /// `dSv_event_tmp_flag_c` reads and the three audio calls that hang off them are omitted.
    fn daichi(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        match self.stage {
            // `if (dComIfGs_isTmpBit(dSv_event_tmp_flag_c::UNK_0440)) { m296++; mDoAud_bgmAllMute(30); }`
            // (`:362-365`) - the gate has no channel, so the arm is fed `false` and stages 0 and 10
            // never advance on their own.
            0 | 10 => {}
            // `if (dComIfGs_isSwitch(getSwbit(), roomNo)) { m296++; }` (`:369-371`)
            1 => {
                if ctx.switches.test(self.swbit) {
                    self.stage += 1;
                }
            }
            // ```
            // if (mCutEndTimer > 0) { mCutEndTimer--; }
            // else { m296++; if (isTmpBit(UNK_0440)) { subBgmStart(JA_BGM_MBOSS); onTmpBit(UNK_0420); } }
            // ```
            // (`:376-384`) - the timer drain and the stage step survive; the sub-BGM start does not.
            2 | 11 => {
                if self.timer > 0 {
                    self.timer -= 1;
                } else {
                    self.stage += 1;
                }
            }
            // ```
            // if (dComIfGs_isSwitch(getSwbit2(), roomNo)) {
            //     m296++;
            //     if (isTmpBit(UNK_0420)) { subBgmStop(); onTmpBit(UNK_0410); }
            // }
            // ```
            // (`:389-395`)
            3 | 12 => {
                if ctx.switches.test(self.swbit2) {
                    self.stage += 1;
                }
            }
            _ => {}
        }
    }

    /// `demoProc` (`:217-256`), with `mStaffIdx` pinned to the compile-time [`STAFF_TAGMK`].
    fn demo_proc(&mut self, ctx: &mut Ctx<'_, '_, '_>, actor: &Actor) {
        // ```
        // s32 iVar1 = getNowEventAction();
        // if (dComIfGp_evmng_getIsAddvance(mStaffIdx)) { demoInitCom(); switch (iVar1) { ... } }
        // switch (iVar1) { case 0: demoProcWait(); break; case 1: demoProcMake(); break;
        //                  default: dComIfGp_evmng_cutEnd(mStaffIdx); break; }
        // demoProcCom();
        // ```
        // `getNowEventAction` (`:210-213`) is `getMyActIdx(mStaffIdx, action_table, 4, FALSE, 1)`
        // against `{"WAIT", "MAKE", "DELETE", "SETGOAL"}` (`:211`), so the index is the position of
        // the running cut's own name in that four-entry table; a `WAIT` cut - or no cut at all -
        // yields `None`, which is this module's spelling of index 0. `demoInitCom` (`:202-203`) and
        // `demoProcCom` (`:206-207`) are both empty in the decomp.
        let cut = ctx.active.staff_cut(ctx.list, STAFF_TAGMK);
        let index = cut.and_then(|(cut, _)| {
            ctx.list.cut_name(cut).and_then(|name| match name {
                "MAKE" => Some(DEMO_MAKE),
                "DELETE" => Some(DEMO_DELETE),
                "SETGOAL" => Some(DEMO_SETGOAL),
                _ => None,
            })
        });
        if let Some((_, advanced)) = cut
            && advanced
        {
            match index {
                // `demoInitWait` (`:182-188`):
                // `int* timer = getMyIntegerP(mStaffIdx, "Timer");
                //  mCutEndTimer = (timer != NULL) ? *timer : 0;`
                None => {
                    self.timer = ctx
                        .list
                        .staff_of(STAFF_TAGMK)
                        .and_then(|staff| ctx.list.number(staff, TIMER_KEY))
                        .and_then(|value| i16::try_from(value).ok())
                        .unwrap_or(0);
                }
                Some(DEMO_MAKE) => {
                    // ```
                    // fopAc_ac_c* a_item = fopAcM_createItemForKP2(&(-3310, 1463, -198484),
                    //     dItemNo_JOY_PENDANT_e, roomNo, &(0, 0x4000, 0), NULL, 10.0, 10.0, -2.1, 0);
                    // m2B4 = fpcM_GetID(a_item);
                    // dComIfGp_event_setTalkPartner(a_item);
                    // ```
                    // (`demoInitMake`, `:163-173`). The position is absolute world coordinates and
                    // the yaw is `0x4000`; `dComIfGp_event_setTalkPartner` is the absent
                    // attention/catch channel. The trailing `10.0, 10.0, -2.1` triple is the item's
                    // launch velocity, which `fopAcM_createItemForKP2` hands to the item actor's own
                    // `create` rather than to `fopAc_prm`, so it is not invented here.
                    self.pendant = Some(ctx.commands.spawn(Actor {
                        name: super::pickup::ITEM_ACTORS[0].to_owned(),
                        parameters: ITEM_JOY_PENDANT as u32,
                        pos: PENDANT_POS,
                        angle: [0, PENDANT_YAW, 0],
                        set_id: 0,
                        room: actor.room,
                        ..Default::default()
                    }).id());
                }
                Some(DEMO_DELETE) => {
                    // ```
                    // fopAc_ac_c* pfVar1 = fopAcM_SearchByID(m2B4);
                    // if (pfVar1 != NULL) { fopAcM_delete(pfVar1); }
                    // ```
                    // (`demoInitDelete`, `:154-159`) - this despawns the pendant, not the sign. The
                    // `SearchByID` guard becomes the `Option`, so a `None` pendant is already the
                    // decomp's "not found" arm.
                    if let Some(pendant) = self.pendant {
                        let _ = ctx.commands.entity(pendant).despawn();
                    }
                }
                Some(DEMO_SETGOAL) => {
                    // ```
                    // fopAc_ac_c* pfVar1 = fopAcM_SearchByID(m2B4);
                    // if (pfVar1 != NULL) { dComIfGp_evmng_setGoal(&pfVar1->current.pos); }
                    // ```
                    // (`demoInitSetgoal`, `:145-151`) - the event manager's camera goal is a
                    // demo-camera channel this sim does not model. The `SearchByID` lookup is
                    // transcribed and the goal write omitted, so the arm is a no-op.
                }
                _ => {}
            }
        }
        match index {
            // ```
            // if (mCutEndTimer > 0) { mCutEndTimer--; } else { dComIfGp_evmng_cutEnd(mStaffIdx); }
            // ```
            // `demoProcWait` (`:192-198`).
            None => {
                if self.timer > 0 {
                    self.timer -= 1;
                } else {
                    ctx.cut_end();
                }
            }
            // ```
            // dComIfGp_evmng_cutEnd(mStaffIdx); return FALSE;
            // ```
            // `demoProcMake` (`:176-179`) - the MAKE cut lasts exactly one tick.
            Some(DEMO_MAKE) => ctx.cut_end(),
            // `default: dComIfGp_evmng_cutEnd(mStaffIdx); break;` (`:251-253`)
            _ => ctx.cut_end(),
        }
    }

    /// `fopAcM_orderOtherEventId(this, mEventIdx)` (`:288`, `:352`) - the one-argument overload. The
    /// `stage_of` guard is mandatory (bug class 5): re-ordering every tick resets `Stage::Order` and
    /// livelocks the demo.
    fn order(&mut self, ctx: &mut Ctx<'_, '_, '_>, entity: Entity) {
        if let Some(index) = self.event_idx
            && ctx.active.stage_of(index).is_none()
        {
            ctx.active.order(ctx.list, index, entity);
        }
    }
}

impl Ctx<'_, '_, '_> {
    /// `dComIfGp_evmng_endCheck(mEventIdx)` (`:260`, `:295`, `:338`).
    fn ended(&self, index: Option<usize>) -> bool {
        index.is_some_and(|index| self.active.end_check(index))
    }

    /// `eventInfo.checkCommandDemoAccrpt()` (`:271`, `:349`) is `mCommand == dEvtCmd_INDEMO_e`
    /// (`f_op_actor.h:177`), a latch set on both ordered actors when the event starts
    /// (`d_event.cpp:314-316`) and cleared at `demoEnd` (`:331-337`), so it stays true through the
    /// CLOSE stage. `src/sim/htobi3.rs:160-166` establishes the sim's reduction of exactly that
    /// latch as "an event this actor ordered is in `Play` or `Close`", and the ordered event here
    /// is `mEventIdx`.
    fn demo_accepted(&self, index: Option<usize>) -> bool {
        index.is_some_and(|index| {
            matches!(
                self.active.stage_of(index),
                Some(Stage::Play) | Some(Stage::Close)
            )
        })
    }

    /// `dComIfGp_evmng_cutEnd(mStaffIdx)` (`:177`, `:196`, `:252`).
    fn cut_end(&mut self) {
        self.active.end_staff_cut(self.list, self.flags, STAFF_TAGMK);
    }
}

/// `dComIfGp_evmng_getMyIntegerP(mStaffIdx, "Timer")` (`:183`).
const TIMER_KEY: &str = "Timer"; // <- d_a_tag_mk.cpp:183

/// `fopAcM_searchActorDistanceXZ(this, other)` is `(*other_pos - *this_pos).absXZ()`
/// (`f_op_actor_mng.cpp:511-515`) - a raw XZ distance, not squared.
fn distance_xz(actor: &Actor, player: Vec3) -> f32 {
    (player.x - actor.pos.x).hypot(player.z - actor.pos.z)
}

pub fn is_tagmk(name: &str) -> bool {
    TAGMK_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    switches: Res<Switches>,
    mut flags: ResMut<EventFlags>,
    day: Res<DayNight>,
    equipped: Res<Equipped>,
    charts: Res<Charts>,
    players: Query<&Player>,
    mut signs: Query<(Entity, &Actor, &mut Tagmk), With<Tagmk>>,
    mut commands: Commands,
) {
    let player = players.iter().next().copied();
    for (entity, actor, mut sign) in &mut signs {
        if !sign.entered {
            sign.create_init(actor, &equipped, charts.total(), &list);
        }
        let mut ctx = Ctx {
            list: &list,
            active: &mut active,
            switches: &switches,
            flags: &mut flags,
            day: &day,
            player,
            commands: &mut commands,
        };
        sign.execute(&mut ctx, entity, actor);
    }
}
