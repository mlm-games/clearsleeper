//! The Tower of the Gods monuments - `fpcNm_Obj_HSH_e`, `d_a_obj_hsehi1.cpp`, the actor behind the
//! two stage strings `Hsh` and `Hsh2`.
//!
//! COLLISION NOTE: `src/sim/event.rs:887` tests `staff.name == "Hsh"` in the event cut dispatcher.
//! That is a staff-name comparison, not a stage-placement claim: no `spawn_actor` branch in
//! `src/sim/mod.rs` tests `"Hsh"` or `"Hsh2"`, and this file is their only claimant. The staff name
//! below is the same string retail uses (`dComIfGp_evmng_getMyStaffId("Hsh", NULL, 0)`, `:509`).
//!
//! `daObj_hsh_c::argument` is `fopAc_ac_c::argument` (`f_op_actor.h:282`), which `dStage_actorCreate`
//! fills from the OBJNAME table's third column (`d_stage.cpp:1306`, `d_stage.h:992`). `Hsh` carries
//! `0` and `Hsh2` carries `1` (`d_stage.cpp:1138-1139`), so `argument` is a property of the NAME,
//! not of the stage parameter - it is passed to `spawn` by the wiring rather than decoded.
//!
//! `argument == 0` is the Command-Melody monument ("Takuto (wind waker) stone monument", `:243`)
//! and `argument == 1` the plain message monument ("Message stone monument", `:247`).
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE. `daObj_hsh_c` registers no `dCcD`
//! collider - `mObjAcch` / `mAcchCir` are `dBgS` probes (`:271`) and `mpBgW` is a `dBgW` moving
//! background (RULE 3b) - and the profile `Group` is `fopAc_ACTOR_e` (`:861`). Neither
//! `combat::HitPoints` nor `enemy::Hurt` is attached; attaching them would let `combat::control`
//! despawn an actor retail cannot damage.
//!
//! `daObj_hsh_IsDelete` (`:825-827`) is the bare "accepts a delete request" form of RULE 6b. The two
//! real destructions are `deleteAction`'s `fopAcM_delete(this)` (`:397`), transcribed, and
//! `create`'s `cPhs_ERROR_e` (`:221-223`), discharged on the first tick per RULE 6b.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::{Actor, atan2s};
use super::collision::Collision;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, OrderFlags};
use super::ground;
use super::player::Player;

/// `d_stage.cpp:1138-1139`: `OBJNAME("Hsh", fpcNm_Obj_HSH_e, 0, 54)` and
/// `OBJNAME("Hsh2", fpcNm_Obj_HSH_e, 1, 54)`. The object archives are `"Hsehi1"` (`argument == 0`,
/// `:228`) and `"Hsehi2"` (`argument != 0`, `:230`) - neither is a stage string.
pub const HSH_ACTORS: [&str; 2] = ["Hsh", "Hsh2"];

/// `dComIfGp_evmng_getMyStaffId("Hsh", NULL, 0)` (`:509`) - the staff cut table `cut_name_tbl`
/// (`:491-500`) is then indexed by this actor.
const STAFF: &str = "Hsh"; // <- d_a_obj_hsehi1.cpp:509

/// `argument == 0`, from `OBJNAME("Hsh", ..., 0, 54)` (`d_stage.cpp:1138`). The sibling row
/// `OBJNAME("Hsh2", ..., 1, 54)` (`d_stage.cpp:1139`) gives `argument == 1`; that value is only
/// ever compared against zero, so it is left for the wiring to pass rather than stored as a
/// constant with a single reader.
const ARG_MONUMENT: i8 = 0; // <- d_stage.cpp:1138

/// `hio_prm_c::mAttnDist = 250.0f` (`:22`), read at `:451`.
const ATTN_DIST: f32 = 250.0; // <- d_a_obj_hsehi1.cpp:451 (l_HIO.prm.mAttnDist, init :22)
/// `hio_prm_c::mAttnAngle = 0x4000` (`:24`), read at `:452`.
const ATTN_ANGLE: i16 = 0x4000; // <- d_a_obj_hsehi1.cpp:452 (l_HIO.prm.mAttnAngle, init :24)
/// `if (mAttentionLatch != 0) { dist += 40.0f; ... }` (`:460`).
const ATTN_DIST_LATCH: f32 = 40.0; // <- d_a_obj_hsehi1.cpp:460
/// `if (mAttentionLatch != 0) { ...; angle += 0x71C; }` (`:461`).
const ATTN_ANGLE_LATCH: i16 = 0x71C; // <- d_a_obj_hsehi1.cpp:461

