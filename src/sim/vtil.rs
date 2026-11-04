//! `daObjVtil_c` (`tww/src/d/actor/d_a_obj_vtil.cpp`): the stone idol behind the five stage names
//! `Vtil1` .. `Vtil5` (`tww/src/d/d_stage.cpp:1174-1178`), process `fpcNm_Obj_Vtil_e`
//! (`tww/src/c/c_dylink.cpp:447`), profile `g_profile_Obj_Vtil` (`d_a_obj_vtil.cpp:535-550`),
//! archive `"Vtil"` (`d_a_obj_vtil.cpp:57`).
//!
//! What survives here is the whole state machine: [`Vtil::hit_co`] (`d_a_obj_vtil.cpp:366-378`) and
//! [`Vtil::hit_bg`] (`d_a_obj_vtil.cpp:403-440`) ahead of one [`Mode`] arm of the `_execute`
//! dispatch table (`d_a_obj_vtil.cpp:462-470`), with the four mode bodies
//! (`d_a_obj_vtil.cpp:233-256`, `d_a_obj_vtil.cpp:268-282`, `d_a_obj_vtil.cpp:326-333`,
//! `d_a_obj_vtil.cpp:359-363`) and the four transitions (`d_a_obj_vtil.cpp:222-230`,
//! `d_a_obj_vtil.cpp:259-265`, `d_a_obj_vtil.cpp:285-299`, `d_a_obj_vtil.cpp:336-356`) they call.
//! The idol is picked up, thrown, and sinks; a weapon hit reports to the `agb*` figure's beat
//! counter (`tww/src/d/d_com_static.cpp:96-108`) and a finished sinking to its dispose counter
//! (`tww/src/d/d_com_static.cpp:81-93`).
//!
//! Three subjects change what the actor can do, and each is a RULE 3b absence rather than a
//! reduction:
//!
//! * **The per-stage saved treasure-box table**, which decides whether the idol is created at all.
//!   `check_ev_bit` (`d_a_obj_vtil.cpp:123-125`) is `dComIfGs_isStageTbox` of box 15 in the stage
//!   `l_daObjVtil_scene_no_table` gives for the type (`d_a_obj_vtil.cpp:29`), and
//!   `tww/src/d/d_com_inf_game.cpp:710-718` answers the live table only for the stage you are standing in.
//!   `src/sim/chest.rs` carries two 32-slot tables and the routing in `src/sim/npc_tc.rs:982` knows
//!   two of them, so asking it about stages 3, 4, 5, 6 and 7 would answer false everywhere and
//!   RULE 6b would despawn every idol. The gate therefore folds to "created", and `mType` has
//!   nothing left to do: its only two readers are the archive's model index
//!   (`d_a_obj_vtil.cpp:68`, the renderer) and that table (`d_a_obj_vtil.cpp:124`), so it is not
//!   stored at all (RULE 7).
//! * **`mAcch`, the ground/roof/wall/water checker.** Its answers arrive through
//!   [`Vtil::crr_pos`], run at the end of every mode body exactly where `mAcch.CrrPos` is called
//!   (`d_a_obj_vtil.cpp:255`, `d_a_obj_vtil.cpp:279`, `d_a_obj_vtil.cpp:332`,
//!   `d_a_obj_vtil.cpp:362`), so each read at the top of the next tick sees the previous tick's
//!   correction - retail's own one-tick latency. The four `Chk*` it answers are folded:
//!   `ChkSeaIn` and `GetSeaHeight` to `super::water::probe`, `ChkGroundHit` and `ChkGroundLanding`
//!   to `super::ground::grounded`, and `ChkWallHit` / `ChkRoofHit` (`d_a_obj_vtil.cpp:411`) are
//!   dropped because the crate's `Collision` has no wall or roof plane to hit, the same absence
//!   `src/sim/stone.rs:267` records.
//! * **The statue's own attack**, which `dCcD_Cyl`'s AT bit carries.
//!   `to_throw_mode` turns it on (`d_a_obj_vtil.cpp:286`) and
//!   `to_wait_mode` turns it off (`d_a_obj_vtil.cpp:223`), but a
//!   `bodies::TABLE` row is static, so the row leaves `at_sprm` at `0x00` and
//!   [`Vtil::hit_co`] cannot see a `ChkAtHit` to branch on - see its doc for what that costs.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::agbsw0;
use super::carry::Carry;
use super::collision::Collision;
use super::contact_run::ContactResults;
use super::events::{SimEvent, SimEvents};
use super::ground;
use super::player::Player;
use super::sea::Sea;
use super::stream;
use super::vibration::Vibration;
use super::water;

