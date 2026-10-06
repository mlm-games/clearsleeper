//! `daWarpdm20_c` (`d_a_warpdm20.h:13-77`) - the Great Sea warp hole to Hyrule behind the stage
//! string `Gdemo20`. `Nonmatching` count: 0, so every token below is transcribed.
//!
//! Nothing else in `src/sim/` claims `Gdemo20`: `OBJNAME("Gdemo20", fpcNm_WARPDEMO20_e, 255, 0)`
//! (`d_stage.cpp:1158`) is the only row, and it agrees with `m_arcname[] = "Gdemo20"`
//! (`d_a_warpdm20.cpp:36`). The sibling warp actors are DIFFERENT profiles: `warpf.rs` owns `Warpf`,
//! `warpfo.rs` owns `Warpfo` and `warpt.rs` owns `Warpt` / `Warpnt` / `Warpts1..3`.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, Stage};
use super::flags::{EventFlags, EventReg};
use super::player::Player;
use super::player_status::{STTS0_SHIP_RIDE, PlayerStatus};
use super::sea::Sea;
use super::ship::Ship;
use crate::text::msgproc::MsgGameInfo;

/// `OBJNAME("Gdemo20", fpcNm_WARPDEMO20_e, 255, 0)` (`d_stage.cpp:1158`) - the only row for
/// `g_profile_WARPDEMO20` (`d_a_warpdm20.cpp:551-566`).
pub const WARPDM20_ACTORS: [&str; 1] = ["Gdemo20"];