/// `dSv_event_flag_c::UNK_2510` = `0x2510` (`d_save_event_flag.inc:283`), tested at `:221`.
const FLAG_2510: u32 = 0x2510; // <- d_a_obj_hsehi1.cpp:221
/// `dSv_event_flag_c::UNK_2B10` = `0x2B10` (`d_save_event_flag.inc:329`) - "Command melody monument
/// in Tower of the Gods has been revealed". Read at `:277` and `:768`, set at `:652`.
const FLAG_2B10: u32 = 0x2B10; // <- d_a_obj_hsehi1.cpp:277, :652, :768
/// `fopAcM_seStartCurrent(this, JA_SE_OBJ_ST_TAKT_GR_ON, 0)` (`:656` and `:677`).
const SE_TAKT_GR_ON: u32 = 0x6A05; // <- JAZelAudio_SE.h:2028 (JA_SE_OBJ_ST_TAKT_GR_ON)

/// `i_this->mAppearDeleteTimer = 30;` (`:658`), `initialAppearEvent`.
const APPEAR_TIMER: i32 = 30; // <- d_a_obj_hsehi1.cpp:658
/// `i_this->mAppearDeleteTimer = 60;` (`:679`), `initialDeleteEvent`.
const DELETE_TIMER: i32 = 60; // <- d_a_obj_hsehi1.cpp:679

/// `i_this->field_0x514 = -1;` (`:268`) - "nothing latched yet".
const LATCH_NONE: i16 = -1; // <- d_a_obj_hsehi1.cpp:268
/// `if (field_0x514 == -1) { field_0x514 = 3; }` (`:346`) - the message monument's plain talk latch.
const LATCH_TALK: i16 = 3; // <- d_a_obj_hsehi1.cpp:346
/// `if (field_0x514 == -1) { field_0x514 = 5; }` (`:338`) - the Command-Melody item latch.
const LATCH_ITEM: i16 = 5; // <- d_a_obj_hsehi1.cpp:338

/// `eventInfo.onCondition(dEvtCnd_CANTALK_e)` (`:406`, `:412`) is `dEvtCnd_CANTALK_e`
/// (`f_op_actor.h:145`).
const EVT_CND_CANTALK: u16 = 0x0001; // <- d_a_obj_hsehi1.cpp:406 (dEvtCnd_CANTALK_e)
/// `eventInfo.onCondition(dEvtCnd_CANTALKITEM_e)` (`:411`) is `dEvtCnd_CANTALKITEM_e`
/// (`f_op_actor.h:150`).
const EVT_CND_CANTALKITEM: u16 = 0x0020; // <- d_a_obj_hsehi1.cpp:411 (dEvtCnd_CANTALKITEM_e)

/// `fopAc_CULLBOX_0_e` (`:862`) resolves through `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`);
/// this file calls neither `fopAcM_setCullSizeBox` nor `fopAcM_SetMin` / `SetMax`.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// The monument's own action (`mAction`), not its event cut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Wait,
    Off,
    Delete,
}

/// `cut_name_tbl` (`:491-500`), indexed by `dComIfGp_evmng_getMyActIdx` at `:513`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cut {
    Wait,
    Disp,
    MsgSet,
    Talk,
    Tact,
    Judge,
    Appear,
    Delete,
}

impl Cut {
    fn of(name: &str) -> Option<Self> {
        match name {
            "WAIT" => Some(Self::Wait),
            "Disp" => Some(Self::Disp),
            "MsgSet" => Some(Self::MsgSet),
            "Talk" => Some(Self::Talk),
            "Tact" => Some(Self::Tact),
            "Judge" => Some(Self::Judge),
            "Appear" => Some(Self::Appear),
            "Delete" => Some(Self::Delete),
            _ => None,
        }
    }