/// `d_stage.cpp:1174-1178`, the five `OBJNAME` rows that share `fpcNm_Obj_Vtil_e`. They differ only
/// in the third field - the macro's `sub`, `dStage_objectNameInf::argument`
/// (`tww/include/d/d_stage.h:1000`), which retail copies into `fopAc_ac_c::argument`
/// (`tww/src/f_op/f_op_actor.cpp:276`) and this TU never reads, so no argument is stored here
/// (RULE 7).
pub const VTIL_ACTORS: [&str; 5] = ["Vtil1", "Vtil2", "Vtil3", "Vtil4", "Vtil5"];

/// The `gravity = -6.0f` the idol is born with. Four places write it:
/// `_create` (`d_a_obj_vtil.cpp:97`), `to_wait_mode` (`d_a_obj_vtil.cpp:227`),
/// `to_throw_mode` (`d_a_obj_vtil.cpp:297`) and `calc_throw` (`d_a_obj_vtil.cpp:321`).
const GRAVITY: f32 = -6.0; // <- d_a_obj_vtil.cpp:97

/// `actor->maxFallSpeed = -100.0f` (`tww/src/f_op/f_op_actor.cpp:286`), the clamp
/// `posMoveF_stream` applies to `speed.y` (`tww/src/d/d_a_obj.cpp:159-161`). This TU never writes it.
const MAX_FALL: f32 = -100.0; // <- f_op_actor.cpp:286

/// `eyePos.y += 110.0f` (`d_a_obj_vtil.cpp:100`), the point every idol sound is started at.
const EYE_LIFT: f32 = 110.0; // <- d_a_obj_vtil.cpp:100

/// The model height, which `check_sink_end` adds to the position (`d_a_obj_vtil.cpp:392`) and the
/// cylinder, the cull box and the roof-check height all carry as well (`:54`, `:94`, `:167`).
const HEIGHT: f32 = 160.0; // <- d_a_obj_vtil.cpp:392

/// `fopAcM_setCullSizeBox(this, -50.0f, 0.0f, -50.0f, 50.0f, 160.0f, 50.0f)`
/// (`d_a_obj_vtil.cpp:94`), the box the profile's `fopAc_CULLBOX_CUSTOM_e` (`:549`) makes the
/// actor's own.
const CULL_MIN: [f32; 3] = [-50.0, 0.0, -50.0]; // <- d_a_obj_vtil.cpp:94
const CULL_MAX: [f32; 3] = [50.0, HEIGHT, 50.0]; // <- d_a_obj_vtil.cpp:94

/// `speed.y = 27.0f` and `speedF = 36.0f`, the launch `to_throw_mode` writes (`d_a_obj_vtil.cpp:295`,
/// `:296`), over the `gravity = -6.0f` of `:297`.
const THROW_LIFT: f32 = 27.0; // <- d_a_obj_vtil.cpp:295
const THROW_SPEED_F: f32 = 36.0; // <- d_a_obj_vtil.cpp:296

/// `gravity = -2.0f`, the sink's own fall (`d_a_obj_vtil.cpp:345`), and the
/// `magnitude > 30.0` rescale of `speed` and `speedF` that follows it (`:346-350`).
const SINK_GRAVITY: f32 = -2.0; // <- d_a_obj_vtil.cpp:345
const SINK_CAP: f32 = 30.0; // <- d_a_obj_vtil.cpp:347

/// `daObj::posMoveF_stream(this, mStts.GetCCMoveP(), &cXyz::Zero, 0.2f, 0.02f)`, the
/// resist pair `mode_sink` moves against (`d_a_obj_vtil.cpp:360`) and the
/// pair `calc_throw` answers under water (`d_a_obj_vtil.cpp:315`, `d_a_obj_vtil.cpp:316`).
const SINK_DAMP: f32 = 0.2; // <- d_a_obj_vtil.cpp:360
const SINK_DRAG: f32 = 0.02; // <- d_a_obj_vtil.cpp:360

/// `calc_throw`'s submersion fraction: full drag at `-160.0f` of depth and half of it below that,
/// scaled by `0.003125f` in between (`d_a_obj_vtil.cpp:307-312`), and the four coefficients the
/// fraction is mixed between (`:315-317`) plus the dry-arm constants (`:319-321`).
const THROW_DEPTH: f32 = -160.0; // <- d_a_obj_vtil.cpp:309
const THROW_DEPTH_RATIO: f32 = 0.5; // <- d_a_obj_vtil.cpp:310
const THROW_DEPTH_SCALE: f32 = 0.003125; // <- d_a_obj_vtil.cpp:312
const THROW_DRAG_WATER: f32 = 0.2; // <- d_a_obj_vtil.cpp:315
const THROW_DRAG_AIR: f32 = 0.002; // <- d_a_obj_vtil.cpp:315
const THROW_INERT_WATER: f32 = 0.02; // <- d_a_obj_vtil.cpp:316
const THROW_INERT_AIR: f32 = 0.0002; // <- d_a_obj_vtil.cpp:316
const THROW_BUOYANCY: f32 = 4.0; // <- d_a_obj_vtil.cpp:317

