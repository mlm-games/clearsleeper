use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::area::StageName;
use super::collision::Collision;
use super::event::{ActiveEvent, EventList, Stage};
use super::ground;
use super::player::Player;
use super::transition::{NextStage, Target};

/// `OBJNAME("Ypit00", fpcNm_OBJ_HOLE_e, 0, 0)` and `OBJNAME("Pitfall", fpcNm_OBJ_HOLE_e, 1, 0)`
/// (`d_stage.cpp:812-813`) - `fpcNm_OBJ_HOLE_e` is `d_a_obj_hole.cpp`. The archive is a DIFFERENT
/// string: `daObj_Hole_c::m_arc_name[] = "Aana"` (`d_a_obj_hole.cpp:21`), which appears nowhere in
/// the OBJNAME table, so the stage strings must come from `d_stage.cpp`, not the arcname. Neither
/// `Ypit00` nor `Pitfall` is claimed by any other `*_ACTORS` array in `src/sim/`.
pub const HOLE_ACTORS: [&str; 2] = ["Ypit00", "Pitfall"];

/// `daObj_Hole_HIO_c::daObj_Hole_HIO_c()` (`d_a_obj_hole.cpp:29-35`) sets `m08 = 65.0f`. Unlike
/// `daobj_stairHIO_c` this HIO is NEVER handed to `mDoHIO_createChild` (the file makes no such call
/// and `daObj_Hole_c::_createHeap` (`:218-231`) only builds the model), so nothing in the decomp
/// ever writes it: these are PROVEN values, not stage data.
const HIO_M08: f32 = 65.0; // <- d_a_obj_hole.cpp:32 (daObj_Hole_HIO_c::m08)
/// The remaining `daObj_Hole_HIO_c` constructor values are all PROVEN and all unused here:
/// `m0C = 0` (`d_a_obj_hole.cpp:33`) is what makes `modeWait` take the `mScaleLocal` arm at `:105`
/// rather than `l_HIO.m0C * 10` at `:102`; `m10 = 2.0f` (`:34`) is only the `adjustPos.y +=` lift in
/// `setMtx` (`:43`) and `debugDraw`'s only statement (`:182`); `m05 = 0` (`:31`) gates that debug
/// draw (`:187-189`); `mNo = -1` (`:30`) is the HIO handle, which `daObj_Hole_c` never uses. None is
/// stored, because each has no surviving reader (RULE 12b).

/// `enum Mode { MODE_WAIT = 0, MODE_EVENT = 1, MODE_NULL }` (`d_a_obj_hole.h:15-19`) - the
/// `mode_tbl` index of `modeProc` (`d_a_obj_hole.cpp:150-161`).
const MODE_WAIT: i32 = 0; // <- d_a_obj_hole.h:16
const MODE_EVENT: i32 = 1; // <- d_a_obj_hole.h:17

/// `scaleMag += l_HIO.m0C * 10;` / `scaleMag += mScaleLocal * 10;` (`d_a_obj_hole.cpp:53`, `:56`,
/// `:102`, `:105`) - the local scale multiplier applied to `home.angle.z`.
const LOCAL_SCALE_MULT: f32 = 10.0; // <- d_a_obj_hole.cpp:53
/// `dLib_checkPlayerInCircle(current.pos, scaleMag, 20.0f)` (`d_a_obj_hole.cpp:108`) - the vertical
/// half-height of the trigger cylinder.
const CIRCLE_HALF_HEIGHT: f32 = 20.0; // <- d_a_obj_hole.cpp:108

/// `fopAc_CULLBOX_4_e` (`d_a_obj_hole.cpp:335`) resolves to `l_cullSizeBox[4]`
/// (`f_op_actor_mng.cpp:562`) - a real box, so `cull()` is derivable even though the file makes no
/// `fopAcM_setCullSizeBox` call. `fopAcM_setCullSizeFar(this, 10.0f)` (`d_a_obj_hole.cpp:206`)
/// overrides only `i_actor->cullSizeFar` (`f_op_actor_mng.h:316-318`), so it leaves the box alone.
const CULL_MIN: [f32; 3] = [-100.0, 0.0, -100.0]; // <- f_op_actor_mng.cpp:562 (fopAc_CULLBOX_4_e)
const CULL_MAX: [f32; 3] = [100.0, 800.0, 100.0]; // <- f_op_actor_mng.cpp:562 (fopAc_CULLBOX_4_e)

