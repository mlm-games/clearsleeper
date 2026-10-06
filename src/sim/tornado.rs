use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::EventFlags;
use super::player::Player;
use super::player_status::{PlayerStatus, STTS0_SHIP_RIDE};
use super::ship::SHIP_ACTOR;
use super::walk;
use crate::text::msgproc::MsgGameInfo;

/// `OBJNAME("Ytrnd00", fpcNm_TORNADO_e, 255, 58)` (`d_stage.cpp:987`) is `fpcNm_TORNADO_e` =
/// `d_a_tornado.cpp`. The archive is a DIFFERENT string - `static char l_arcName[] = "Trnd"`
/// (`d_a_tornado.cpp:15`) - and so are the two models it loads, `BDL_YTRND00` (`:255`) and
/// `BDL_YWUWT00` (`:267`), so the OBJNAME row above is the authority, not the arcname.
///
/// `src/sim/sage.rs:27` also holds `const TORNADO_ACTOR: &str = "Ytrnd00";` - that is NOT a stage
/// claim but a spawn TARGET: `sage.rs:872-875` uses it to FIND the tornado entity the King of Red
/// Lions rides. It is in no `*_ACTORS` array, so no stage string collides.
pub const TORNADO_ACTORS: [&str; 1] = ["Ytrnd00"];

/// `cLib_addCalcAngleS(&current.angle.y, target, 10, 0x1000, 0x100)` (`d_a_tornado.cpp:148`) - the
/// turn toward the player while no event is running.
const TURN_SCALE: i16 = 10; // <- d_a_tornado.cpp:148
const TURN_MAX_STEP: i16 = 0x1000; // <- d_a_tornado.cpp:148
const TURN_MIN_STEP: i16 = 0x100; // <- d_a_tornado.cpp:148
/// `cLib_chaseF(&speedF, 20.0f, 0.2f)` (`d_a_tornado.cpp:149`) - the cruise speed.
const CRUISE_SPEED: f32 = 20.0; // <- d_a_tornado.cpp:149
const CRUISE_STEP: f32 = 0.2; // <- d_a_tornado.cpp:149
/// `if (diff.abs2XZ() < SQUARE(10000.0f))` (`d_a_tornado.cpp:154`) - the radius inside which a
/// riding ship is captured. It equals `daTornado_HIO_c0::start_dis` (`:18`) in value, but `execute`
/// uses the bare literal at `:154`, so the literal is what is cited.
const CAPTURE_RADIUS: f32 = 10000.0; // <- d_a_tornado.cpp:154
/// `if (diff.abs2XZ() > SQUARE(7500.0f)) { diff.normalize(); current.pos = home.pos + diff *
/// 7500.0f; }` (`d_a_tornado.cpp:164-166`) - the leash, again a bare literal at both use sites and
/// equal in value to `daTornado_HIO_c0::move_dis` (`:17`).
const LEASH_RADIUS: f32 = 7500.0; // <- d_a_tornado.cpp:164
/// `mAngle2 += 250;` (`d_a_tornado.cpp:112`) and `mAngle1 += 500;` (`:113`) - the two joint phases,
/// both `s16` (`d_a_tornado.h:56-57`).
const ANGLE2_STEP: i16 = 250; // <- d_a_tornado.cpp:112
const ANGLE1_STEP: i16 = 500; // <- d_a_tornado.cpp:113
/// `m32c = 10000.0f;` (`d_a_tornado.cpp:114`) is written every tick and read NOWHERE in the decomp,
/// and is a distinct field from the `s16 m31c` that `setScaleOn` touches (`d_a_tornado.h:37`). Dead,
/// not stored.
/// `mPtclTimer = 10;` (`d_a_tornado.cpp:173`) is the wind-particle cadence; the particle is RULE 3b
/// absent, so the timer has no surviving reader either.
/// `static cXyz wind_scale(20.0f,20.0f,20.0f);` (`d_a_tornado.cpp:100`) feeds only
/// `dComIfGp_particle_set` (`:174`), and `current.pos.y -= 2000.0f` (`:303`) is the `param == 1`
/// create variant that nothing in the decomp spawns.
const SE_TORNADE_SUS: u32 = 0x106B; // <- d_a_tornado.cpp:177 (JA_SE_OBJ_TORNADE_SUS, JAZelAudio_SE.h:302)
/// `dComIfGs_isEventBit(dSv_event_flag_c::UNK_2710)` (`d_a_tornado.cpp:179`, `:314`,
/// `d_save_event_flag.inc:297`) - "the Ballad of Gales tornado has done its job". It arms BOTH the
/// `create` refusal (`:314-316`) and `execute`'s stand-down arm (`:179-191`).
const EVT_2710: u32 = 0x2710; // <- d_save_event_flag.inc:297
/// `fopAcM_SetParam(this, 0)` (`d_a_tornado.cpp:313`) - the `else` arm rewrites its own parameter,
/// so `param` is 0 for the rest of that actor's life and only the `if (param == 0)` arm (`:145`)
/// is reachable for it.
const PARAM_DEFAULT: u32 = 0; // <- d_a_tornado.cpp:313
/// `mBtkFrame += 1.0f` (`d_a_tornado.cpp:103`) and `mBtkUnderFrame += 1.0f` (`:108`) - the two BTK
/// frame indices. Their advance IS transcribed; the wrap needs `getBtkAnm()->getFrameMax()` from
/// the BTK archive (RULE 3b) and is not.
const BTK_STEP: f32 = 1.0; // <- d_a_tornado.cpp:103
/// `mBrkFrame += 0.3f;` (`d_a_tornado.cpp:185`) - the stand-down tev-register dissolve.
const BRK_STEP: f32 = 0.3; // <- d_a_tornado.cpp:185