/// `speedF *= 0.6f`, the survivable throw landing (`d_a_obj_vtil.cpp:417`), tested against
/// `mPrevSpeedY < gravity - 1.0f` (`:415`).
const THROW_SLOW: f32 = 0.6; // <- d_a_obj_vtil.cpp:417
const THROW_SLOW_MARGIN: f32 = 1.0; // <- d_a_obj_vtil.cpp:415

/// `mAcch.GetSeaHeight() > 70.0f + current.pos.y` (`d_a_obj_vtil.cpp:383`), the depth the idol has
/// to reach to start sinking, and `50.0f + (160.0f + current.pos.y)` (`:392`), the depth at which it
/// has gone for good.
const SINK_TOP: f32 = 70.0; // <- d_a_obj_vtil.cpp:383
const SINK_END: f32 = 50.0; // <- d_a_obj_vtil.cpp:392

/// `if (distance <= 190.0f)`, `check_circle`'s player-proximity answer (`d_a_obj_vtil.cpp:454`).
const CIRCLE_RADIUS: f32 = 190.0; // <- d_a_obj_vtil.cpp:454

/// `StartShock(strength + 1, 1, cXyz(0.0f, 1.0f, 0.0f))` (`d_a_obj_vtil.cpp:445`): the `1` is the
/// flags word, which selects the motor rumble alone, and the vector is its coordinate.
const SHOCK_NEAR: i32 = 1; // <- d_a_obj_vtil.cpp:445
const SHOCK_FLAGS: i32 = 1; // <- d_a_obj_vtil.cpp:445
const SHOCK_DIR: Vec3 = Vec3::new(0.0, 1.0, 0.0); // <- d_a_obj_vtil.cpp:445

/// `mDoAud_seStart(JA_SE_OBJ_PUT_STONE, &eyePos, sound, ...)` (`d_a_obj_vtil.cpp:190`) and
/// `mDoAud_seStart(JA_SE_OBJ_FALL_WATER_S, &eyePos, sound, ...)` (`:212`), both at `eyePos`, so both
/// are positional.
const PUT_STONE_SE: u32 = 0x6929; // <- JAZelAudio_SE.h:1830
const FALL_WATER_SE: u32 = 0x6918; // <- JAZelAudio_SE.h:1813

/// `mAcch.GetSeaHeight()` with no sea under the idol. `ChkSeaIn` (`:383`, `:392`, `:303`) is the
/// gate every reader is behind, so the value itself is never compared while it stands.
const NO_SEA: f32 = f32::NEG_INFINITY;

/// `mMode`'s four values, in `mode_exe[]` order (`d_a_obj_vtil.cpp:462-465`): `0`
/// [`Mode::Wait`] (`:463`), `1` [`Mode::Carry`] (`:463`), `2` [`Mode::Throw`] (`:464`) and `3`
/// [`Mode::Sink`] (`:464`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Wait,
    Carry,
    Throw,
    Sink,
}

/// `daObjVtil_c` (`tww/include/d/actor/d_a_obj_vtil.h:11-68`) - the standing, throwable,
/// sinkable stone idol, reduced to the fields the simulation reads.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Vtil {
    /// `mMode` (`d_a_obj_vtil.h:63`) - the `_execute` dispatch index (`d_a_obj_vtil.cpp:469`).
    pub mode: Mode,
    /// `mDeleteState` (`d_a_obj_vtil.h:64`) - `0` runs the mode body (`d_a_obj_vtil.cpp:468`) and
    /// `1` deletes the actor (`d_a_obj_vtil.cpp:482-485`); the `2` written after the delete is
    /// not stored, because the entity is gone by then.
    pub delete_state: u8,
    /// `mFirstLanding` (`d_a_obj_vtil.h:65`) - `1` from `_create` (`d_a_obj_vtil.cpp:103`) until
    /// the first landing clears it (`d_a_obj_vtil.cpp:243`), so the dust the idol kicks is
    /// skipped on the drop it was created with.
    pub first_landing: bool,
    /// `mPrevSpeedY` (`d_a_obj_vtil.h:62`) - `speed.y` at the end of every mode body, at
    /// `d_a_obj_vtil.cpp:254`, `d_a_obj_vtil.cpp:278`, `d_a_obj_vtil.cpp:331` and
    /// `d_a_obj_vtil.cpp:361`, and read by the throw's landing test at
    /// `d_a_obj_vtil.cpp:415`.
    pub prev_speed_y: f32,
    /// `mAcch.ChkGroundHit()` - a floor under the idol, from [`Vtil::crr_pos`]. Read as `ground` in
    /// `hit_bg` (`:404`).
    pub grounded: bool,
    /// `mAcch.ChkGroundLanding()` - the one-tick edge of [`Vtil::grounded`], which is what
    /// `mode_wait` branches on (`:237`) and `mAcch.ClrGroundLanding()` clears (`:245`).
    pub landed: bool,
    /// `mAcch.ChkSeaIn()` (`:303`, `:382`, `:391`).
    pub wet: bool,
    /// `mAcch.GetSeaHeight()` (`:304`, `:383`, `:392`), or [`NO_SEA`] out at sea.
    pub sea_y: f32,
    /// `eyePos` (`fopAc_ac_c`), written once by `_create` (`:99-100`) and never again - the idol
    /// keeps reporting its sounds at the height of the ground it was placed on.
    pub eye: Vec3,
    /// `speed` (`fopAc_ac_c::speed`).
    pub speed: Vec3,
    /// `speedF` (`fopAc_ac_c::speedF`).
    pub speed_f: f32,
    /// `gravity` (`fopAc_ac_c::gravity`), which `posMoveF_stream` adds to `speed.y`
    /// (`tww/src/d/d_a_obj.cpp:153`).
    pub gravity: f32,
}