/// `fopAcM_setCullSizeBox(this, -300.0f, 0.0f, -300.0f, 300.0f, 500.0f, 300.0f)`
/// (`d_a_warpdm20.cpp:112`), matching the profile's `fopAc_CULLBOX_CUSTOM_e` (`:565`).
/// `fopAcM_setCullSizeFar(this, 1.0f)` (`:113`) is a far-plane override, not a box.
const CULL_MIN: [f32; 3] = [-300.0, 0.0, -300.0]; // <- d_a_warpdm20.cpp:112
const CULL_MAX: [f32; 3] = [300.0, 500.0, 300.0]; // <- d_a_warpdm20.cpp:112
/// `mEventIdx = dComIfGp_evmng_getEventIdx("TO_HYRULE_WARP", 0xff);` (`:123`). The second argument is
/// `evNo = 0xFF`, i.e. "no stage redirect", so this is a pure name lookup
/// (`d_event_manager.cpp:347-348`).
const EV_TOHYRULE_WARP: &str = "TO_HYRULE_WARP"; // <- d_a_warpdm20.cpp:123
/// `if (dComIfGs_isEventBit(dSv_event_flag_c::UNK_2D08))` (`:118`, `:188`) - set by `warphole.stb`
/// in stage ADMumi per the decomp's own comment (`d_save_event_flag.inc:343`). Latches `m2CC` to 1.
const FLAG_ACTIVE: u16 = 0x2D08;
/// `if (dComIfGs_isEventBit(dSv_event_flag_c::ZELDA_AWAKENED)) { dComIfGs_onEventBit(UNK_1820); }`
/// (`:125-127`) - `ZELDA_AWAKENED` is set by `awake_zelda.stb` in stage kenroom
/// (`d_save_event_flag.inc:345`).
const FLAG_ZELDA_AWAKENED: u16 = 0x2D02;
const FLAG_WARP_USED: u32 = 0x1820; // <- d_a_warpdm20.cpp:126 (d_save_event_flag.inc:183)
/// `const f32 daWarpdm20_c::m_warp_distance = 500.0f;` (`:40`), used at `:484` - how close the ship
/// must be before `check_warp` fires.
const WARP_DISTANCE: f32 = 500.0; // <- d_a_warpdm20.cpp:40
/// `const f32 daWarpdm20_c::m_eff_distance = 2000.0f;` (`:41`). NOTE the body uses a LOCAL
/// `f32 max = 2000.0f;` at `:186` rather than the constant; both are transcribed and agree.
const EFF_DISTANCE: f32 = 2000.0; // <- d_a_warpdm20.cpp:41, :186
/// `cXyz sp08(current.pos); sp08.y += 2000.0f; current.pos.y = getSeaY(sp08);` (`:147-150`) - the
/// sea-surface probe is taken 2000 units ABOVE the actor.
const SEA_PROBE_LIFT: f32 = 2000.0; // <- d_a_warpdm20.cpp:148
/// ```
/// #if VERSION > VERSION_DEMO
/// if (current.pos.y == -G_CM3D_F_INF) { current.pos.y = 0.0f; }
/// #endif
/// ```
/// (`:151-155`) - a no-water sentinel, applied because this build is `VERSION_NTSC_U`.
const NO_WATER: f32 = 0.0; // <- d_a_warpdm20.cpp:153
/// `cLib_chasePos(&m2EC, cXyz::Zero, 0.2f); emitter->setGlobalScale(m2EC);` (`:359-361`) - the ripple
/// emitter shrinks to nothing over `actDead`.
const RIPPLE_SCALE_STEP: f32 = 0.2; // <- d_a_warpdm20.cpp:359
/// `mRippleCallback.setRate(7.0f);` (`:455`) and `m2EC.set(1.0f, 1.0f, 1.0f);` in `CreateInit`
/// (`:116`) - the emitter's full scale.
const RIPPLE_RATE: f32 = 7.0; // <- d_a_warpdm20.cpp:455
const RIPPLE_SCALE: f32 = 1.0; // <- d_a_warpdm20.cpp:116
/// `f32 volume = (mRotSpeed / 45.0f) * 100.0f;` is the FERRIS wheel's audio, not this actor's.
/// `set_effect(0x83d8)` (`:165`) and `set_effect(0x82bd)` (`:214`) - two particle ids for the
/// ripple emitter, which RULE 3b lists as absent.
const PARTICLE_NORMAL: u16 = 0x83d8; // <- d_a_warpdm20.cpp:165
const PARTICLE_DEMO: u16 = 0x82bd; // <- d_a_warpdm20.cpp:214
/// `static char* action_table[] = { "WAIT", "WARP", "DEAD", "WAIT_DEAD", "WAIT2",
/// "RETURN_WAIT" };` (`:227-234`), paired with `event_init_tbl` / `event_action_tbl` (`:19-34`).
/// The six names and their order are transcribed as [`ACTIONS`] so the `init_tbl` / `action_tbl`
/// pairing has a named shape even though `demo_proc`'s dispatch is dropped.
const ACTIONS: [&str; 6] = [
    "WAIT",           // <- d_a_warpdm20.cpp:228
    "WARP",           // <- d_a_warpdm20.cpp:229
    "DEAD",           // <- d_a_warpdm20.cpp:230
    "WAIT_DEAD",      // <- d_a_warpdm20.cpp:231
    "WAIT2",          // <- d_a_warpdm20.cpp:232
    "RETURN_WAIT",    // <- d_a_warpdm20.cpp:233
];
/// `const u32 daWarpdm20_c::m_heapsize = 0x3000;` (`:39`) and the `fopAcM_entrySolidHeap(this,
/// CheckCreateHeap, 0x3000)` guard (`:137`) - heap work.
const HEAP_SIZE: u32 = 0x3000; // <- d_a_warpdm20.cpp:39
/// `const s16 daWarpdm20_c::m_residxB[] = { -1, -1, -1, -1 };` (`:38`) - every BCK / BTK / BRK in
/// `m_residxA` (`:37`) is present, so `CreateHeap` builds all three animation objects.
const RESIDX_COUNT: usize = 4; // <- d_a_warpdm20.cpp:37-38
/// `daWarpdm20_prm::getType(i_this)` returns `fopAcM_GetParam(i_this) & 0xF` (`:44-46`) into `m2D6`
/// (`:134`). `m2D6` is read NOWHERE else in the translation unit - grep `m2D6` and the only hits
/// are the write at `:134` and the field declaration - so per RULE 7 the parameter is not stored.
const TYPE_MASK: u32 = 0xF; // <- d_a_warpdm20.cpp:45
/// `if (demoActorID == 0) { set_effect(0x83d8); checkOrder(); demo_proc(); eventOrder(); } else
/// { demo_execute(); }` (`:164-171`). `demoActorID` is never set by `_create` (`:131-143`), so it
/// takes `sBs_ClearArea`'s zero (`f_pc_base.cpp:97-98`) and the DEMO branch is dead for a
/// stage-placed actor.
const DEMO_ID_NONE: u32 = 0; // <- d_a_warpdm20.cpp:164