/// Ballad of Gales' tornado - `daTornado_c` (`d_a_tornado.h:15-69`), the actor behind the stage
/// string `Ytrnd00`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. Checked, not assumed, because a tornado is
/// exactly the sort of thing one would expect to be damageable: `daTornado_c` is a bare
/// `fopAc_ac_c` holding two models, six animators, the joint arrays, `mCenter` and a
/// `dPa_followEcallBack` (`d_a_tornado.h:47-68`) and it registers NO `dCcD` collider - no `dCcD_Stts`,
/// no `dCcD_Cyl`, no `dCcD_Src*` anywhere in the translation unit. The profile `Group` is
/// `fopAc_ACTOR_e` (`:361`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached - attaching
/// them would let `combat::control` despawn a tornado retail cannot damage.
///
/// `daTornado_IsDelete` (`:232-235`) is the bare "accepts a delete request" form of RULE 6b. The
/// real destruction is `fopAcM_delete(this)` at `:187` and `:207`. The `:207` one needs
/// `m31c != 0` (`:200`), and `m31c`'s ONLY writer is `setScaleOn()` (`d_a_tornado.h:37`) called
/// from `d_a_ship.cpp:2742` and `:2832` - an un-ported ship path - so it holds the `fopAcM_ct`
/// zero-fill (`f_pc_base.cpp:98`) and that arm is DEAD. The `:187` one needs a BRK frame count from
/// the archive. Neither delete is reachable here; see OMITTED.
///
/// `create` (`:288-333`) can return `cPhs_ERROR_e` from `fopAcM_entrySolidHeap` (`:297-299`, heap
/// work) and from the `else` arm's `if (isEventBit(UNK_2710)) { return cPhs_ERROR_e; }`
/// (`:314-316`). That second one is a REAL refusal, and like every `cPhs_ERROR_e` it cancels the
/// actor inside Create (`f_pc_create_req.cpp:106-108`), so it is a spawn-then-despawn on the FIRST
/// TICK and never `Option<Self> -> None` (RULE 6b).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Tornado {
    /// `fopAc_ac_c::speedF` (`d_a_tornado.h:51`) - the cruise speed, chased to [`CRUISE_SPEED`] at
    /// `:149` and ZEROED at `:156` once a riding ship is captured.
    pub speed_f: f32,
    /// `home.pos` (`f_op_actor.h:288`), the placement position. `fopAc_Init` seeds both `current`
    /// and `old` from it (`f_op_actor.cpp:282-284`), and `execute` measures the leash from it
    /// (`:162`, `:166`).
    pub home: Vec3,
    /// `mAngle1` (`d_a_tornado.h:56`, an `s16`) - advanced by [`ANGLE1_STEP`] at `:113`, read only
    /// at `:138` on the joint-matrix path, which is renderer work.
    pub angle1: i16,
    /// `mAngle2` (`d_a_tornado.h:57`, an `s16`) - advanced by [`ANGLE2_STEP`] at `:112`, read only
    /// at `:139-141`, likewise renderer-only.
    pub angle2: i16,
    /// `mBtkFrame` (`d_a_tornado.h:60`, an `f32`) - advanced at `:103-106`.
    pub btk_frame: f32,
    /// `mBrkFrame` (`d_a_tornado.h:62`, an `f32`) - advanced by [`BRK_STEP`] at `:185`, in the
    /// stand-down arm only.
    pub brk_frame: f32,
    /// `create`'s `cPhs_ERROR_e` refusal (`:314-316`) - `cPhs_ERROR_e` cancels the actor inside
    /// Create, so this is a spawn-then-despawn on the first tick, never `Option<Self> -> None`
    /// (RULE 6b).
    pub refused: bool,
    /// One-shot guard so `create`'s single `dComIfGs_isEventBit(UNK_2710)` read (`:314`) happens
    /// once, on the first tick, where `EventFlags` is available.
    pub entered: bool,
}