    /// `event_init_tbl` (`:469-478`): `[0] initialDefault`, `[1] initialLinkDispEvent`,
    /// `[2] initialMsgSetEvent`, `[3] initialDefault`, `[4] initialDefault`, `[5]
    /// initialJudgeEvent`, `[6] initialAppearEvent`, `[7] initialDeleteEvent`.
    ///
    /// `initialDefault` (`:547-548`) is empty. `initialLinkDispEvent` (`:556-590`) reads the staff
    /// strings `target` / `disp` and toggles the player's and the actor's draw flags - renderer.
    /// `initialMsgSetEvent` (`:593-604`), `initialJudgeEvent` (`:638-648`) and the two
    /// `particle_set` bursts (`:653-654`, `:674-675`) are the message box, the Tact check and the
    /// particle system.
    fn init(&self, shrine: &mut Hsh, flags: &mut EventFlags, events: &mut SimEvents) {
        match self {
            Self::Appear => {
                // `dComIfGs_onEventBit(dSv_event_flag_c::UNK_2B10);` (:652)
                flags.set(FLAG_2B10);
                // `fopAcM_seStartCurrent(this, JA_SE_OBJ_ST_TAKT_GR_ON, 0);` (:656)
                events.list.push(SimEvent::Se(SE_TAKT_GR_ON));
                // `i_this->mAppearDeleteTimer = 30;` (:658)
                shrine.timer = APPEAR_TIMER;
                // `setAction(&daObj_hsh_c::waitAction, NULL);` (:659)
                shrine.set_mode(Mode::Wait);
            }
            Self::Delete => {
                // `fopAcM_seStartCurrent(this, JA_SE_OBJ_ST_TAKT_GR_ON, 0);` (:677)
                events.list.push(SimEvent::Se(SE_TAKT_GR_ON));
                // `i_this->mAppearDeleteTimer = 60;` (:679)
                shrine.timer = DELETE_TIMER;
                // `setAction(&daObj_hsh_c::deleteAction, NULL);` (:680)
                shrine.set_mode(Mode::Delete);
            }
            Self::Wait | Self::Disp | Self::MsgSet | Self::Talk | Self::Tact | Self::Judge => {}
        }
    }

    /// `event_action_tbl` (`:480-489`): `[0]` and `[1]` are `actionDefault`, `[2]` is
    /// `actionMsgSetEvent`, `[3]` is `actionMessageEvent`, `[4]` is `actionTactEvent`, `[5]` is
    /// `actionDefault`, `[6]` is `actionAppearEvent`, `[7]` is `actionDeleteEvent`.
    ///
    /// `actionDefault` (`:551-553`) is a bare `TRUE`. `actionAppearEvent` (`:663-671`) and
    /// `actionDeleteEvent` (`:684-691`) both end the cut once `cLib_calcTimer` reports zero, after
    /// `offOffDraw()` / `drawStop()` - which are the draw-bit and emitter calls.
    fn act(&self, shrine: &mut Hsh) -> bool {
        match self {
            Self::Wait | Self::Disp | Self::Tact | Self::Judge => true,
            // `actionMsgSetEvent` (`:607-609`) is `talk_init()` and `actionMessageEvent`
            // (`:612-619`) is `talk(talkMode)`; both drive `fopMsgM_messageSet`, which this sim
            // does not model, so their real return values cannot be evaluated here.
            Self::MsgSet | Self::Talk => false,
            Self::Appear | Self::Delete => calc_timer(&mut shrine.timer),
        }
    }
}

/// `cLib_calcTimer` (`c_lib.h:85-91`), as `actionAppearEvent` (`:664`) and `actionDeleteEvent`
/// (`:685`) call it.
fn calc_timer(timer: &mut i32) -> bool {
    if *timer != 0 {
        *timer -= 1;
    }
    *timer == 0
}

