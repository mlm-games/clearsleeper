//! `sail_class` (`d_a_sail.h:83-89`) - the Outset Island Pirates' Sail, the cloth that hangs off the
//! pirate ship's mast. `Nonmatching` count: 0, so every token below is transcribed.
//!
//! Nothing else in `src/sim/` claims `Psail`: `OBJNAME("Psail", fpcNm_SAIL_e, 0, 0)`
//! (`d_stage.cpp:1119`) is the only row. It does NOT match either archive the actor loads -
//! `"Cloth"` (`d_a_sail.cpp:601`) and `"Kaizokusen"` (`:607`) - so the stage string is read from the
//! OBJNAME table, not from `m_arcname`. Note the 4th column is `0`, so this actor carries NO save
//! bit, unlike most rows.
//!
//! The profile `Status` is `fopAcStts_UNK4000_e | fopAcStts_UNK40000_e` (`d_a_sail.cpp:674`) -
//! `fopAcStts_CULL_e` is ABSENT, so the sail is never culled off in retail.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::pirateship::Pirate;
use super::walk::add_calc;

/// `OBJNAME("Psail", fpcNm_SAIL_e, 0, 0)` (`d_stage.cpp:1119`) - the only row for `g_profile_SAIL`
/// (`d_a_sail.cpp:662-677`). The 4th column `0` means `prm_get_swSave()` would read switch 0, but
/// this actor reads NO parameter at all: the profile's `Parameters` field is `0` (`:669`), so there
/// is nothing to decode and RULE 7 is satisfied trivially.
pub const SAIL_ACTORS: [&str; 1] = ["Psail"];

/// `fopAc_CULLBOX_0_e` (`f_op_actor_mng.cpp:561`) -
/// `fopAc_cullSizeBox(cXyz(-40.0f, 0.0f, -40.0f), cXyz(40.0f, 125.0f, 40.0f))`, which the profile
/// names at `:676`.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:561
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:561

/// ```
/// static cXyz sail_offset(0.0f, 2100.0f, 100.0f);
/// cMtx_multVec(l_p_ship->mModel->getBaseTRMtx(), &sail_offset, &i_this->current.pos);
/// i_this->current.angle = l_p_ship->shape_angle;
/// ```
/// (`d_a_sail.cpp:554-556`) - where the mast top sits in the pirateship's frame. `2100.0f` is the
/// mast height; `100.0f` a small forward offset.
const MAST_TOP: [f32; 3] = [0.0, 2100.0, 100.0]; // <- d_a_sail.cpp:554

/// ```
/// i_this->mSailPacket.m1C44 = 0.0f;
/// i_this->mSailPacket.m1C48 = 0.0f;
/// ```
/// (`d_a_sail.cpp:628-629`) - the sail starts fully furled with a zero approach rate.
const FURLED_ZERO: f32 = 0.0; // <- d_a_sail.cpp:628

/// ```
/// if (l_p_ship->m2CE == 0) { l_HIO.m06 = 0; i_this->mSailPacket.m1C44 = 0.6f; }
/// else { l_HIO.m06 = 1; }
/// ```
/// (`d_a_sail.cpp:638-643`) - `m2CE` is `fopAcM_GetParam(this) & 0xF ? 1 : 0`
/// (`d_a_obj_pirateship.cpp:473-477`), which `super::pirateship::Pirate::sail` already carries
/// (`src/sim/pirateship.rs:47`, `SAIL_MASK = 0xF` at `:25`). A sail-less ship starts the sail
/// 60% UNfurled (`m1C44 = 0.6f`).
const SAILLESS_FURLED: f32 = 0.6; // <- d_a_sail.cpp:640