impl Tornado {
    /// `daTornado_c::create` (`:288-333`) minus `dComIfG_resLoad` (`:293`), `fopAcM_ct` (`:294`),
    /// `fopAcM_entrySolidHeap` / `createHeap` (`:297`, `:254-280`), the `param == 1` / `param == 2`
    /// scale arms (`:302-311` - nothing in the decomp or in this sim spawns either), the three
    /// `setBaseScale` calls and their `cXyz` literals (`:289-291`, `:304-318`), the two
    /// `setBaseTRMtx` calls (`:324-325`), the joint-callback install loop (`:326-329`),
    /// `mpModel->setUserArea` (`:330`), `dKyw_tornado_Notice` (`:317`, see OMITTED) and the two
    /// `dComIfGp_particle_set` emitters (`:320`, `:174`) - all archive, model-matrix, `dKy` wind and
    /// particle work.
    ///
    /// `fopAcM_SetParam(this, 0)` (`:313`) means the `else` arm's `param` stays 0, so
    /// [`PARAM_DEFAULT`] needs no field and only the `if (param == 0)` arm of `execute` can run.
    ///
    /// `fopAc_Init` seeds `current` and `old` from `home` (`f_op_actor.cpp:282-284`); RULE 2 makes
    /// the tornado's scale exactly 1.0, and `mpModel->setBaseScale` is renderer work.
    pub fn spawn(_parameters: u32) -> Self {
        Self {
            // `speedF` (`d_a_tornado.h:51`), `mAngle1` / `mAngle2` (`:56-57`), `mBtkFrame` (`:60`),
            // `mBrkFrame` (`:62`) and `home.pos` all take `fopAcM_ct`'s `sBs_ClearArea` zero-fill
            // (`f_pc_base.cpp:98`) - the tornado is stationary until the first `cLib_chaseF`.
            speed_f: 0.0,
            home: Vec3::ZERO,
            angle1: 0,
            angle2: 0,
            btk_frame: 0.0,
            brk_frame: 0.0,
            refused: false,
            entered: false,
        }
    }

    /// `daTornado_c::create`'s `else` arm (`:312-322`): `mCenter = current.pos` (`:319`) - i.e. the
    /// tornado's centre is its placement position, which is the [`Tornado::home`] anchor the leash
    /// is measured from. `refused` records the `cPhs_ERROR_e` at `:314-316`.
    fn create_init(&mut self, actor: &Actor, flags: &EventFlags) {
        // `if (dComIfGs_isEventBit(dSv_event_flag_c::UNK_2710)) { return cPhs_ERROR_e; }` (:314-316)
        self.refused = flags.test(EVT_2710);
        self.home = actor.pos;
        self.entered = true;
    }

    /// `daTornado_c::execute` (`:99-225`).
    ///
    /// Dropped, all renderer / archive / `dKy` work: `mBck.play()` (`:102`), the two BTK frame
    /// WRAPS (`:104-111`, whose `getFrameMax()` is archive data), the whole `fVar8` block and the
    /// 11-joint loop (`:115-142`, which only feed `mJointX` / `mJointZ` and therefore
    /// `jointCallBack`'s model matrix), the `param == 1` and `else if (m31c != 0)` arms
    /// (`:192-210`, both dead - see the struct doc), the two `setBaseTRMtx` calls (`:211-212`,
    /// `:221-222`), `attention_info.position` (`:213`, RULE 3b), `eyePos = current.pos` (`:214`),
    /// `mCenter.set(...)` (`:215`) and the `mPtclCb` particle colour (`:216-220`).
    fn execute(&mut self, actor: &mut Actor, ctx: &mut Context<'_>) {
        // ```
        // mAngle2 += 250;    (:112)
        // mAngle1 += 500;    (:113)
        // m32c = 10000.0f;  (:114)   - write-only, dropped
        // ```
        self.angle2 = self.angle2.wrapping_add(ANGLE2_STEP);
        self.angle1 = self.angle1.wrapping_add(ANGLE1_STEP);
        // `mBtkFrame += 1.0f;` (:103) - the wrap at `:104-106` needs the archive's frame count.
        self.btk_frame += BTK_STEP;
        // `mBtkUnderFrame += 1.0f;` (:108) - same, and its field is renderer-only besides.

        // `param = fopAcM_GetParam(this); if (param == 0) { ... }` (:144-145)
        if ctx.parameters == PARAM_DEFAULT {
            self.live_arm(actor, ctx);
        }
    }