/// The Tower of the Gods monument.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hsh {
    /// `argument` (`f_op_actor.h:282`), taken from the OBJNAME row, not from the stage parameter.
    pub argument: i8,
    /// `mAction` (`:298`) - which of `waitAction` / `offAction` / `deleteAction` runs.
    mode: Mode,
    /// `mActionMode` (`:315`), reset to 0 by `setAction` (`:315`) and to -1 by its old-action call
    /// (`:308`). Every arm here treats -1 as "do nothing", so only 0 and 1 are observable.
    pub step: i32,
    /// `field_0x514` (`:268`, `-1` at init), latched by `waitAction` (`:338`, `:346`).
    pub latch: i16,
    /// `mAttentionLatch` (`d_a_obj_hsehi1.h`), assigned by `waitAction` (`:332`) and read by
    /// `chkAttention` (`:459`).
    pub attention: bool,
    /// `mAppearDeleteTimer` (`:658`, `:679`).
    pub timer: i32,
    /// One-shot guard: `create`'s `cPhs_ERROR_e` (`:221-223`) and `init`'s action choice
    /// (`:276-285`) both read the `EventFlags` resource, which `spawn` cannot see, so both are
    /// evaluated on the first tick instead.
    armed: bool,
}

impl Hsh {
    /// `create` (`:211-256`) and `init` (`:264-294`), minus `fopAcM_ct_Retail` (`:214`),
    /// `fopAcM_ct_Demo` (`:233`), the `dComIfG_resLoad` (`:227-231`), `fopAcM_entrySolidHeap` /
    /// `checkCreateHeap` / `createHeap` (`:234-239`, `:164-203`), `fopAcM_SetMtx` (`:240`), the
    /// `mDoHIO_createChild` (`:241-250`), `setBaseMtx` (`:272`), the `dComIfG_Bgsp()->Regist` /
    /// `mpBgW->Move()` (`:273-274`), the `mEventId[i] = dComIfGp_evmng_getEventIdx(...)`
    /// (`:287-289`, the RULE 3b-absent stage event-info table) and the
    /// `eventInfo.setXyCheckCB` / `setXyEventCB` installs (`:291-292`) - archive, heap, HIO,
    /// moving-background and event-info work.
    ///
    /// The stage parameter IS decoded by retail - `mSwitchNo = fopAcM_GetParam(this) & 0xff`
    /// (`:265`) and `mPrmMsgNo = (fopAcM_GetParam(this) >> 8) & 0xffff` (`:266`) - but both fields
    /// have exactly one reader each, `dComIfGs_onSwitch(mSwitchNo, roomNo)` (`:720`) and
    /// `next_msgStatus`'s `msg == mPrmMsgNo` (`:757`), and both sit inside the `fopMsgM` message
    /// box. With that omitted neither field has a reader, so RULE 12b drops them and no shift or
    /// mask is stored.
    pub fn spawn(_parameters: u32, argument: i8) -> Self {
        Self {
            argument,
            mode: Mode::Wait,
            step: 0,
            latch: LATCH_NONE,
            attention: false,
            timer: 0,
            armed: true,
        }
    }