/// `dComIfGp_evmng_endCheck("DEFAULT_PITFALL")` and `fopAcM_orderOtherEvent(this,
/// "DEFAULT_PITFALL")` (`d_a_obj_hole.cpp:126`, `:136`) - the one event this actor uses.
/// `dComIfGp_evmng_getMyStaffId("Ypit00")` (`:124`) is assigned to a local `staffId` that nothing
/// reads and the class stores no staff id, so that string has no surviving use.
const EVENT_PITFALL: &str = "DEFAULT_PITFALL"; // <- d_a_obj_hole.cpp:126
/// `if (mExitIdx != 0xFF)` (`d_a_obj_hole.cpp:127`) - the discriminator between the two
/// stage-transition arms of `modeEvent`.
const NO_EXIT: u8 = 0xFF; // <- d_a_obj_hole.cpp:127
/// `if (mHasModel == 0xFF)` (`d_a_obj_hole.cpp:49`, `:191`, `:266`) - the only value that carries a
/// model, the only one whose `dBgS_ObjLinChk` line check the renderer uses, and the only one that
/// allocates a heap (`:267`). NOT stored: `getArg`'s `mHasModel = param >> 8` (`:237`) and its
/// `if (mHasModel >= 1) { mHasModel = 0xFF; }` fold (`:244-246`) are transcribed in `spawn` for the
/// record, but every reader is either `setMtx`'s model-matrix branch (`:49`, renderer) or
/// `entrySolidHeap` (`:267`, heap work), so a stored copy would be dead code (RULE 7 / RULE 12b).
/// `fopAcM_GetParamBit(fopAcM_GetParam(this), 0, 16)` (`d_a_obj_hole.cpp:235`) =
/// `(param >> 0) & ((1 << 16) - 1)` over the WHOLE u32 (`f_op_actor_mng.h:127-133`).
const PARAM_SHIFT: u32 = 0; // <- d_a_obj_hole.cpp:235
const PARAM_WIDTH: u32 = 16; // <- d_a_obj_hole.cpp:235

/// The grotto mouths that swallow the player - `daObj_Hole_c` (`d_a_obj_hole.h:8-52`), the actor
/// behind the stage strings `Ypit00` and `Pitfall`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daObj_Hole_c` is a bare `fopAc_ac_c`
/// holding a model, a `dBgS_ObjLinChk`, a `dBgS_ObjAcch` and a `dBgS_AcchCir`
/// (`d_a_obj_hole.h:47-51`); it registers no `dCcD` collider of any kind. `mAcch` is a
/// `dBgS_ObjAcch` - actor-versus-BACKGROUND, not actor-versus-actor - and it is widened by
/// `SetWallNone` / `SetRoofNone` in the same `createInit` (`:211-212`) that sets it, so it does
/// nothing. The profile `Status` is `fopAcStts_UNK40000_e` alone (`:333`) and `Group` is
/// `fopAc_ACTOR_e` (`:334`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `daObjHoleIsDelete` (`:308-311`) is the bare "accepts a delete request" form of RULE 6b. The real
/// destruction is the two stage transitions in `modeEvent` (`:128`, `:131`), not a delete.
/// `_create` can return `cPhs_ERROR_e` only from `fopAcM_entrySolidHeap` returning 0 (`:269-271`),
/// which is heap work, so `spawn` never refuses.
///
/// `setMtx` (`:38-71`) writes `shape_angle.y = fopCamM_GetAngleY(dComIfGp_getCamera(0)) + 0x8000`
/// (`:46`) and then reads all three `shape_angle` components back into the model matrix (`:66-67`).
/// That is renderer work, and it is also the ONLY reader of the camera, so `super::kytag00::
/// CameraEye` - which is write-never and permanently `Vec3::ZERO` and would silently yield a wrong
/// yaw - is deliberately not touched. `triPla->CalcAngleXz(&shape_angle.x, &shape_angle.z)` (`:87`)
/// is dropped for the same reason; `triPla->getCrossY(current.pos, &current.pos.y)` (`:88`) is real
/// and is transcribed.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Hole {
    /// `mMode` (`d_a_obj_hole.h:43`, an `s32`) - the `mode_tbl` index of `modeProc`
    /// (`d_a_obj_hole.cpp:163-169`). `createInit` runs `modeProcInit(MODE_WAIT)` (`:203`) and
    /// `modeWait` escalates to `MODE_EVENT` (`:109`); nothing returns to `MODE_WAIT`.
    pub mode: i32,
    /// `mExitIdx` (`d_a_obj_hole.h:45`, a `u8`) - `param & 0xFF` (`:236`), the scls-table index of
    /// the `dLib_setNextStageBySclsNum` arm (`:128`). Carried because it IS the branch selector
    /// between the two stage-transition arms, even though the scls arm itself is omitted.
    pub exit_idx: u8,
    /// `mScaleLocal` (`d_a_obj_hole.h:46`, a `u16`) - `home.angle.z` with `-1` folded to 0
    /// (`:239-242`). Fed into the trigger radius as `mScaleLocal * 10` (`:105`).
    pub scale_local: u16,
    /// `mOpenEventIdx` equivalent: the `DEFAULT_PITFALL` slot from `getEventIdx(name, 0xff)` (the
    /// two-argument `dComIfGp_evmng_startCheck` / `endCheck` default `evNo` to `-1`, i.e. `0xFF`,
    /// `d_com_inf_game.h:3659`, `:3663`, so both are pure name lookups). `None` is the -1.
    pub event_slot: Option<usize>,
    /// One-shot guard so the `DEFAULT_PITFALL` lookup and `createInit`'s `modeProcInit(MODE_WAIT)`
    /// - which reaches `getPosAndAngle` - run once, on the first tick, where `Collision` is
    /// available.
    pub entered: bool,
}