    /// The `if (param == 0) { ... }` body of `execute` (`:145-191`).
    fn live_arm(&mut self, actor: &mut Actor, ctx: &mut Context<'_>) {
        // `if (!dComIfGp_event_runCheck()) { ... }` (:146) - `runCheck()` is
        // `mMode != dEvtMode_NONE_e` (`d_event.h:133-138`) on the single running event, which is
        // exactly `MsgGameInfo::event_run_check()`.
        if !ctx.info.event_run_check() {
            // ```
            // s16 target = fopAcM_searchActorAngleY(this, dComIfGp_getLinkPlayer());   (:147)
            // cLib_addCalcAngleS(&current.angle.y, target, 10, 0x1000, 0x100);          (:148)
            // cLib_chaseF(&speedF, 20.0f, 0.2f);                                        (:149)
            // ```
            if let Some(player) = ctx.player {
                let target = target_yaw(actor.pos, player);
                // Two-phase borrow: the RHS reads `actor.angle[1]` before the store.
                let turned = walk::add_calc_angle_s(
                    actor.angle[1],
                    target,
                    TURN_SCALE,
                    TURN_MAX_STEP,
                    TURN_MIN_STEP,
                );
                actor.angle[1] = turned;
            }
            walk::chase_f(&mut self.speed_f, CRUISE_SPEED, CRUISE_STEP);
        }

        // ```
        // if (dComIfGp_checkPlayerStatus0(0, daPyStts0_SHIP_RIDE_e) && getShipActor() != NULL) {
        //     daShip_c* ship = getShipActor();
        //     cXyz diff = ship->current.pos - current.pos;
        //     if (diff.abs2XZ() < SQUARE(10000.0f)) { ship->onTornadoFlg(GetID(this)); speedF = 0.0f; }
        // }
        // ```
        // (:151-158). `ship->onTornadoFlg(fopAcM_GetID(this))` writes the ship actor's `mTornadoID`
        // (`d_a_ship.h:189`), which `super::ship::Ship` does not carry - see OMITTED. The
        // `speedF = 0.0f` half is real.
        if ctx.status.check0(STTS0_SHIP_RIDE) {
            if let Some(ship) = ctx.ship {
                if abs2_xz(ship - actor.pos) < CAPTURE_RADIUS * CAPTURE_RADIUS {
                    self.speed_f = 0.0;
                }
            }
        }

        // ```
        // if (!dComIfGp_event_runCheck()) {
        //     fopAcM_posMoveF(this, NULL);
        //     cXyz diff = current.pos - home.pos; diff.y = 0;
        //     if (diff.abs2XZ() > SQUARE(7500.0f)) { diff.normalize(); current.pos = home.pos + diff * 7500.0f; }
        // }
        // ```
        // (:160-168). `fopAcM_posMoveF(this, NULL)` (`f_op_actor_mng.cpp:482-485`) is
        // `fopAcM_calcSpeed` then `fopAcM_posMove`. `calcSpeed` (`:454-466`) is
        // `speed.x = speedF * cM_ssin(angle.y); speed.y = speed.y + gravity; speed.z = speedF *
        // cM_scos(angle.y)` with the y clamped up to `maxFallSpeed`; `gravity` is 0 here (never
        // written - no initialiser, so the zero-fill of `f_pc_base.cpp:98`, and `daTornado_c` calls
        // no `fopAcM_SetGravity`) and `maxFallSpeed` is `-100.0f` (`f_op_actor.cpp:283`), so the
        // clamp never fires. `posMove` (`:469-479`) adds `speed` with a NULL extra offset.
        if !ctx.info.event_run_check() {
            let (sin, cos) = walk::from_s16(actor.angle[1]).sin_cos();
            actor.pos.x += self.speed_f * sin;
            actor.pos.z += self.speed_f * cos;
            // `cXyz diff = current.pos - home.pos; diff.y = 0;` (:162-163)
            let mut diff = actor.pos - self.home;
            diff.y = 0.0;
            // `if (diff.abs2XZ() > SQUARE(7500.0f)) { diff.normalize(); current.pos = home.pos +
            // diff * 7500.0f; }` (:164-166). `cXyz::normalize()` divides by `sqrtf(abs2())`, so a
            // zero horizontal offset would divide by zero - retail only reaches that when the
            // tornado is exactly at `home`, where `abs2XZ() == 0` is NOT `> LEASH_RADIUS^2`, so the
            // guard covers it.
            if abs2_xz(diff) > LEASH_RADIUS * LEASH_RADIUS {
                let length = abs2_xz(diff).sqrt();
                actor.pos = self.home + diff / length * LEASH_RADIUS;
            }
        }

        // ```
        // if (mPtclTimer != 0) { mPtclTimer -= 1; }
        // else { mPtclTimer = 10; dComIfGp_particle_set(dPa_name::ID_AK_SN_TORNADOWIND00,
        //             &current.pos, NULL, (cXyz*)&wind_scale); }
        // ```
        // (:170-175) - the timer only feeds the particle, so neither survives.
        // `fopAcM_seStartCurrent(this, JA_SE_OBJ_TORNADE_SUS, 100);` (:177)
        ctx.events.list.push(SimEvent::Se(SE_TORNADE_SUS));

        // `if (dComIfGs_isEventBit(dSv_event_flag_c::UNK_2710)) { ... }` (:179-191) - the
        // stand-down. `mPtclCb.remove()` (`:180`) and `ship->offTornadoFlg()` (`:183`) are particle
        // and ship-flag writes (omitted), `mBrkFrame += 0.3f` (`:185`) is transcribed, and the
        // `fopAcM_delete(this)` at `:187` needs `mBrk.getBrkAnm()->getFrameMax()` (archive data).
        if ctx.flags.test(EVT_2710) {
            self.brk_frame += BRK_STEP;
        } else {
            // `dKyw_tornado_Notice(&current.pos);` (:190) - see OMITTED.
        }
    }
}