impl Vtil {
    /// `_create` (`d_a_obj_vtil.cpp:80-112`), reduced to the state it leaves behind. `at` is
    /// `current.pos` at that moment, which `eyePos` is built from (`:99-100`).
    ///
    /// Not carried from `_create`, all of it renderer, archive or absent-subsystem work: the
    /// `check_ev_bit` gate (`:87`, see the module header), `dComIfG_resLoad` (`:88`) and
    /// `fopAcM_entrySolidHeap` (`:91`) with `create_heap` (`:66-77`) behind it, `fopAcM_SetMtx`
    /// (`:92`) and `init_mtx` (`:138-144`), `init_co` (`:147-155` - the `M_co_cyl_data` cylinder
    /// `:30-56` is a `bodies::TABLE` row and the `mStts.Init(200, 0xff, this)` of `:148` is a
    /// `weight.rs` row), `renew_attention_pos` (`:98`, `:176-180`) and the two
    /// `attention_info` writes (`:101-102`), `dKy_plight_set` (`:105`) and `model = mpModel`
    /// (`:106`).
    ///
    /// `init_bgc` (`:158-173`) is also not run here: its `fopAcM_posMoveF` (`:170`) and
    /// `mAcch.CrrPos` (`:171`) need the ground, and its `mAcch.ClrGroundLanding()` (`:172`)
    /// is the landing latch [`Vtil::landed`] already starts cleared. The
    /// `gravity = -6.0f` that `_create` sets at `:97` and the `to_wait_mode()`
    /// it calls at `:104` are what [`Vtil::to_wait_mode`] leaves, and
    /// `mFirstLanding = 1` at `:103` is [`Vtil::first_landing`].
    pub fn spawn(at: Vec3) -> Self {
        Self {
            mode: Mode::Wait,
            delete_state: 0,
            first_landing: true,
            prev_speed_y: 0.0,
            grounded: false,
            landed: false,
            wet: false,
            sea_y: NO_SEA,
            eye: Vec3::new(at.x, at.y + EYE_LIFT, at.z),
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: GRAVITY,
        }
    }

    /// The cull box `fopAcM_setCullSizeBox` hands the shell (`d_a_obj_vtil.cpp:94`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `to_wait_mode` (`:222-230`): the three collider bits (`:223-225`, the `at_sprm` reduction the
    /// module header names), `speedF = 0.0f` (`:226`), `gravity = -6.0f` (`:227`), the
    /// `mStts.Init(200, 0xff, this)` re-init (`:228`, which `weight.rs` already supplies) and
    /// `mMode = 0` (`:229`).
    pub fn to_wait_mode(&mut self) {
        self.speed_f = 0.0;
        self.gravity = GRAVITY;
        self.mode = Mode::Wait;
    }

    /// `mode_wait` (`:233-256`): the grab latch (`:234-235`), then the landing report (`:237-245`)
    /// and the settle (`:252`), then the two tail statements (`:254-255`) that every arm shares.
    fn mode_wait(
        &mut self,
        actor: &mut Actor,
        held: bool,
        player: Option<&Player>,
        events: &mut SimEvents,
        shake: &mut Vibration,
        solid: Option<&Collision>,
        sea: Option<&Sea>,
    ) {
        if held {
            self.to_carry_mode(); // :235
        } else {
            if self.landed {
                if self.first_landing {
                    self.first_landing = false; // mFirstLanding = 0 (:243)
                } else {
                    // make_smoke() (:239) is `daObj::make_land_effect(this, &mAcch.m_gnd, 1.0f)`
                    // (`:183-185`), a dust particle off the hit ground polygon.
                    self.se_smoke(events);
                    self.make_vib(player, actor.pos, shake); // :241
                }
                // mAcch.ClrGroundLanding() (:245) is what [`Vtil::crr_pos`] recomputes.
            }
            // ChkGroundHit() (:247-251) only raises and lowers the CARRY attention flag, and
            // attention is not ported.
            self.settle(actor); // fopAcM_posMoveF(this, mStts.GetCCMoveP()) (:252)
        }
        self.prev_speed_y = self.speed.y; // :254
        self.crr_pos(actor, solid, sea); // :255
    }