impl Hole {
    /// `daObj_Hole_c::_create` (`:250-279`) plus `getArg` (`:234-247`), minus `fopAcM_ct` (`:257`),
    /// `dComIfG_resLoad` (`:260`), `fopAcM_entrySolidHeap` / `_createHeap` (`:267`, `:218-231`),
    /// `fopAcM_SetMtx` (`:205`), `setMtx` (`:214`) and the whole of `createInit`'s `dBgS` setup
    /// (`:208-212`).
    ///
    /// The `DEFAULT_PITFALL` name has no `getEventIdx` in retail - `modeEvent` resolves it by name
    /// at both use sites - but the sim's `EventList` is the same lookup either way, so it is done
    /// once on the first tick rather than twice per tick.
    pub fn spawn(parameters: u32, angle_z: i16) -> Self {
        // `u16 param = fopAcM_GetParamBit(fopAcM_GetParam(this), 0, 16);` (:235)
        let param = ((parameters >> PARAM_SHIFT) & ((1 << PARAM_WIDTH) - 1)) as u16;
        // `mExitIdx = param & 0xFF;` (:236)
        let exit_idx = (param & 0xFF) as u8;
        // `mHasModel = param >> 8;` (:237) then `if (mHasModel >= 1) { mHasModel = 0xFF; }`
        // (:244-246) - transcribed for the record and then deliberately dropped: see the note on
        // [`NO_EXIT`] above and RULE 7 / RULE 12b.
        let _has_model = match (param >> 8) as u8 {
            0 => 0u8,
            _ => 0xFFu8,
        };
        // `mScaleLocal = home.angle.z; if (mScaleLocal == 0xFFFF) { mScaleLocal = 0; }` (:239-242).
        // `home.angle` is `csXyz` (`d_a_obj_hole.h:47` is the next member), so `.z` is the s16 the
        // DZS carries; the store into the `u16` reinterprets -1 as 0xFFFF, which is the test's
        // subject, and a negative value other than -1 wraps - both copied verbatim.
        let scale_local = if angle_z == -1 { 0 } else { angle_z as u16 };
        Self {
            mode: MODE_WAIT,
            exit_idx,
            scale_local,
            event_slot: None,
            entered: false,
        }
    }

    /// `fopAc_CULLBOX_4_e` (`d_a_obj_hole.cpp:335`) -> `l_cullSizeBox[4]`
    /// (`f_op_actor_mng.cpp:562`). `fopAcM_setCullSizeFar(this, 10.0f)` (`:206`) overrides only the
    /// far plane (`f_op_actor_mng.h:316-318`), so the box above is untouched - `setCullSizeFar` is
    /// cited as [`CULL_FAR`] because the sim's `cull()` channel carries the box alone.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `createInit` (`:202-215`) - only its `modeProcInit(MODE_WAIT)` (`:203`) survives, because
    /// `modeProc(PROC_INIT_e, MODE_WAIT)` (`:163-166`) calls `modeWaitInit` (`:93-95`), which calls
    /// `getPosAndAngle` (`:74-90`) and needs `Collision`.
    fn create_init(&mut self, actor: &mut Actor, list: &EventList, collision: Option<&Collision>) {
        self.event_slot = list.event_index(EVENT_PITFALL);
        self.mode_proc_init(actor, collision);
        self.entered = true;
    }