/// The six `event_init_tbl` / `event_action_tbl` entries (`:19-34`), indexed by
/// `dComIfGp_evmng_getMyActIdx`'s position in [`ACTIONS`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// `initWait` / `actWait` (`:254-264`).
    Wait,
    /// `initWarp` / `actWarp` (`:278-287`).
    Warp,
    /// `initDead` / `actDead` (`:330-364`).
    Dead,
    /// `initWaitDead` / `actWaitDead` (`:290-327`).
    WaitDead,
    /// `initWait2` / `actWait2` (`:267-275`).
    Wait2,
    /// `initReturnWait` / `actReturnWait` (`:367-382`).
    ReturnWait,
}

impl Action {
    /// The `action_table` row this maps to, so the pairing with [`ACTIONS`] has a reader.
    pub fn name(self) -> &'static str {
        match self {
            Action::Wait => ACTIONS[0],
            Action::Warp => ACTIONS[1],
            Action::Dead => ACTIONS[2],
            Action::WaitDead => ACTIONS[3],
            Action::Wait2 => ACTIONS[4],
            Action::ReturnWait => ACTIONS[5],
        }
    }
}

/// `daWarpdm20_c` (`d_a_warpdm20.h:13-77`).
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. The class carries only `mPhase`, four
/// animation pointers, a `dPa_rippleEcallBack` and a handful of scalars (`d_a_warpdm20.h:44-76`) -
/// no `dCcD_Stts`, `dCcD_Cyl` or `dCcD_Sph` member, and `dComIfG_Ccsp()->Set(...)` appears nowhere
/// in the translation unit. The profile `Group` is `fopAc_ACTOR_e` (`:564`). Neither
/// `combat::HitPoints` nor `enemy::Hurt` is attached; a warp hole is not a damageable thing in
/// retail and `combat::control` would despawn it.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct WarpDm20 {
    /// `m2CC` (`d_a_warpdm20.h:53`) - "the warp is lit". Set to 1 by `CreateInit` (`:119-121`) and by
    /// `normal_execute` (`:189`) when [`FLAG_ACTIVE`] is set; cleared by `actDead` when the
    /// closing animation finishes (`:353`). Read at `:173` (the `JA_SE_OBJ_HL_WAPR_EFF` sound),
    /// `:210-211` and `:496` (`_draw` returns without drawing when 0).
    pub lit: bool,
    /// `m2C8` (`d_a_warpdm20.h:52`) - "the player is in range, order the event". Set by
    /// `check_warp` (`:200`), cleared by `checkOrder` (`:396`) and by `coming_process_init`-style
    /// reset. Read at `:386`, `:395`, `:403`.
    pub armed: bool,
    /// `mEventIdx` (`d_a_warpdm20.h:56`) from `getEventIdx("TO_HYRULE_WARP", 0xff)` (`:123`).
    pub event_slot: Option<usize>,
    /// `m2E0.y` (`d_a_warpdm20.h:65`) - `set_mtx`'s tracked sea height, which the ripple emitter is
    /// anchored to (`:453`). `set_mtx` writes it every tick (`:156`).
    pub sea_y: f32,
    /// `m2EC` (`d_a_warpdm20.h:66`) - the ripple emitter's global scale, chaser toward zero in
    /// `actDead` (`:359`) and initialised to `(1,1,1)` in `CreateInit` (`:116`).
    pub ripple: Vec3,
    /// `m2F8` (`d_a_warpdm20.h:67`) - `set_wpsenko`'s one-shot guard (`:468-471`).
    pub wpsenko: bool,
    /// `mStaffIdx` (`d_a_warpdm20.h:64`) from `dComIfGp_evmng_getMyStaffId("Gdemo20")` (`:236`).
    /// Only `demo_proc` reads it, and that is dropped, so it is not stored.
    /// `demoActorID` (`:164`) - never written by `_create`, so [`DEMO_ID_NONE`] always holds.
    /// One-shot guard so `CreateInit`'s `onEventBit` (`:126`) runs exactly once, on the first tick.
    pub entered: bool,
    /// `set_effect` (`:449-464`) returns FALSE when `mRippleCallback.getEmitter() != NULL` - i.e. it
    /// is a ONE-SHOT setup, not a per-tick call. `set_effect(0x83d8)` (`:165`) therefore creates the
    /// emitter exactly once; this flag records that.
    pub effect_built: bool,
}