    /// `fopAc_CULLBOX_0_e` (`:862`) -> `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `setAction` (`:306-325`). The old action is invoked with `mActionMode = -1` first
    /// (`:307-311`), which every one of `waitAction` / `offAction` / `deleteAction` treats as a
    /// no-op, so only the reset (`:315-320`) and the immediate new-action call (`:322`) survive.
    fn set_mode(&mut self, mode: Mode) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        self.step = 0;
        self.run_mode();
    }

    /// `action` (`:297-303`) minus its `if (!mAction)` guard, which never fires because
    /// `create` / `init` always install an action. Returns `true` for `deleteAction`'s
    /// `fopAcM_delete(this)` (`:397`), a real despawn.
    ///
    /// `waitAction` (`:328-354`) and `offAction` (`:385-390`) share the mode-0 arm verbatim; only
    /// Wait's mode-1 arm, the attention latch, has a body, and that runs from `control`.
    fn run_mode(&mut self) -> bool {
        match self.mode {
            Mode::Wait | Mode::Off => {
                if self.step == 0 {
                    self.step += 1;
                }
                false
            }
            Mode::Delete => {
                if self.step == 0 {
                    self.step += 1;
                    false
                } else {
                    true
                }
            }
        }
    }

    /// `waitAction`'s latch half, `:332-350`.
    ///
    /// `mAttentionLatch = chkAttention(current.pos, shape_angle.y)` (`:332`),
    /// `if (argument == 0) { if (mAttentionLatch) { ...; if (field_0x514 == -1) field_0x514 = 5; }
    /// else { ... } } else if (mAttentionLatch) { ...; if (field_0x514 == -1) field_0x514 = 3; }
    /// else { ... }` (`:334-350`). The `cLib_onBit` / `cLib_offBit` pairs write
    /// `attention_info.flags`, the attention subsystem RULE 3b lists as absent.
    fn latch_attention(&mut self, attention: bool) {
        self.attention = attention;
        if self.latch != LATCH_NONE {
            return;
        }
        if self.argument == ARG_MONUMENT {
            if attention {
                self.latch = LATCH_ITEM;
            }
        } else if attention {
            self.latch = LATCH_TALK;
        }
    }

    /// `chkAttention` (`:449-467`).
    ///
    /// `f32 dist = l_HIO.prm.mAttnDist;` (`:451`), `int angle = l_HIO.prm.mAttnAngle;` (`:452`),
    /// `delta` / `d` / `targetAngle` (`:454-458`), the latch widening (`:459-462`),
    /// `s16 diff = targetAngle; diff -= i_angleY;` (`:463-464`) and
    /// `return angle > abs(diff) && dist > d;` (`:466`). `abs(diff)` is `abs` on an `int` in retail
    /// (`c_lib.h`), so the `s16` difference is widened before the magnitude is taken (RULE 10).
    fn chk_attention(&self, actor: &Actor, player: Vec3) -> bool {
        let mut dist = ATTN_DIST;
        let mut angle = i32::from(ATTN_ANGLE);
        if self.attention {
            dist += ATTN_DIST_LATCH;
            angle += i32::from(ATTN_ANGLE_LATCH);
        }
        let delta_x = player.x - actor.pos.x;
        let delta_z = player.z - actor.pos.z;
        let reach = (delta_x * delta_x + delta_z * delta_z).sqrt();
        let bearing = atan2s(delta_x, delta_z);
        let diff = bearing.wrapping_sub(actor.shape_angle[1]).unsigned_abs();
        angle > i32::from(diff) && dist > reach
    }

    /// `eventProc` (`:503-537`), reduced to the staff machine.
    ///
    /// `if (eventInfo.checkCommandDemoAccrpt() && field_0x514 != -1) { onEventAccept();
    /// field_0x514 = 0xff; }` (`:504-507`) is the `dEvtCmd_INDEMO_e` order queue, a per-actor
    /// event command this sim does not carry - the same omission `pscnchg.rs:99`,
    /// `hami3.rs:137` and `tagetc.rs:201` document - so `field_0x514` keeps only `-1`, `3` and
    /// `5` and `isEventAccept()` stays false, which also makes `eventProc`'s
    /// `if (isEventAccept()) { ... return TRUE; }` arm (`:526-531`) and `eventOrder`'s
    /// `if (!isEventAccept())` guard (`:404`) trivial. `!checkCommandTalk()` (`:511`) is the
    /// proven-false TALK command.
    fn event_proc(
        &mut self,
        list: &EventList,
        cut: usize,
        advance: bool,
        flags: &mut EventFlags,
        events: &mut SimEvents,
    ) -> bool {
        // `actIdx == -1` (`:514`) is the unmatched cut, and an action's TRUE result ends the cut
        // (`:521-523`); both call `dComIfGp_evmng_cutEnd(staffIdx)`.
        let Some(cut) = list.cut_name(cut).and_then(Cut::of) else {
            return true;
        };
        if advance {
            cut.init(self, flags, events);
        }
        cut.act(self)
    }

    /// `eventOrder` (`:403-422`), reduced to the two reachable `onCondition` calls.
    ///
    /// `field_0x514 == 4` (`:405-409`) would also call `fopAcM_orderSpeakEvent(this)`; nothing in
    /// this translation unit ever writes 4, so that arm is unreachable and dropped. The
    /// `field_0x514 != -1 && field_0x514 < 2` arm (`:416-419`) would call
    /// `fopAcM_orderOtherEventId(this, mEventId[mEventSelector])`, whose id comes from the absent
    /// stage event-info table, and nothing here writes 0 or 1 either. `field_0x514 == 5` also
    /// calls `eventInfo.setEventName("hsehi1_talk")` (`:413-415`), the event-name channel this sim
    /// does not carry.
    fn event_order(&self, order: &mut OrderFlags) {
        if self.latch == LATCH_TALK {
            order.set(EVT_CND_CANTALK);
        } else if self.latch == LATCH_ITEM {
            order.set(EVT_CND_CANTALKITEM);
            order.set(EVT_CND_CANTALK);
        }
    }
}