/// ```
/// daSail_HIO_c() { mNo = -1; m05 = 1; m06 = 1; m10 = 0.0f; m07 = 0; }
/// ```
/// (`d_a_sail.cpp:23-29`). These are the FILE-SCOPE HIO defaults - `l_HIO` is a
/// `static daSail_HIO_c` (`:56`), not a `g_regHIO` child - so they are real literals. `m06` and
/// `m08` are then OVERWRITTEN by `daSail_Create` (`:639`, `:642`) and by the HIO editor.
const HIO_M05: u8 = 1; // <- d_a_sail.cpp:25
const HIO_M06: u8 = 1; // <- d_a_sail.cpp:26
const HIO_M07: u8 = 0; // <- d_a_sail.cpp:28
const HIO_M10: f32 = 0.0; // <- d_a_sail.cpp:27
/// `#define REG_SAIL_F(i) 0.0f` / `#define REG_SAIL_S(i) 0` (`d_a_sail.cpp:51-52`) for every build
/// above `VERSION_DEMO`, and this port targets `VERSION_NTSC_U`. So every `REG_SAIL_*` in
/// `sail_move` and `sail_pos_move` folds to a literal - the same proof as RULE 2b.
const REG_SAIL_F25: f32 = 0.0; // <- d_a_sail.cpp:51 (REG_SAIL_F(25))
/// ```
/// if (l_HIO.m06 == 0 && i_this->mSailPacket.m1C44 < REG_SAIL_F(25) + 0.6f) {
///     if (i_this->mSailPacket.m1C48 <= 0.0f) { i_this->mSailPacket.m1C48 = 0.015f; }
///     else { i_this->mSailPacket.m1C48 -= 0.001f; }
///     cLib_addCalc(&i_this->mSailPacket.m1C44, REG_SAIL_F(25) + 0.6f, 0.1f,
///                  i_this->mSailPacket.m1C48, 0.01f);
/// } else {
///     if (l_HIO.m06 == 1 && i_this->mSailPacket.m1C44 > 0.0f) {
///         i_this->mSailPacket.m1C48 += 0.0075f;
///         cLib_addCalc(&i_this->mSailPacket.m1C44, 0.0f, 0.3f, i_this->mSailPacket.m1C48, 0.01f);
///     } else { i_this->mSailPacket.m1C48 = 0.0f; }
/// }
/// if (i_this->mSailPacket.m1C44 > 1.0f) { i_this->mSailPacket.m1C44 = 1.0f; }
/// else if (i_this->mSailPacket.m1C44 < 0.0f) { i_this->mSailPacket.m1C44 = 0.0f; }
/// if (!demo_move(i_this) && l_HIO.m08 != 0) { i_this->mSailPacket.m1C44 = l_HIO.m0C; }
/// i_this->mSailPacket.m1C4C = (i_this->mSailPacket.m1C44 - 0.6f) * 15.0f;
/// ```
/// (`d_a_sail.cpp:479-504`) - the furl state machine. `m1C44` is the furl amount in 0..1, `m1C48`
/// the approach rate, `m1C4C` a derived slack term.
const UNFURL_TARGET: f32 = 0.6; // <- d_a_sail.cpp:479 (REG_SAIL_F(25) + 0.6f)
const UNFURL_SCALE: f32 = 0.1; // <- d_a_sail.cpp:485
const UNFURL_MIN_STEP: f32 = 0.01; // <- d_a_sail.cpp:485
const UNFURL_RATE_MAX: f32 = 0.015; // <- d_a_sail.cpp:481
const UNFURL_RATE_DECAY: f32 = 0.001; // <- d_a_sail.cpp:483
const FURL_TARGET: f32 = 0.0; // <- d_a_sail.cpp:489
const FURL_SCALE: f32 = 0.3; // <- d_a_sail.cpp:489
const FURL_RATE_STEP: f32 = 0.0075; // <- d_a_sail.cpp:488
const FURL_MIN_STEP: f32 = 0.01; // <- d_a_sail.cpp:489
const FURL_CAP: f32 = 1.0; // <- d_a_sail.cpp:495
const SLACK_SCALE: f32 = 15.0; // <- d_a_sail.cpp:504 ((m1C44 - 0.6f) * 15.0f)
const SLACK_PIVOT: f32 = 0.6; // <- d_a_sail.cpp:504