    /// `modeProcInit(int newMode) { modeProc(PROC_INIT_e, newMode); }` (`d_a_obj_hole.h:23`) - the
    /// `PROC_INIT_e` arm of `modeProc` (`:163-166`): `mMode = newMode; (this->*mode_tbl[mMode].init)()`.
    fn mode_proc_init(&mut self, actor: &mut Actor, collision: Option<&Collision>) {
        self.mode = MODE_WAIT;
        // `mode_tbl[MODE_WAIT].init = &daObj_Hole_c::modeWaitInit` (:152) and `modeWaitInit()` is
        // `{ getPosAndAngle(); }` (:93-95).
        self.get_pos_and_angle(actor, collision);
    }

    /// `daObj_Hole_c::getPosAndAngle` (`:74-90`).
    ///
    /// ```
    /// cXyz posUp(0.0f, 10.0f, 0.0f);   posUp += current.pos;
    /// cXyz posDown(0.0f, -100.0f, 0.0f); posDown += current.pos;
    /// csXyz unused_angle(0, 0, 0);
    /// mLinChk.Set(&posUp, &posDown, this);
    /// if (dComIfG_Bgsp()->LineCross(&mLinChk) != 0) {
    ///     cM3dGPla* triPla = dComIfG_Bgsp()->GetTriPla(mLinChk);
    ///     triPla->CalcAngleXz(&shape_angle.x, &shape_angle.z);
    ///     triPla->getCrossY(current.pos, &current.pos.y);
    /// }
    /// ```
    ///
    /// `dComIfG_Bgsp()->LineCross` is a merged-DZB query, which is what `super::collision::
    /// Collision` holds, so `ground::surface` is the house reduction of it. A missing `Collision`
    /// resource is read as "no plane", i.e. no `LineCross` hit, exactly as retail's `!= 0` guard -
    /// which is why `ground::surface`'s own `None -> Some(0.0)` default must not be reached here.
    fn get_pos_and_angle(&mut self, actor: &mut Actor, collision: Option<&Collision>) {
        let Some(collision) = collision else {
            return;
        };
        let Some(cross) = ground::surface(Some(collision), actor.pos) else {
            return;
        };
        actor.pos.y = cross;
    }

    /// `daObj_Hole_c::_execute` (`:173-177`): `modeProc(PROC_EXEC_e, MODE_NULL); setMtx(); return
    /// false;`. Only the `PROC_EXEC_e` dispatch survives (`setMtx` is model work, and the `return
    /// false` is discussed in OMITTED).
    fn execute(&mut self, actor: &mut Actor, ctx: &mut Context<'_>) {
        match self.mode {
            MODE_WAIT => self.mode_wait(actor, ctx),
            MODE_EVENT => self.mode_event(ctx),
            _ => {}
        }
    }

    /// `daObj_Hole_c::modeWait` (`:98-111`).
    fn mode_wait(&mut self, actor: &Actor, ctx: &Context<'_>) {
        // ```
        // f32 scaleMag = l_HIO.m08 * scale.x;
        // if (l_HIO.m0C != 0) { scaleMag += l_HIO.m0C * 10; } else { scaleMag += mScaleLocal * 10; }
        // if (dLib_checkPlayerInCircle(current.pos, scaleMag, 20.0f)) { modeProcInit(MODE_EVENT); }
        // ```
        // (:99-110). RULE 2 makes `scale.x` exactly 1.0, and `l_HIO.m0C` is the PROVEN 0 of
        // `daObj_Hole_HIO_c`'s constructor (`:33`) - nothing ever writes that HIO - so the `else`
        // arm is the live one.
        let scale_mag = HIO_M08 + f32::from(self.scale_local) * LOCAL_SCALE_MULT;
        if Self::player_in_circle(actor.pos, scale_mag, CIRCLE_HALF_HEIGHT, ctx.player) {
            // `modeProcInit(MODE_EVENT)` (:109) -> `modeProc(PROC_INIT_e, MODE_EVENT)` -> `mMode = 1;
            // (this->*mode_tbl[1].init)()` = `modeEventInit()`, whose body is `return;` (`:117-119`).
            self.mode = MODE_EVENT;
        }
    }