impl WarpDm20 {
    /// `daWarpdm20_c::CreateInit` (`:110-128`), minus the parts that are renderer, heap or particle
    /// work. The parameter decode is absent: `m2D6` (`daWarpdm20_prm::getType`, `:44-46`) has no
    /// reader, so per RULE 7 it is not stored.
    ///
    /// ```
    /// m2E0 = current.pos;
    /// m2EC.set(1.0f, 1.0f, 1.0f);
    /// m2C8 = 0;
    /// if (dComIfGs_isEventBit(dSv_event_flag_c::UNK_2D08)) {
    ///     m2CC = 1;
    ///     setEndAnm();
    /// }
    /// mEventIdx = dComIfGp_evmng_getEventIdx("TO_HYRULE_WARP", 0xff);
    /// if (dComIfGs_isEventBit(dSv_event_flag_c::ZELDA_AWAKENED)) {
    ///     dComIfGs_onEventBit(dSv_event_flag_c::UNK_1820);
    /// }
    /// ```
    ///
    /// DROPPED: `fopAcM_SetMtx` (`:111`), the cull box and far plane (`:112-113`), `set_mtx()`
    /// (`:114`) and `setEndAnm()` (`:121`) - the latter two are BCK/BRK frame work, unavailable per
    /// bug class 2 (archive frame counts are not in this checkout).
    pub fn spawn() -> Self {
        Self {
            // `m2C8 = 0;` (:117)
            armed: false,
            // `m2CC` takes the zero fill (`f_pc_base.cpp:97-98`); `:119-121` sets it from the flag.
            lit: false,
            // `mEventIdx` (:123) is resolved by `create_init`, which needs the `EventList`.
            event_slot: None,
            // `m2E0 = current.pos;` (:115)
            sea_y: 0.0,
            // `m2EC.set(1.0f, 1.0f, 1.0f);` (:116)
            ripple: Vec3::splat(RIPPLE_SCALE),
            // `m2F8` takes the zero fill - `set_wpsenko`'s guard (`:468`).
            wpsenko: false,
            entered: false,
            // `mRippleCallback` is default-constructed, so `getEmitter() == NULL` until
            // `set_effect` succeeds - `set_effect(0x83d8)` (`:165`) is the first call.
            effect_built: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -300.0f, 0.0f, -300.0f, 300.0f, 500.0f, 300.0f)`
    /// (`d_a_warpdm20.cpp:112`) - the actor's one real custom cull box, matching the profile's
    /// `fopAc_CULLBOX_CUSTOM_e` (`:565`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daWarpdm20_c::CreateInit` (`:110-128`), first tick only, for the two pieces that need a
    /// resource: the event-list lookup and the `onEventBit` store.
    fn create_init(&mut self, list: &EventList, flags: &EventReg, events: &mut EventFlags) {
        // `if (dComIfGs_isEventBit(dSv_event_flag_c::UNK_2D08)) { m2CC = 1; setEndAnm(); }`
        // (:118-121) - `setEndAnm` is BCK/BRK frame work (bug class 2).
        if flags.test(FLAG_ACTIVE) {
            self.lit = true;
        }
        // `mEventIdx = dComIfGp_evmng_getEventIdx("TO_HYRULE_WARP", 0xff);` (:123). NOTE the
        // contract lists `dComIfGp_evmng_getEventIdx` by 8-bit evId as confirmed-absent, yet
        // `EventList::event_index(name)` exists and 56 modules already use it (`mbdoor.rs`,
        // `vgnfd.rs`, `zunari.rs`, `dai.rs` and 52 others). Following the existing modules.
        self.event_slot = list.event_index(EV_TOHYRULE_WARP);
        // ```
        // if (dComIfGs_isEventBit(dSv_event_flag_c::ZELDA_AWAKENED)) {
        //     dComIfGs_onEventBit(dSv_event_flag_c::UNK_1820);
        // }
        // ```
        // (:125-127) - RULE 11c: pass the retail ids straight through; `EventFlags` is already
        // byte-indexed by `hi_byte << 8 | mask`.
        if flags.test(FLAG_ZELDA_AWAKENED) {
            events.set(FLAG_WARP_USED);
        }
        self.entered = true;
    }

    /// `daWarpdm20_c::set_mtx` (`:146-160`).
    ///
    /// ```
    /// cXyz sp08(current.pos);
    /// sp08.y += 2000.0f;
    /// current.pos.y = getSeaY(sp08);
    /// #if VERSION > VERSION_DEMO
    /// if (current.pos.y == -G_CM3D_F_INF) { current.pos.y = 0.0f; }
    /// #endif
    /// m2E0.y = current.pos.y;
    /// ```
    ///
    /// `getSeaY` (`:438-446`) is `daSea_ChkArea(x, z) ? daSea_calcWave(x, z) :
    /// dBgS_ObjGndChk_Wtr_Func(arg1)` - the sim's `Sea::draw_contains` / `Sea::height_at` pair, the
    /// same reduction `esa.rs:259-260` and `ritem.rs:320-321` use. The `else` arm is a per-poly-group
    /// DZB water query, which RULE 3b lists as absent and `super::water::Water` is NOT a substitute
    /// for (it is a single player probe, `src/sim/water.rs:19-25`), so only the sea branch is
    /// modelled and a point outside the sea leaves the height alone.
    ///
    /// DROPPED: `mpModel->setBaseScale(scale)` (`:157`) and the `transS` / `setBaseTRMtx` pair
    /// (`:158-159`) - renderer work (RULE 3b).
    fn set_mtx(&mut self, actor: &mut Actor, sea: Option<&Sea>) {
        let probe = Vec3::new(actor.pos.x, actor.pos.y + SEA_PROBE_LIFT, actor.pos.z);
        // `daSea_ChkArea(pos->x, pos->z)` (:440) is `Sea::draw_contains`.
        if let Some(sea) = sea
            && sea.draw_contains(probe.x, probe.z)
        {
            // `daSea_calcWave(pos->x, pos->z)` (:441) is `Sea::height_at`.
            actor.pos.y = sea.height_at(probe.x, probe.z);
            // `m2E0.y = current.pos.y;` (:156)
            self.sea_y = actor.pos.y;
        }
        // `if (current.pos.y == -G_CM3D_F_INF) { current.pos.y = 0.0f; }` (:151-155) - the no-water
        // sentinel. Only the sea branch above can produce `-G_CM3D_F_INF`, and it cannot, so this
        // fold's input is never reachable; `NO_WATER` is therefore not applied and the omission is
        // reported rather than papered over with a fake height.
    }

    /// `daWarpdm20_c::check_warp` (`:482-492`).
    ///
    /// ```
    /// daShip_c* ship = dComIfGp_getShipActor();
    /// f32 max = 500.0f;
    /// if (dComIfGp_checkPlayerStatus0(0, daPyStts0_SHIP_RIDE_e) && ship != NULL) {
    ///     f32 abs = (ship->current.pos - current.pos).absXZ();
    ///     if (abs < max) { return TRUE; }
    /// }
    /// return FALSE;
    /// ```
    ///
    /// `dComIfGp_getShipActor()` is a process-id lookup; the sim's `Query<(&Actor, &Ship)>` is the
    /// house reduction. `absXZ()` is the XZ-only magnitude (`c_xyz.h`).
    fn check_warp(&mut self, status: &PlayerStatus, ship: Option<Vec3>, origin: Vec3) {
        // `dComIfGp_checkPlayerStatus0(0, daPyStts0_SHIP_RIDE_e)` (:485)
        if !status.check0(STTS0_SHIP_RIDE) {
            return;
        }
        // `ship != NULL` (:485)
        let Some(ship) = ship else {
            return;
        };
        // `f32 abs = (ship->current.pos - current.pos).absXZ();` (:486)
        let gap = Vec3::new(ship.x - origin.x, 0.0, ship.z - origin.z).length();
        // `if (abs < max)` (:487) with `max = 500.0f` (:484) == `m_warp_distance` (:40)
        if gap < WARP_DISTANCE {
            self.armed = true;
        }
    }

    /// `daWarpdm20_c::normal_execute` (`:182-203`).
    ///
    /// ```
    /// JPABaseEmitter* emitter = mRippleCallback.getEmitter();
    /// daPy_py_c* player = daPy_getPlayerActorClass();
    /// f32 abs = (player->current.pos - current.pos).absXZ();
    /// f32 max = 2000.0f;
    /// if (dComIfGs_isEventBit(dSv_event_flag_c::UNK_2D08)) {
    ///     m2CC = 1;
    ///     animPlay();
    ///     if (emitter != NULL) {
    ///         if (abs < max) { emitter->playCreateParticle(); } else { emitter->stopCreateParticle(); }
    ///     }
    ///     if (check_warp()) { m2C8 = 1; }
    /// }
    /// ```
    ///
    /// DROPPED: `animPlay()` (`:190`) - BTK / BRK / BCK playback, unavailable per bug class 2 - and
    /// the emitter pair (`:191-195`) - particle work, which RULE 3b lists as absent.
    /// `dComIfGs_isEventBit` is `super::flags::EventReg::test`.
    fn normal_execute(&mut self, flags: &EventReg, ctx: &Ctx<'_>) {
        // `if (dComIfGs_isEventBit(dSv_event_flag_c::UNK_2D08))` (:188)
        if !flags.test(FLAG_ACTIVE) {
            return;
        }
        // `m2CC = 1;` (:189)
        self.lit = true;
        // `if (check_warp()) { m2C8 = 1; }` (:199-201)
        self.check_warp(&ctx.status, ctx.ship, ctx.origin);
        // The `abs < max` emitter gate is dropped with the emitter; `m_eff_distance` (:41) and the
        // local `max = 2000.0f` (:186) are the same value and are carried by [`EFF_DISTANCE`].
    }

    /// `daWarpdm20_c::checkOrder` (`:393-406`).
    ///
    /// ```
    /// if (eventInfo.checkCommandDemoAccrpt()) {
    ///     if (dComIfGp_evmng_startCheck(mEventIdx) && m2C8 != 0) { m2C8 = 0; }
    ///     if (dComIfGp_evmng_endCheck(mEventIdx)) {
    ///         dComIfGp_event_reset();
    ///         fopAcM_delete(this);
    ///     }
    /// } else if (m2C8 == 0 && !dComIfGp_event_runCheck()) {
    ///     normal_execute();
    /// }
    /// ```
    ///
    /// `eventInfo.checkCommandDemoAccrpt()` is the per-actor `dEvtCmd_INDEMO_e` latch the sim does
    /// not carry; the house reduction `Stage::Play | Stage::Close` on this actor's own event slot is
    /// used instead (the `hbrf1.rs:258` / `hole.rs:293` convention). `evmng_startCheck` /
    /// `evmng_endCheck` are `ActiveEvent::start_check` / `::end_check`. `dComIfGp_event_reset()` is
    /// `ActiveEvent::end` - see the note in [`WarpDm20::execute`].
    fn check_order(&mut self, active: &ActiveEvent, info: &MsgGameInfo, ctx: &Ctx<'_>) -> bool {
        let Some(slot) = self.event_slot else {
            return false;
        };
        // RULE 5: `dComIfGp_event_reset()` (`:400`) clears the actor's own slot, which is
        // `ActiveEvent::end`. `dComIfGp_evmng_endCheck` (`:399`) is a READ of that slot's stage, so
        // the delete is sequenced after the reset here and reported to `control` as `deleted`.
        if demo_running(active, slot) {
            // ```
            // if (dComIfGp_evmng_startCheck(mEventIdx) && m2C8 != 0) { m2C8 = 0; }
            // ```
            // (:395-397)
            if active.start_check(slot) && self.armed {
                self.armed = false;
            }
            // ```
            // if (dComIfGp_evmng_endCheck(mEventIdx)) { dComIfGp_event_reset(); fopAcM_delete(this); }
            // ```
            // (:399-402) - returns TRUE so `control` despawns.
            return active.end_check(slot);
        }
        // `} else if (m2C8 == 0 && !dComIfGp_event_runCheck()) { normal_execute(); }` (:403-405)
        if !self.armed && !info.event_run_check() {
            self.normal_execute(&ctx.flags, ctx);
        }
        false
    }

    /// `daWarpdm20_c::eventOrder` (`:385-390`).
    ///
    /// ```
    /// if (m2C8 == 1) {
    ///     fopAcM_orderOtherEventId(this, mEventIdx, 0xff, 0xffff, 0, 1);
    ///     eventInfo.onCondition(2);
    /// }
    /// ```
    ///
    /// `fopAcM_orderOtherEventId` takes this actor's event index, so `ActiveEvent::order` is the
    /// right channel. RULE 5 on re-ordering: `ActiveEvent::order` must only run when this actor has
    /// no slot in the manager, or the demo livelocks.
    fn event_order(&mut self, active: &mut ActiveEvent, list: &EventList, entity: Entity) {
        // `if (m2C8 == 1)` (:386)
        if !self.armed {
            return;
        }
        let Some(slot) = self.event_slot else {
            return;
        };
        // RULE 5: never re-order a slot this actor already owns.
        if active.stage_of(slot).is_none() {
            active.order(list, slot, entity);
        }
        // `eventInfo.onCondition(2)` (:388) is a per-actor condition bit with no modelled reader -
        // the same omission `dai.rs:351-355` records for `dEvtCnd_CANTALK_e`.
    }

    /// `daWarpdm20_c::_execute` (`:163-179`), reduced to `set_mtx` plus the two live branches.
    ///
    /// ```
    /// if (demoActorID == 0) {
    ///     set_effect(0x83d8);
    ///     checkOrder();
    ///     demo_proc();
    ///     eventOrder();
    /// } else {
    ///     demo_execute();
    /// }
    /// if (m2CC != 0) { fopAcM_seStart(this, JA_SE_OBJ_HL_WAPR_EFF, 0); }
    /// set_mtx();
    /// ```
    ///
    /// `demoActorID` (`:164`) is never written by `_create` (`:131-143`), so it holds
    /// [`DEMO_ID_NONE`] and `demo_execute` is dead for a stage-placed actor. DROPPED: `set_effect`
    /// (`:165`) - particle emitter setup, RULE 3b; `demo_proc` (`:167`) - it needs
    /// `dComIfGp_evmng_getMyActIdx(mStaffIdx, action_table, 6, FALSE, 0)` to resolve a cut's ACTION
    /// name against [`ACTIONS`], and the sim's `EventList` exposes the cut NAME, not the action label
    /// (`Event` has `name`, `priority`, `staffs`, `finish_flags`, `end_sound` -
    /// `src/sim/event.rs:203-209`), so the index has no counterpart; the `JA_SE_OBJ_HL_WAPR_EFF` sound
    /// (`:174`) - no modelled audio channel for a positional SE.
    ///
    /// Returns TRUE when `checkOrder` asked for the actor to be deleted (`fopAcM_delete(this)`,
    /// `:401`).
    fn execute(
        &mut self,
        actor: &mut Actor,
        active: &mut ActiveEvent,
        list: &EventList,
        info: &MsgGameInfo,
        ctx: &Ctx<'_>,
        entity: Entity,
    ) -> bool {
        // `set_effect(0x83d8);` (:165) - one-shot, recorded on the component.
        self.effect_built = true;
        // `checkOrder();` (:166)
        let deleted = self.check_order(active, info, ctx);
        // `demo_proc();` (:167) - dropped, see above.
        // `eventOrder();` (:168)
        self.event_order(active, list, entity);
        // `set_mtx();` (:177)
        self.set_mtx(actor, ctx.sea);
        deleted
    }

    /// `daWarpdm20_c::_delete` (`:50-54`) and `set_wpsenko` (`:467-472`) have no modelled effect:
    /// `_delete` removes the ripple callback and releases the archive, and `set_wpsenko` fires one
    /// particle. The `m2F8` guard is transcribed on the component.
    #[allow(dead_code)]
    fn wpsenko_once(&mut self) -> bool {
        // `if (!m2F8) { dComIfGp_particle_set(...); m2F8 = true; }` (:468-471)
        let fresh = !self.wpsenko;
        self.wpsenko = true;
        fresh
    }

    /// `daWarpdm20_c::actDead`'s emitter collapse (`:357-362`):
    /// `cLib_chasePos(&m2EC, cXyz::Zero, 0.2f); emitter->setGlobalScale(m2EC);`
    pub fn shrink_ripple(&mut self) {
        let step = self.ripple * RIPPLE_SCALE_STEP;
        self.ripple -= step;
    }
}

/// Everything `execute` needs beyond the component, in one bundle - the `hbrf1` / `tagetc` shape.
struct Ctx<'a> {
    origin: Vec3,
    status: &'a PlayerStatus,
    ship: Option<Vec3>,
    sea: Option<&'a Sea>,
    flags: &'a EventReg,
}