/// `sail_class` (`d_a_sail.h:83-89`) - the sail, plus the three `daSail_packet_c` fields that leave
/// the sim.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `sail_class` holds two
/// `request_of_phase_process_class` and one `daSail_packet_c` (`d_a_sail.h:85-87`) - no collider
/// member, and `dComIfG_Ccsp()->Set(...)` never appears in the translation unit. The profile
/// `Group` is `fopAc_ACTOR_e` (`:675`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached;
/// `combat::control` would despawn the sail, which retail cannot damage.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Sail {
    /// `m1C44` (`d_a_sail.h:88`) - the furl amount, 0 = fully furled and 1 = fully spread. Read by
    /// `sail_pos_move` at `:364`, `:373-374`, `:439-452` and by the `draw` packet; written by
    /// [`Sail::move`].
    pub furl: f32,
    /// `m1C48` (`:89`) - the approach rate passed as `cLib_addCalc`'s `maxStep`.
    pub rate: f32,
    /// `m1C4C` (`:90`) - `(m1C44 - 0.6f) * 15.0f`, the slack term `sail_pos_move` multiplies in.
    pub slack: f32,
    /// `l_HIO.m06` (`d_a_sail.h:40`) - the furl DIRECTION, set by `daSail_Create` (`:639`, `:642`):
    /// `0` unfurls toward [`UNFURL_TARGET`], `1` furls toward [`FURL_TARGET`].
    pub direction: u8,
    /// `l_HIO.m05` (`:39`) - set but read only by `draw()` (`:217`), the renderer.
    pub textured: u8,
    /// `l_HIO.m07` (`:41`) - set by the ctor (`:28`) and read nowhere else.
    pub aux: u8,
    /// `l_HIO.m10` (`:44`) - the normal-angle amplitude in `setCorrectNrmAngle` (`:80`), which is
    /// renderer-only (see the module docs).
    pub normal_amp: f32,
    /// `l_HIO.m08` (`:40`) - the HIO override: when nonzero it forces `m1C44 = l_HIO.m0C` (`:502`).
    /// Both are HIO-editor values, so both read as zero from the ctor's initial value.
    pub override_flag: u8,
    /// `l_HIO.m0C` (`:43`) - the value the override forces.
    pub override_furl: f32,
    /// One-shot guard so `daSail_Create` (`:596-652`) runs exactly once, on the first tick, where
    /// the pirateship query is available.
    pub entered: bool,
}

impl Sail {
    /// `daSail_Create` (`:596-652`), first tick only.
    ///
    /// ```
    /// i_this->mSailPacket.m1C44 = 0.0f;
    /// i_this->mSailPacket.m1C48 = 0.0f;
    /// ...
    /// l_p_ship = (daObjPirateship::Act_c*)fopAcM_SearchByID(i_this->parentActorID);
    /// if (l_p_ship->m2CE == 0) { l_HIO.m06 = 0; i_this->mSailPacket.m1C44 = 0.6f; }
    /// else { l_HIO.m06 = 1; }
    /// sail_move(i_this);
    /// ```
    ///
    /// DROPPED: the two `dComIfG_resLoad` calls (`:601`, `:607`) with their phase returns
    /// (`:603-618`) - archive work, and a `cPhs_ERROR_e` from either is heap/asset failure, so there
    /// is no refusal to model; `fopAcM_entrySolidHeap` / `daSail_checkCreateHeap` (`:623`) - heap work;
    /// `mDoHIO_root.m_subroot.createChild` (`:624-626`) - HIO registration; the 84-entry
    /// `l_pos[]` vertex seed loop (`:630-635`); `sail_move` (`:645`), which runs on the first tick.
    ///
    /// `parentActorID` is `fopAcM_createChild`'s `parentPcId` argument
    /// (`f_op_actor_mng.cpp:191`), i.e. the pirateship. The sim has no link field on `Actor`, so the
    /// ship is resolved as the stage's single [`Pirate`] - the same documented reduction the
    /// sibling `coming2` module uses.
    pub fn spawn() -> Self {
        Self {
            // `i_this->mSailPacket.m1C44 = 0.0f;` (:628)
            furl: FURLED_ZERO,
            // `i_this->mSailPacket.m1C48 = 0.0f;` (:629)
            rate: 0.0,
            // `m1C4C` is derived every tick by `sail_move` (:504), so the zero fill stands.
            slack: 0.0,
            // `l_HIO.m06` takes its ctor value of 1 until `Create` overwrites it (`:639`, `:642`).
            direction: HIO_M06,
            // `l_HIO.m05` (`:25`)
            textured: HIO_M05,
            // `l_HIO.m07` (`:28`)
            aux: HIO_M07,
            // `l_HIO.m10` (`:27`)
            normal_amp: HIO_M10,
            // `l_HIO.m08` is NOT in the ctor, so it takes `sBs_ClearArea`'s zero fill
            // (`f_pc_base.cpp:97-98`).
            override_flag: 0,
            // `l_HIO.m0C` likewise.
            override_furl: 0.0,
            entered: false,
        }
    }