    /// `daObj_Hole_c::modeEvent` (`:122-138`).
    ///
    /// `int staffId = dComIfGp_evmng_getMyStaffId("Ypit00");` (`:124`) is assigned to a local that
    /// nothing reads, so it is cited as [`STAFF_PITFALL`] and transcribed nowhere.
    fn mode_event(&mut self, ctx: &mut Context<'_>) {
        // `if (eventInfo.checkCommandDemoAccrpt()) { ... }` (:123). That latch is set on this actor
        // precisely because the `else` arm ordered `DEFAULT_PITFALL`, so "the event this actor
        // ordered is in Play or Close" is the exact reading.
        if !Self::demo_running(ctx.active, self.event_slot) {
            // `fopAcM_orderOtherEvent(this, "DEFAULT_PITFALL");` (:136) - re-ordering an event that
            // is already ORDER / PLAY / CLOSE rewrites its state (`d_event_manager.cpp:367`), so
            // the guard the house uses everywhere is required.
            if let Some(slot) = self.event_slot {
                if ctx.active.stage_of(slot).is_none() {
                    let _ = ctx.active.order(ctx.list, slot, ctx.entity);
                }
            }
            return;
        }
        // `if (dComIfGp_evmng_endCheck("DEFAULT_PITFALL")) { ... }` (:126) - `endCheckOld(name)` is
        // `endCheck(getEventIdx(name, 0xFF))` (`d_event_manager.cpp:395-397`), i.e. the event's
        // `mEventState == dEvDtEvent_c::CLOSE_e` (`:386-392`).
        if !self
            .event_slot
            .is_some_and(|slot| ctx.active.end_check(slot))
        {
            return;
        }
        if self.exit_idx != NO_EXIT {
            // `dLib_setNextStageBySclsNum(mExitIdx, current.roomNo);` (:128) - OMITTED: that call
            // resolves `i_sclsnum` through the stage's `stage_scls_info_class` table
            // (`d_lib.cpp:236-258`), which is stage data this checkout does not carry. See the
            // report's OMITTED list.
        } else {
            // `dComIfGp_setNextStage(dComIfGp_getStartStageName(), 0, current.roomNo);` (:131) -
            // the 4th..8th arguments all take their defaults (`d_com_inf_game.h:2237-2239`), and
            // `dComIfGp_getStartStageName()` (`d_com_inf_game.h:2247-2249`) is the current DZS stage
            // name, which `super::area::StageName` carries.
            ctx.next.target = Some(Target {
                stage: ctx.stage.0.clone(),
                room: ctx.room as u8,
                start: 0,
            });
        }
    }

    /// `dLib_checkPlayerInCircle(current.pos, scaleMag, 20.0f)` (`d_a_obj_hole.cpp:108`) =
    /// `dLib_checkActorInCircle` (`d_lib.cpp:269-274`): `distXZ = (center - actor->current.pos)
    /// .absXZ(); distY = fabs(center.y - actor->current.pos.y); return distXZ < radius && distY <
    /// halfHeight;` (`d_lib.cpp:277-284`).
    fn player_in_circle(center: Vec3, radius: f32, half_height: f32, player: Option<Vec3>) -> bool {
        let Some(player) = player else {
            return false;
        };
        let delta = center - player;
        let dist_xz = (delta.x * delta.x + delta.z * delta.z).sqrt();
        dist_xz < radius && (center.y - player.y).abs() < half_height
    }

    /// `eventInfo.checkCommandDemoAccrpt()` as `Stage::Play | Stage::Close` - the two states
    /// `dEvent_manager_c::getMyStaffId` scans (`d_event_manager.cpp:412`).
    fn demo_running(active: &ActiveEvent, slot: Option<usize>) -> bool {
        slot.is_some_and(|slot| {
            matches!(active.stage_of(slot), Some(Stage::Play) | Some(Stage::Close))
        })
    }
}

/// Everything `execute` needs beyond the component, in one bundle so the state machine has a single
/// reader - the `hbrf1` / `tagetc` shape.
struct Context<'a> {
    player: Option<Vec3>,
    list: &'a EventList,
    active: &'a mut ActiveEvent,
    next: &'a mut NextStage,
    stage: &'a StageName,
    room: i8,
    entity: Entity,
}

pub fn is_hole(name: &str) -> bool {
    HOLE_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut next: ResMut<NextStage>,
    stage: Res<StageName>,
    players: Query<&Player>,
    mut holes: Query<(Entity, &mut Actor, &mut Hole)>,
) {
    let player = players.iter().next().map(|link| link.pos);
    for (entity, mut actor, mut hole) in &mut holes {
        let actor = &mut *actor;
        let room = actor.room;
        if !hole.entered {
            hole.create_init(actor, &list, collision.as_deref());
        }
        let mut ctx = Context {
            player,
            list: &list,
            active: &mut active,
            next: &mut next,
            stage: &stage,
            room,
            entity,
        };
        hole.execute(actor, &mut ctx);
    }
}