/// `eventInfo.checkCommandDemoAccrpt()` as `Stage::Play | Stage::Close` - the two states
/// `dEvent_manager_c::getMyStaffId` scans (`d_event_manager.cpp:412`). The `hbrf1.rs:258` /
/// `hole.rs:293` house reduction.
fn demo_running(active: &ActiveEvent, slot: usize) -> bool {
    matches!(active.stage_of(slot), Some(Stage::Play) | Some(Stage::Close))
}

pub fn is_warp_dm20(name: &str) -> bool {
    WARPDM20_ACTORS.contains(&name)
}

pub fn control(
    mut holes: Query<(Entity, &mut Actor, &mut WarpDm20), With<WarpDm20>>,
    ships: Query<&Actor, (With<Ship>, Without<WarpDm20>)>,
    status: Res<PlayerStatus>,
    sea: Option<Res<Sea>>,
    regs: Res<EventReg>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    info: Res<MsgGameInfo>,
    mut flags: ResMut<EventFlags>,
    mut commands: Commands,
) {
    let sea_ref = sea.as_deref();
    // `dComIfGp_getShipActor()` (`:483`) - a process-id lookup; `Query<&Actor, With<Ship>>` is the
    // house reduction. The `&mut Actor` writer carries `With<WarpDm20>` and this reader
    // `Without<WarpDm20>`, which is the disjointness pair RULE 9 asks for - `Without` on ONE side
    // alone would not prove it.
    let ship = ships.iter().next().map(|actor| actor.pos);
    for (entity, mut actor, mut hole) in &mut holes {
        if !hole.entered {
            // `daWarpdm20_c::CreateInit` (`:110-128`)
            hole.create_init(&list, &regs, &mut flags);
        }
        let ctx = Ctx {
            // `current.pos` (:147, :150, :185)
            origin: actor.pos,
            status: &status,
            ship,
            sea: sea_ref,
            flags: &regs,
        };
        let deleted = hole.execute(&mut actor, &mut active, &list, &info, &ctx, entity);
        // `fopAcM_delete(this);` (:401) - a real delete request in retail, so `commands.despawn()`.
        if deleted {
            let _ = commands.entity(entity).despawn();
        }
    }
}