    /// `to_carry_mode` (`:259-265`): the three collider bits (`:260-262`), the attention flag off
    /// (`:263`) and `mMode = 1` (`:264`).
    fn to_carry_mode(&mut self) {
        self.mode = Mode::Carry;
    }

    /// `mode_carry` (`:268-282`). The position and speed snapshot of `:269-270` and the restore of
    /// `:280-281` are kept as written: `mAcch.CrrPos` (`:279`) would push a carried idol out of the
    /// floor it is being held through, and the two lines undo that correction along with everything
    /// the release branch just wrote to `speed`. Without a wall-carrying `CrrPos` the restore is
    /// inert in this port, and `src/sim/koisi.rs` keeps its own equivalent for the same reason.
    fn mode_carry(
        &mut self,
        actor: &mut Actor,
        held: bool,
        solid: Option<&Collision>,
        sea: Option<&Sea>,
    ) {
        let pos = actor.pos; // cXyz pos = current.pos (:269)
        let start_speed = self.speed; // cXyz prevSpeed = speed (:270)
        if !held {
            if self.speed_f > 0.0 {
                self.to_throw_mode(); // :273
            } else {
                self.to_wait_mode(); // :275
            }
        }
        self.prev_speed_y = self.speed.y; // :278
        self.crr_pos(actor, solid, sea); // :279
        actor.pos = pos; // :280
        self.speed = start_speed; // :281
    }

    /// `to_throw_mode` (`:285-299`): the three collider bits (`:286-288`, of which only
    /// `OnAtSetBit` has no home in a static row), the four `mAcch` clears (`:289-292`) and the
    /// `OnLineCheck` (`:293`) that belong to the checker, the attention flag (`:294`), then the
    /// launch `speed.y = 27.0f` (`:295`), `speedF = 36.0f` (`:296`) and `gravity = -6.0f` (`:297`)
    /// and `mMode = 2` (`:298`).
    fn to_throw_mode(&mut self) {
        self.speed.y = THROW_LIFT;
        self.speed_f = THROW_SPEED_F;
        self.gravity = GRAVITY;
        self.mode = Mode::Throw;
    }

    /// `mode_carry`'s release test (`:272`) as `src/sim/carry.rs` calls it. Retail's thrower writes
    /// the speed through the grab object - `grab_actor->speedF = m_HIO->mGrab.m.field_0x84`
    /// (`tww/src/d/d_a_player_grab.inc:572`) and `grab_actor->speed.y` just after it (`:573`) - and
    /// `daPy_py_c`'s HIO tunables are not ported, but this TU reads only the sign of that write
    /// (`:272`) before `to_throw_mode` overwrites both values (`:295-296`). So the throw is armed
    /// with the idol's own launch speed, and the direction needs no argument either:
    /// `posMoveF_stream` rebuilds `speed.x`/`speed.z` from `speedF` and `current.angle.y`
    /// (`tww/src/d/d_a_obj.cpp:152-154`), which `src/sim/carry.rs:127` has been steering onto the
    /// player every tick it held the idol.
    pub fn toss(&mut self) {
        self.speed_f = THROW_SPEED_F; // the value to_throw_mode writes anyway (:296)
    }

    /// `calc_throw` (`:302-323`): the submersion fraction of the sea surface over `current.pos.y`
    /// and the three coefficients mixed by it. The sea answer is [`Vtil::wet`] and [`Vtil::sea_y`],
    /// which is the previous tick's `mAcch` state, exactly as `mAcch.ChkSeaIn()` is.
    fn calc_throw(&self, at: Vec3) -> (f32, f32, f32) {
        if !self.wet {
            return (GRAVITY, THROW_DRAG_AIR, THROW_INERT_AIR); // :319-321
        }
        let distance = at.y - self.sea_y; // :305
        let depth = if distance >= 0.0 {
            0.0 // :307-308
        } else if distance <= THROW_DEPTH {
            THROW_DEPTH_RATIO // :309-310
        } else {
            THROW_DEPTH_SCALE * -distance // :312
        };
        let above = 1.0 - depth; // :314
        (
            GRAVITY + THROW_BUOYANCY * depth,                    // :317
            THROW_DRAG_WATER * depth + THROW_DRAG_AIR * above,   // :315
            THROW_INERT_WATER * depth + THROW_INERT_AIR * above, // :316
        )
    }