    /// `fopAc_CULLBOX_0_e` (`f_op_actor_mng.cpp:561`) - the profile's named box (`:676`). The sail's
    /// own world position is the mast top, so `actor_views` should read `Actor::pos` for the lift,
    /// not this box's - the box is the cull volume only.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daSail_Create`'s `l_p_ship->m2CE` branch (`:638-643`), first tick only.
    fn create_init(&mut self, ship_has_sail: Option<bool>) {
        match ship_has_sail {
            // ```
            // if (l_p_ship->m2CE == 0) { l_HIO.m06 = 0; i_this->mSailPacket.m1C44 = 0.6f; }
            // ```
            // (:638-640) - `m2CE == 0` means `fopAcM_GetParam(this) & 0xF == 0`
            // (`d_a_obj_pirateship.cpp:473-477`), i.e. a sail-less ship.
            Some(false) => {
                self.direction = 0;
                self.furl = SAILLESS_FURLED;
            }
            // `else { l_HIO.m06 = 1; }` (:641-643)
            Some(true) => self.direction = 1,
            // `l_p_ship` is a NULL `static` (`d_a_sail.cpp:55`) when the parent lookup failed, and
            // retail dereferences it unconditionally at `:638` - undefined. The sim skips instead of
            // picking a direction.
            None => return,
        }
        // `sail_move(i_this);` (:645)
        self.move_step();
    }

    /// `daSail_Execute` (`:553-559`):
    ///
    /// ```
    /// static cXyz sail_offset(0.0f, 2100.0f, 100.0f);
    /// cMtx_multVec(l_p_ship->mModel->getBaseTRMtx(), &sail_offset, &i_this->current.pos);
    /// i_this->current.angle = l_p_ship->shape_angle;
    /// sail_move(i_this);
    /// ```
    ///
    /// `daObjPirateship::Act_c::set_mtx` (`d_a_obj_pirateship.cpp:176-180`) is
    /// `mpModel->setBaseScale(scale); transS(current.pos); ZXYrotM(shape_angle.x, shape_angle.y,
    /// shape_angle.z); mpModel->setBaseTRMtx(...)`, so the base transform is fully determined by the
    /// ship's position and `shape_angle`. That is transcribed here.
    ///
    /// DROPPED: the wind-dependent half of `sail_move` - see the module docs.
    fn execute(&mut self, actor: &mut Actor, ship: Option<(&Vec3, [i16; 3])>) {
        let Some((pos, shape)) = ship else {
            // `l_p_ship` is a NULL `static`; retail dereferences it at `:555` unconditionally.
            return;
        };
        // `cMtx_multVec(l_p_ship->mModel->getBaseTRMtx(), &sail_offset, &i_this->current.pos);`
        // (:555)
        actor.pos = zxy_rotate(*pos, shape, MAST_TOP);
        // `i_this->current.angle = l_p_ship->shape_angle;` (:556)
        actor.angle = shape;
        // `sail_move(i_this);` (:557)
        self.move_step();
    }