/// `cXyz::abs2XZ()` - `x * x + z * z`, the value `d_a_tornado.cpp:154` and `:164` compare against
/// `SQUARE(...)`.
fn abs2_xz(delta: Vec3) -> f32 {
    delta.x * delta.x + delta.z * delta.z
}

/// `fopAcM_searchActorAngleY(this, dComIfGp_getLinkPlayer())` (`d_a_tornado.cpp:147`) =
/// `cLib_targetAngleY(pos_a, pos_b)` (`f_op_actor_mng.cpp:488-490`) = `cM_atan2s(rhs->x - lhs->x,
/// rhs->z - lhs->z)` (`c_lib.cpp:343-345`). `walk::to_s16` is the one radians-to-`i16` path
/// (RULE 5b) and `cM_atan2s` already works in these units, so this is `atan2` of the same pair.
fn target_yaw(from: Vec3, to: Vec3) -> i16 {
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

pub fn is_tornado(name: &str) -> bool {
    TORNADO_ACTORS.contains(&name)
}

/// Everything `execute` needs beyond the component - the `hbrf1` / `tagetc` shape.
struct Context<'a> {
    flags: &'a EventFlags,
    status: &'a PlayerStatus,
    events: &'a mut SimEvents,
    info: &'a MsgGameInfo,
    player: Option<Vec3>,
    ship: Option<Vec3>,
    parameters: u32,
}

pub fn control(
    flags: Res<EventFlags>,
    status: Res<PlayerStatus>,
    mut events: ResMut<SimEvents>,
    info: Res<MsgGameInfo>,
    players: Query<&Player>,
    mut tornadoes: Query<(Entity, &mut Actor, &mut Tornado)>,
    ships: Query<&Actor>,
    mut commands: Commands,
) {
    let player = players.iter().next().map(|link| link.pos);
    // `dComIfGp_getShipActor()` is the live ship; the decomp null-checks it at `:118` / `:151`
    // without a name test here, and `fopAcM_GetName(ship_p) == fpcNm_SHIP_e` is the sim's own
    // way of identifying it (`d_stage.cpp:650`, `src/sim/ship.rs:12`).
    let ship = ships
        .iter()
        .find(|actor| actor.name == SHIP_ACTOR)
        .map(|actor| actor.pos);
    let mut gone: Vec<Entity> = Vec::new();
    for (entity, mut actor, mut tornado) in &mut tornadoes {
        let actor = &mut *actor;
        if !tornado.entered {
            tornado.create_init(actor, &flags);
        }
        if tornado.refused {
            // `cPhs_ERROR_e` from `create` (`:314-316`) cancels the actor inside Create, so the
            // entity exists for exactly one tick and is gone on the next (RULE 6b).
            gone.push(entity);
            continue;
        }
        let mut ctx = Context {
            flags: &flags,
            status: &status,
            events: &mut events,
            info: &info,
            player,
            ship,
            parameters: actor.parameters,
        };
        tornado.execute(actor, &mut ctx);
    }
    for entity in gone {
        let _ = commands.entity(entity).despawn();
    }
}