    /// `mode_throw` (`:326-333`).
    fn mode_throw(&mut self, actor: &mut Actor, solid: Option<&Collision>, sea: Option<&Sea>) {
        let (grav, drag, drag2) = self.calc_throw(actor.pos); // :328
        self.gravity = grav; // :329
        // daObj::posMoveF_stream(this, mStts.GetCCMoveP(), &cXyz::Zero, drag, drag2) (:330)
        self.stream_move(actor, drag, drag2);
        self.prev_speed_y = self.speed.y; // :331
        self.crr_pos(actor, solid, sea); // :332
    }

    /// `to_sink_mode` (`:336-356`): the three collider bits (`:337-339`), the `mAcch` roof, wall,
    /// ground and water clears and `OnLineCheck` (`:340-344`) that belong to the checker, the
    /// `gravity = -2.0f` (`:345`) and the `30.0` clamp of the launch magnitude (`:346-350`), then the
    /// splash, its sound and `mMode = 3` (`:355`).
    fn to_sink_mode(&mut self, events: &mut SimEvents) {
        self.gravity = SINK_GRAVITY;
        let magnitude = (self.speed.y * self.speed.y + self.speed_f * self.speed_f).sqrt();
        if magnitude > SINK_CAP {
            let ratio = SINK_CAP / magnitude; // :348
            self.speed *= ratio; // :349
            self.speed_f *= ratio; // :350
        }
        // make_splash() (:352) is `fopKyM_createWpillar(&pos, 1.0f, 0.75f, 0)` (`:194-197`), a
        // water pillar at the sea surface.
        self.se_splash(events);
        self.mode = Mode::Sink;
    }

    /// `mode_sink` (`:359-363`).
    fn mode_sink(&mut self, actor: &mut Actor, solid: Option<&Collision>, sea: Option<&Sea>) {
        self.stream_move(actor, SINK_DAMP, SINK_DRAG); // :360
        self.prev_speed_y = self.speed.y; // :361
        self.crr_pos(actor, solid, sea); // :362
    }

    /// `daObj::posMoveF_stream(this, mStts.GetCCMoveP(), &cXyz::Zero, drag, drag2)` (`:330`, `:360`)
    /// = `posMoveF_grade` with a NULL ground normal (`tww/src/d/d_a_obj.cpp:94-95`), which zeroes the
    /// grade pull and leaves the resist pair plus `speed.y + gravity`, then `fopAcM_posMove` (`:173`)
    /// integrates `pos += speed`. `mStts.GetCCMoveP()` is the `cCd` momentum channel this crate does
    /// not carry, the same absence `src/sim/koisi.rs` records for its own calls.
    fn stream_move(&mut self, actor: &mut Actor, damp: f32, drag: f32) {
        let (speed, speed_f, facing) = stream::move_stream(
            self.speed,
            Vec3::ZERO,
            actor.angle[1],
            self.speed_f,
            self.gravity,
            MAX_FALL,
            damp,
            drag,
        );
        self.speed = speed;
        self.speed_f = speed_f;
        actor.angle[1] = facing; // current.angle.y = cM_atan2s(speed_x, speed_z) (:170)
        actor.gravity = self.gravity;
        actor.pos += self.speed;
    }

    /// `fopAcM_posMoveF(this, mStts.GetCCMoveP())` (`:252`) = `fopAcM_calcSpeed`
    /// (`tww/src/f_op/f_op_actor_mng.cpp:454-466`) then `fopAcM_posMove` (`:469`).
    fn settle(&mut self, actor: &mut Actor) {
        let (speed, speed_f) = stream::calc_speed(
            self.speed_f,
            actor.angle[1],
            self.speed.y,
            self.gravity,
            MAX_FALL,
        );
        self.speed = speed;
        self.speed_f = speed_f;
        actor.pos += self.speed;
    }

    /// `mAcch.CrrPos(*dComIfG_Bgsp())` (`:255`, `:279`, `:332`, `:362`) reduced to the two answers
    /// the idol reads back: whether there is a floor at the position it just computed, and the sea
    /// plane over it. The landing latch is the edge between one tick's floor answer and the next, so
    /// `mAcch.ClrGroundLanding()` (`:245`) needs no statement of its own.
    ///
    /// NOT carried, with the decomp line: the position correction itself and the polygon handle
    /// `mAcch.m_gnd` (the `make_smoke` material read of `:189`), `mAcch.m_wtr` (`:206`), and the
    /// roof, wall and water-state latches `:289-292`, `:340-343` and `:165-168` write.
    fn crr_pos(&mut self, actor: &Actor, solid: Option<&Collision>, sea: Option<&Sea>) {
        let below = ground::grounded(solid, actor.pos);
        self.landed = below && !self.grounded;
        self.grounded = below;
        let (in_sea, top) = water::probe(sea, actor.pos.x, actor.pos.z);
        self.wet = in_sea; // mAcch.ChkSeaIn() (:303)
        self.sea_y = if in_sea { top } else { NO_SEA }; // mAcch.GetSeaHeight() (:304)
    }