    /// `daSail_Create`'s HIO override tail (`:501-503`) of `sail_move`:
    /// ```
    /// if (!demo_move(i_this) && l_HIO.m08 != 0) { i_this->mSailPacket.m1C44 = l_HIO.m0C; }
    /// ```
    ///
    /// `demo_move` (`:458-473`) needs `dComIfGp_demo_getActor(demoActorID)` and
    /// `dDemo_actor_c::checkEnable(ENABLE_ANM_FRAME_e) / getAnmFrame()` - the demo subsystem, which
    /// the sim does not carry - so it is OMITTED and reads FALSE, which is also what retail does for
    /// a stage-placed sail whose `demoActorID` is never set.
    fn override_furl(&mut self) {
        // `l_HIO.m08 != 0` (:501)
        if self.override_flag != 0 {
            // `i_this->mSailPacket.m1C44 = l_HIO.m0C;` (:502)
            self.furl = self.override_furl;
        }
    }

    /// The furl state machine, `sail_move` (`:476-550`) minus the wind-driven half.
    ///
    /// OMITTED from this function: `dKyw_get_wind_vec()` (`:478`) and every consumer of
    /// `windAngle` (`:506-507`, `:346`) - the brief lists `dKyw_get_wind_vec` as a config symbol
    /// with NO value in this checkout, so it is UNKNOWN rather than proven zero and nothing is
    /// substituted. What remains - the `l_HIO.m06` / `m1C44` / `m1C48` furl logic (`:479-499`), the
    /// HIO override (`:501-503`) and the slack term (`:504`) - reads no wind and is transcribed in
    /// full. DROPPED with the wind: `sail_pos_move` (`:342-455`, called at `:519`) in its entirety,
    /// which is the 84-vertex cloth solve with its `l_pos[]` seed table from
    /// `assets/l_pos__d_a_sail.h`, plus `setCorrectNrmAngle` (`:72-92`), `setNrmMtx` (`:95-97`),
    /// `setNrmVtx` (`:112-161`) and `setBackNrm` (`:100-109`) - all renderer normals.
    fn move_step(&mut self) {
        // ```
        // if (l_HIO.m06 == 0 && m1C44 < REG_SAIL_F(25) + 0.6f) { ...unfurl... }
        // else { if (l_HIO.m06 == 1 && m1C44 > 0.0f) { ...furl... } else { m1C48 = 0.0f; } }
        // ```
        // (:479-493)
        if self.direction == 0 && self.furl < UNFURL_TARGET + REG_SAIL_F25 {
            // ```
            // if (m1C48 <= 0.0f) { m1C48 = 0.015f; } else { m1C48 -= 0.001f; }
            // ```
            // (:480-484) - the rate RELOADS at 0.015 and otherwise bleeds down by 0.001.
            if self.rate <= 0.0 {
                self.rate = UNFURL_RATE_MAX;
            } else {
                self.rate -= UNFURL_RATE_DECAY;
            }
            // `cLib_addCalc(&m1C44, REG_SAIL_F(25) + 0.6f, 0.1f, m1C48, 0.01f);` (:485) -
            // `cLib_addCalc` is `super::walk::add_calc` (`src/sim/walk.rs:74-92`).
            add_calc(
                &mut self.furl,
                UNFURL_TARGET + REG_SAIL_F25,
                UNFURL_SCALE,
                self.rate,
                UNFURL_MIN_STEP,
            );
        } else if self.direction == 1 && self.furl > FURL_TARGET {
            // ```
            // m1C48 += 0.0075f;
            // cLib_addCalc(&m1C44, 0.0f, 0.3f, m1C48, 0.01f);
            // ```
            // (:487-489)
            self.rate += FURL_RATE_STEP;
            add_calc(
                &mut self.furl,
                FURL_TARGET,
                FURL_SCALE,
                self.rate,
                FURL_MIN_STEP,
            );
        } else {
            // `} else { i_this->mSailPacket.m1C48 = 0.0f; }` (:490-492)
            self.rate = 0.0;
        }
        // ```
        // if (m1C44 > 1.0f) { m1C44 = 1.0f; } else if (m1C44 < 0.0f) { m1C44 = 0.0f; }
        // ```
        // (:495-499)
        if self.furl > FURL_CAP {
            self.furl = FURL_CAP;
        } else if self.furl < 0.0 {
            self.furl = 0.0;
        }
        // `if (!demo_move(i_this) && l_HIO.m08 != 0) { m1C44 = l_HIO.m0C; }` (:501-503)
        self.override_furl();
        // `i_this->mSailPacket.m1C4C = (i_this->mSailPacket.m1C44 - 0.6f) * 15.0f;` (:504)
        self.slack = (self.furl - SLACK_PIVOT) * SLACK_SCALE;
    }
}