pub fn is_hsh(name: &str) -> bool {
    HSH_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    mut flags: ResMut<EventFlags>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut order: ResMut<OrderFlags>,
    mut events: ResMut<SimEvents>,
    players: Query<&Player>,
    mut shrines: Query<(Entity, &mut Actor, &mut Hsh)>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let player_pos = players.iter().next().map(|player| player.pos);
    let mut spent: Vec<Entity> = Vec::new();
    for (entity, mut actor, mut shrine) in &mut shrines {
        if shrine.armed {
            shrine.armed = false;
            // `if (argument == 0 && dComIfGs_isEventBit(dSv_event_flag_c::UNK_2510))
            // return cPhs_ERROR_e;` (`:221-223`) - Create refused the process, so the actor never
            // executes even once.
            if shrine.argument == ARG_MONUMENT && flags.test(FLAG_2510) {
                spent.push(entity);
                continue;
            }
            // `if (argument == 0) { if (dComIfGs_isEventBit(UNK_2B10)) setAction(waitAction);
            // else { onOffDraw(); setAction(offAction); } } else { setAction(waitAction); }`
            // (`:276-285`). `onOffDraw` (`:126-131`) is the draw bit plus the `dBgW` release.
            let wait = shrine.argument != ARG_MONUMENT || flags.test(FLAG_2B10);
            shrine.set_mode(if wait { Mode::Wait } else { Mode::Off });
        }
        // `execute` (`:766-793`), minus `fopAcM_SetStatusMap(this, 0x4)` /
        // `fopAcM_OffStatus(this, fopAcStts_SHOWMAP_e)` (`:767-773`), the per-actor status bits
        // RULE 3b lists as absent, and the room / poly-info writes of `:778-782`, which need the
        // `dBgS` room id and poly colour the merged-DZB `Collision` does not expose.
        //
        // `mObjAcch.CrrPos(*dComIfG_Bgsp());` (`:775`) - only the ground response is transcribed;
        // its wall and roof corrections are not, the same reduction `ki.rs:607-622` makes. The
        // `solid.is_some()` guard matters: `ground::surface` answers `Some(0.0)` with no
        // `Collision` (`ground.rs:30-35`), which would otherwise snap the monument to the world
        // origin - `enemy.rs:409,435` guards its own ground path the same way.
        if let Some(solid) = solid
            && let Some(floor) = ground::surface(Some(solid), actor.pos)
            && actor.pos.y < floor
        {
            actor.pos.y = floor;
        }
        // `if (eventProc() == 0) { checkOrder(); action(NULL); }` (`:785-788`) - `checkOrder`
        // (`:425-432`) is the talk-XY order, which needs `dComIfGp_event_chkTalkXY()` and the
        // omitted `field_0x514 = 0xff` writes.
        let owned = match active.staff_cut(&list, STAFF) {
            Some((cut, advance)) => {
                if shrine.event_proc(&list, cut, advance, &mut flags, &mut events) {
                    active.end_staff_cut(&list, &mut flags, STAFF);
                }
                true
            }
            None => false,
        };
        if !owned {
            // `action(NULL);` (`:787`). `waitAction`'s mode-1 arm is the latch half; it reads the
            // player through `dComIfGp_getPlayer(0)` inside `chkAttention` (`:450`).
            if shrine.mode == Mode::Wait
                && shrine.step != 0
                && let Some(player) = player_pos
            {
                let attention = shrine.chk_attention(&actor, player);
                shrine.latch_attention(attention);
            }
            if shrine.run_mode() {
                spent.push(entity);
                continue;
            }
        }
        // `eventOrder();` (`:790`); `setBaseMtx()` (`:791`) is renderer work.
        shrine.event_order(&mut order);
    }
    for entity in spent {
        let _ = commands.entity(entity).despawn();
    }
}