    /// `check_sink` (`:381-387`): the surface more than `70.0f` over the idol's feet.
    fn check_sink(&self, at: Vec3) -> bool {
        self.wet && self.sea_y > SINK_TOP + at.y // :383
    }

    /// `check_sink_end` (`:390-396`): the surface more than `50.0f` over the idol's head, which is
    /// `160.0f` up from its feet (`:392`).
    fn check_sink_end(&self, at: Vec3) -> bool {
        self.wet && self.sea_y > SINK_END + (HEIGHT + at.y) // :392
    }

    /// `hit_co` (`:366-378`), the collider half of `_execute` (`:466`).
    ///
    /// `mStts.Move()` (`:367`) is the `cCcD_Stts` momentum accumulator, which this crate does not
    /// carry, and the `ChkAtHit` arm (`:368-370`) goes with it: the cylinder's `AT_SPRM_SET` bit is
    /// only on while the idol is in flight (`:286`, off again at `:223`, `:260` and `:337`) and a
    /// `bodies::TABLE` row is static, so the port never has a live AT body to be hit. What that
    /// loses is `speedF *= 0.8f` (`:370`), the in-flight stub of a thrown idol against something
    /// solid, and the outbound `AT_TYPE_UNK8` damage the row's `SrcObjAt Type` and `SrcObjAt Atp`
    /// (`:33-34`) would carry, which `src/sim/fallrock.rs:11-18` and `src/sim/toge.rs:94` already
    /// record for the same two words.
    ///
    /// The `ChkTgHit` arm (`:371-376`) is the report the figure waits for, and of its four
    /// statements only that one survives: `daObj::HitSeStart(&eyePos, current.roomNo, &mCyl, 13)`
    /// (`:373`) is silent for this object because the row's `SrcGObjTg Se` is `0` (`:44`) and
    /// `dCcG_SE_NONE` is the value that `tww/src/d/d_a_obj.cpp:274` reads and `:280` gates the whole
    /// body on, so no cue is ever reached at `:310`; `set_sound(150, 5)` (`:374`, `:217-219`) is
    /// `dKy_Sound_set`, a sound-position marker with no home here; and `daObj::HitEff_hibana`
    /// (`:375`) is the spark.
    ///
    /// `mCyl.ClrTgHit()` (`d_a_obj_vtil.cpp:376`) needs no statement: `ContactResults` is rebuilt
    /// every step, so a record is a one-tick edge already. Reading it here carries retail's own
    /// latency, since `dComIfG_Ccsp()->Set(&mCyl)` (`d_a_obj_vtil.cpp:473`) registers the cylinder
    /// at the END of the tick.
    fn hit_co(&mut self, entity: Entity, results: &ContactResults) {
        if results
            .at_tg
            .iter()
            .any(|record| record.tg == entity && record.set_tg)
        {
            agbsw0::inc_tcl_beat(); // tell_agb_attack() (:372)
        }
    }

    /// `make_vib` (`:443-446`): the pattern index is `check_circle() & 1` plus one, so the rumble is
    /// stronger when [`Vtil::check_circle`] says the player is close (`:444`).
    fn make_vib(&self, player: Option<&Player>, at: Vec3, shake: &mut Vibration) {
        let strength = Self::check_circle(player, at) as i32; // check_circle() & 1 (:444)
        let _ = shake.start_shock(strength + SHOCK_NEAR, SHOCK_FLAGS, SHOCK_DIR);
    }

    /// `check_circle` (`:449-458`): `delta.absXZ()` of the player over the idol, compared to
    /// `190.0f` (`:452-454`). An empty room is the one state where retail would follow
    /// `dComIfGp_getPlayer(0)` (`:450`) into a null read, so it answers false here.
    fn check_circle(player: Option<&Player>, at: Vec3) -> bool {
        let Some(player) = player else {
            return false;
        };
        let dx = player.pos.x - at.x;
        let dz = player.pos.z - at.z;
        dx * dx + dz * dz <= CIRCLE_RADIUS * CIRCLE_RADIUS
    }

    /// `se_smoke` (`:188-191`). The `sound` selector is
    /// `dComIfG_Bgsp()->GetMtrlSndId(mAcch.m_gnd)` (`:189`), a per-poly-group material read the
    /// crate does not carry, and `dComIfGp_getReverb` (`:190`) is the room's reverb send; the
    /// positional start itself is what `SeAt` publishes.
    fn se_smoke(&self, events: &mut SimEvents) {
        events.list.push(SimEvent::SeAt(PUT_STONE_SE, self.eye));
    }