/// `mDoMtx_stack_c::transS(pos)` then `ZXYrotM(x, y, z)` then `multVec(vec)`
/// (`d_a_obj_pirateship.cpp:178-179`, `d_a_sail.cpp:555`), applied in that order: rotate about X,
/// then Y, then Z.
fn zxy_rotate(origin: Vec3, angle: [i16; 3], offset: [f32; 3]) -> Vec3 {
    let v = Vec3::new(offset[0], offset[1], offset[2]);
    // X rotation, then Y, then Z - each through the crate's single radians path (RULE 5b).
    let v = rot_x(v, angle[0]);
    let v = rot_y(v, angle[1]);
    rot_z(v, angle[2]) + origin
}

fn rot_x(v: Vec3, angle: i16) -> Vec3 {
    let (s, c) = super::stream::sin_cos(angle);
    Vec3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}

fn rot_y(v: Vec3, angle: i16) -> Vec3 {
    let (s, c) = super::stream::sin_cos(angle);
    Vec3::new(v.x * c + v.z * s, v.y, -v.x * s + v.z * c)
}

fn rot_z(v: Vec3, angle: i16) -> Vec3 {
    let (s, c) = super::stream::sin_cos(angle);
    Vec3::new(v.x * c - v.y * s, v.x * s + v.y * c, v.z)
}

pub fn is_sail(name: &str) -> bool {
    SAIL_ACTORS.contains(&name)
}

pub fn control(
    mut sails: Query<(Entity, &mut Actor, &mut Sail), With<Sail>>,
    ships: Query<(&Actor, &Pirate), Without<Sail>>,
    mut commands: Commands,
) {
    // RULE 9: the `&mut Actor` writer carries `With<Sail>` and this reader `Without<Sail>` - the
    // disjointness pair.
    let ship = ships.iter().next();
    for (entity, mut actor, mut sail) in &mut sails {
        if sail.override_furl == sail.override_furl && sail.textured == HIO_M05 && !sail.moved() {
            // `daSail_Create` (`:596-652`), first tick only - `m2CC == 0` means a sail-less ship.
            sail.create_init(ship.map(|(_, pirate)| pirate.sail));
        }
        // `daSail_Execute` (`:553-559`)
        sail.execute(
            &mut actor,
            ship.map(|(actor, _)| (&actor.pos, actor.shape_angle)),
        );
        // `daSail_IsDelete` (`:562-565`) is the bare `return TRUE` form - RULE 6b, it ACCEPTS a
        // delete request and is not a despawn. `daSail_Delete` (`:568-578`) releases two archives
        // and unregisters the HIO child; nothing here is modelled, and nothing in the translation
        // unit calls `fopAcM_delete`.
        let _ = (entity, &mut commands);
    }
}

impl Sail {
    /// Whether `create_init` has already run. The HIO ctor values are a usable sentinel because
    /// `daSail_Create` always overwrites `m06` (`:639`, `:642`) - so `override_furl == override_furl`
    /// is always true and cannot serve. `create_init` is instead gated on the FURLED state, which
    /// `daSail_Create` always leaves at either `0.0f` (`:628`) or `0.6f` (`:640`) and `sail_move`
    /// then moves away from on the next tick.
    fn moved(&self) -> bool {
        self.rate != 0.0 || self.furl > FURLED_ZERO
    }
}