    /// `se_splash` (`:200-214`). Here the absent material read has a documented fallback of its own:
    /// `u32 sound = 19` (`:204`) is only replaced by a `GetMtrlSndId` read (`:208`) when a polygon
    /// index answers (`:207`), and the crate carries neither, so the decomp's own default stands -
    /// but the selector is not part of [`SimEvent::SeAt`], so it is dropped. The
    /// `set_sound(125, 5)` of `:213` goes with the other `dKy_Sound_set` call.
    fn se_splash(&self, events: &mut SimEvents) {
        events.list.push(SimEvent::SeAt(FALL_WATER_SE, self.eye));
    }

    /// `hit_bg` (`:403-440`), the background half of `_execute` (`:467`), on the previous tick's
    /// ground and sea answers.
    ///
    /// The `ground` term of `:411` is the crate's floor probe and the two omitted acch reads are
    /// `ChkWallHit` and `ChkRoofHit` (`:411`), which need a wall and a roof plane.
    /// `camera_off()` (`:426`, `:437`) is `dComIfGp_getCamera(0)->mCamera.ForceLockOff` (`:398-400`),
    /// and the camera's lock-on is not ported. Both `#if VERSION == VERSION_DEMO` arms (`:412-413`)
    /// are out by RULE 0.
    fn hit_bg(
        &mut self,
        at: Vec3,
        player: Option<&Player>,
        events: &mut SimEvents,
        shake: &mut Vibration,
    ) {
        let ground = self.grounded; // :404
        let sinking = self.check_sink(at); // :405
        match self.mode {
            Mode::Wait => {
                if sinking {
                    self.to_sink_mode(events); // :408
                }
            }
            Mode::Carry => {} // no arm of :406-431 tests mMode == 1
            Mode::Throw => {
                if ground {
                    if self.prev_speed_y < self.gravity - THROW_SLOW_MARGIN {
                        self.speed_f *= THROW_SLOW; // :417
                    } else {
                        // The `if (ground)` of :419 is the same term as the outer test once the two
                        // omitted acch arms of :411 fold away, so its three statements run here.
                        // make_smoke() (:420), the dropped particle again.
                        self.se_smoke(events); // :421
                        self.make_vib(player, at, shake); // :422
                        self.speed_f = 0.0; // :424
                        self.to_wait_mode(); // :425
                    }
                } else if sinking {
                    self.to_sink_mode(events); // :429
                }
            }
            Mode::Sink => {
                if self.check_sink_end(at) {
                    agbsw0::inc_tcl_dispose(); // tell_agb_sink() (:433)
                    self.delete_state = 1; // :434
                } else if !sinking {
                    self.to_wait_mode(); // :436
                }
            }
        }
    }
}

pub fn is_vtil(name: &str) -> bool {
    VTIL_ACTORS.contains(&name)
}

/// `_execute` (`d_a_obj_vtil.cpp:461-487`) for every idol in the room. Reads: the previous step's
/// `ContactResults` records and each idol's own state; writes: `Actor::pos`, `Actor::angle`,
/// `Actor::gravity`, the sound and rumble channels, the figure's two counters, and the despawn of an
/// idol that finished sinking.
///
/// `renew_attention_pos` and `init_mtx` (`:470-471`) are the two statements between the dispatch and
/// the collider, and `mCyl.MoveCAtTg(current.pos)` plus `dComIfG_Ccsp()->Set(&mCyl)` (`:472-473`) are
/// the crate's `bodies::follow`. The `mLight` block (`:474-480`) is a point light.
pub fn control(
    collision: Option<Res<Collision>>,
    sea: Option<Res<Sea>>,
    results: Res<ContactResults>,
    carry: Res<Carry>,
    mut events: ResMut<SimEvents>,
    mut shake: ResMut<Vibration>,
    players: Query<&Player>,
    mut statues: Query<(Entity, &mut Actor, &mut Vtil)>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let water = sea.as_deref();
    let player = players.iter().next();
    for (entity, mut actor, mut statue) in &mut statues {
        let actor = &mut *actor;
        let statue = &mut *statue;
        statue.hit_co(entity, &results); // :466
        statue.hit_bg(actor.pos, player, &mut events, &mut shake); // :467
        if statue.delete_state == 0 {
            // (this->*mode_exe[mMode])() (:469), dispatched on the mode as it stands here, so a
            // transition the two calls above made is not run until the next tick.
            let held = carry.held == Some(entity);
            match statue.mode {
                Mode::Wait => {
                    statue.mode_wait(actor, held, player, &mut events, &mut shake, solid, water)
                }
                Mode::Carry => statue.mode_carry(actor, held, solid, water),
                Mode::Throw => statue.mode_throw(actor, solid, water),
                Mode::Sink => statue.mode_sink(actor, solid, water),
            }
        }
        if statue.delete_state == 1 {
            let _ = commands.entity(entity).despawn(); // fopAcM_delete(this) (:483)
        }
    }
}
