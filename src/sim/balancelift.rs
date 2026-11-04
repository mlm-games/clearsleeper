//! `daBalancelift_c` (`tww/src/d/actor/d_a_obj_balancelift.cpp`): the Tower-of-the-Gods pair of
//! scales. One anchor point at the top, a platform hung `mLength` below it, a quaternion that keeps
//! that platform swinging like a pendulum under a spring, and a weight counter that decides how far
//! down the scale hangs. Archive "Hten1" (`d_a_obj_balancelift.cpp:131`), process
//! `fpcNm_Balancelift_e`, stage name `"Blift"` (`tww/src/d/d_stage.cpp:919`).
//!
//! What survives here is the whole of the motion: [`Balancelift::calc_weight`]
//! (`d_a_obj_balancelift.cpp:180-218`), [`Balancelift::calc_quat`] (`d_a_obj_balancelift.cpp:134-157`),
//! [`Balancelift::set_mtx`] (`d_a_obj_balancelift.cpp:160-177`) as far as its one read-back goes, and
//! [`Balancelift::tg_hit`] (`d_a_obj_balancelift.cpp:359-392`) - the bomb, wind and sword reactions
//! that kick the platform and stretch the chain. The tunables are all read at the
//! `daBalancelift_HIO_c` constructor (`d_a_obj_balancelift.cpp:77-89`), which is how they became the
//! constants below, and the two debug floats of `d_a_obj_balancelift.cpp:149` and
//! `d_a_obj_balancelift.cpp:154` are what `REG10_F(23)` and `REG10_F(24)` add to those slerp weights,
//! and a release build leaves them at zero (`tww/src/d/d_s_play.cpp:78-87`), so the weights are the
//! literals 0.1 and 0.15.
//!
//! Three subjects are NOT carried, and each of them changes what the actor can do:
//!
//! * **The room's PATHS block.** `CreateInit` reads a path number out of the parameter word and asks
//!   `dPath_GetRoomPath` for it (`d_a_obj_balancelift.cpp:242-244`, `tww/src/d/d_path.cpp:22-39`); with
//!   a path it takes its own position from `mpPath->m_points[0]` and creates the OTHER scale as a child
//!   at `mpPath->m_points[1]` (`d_a_obj_balancelift.cpp:246-251`,
//!   `tww/include/f_op/f_op_actor_mng.h:503-508`). `src/dzs.rs` parses no PATHS block (RULE 3b), so the
//!   call answers null in this port, the guard at `d_a_obj_balancelift.cpp:245` fails, and neither half
//!   runs: the lifter keeps the position the room placed it at, no partner is ever created, and the
//!   path number is not kept either, because nothing is left to branch on it.
//! * **The pair, and with it the scale's tilt.** The child half of `CreateInit`
//!   (`d_a_obj_balancelift.cpp:262-272`) is what aims `mpWeight` and `mpUpdateFlags` at the PARENT's
//!   counters, and it is unreachable for the same reason. So `mSide` is `0` forever, which makes the
//!   raise at `d_a_obj_balancelift.cpp:190` the only bit ever set, the comparison at
//!   `d_a_obj_balancelift.cpp:201` never true, and the counter never reset; the two side-1 arms
//!   (`d_a_obj_balancelift.cpp:191-200`, and the subtracting `*mpWeight` of
//!   `d_a_obj_balancelift.cpp:127`) belong to an actor this port cannot spawn. `mSide` and
//!   `mUpdateFlags` are therefore not stored at all (RULE 7: nothing reads a constant), and the length
//!   target always comes from the `mSide == 0` arm.
//! * **`pm_bgw`, the platform's own background.** `CreateHeap` builds it from the archive's DZB: the
//!   `pm_bgw` of `d_a_obj_balancelift.cpp:233-236` is that new object and its `dBgW::Set`. The
//!   `CreateInit` line at `d_a_obj_balancelift.cpp:281` registers it and the
//!   `SetRideCallback` of `d_a_obj_balancelift.cpp:292` publishes the ride callback, while the
//!   `pm_bgw->Move()` of `d_a_obj_balancelift.cpp:357` moves it. The port has no per-actor DZB and no
//!   polygon list for a model archive, so the platform is not a solid the way retail's is: it neither
//!   blocks a trace nor carries its rider, because the callback's `dBgS_MoveBGProc_Typical`
//!   (`tww/src/d/d_bg_s.cpp:489-498`) re-places the rider from the background's matrix delta and there
//!   is no matrix here. What IS ported is the callback's BODY (`d_a_obj_balancelift.cpp:117-129`): the
//!   swing impulse it adds and the weight it counts. Retail dispatches it from the rider's own ground
//!   check (`tww/src/d/d_bg_s_acch.cpp:173`), which the port has no per-actor version of, so
//!   [`Balancelift::carries`] stands in for it geometrically and the call sits in the lifter's own
//!   tick instead of the rider's.
//!
//! Renderer and heap work, dropped with the line each is read at: `CreateHeap` itself
//! (`d_a_obj_balancelift.cpp:226-238`), whose failure `fopAcM_entrySolidHeap` turns into `cPhs_ERROR_e`
//! (`d_a_obj_balancelift.cpp:315-319`) - `spawn` never refuses; `M_mdl` and `_draw`
//! (`d_a_obj_balancelift.cpp:171`, `d_a_obj_balancelift.cpp:400-406`); the model matrix's
//! `quatM(&mPlatformQuat)` and `scaleM(l_HIO.mScale)` (`d_a_obj_balancelift.cpp:169-170`) with the
//! `scale.setall(2.0f)` they match (`d_a_obj_balancelift.cpp:290`), which is the only reader of
//! `mPlatformQuat`, so the second slerp of `calc_quat` (`d_a_obj_balancelift.cpp:152-156`) and its
//! field go with it, and `mTop` with them: the `mTop` of `d_a_obj_balancelift.cpp:164-165` is the point
//! 300 above the platform, which only the `calc_quat` arm of `d_a_obj_balancelift.cpp:142-145` reads,
//! to produce `mPlatformQuat`. The `mBgMtx` of `d_a_obj_balancelift.cpp:172` matches the `MTXCopy` at
//! `d_a_obj_balancelift.cpp:322`, and the `Release` of `d_a_obj_balancelift.cpp:334` drops the
//! background itself. The `dChain_packet_create` at `d_a_obj_balancelift.cpp:231` is the drawn chain,
//! whose `mChainJoint` of `d_a_obj_balancelift.cpp:166` is one of the three `points` that
//! `mPlatformPos` and `current.pos` fill at `d_a_obj_balancelift.cpp:173-176`; the
//! `dComIfGp_particle_set(dPa_name::ID_AK_JN_NG, hitPos)`
//! (`d_a_obj_balancelift.cpp:388`, `tww/include/d/d_particle_name.h:45`) of the sword arm, which has no
//! queue in [`super::events`]; and the debug panel - `mDoHIO_createChild`
//! (`d_a_obj_balancelift.cpp:298-300`) and `mDoHIO_deleteChild` (`d_a_obj_balancelift.cpp:340-343`).
//! `mStts.Init(0xc0, 0xff, this)` (`d_a_obj_balancelift.cpp:293`) is the collision weight, which
//! [`super::weight`] tabulates, and the `mStts` move of `d_a_obj_balancelift.cpp:358` is what the
//! shared contact pass carries (`super::contact_run`). The `IsDelete` of
//! `d_a_obj_balancelift.cpp:413-415` is `TRUE` and the profile status of `d_a_obj_balancelift.cpp:437`
//! is `fopAcStts_CULL_e | fopAcStts_UNK40000_e`, with no `fopAcStts_FREEZE_e`, so the lifter is culled
//! but never frozen.
//!
//! The colliders are the reference's own: [`super::bodies`] holds the `"Blift"` row transcribed from
//! `l_cyl_src` (`d_a_obj_balancelift.cpp:45-73`) plus the `mCyl` setup of
//! `d_a_obj_balancelift.cpp:294-297`, and the `mCyl.SetC(mPlatformPos)` at `d_a_obj_balancelift.cpp:394`
//! is why [`Balancelift::execute`] re-anchors that body to the platform rather than leaving it on
//! `Actor::pos`. Its CO group (`cCcD_CoSPrm_Set_e | cCcD_CoSPrm_IsPlayer_e | cCcD_CoSPrm_VsGrpAll_e`,
//! `d_a_obj_balancelift.cpp:54`) can only ever correct another actor: the player registers no CO body
//! at all (`super::contact_run`'s `player_velocity` note), so the pan does not stop Link from walking
//! through it.
use bevy_ecs::prelude::*;
use glam::{Mat4, Quat, Vec3};

use super::actor::Actor;
use super::armos::ARMOS_ACTORS;
use super::carry::Carry;
use super::contact_run::{ContactBodies, ContactResults};
use super::events::{SimEvent, SimEvents};
use super::makar::MAKAR_ACTOR;
use super::player::Player;

/// `OBJNAME("Blift", fpcNm_Balancelift_e, 255, 0)` - the stage name, whose `255` leaves the whole
/// parameter word to the actor.
pub const BLIFT_ACTORS: [&str; 1] = ["Blift"]; // <- d_stage.cpp:919

/// `fopAcM_setCullSizeBox(this, -150.0f, -150.0f, -150.0f, 150.0f, 1000.0f, 150.0f)`.
const CULL_MIN: [f32; 3] = [-150.0, -150.0, -150.0]; // <- d_a_obj_balancelift.cpp:321
const CULL_MAX: [f32; 3] = [150.0, 1000.0, 150.0]; // <- d_a_obj_balancelift.cpp:321

/// `l_cyl_src`'s `cM3dGCylS` radius, the width of the pan. The row in [`super::bodies`] carries the 200
/// height with it; only the radius is read here, as the standing test's reach.
const CYL_RADIUS: f32 = 30.0; // <- d_a_obj_balancelift.cpp:70

/// `daBalancelift_HIO_c`'s constructor, the eight constants the simulation runs on.
const SPRING: f32 = 0.05; // <- d_a_obj_balancelift.cpp:80
const RIDE_FORCE: f32 = 0.005; // <- d_a_obj_balancelift.cpp:81
const DAMPING: f32 = 0.9; // <- d_a_obj_balancelift.cpp:82
const HIT_FORCE: f32 = 25.0; // <- d_a_obj_balancelift.cpp:83
const CHAIN_DAMPING: f32 = 0.65; // <- d_a_obj_balancelift.cpp:84
const CHAIN_SPRING: f32 = 0.9; // <- d_a_obj_balancelift.cpp:85
const TRAVEL: f32 = 440.0; // <- d_a_obj_balancelift.cpp:86
const MAX_LENGTH: f32 = 1800.0; // <- d_a_obj_balancelift.cpp:87

/// The spring that pulls `mLength` toward its target and the damping on it: the two literals in
/// `calc_weight` that are not `l_HIO` fields.
const LENGTH_SPRING: f32 = 0.005; // <- d_a_obj_balancelift.cpp:206
const LENGTH_DAMPING: f32 = 0.92; // <- d_a_obj_balancelift.cpp:207

/// `0.1f + REG10_F(23)`, the swing slerp's weight, the debug half of which is zero in a release build.
const SLERP_SWING: f32 = 0.1; // <- d_a_obj_balancelift.cpp:149

/// The length change that starts the moving sound.
const LENGTH_SOUND: f32 = 1.0; // <- d_a_obj_balancelift.cpp:215

/// The three pushes the hit reactions make, the drop of the point they aim at, and the gain on the
/// length term that every one of them adds.
const BOMB_PUSH: f32 = -20.0; // <- d_a_obj_balancelift.cpp:369
const WIND_PUSH: f32 = -12.0; // <- d_a_obj_balancelift.cpp:377
const SWORD_PUSH: f32 = -8.0; // <- d_a_obj_balancelift.cpp:380
const PLATFORM_DROP: f32 = 50.0; // <- d_a_obj_balancelift.cpp:364
const LENGTH_GAIN: f32 = 2.0; // <- d_a_obj_balancelift.cpp:370

/// `JA_SE_OBJ_TENBIN_LIFT_MOVE` and `JA_SE_OBJ_COL_SWS_NMTLP`.
const MOVE_SE: u32 = 0x3024; // <- d_a_obj_balancelift.cpp:216, JAZelAudio_SE.h:619
const SWORD_SE: u32 = 0x6817; // <- d_a_obj_balancelift.cpp:390, JAZelAudio_SE.h:1742

/// `AT_TYPE_SWORD` and `AT_TYPE_WIND`, the two `ChkAtType` tests of the sword and wind arms.
const AT_TYPE_SWORD: u32 = 1 << 1; // <- d_cc_d.h:14
const AT_TYPE_WIND: u32 = 1 << 21; // <- d_cc_d.h:32

/// `fpcNm_BOMB_e`, the process name the first hit arm of `_execute` compares at
/// `d_a_obj_balancelift.cpp:362`, spelled `"Bomb"` by the stage table (`tww/src/d/d_stage.cpp:1247`).
const BOMB_ACTOR: &str = "Bomb";

/// The fifteen `fpcNm_Obj_Try_e` stage strings (`tww/src/d/d_stage.cpp:927-941`), each worth one unit
/// of weight (`d_a_obj_balancelift.cpp:95-96`) with the `fpcNm_AM2_e`, `fpcNm_NPC_OS_e` and
/// `fpcNm_NPC_CB1_e` names of the same chain (`:107-110`) - `"amos2"`, `"Os"`, `"Os1"`, `"Os2"` and
/// `"Cb1"`, which the stage table gives at `tww/src/d/d_stage.cpp:677`, `:706-708`, `:731`.
const TRY_ACTORS: [&str; 15] = [
    "Hseki1", "Hseki6", "Hseki3", "Hseki4", "Hseki5", "Hmon1", "Hmon1d", "Hmon2", "Hmon2d",
    "Hseki31", "Hseki41", "Hseki51", "Hseki32", "Hseki42", "Hseki52",
];
const OS_ACTORS: [&str; 3] = ["Os", "Os1", "Os2"];

/// The port's own standing tolerance, the same value `super::pushable` gives the one-polygon
/// tolerance of the retail ground check this test stands in for.
const RIDE_BAND: f32 = 1.0;

/// One hit object's weight, `daObjBlift_ride_actor_check` (`d_a_obj_balancelift.cpp:92-114`) for
/// everything but the player, whose own arm recurses and needs the grab id.
fn rider_weight(name: &str) -> i16 {
    // `fpcNm_Obj_Try_e` at `:95-96`, then the three NPC names tested in one `else if` chain at
    // `:107-110`; every other process falls through `fopAc_IsActor` and the name tests to the
    // initialised `int weight = 0` (`:93`).
    if TRY_ACTORS.contains(&name)
        || ARMOS_ACTORS.contains(&name)
        || OS_ACTORS.contains(&name)
        || name == MAKAR_ACTOR
    {
        1
    } else {
        0
    }
}

/// `daObj::quat_rotBaseY2` (`tww/src/d/d_a_obj.cpp:199-220`): the shortest-arc quaternion from
/// `cXyz::BaseY` onto `axis`. Both degenerate answers are the identity - a vector whose
/// `normalizeRS` fails (`tww/include/SSystem/SComponent/c_xyz.h:111`, squared length under 8e-11)
/// takes `zero_quat` at `d_a_obj.cpp:218`, and an anti-parallel one, whose denominator is at or under
/// `0.0001`, takes the same value at `d_a_obj.cpp:215`.
fn quat_rot_base_y2(axis: Vec3) -> Quat {
    let square = axis.length_squared();
    if square < 8e-11 {
        return Quat::IDENTITY;
    }
    let unit = axis / square.sqrt();
    let half = (2.0 * (unit.y + 1.0)).sqrt();
    if half <= 0.0001 {
        return Quat::IDENTITY;
    }
    // `cXyz::BaseY.outprod(sp18)` for `(0, 1, 0)` crossed into `(x, y, z)` is `(z, 0, -x)`, and the
    // scalar is `0.5 * half` while each component is divided by `half` itself
    // (`d_a_obj.cpp:209-213`).
    Quat::from_xyzw(unit.z / half, 0.0, -unit.x / half, 0.5 * half)
}

/// `cXyz::normalizeRS` (`tww/src/SSystem/SComponent/c_xyz.cpp:120-128`): true, with the vector
/// replaced by its unit form, or false with the vector left exactly as it was when the squared
/// length is under the same `isNearZeroSquare` bound.
fn normalize_rs(v: &mut Vec3) -> bool {
    let square = v.length_squared();
    if square < 8e-11 {
        return false;
    }
    *v /= square.sqrt();
    true
}

/// `cXyz::abs2XZ` (`tww/include/SSystem/SComponent/c_xyz.h:115-123`): the SQUARED XZ length, with y
/// dropped before the sum, which is what `calc_weight` subtracts from the target's square.
fn abs2_xz(v: Vec3) -> f32 {
    v.x * v.x + v.z * v.z
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Balancelift {
    /// `mSwingPos` (`d_a_obj_balancelift.h:30`), the point the platform's own spring pulls at.
    pub swing_pos: Vec3,
    /// `mSwingVelocity` (`d_a_obj_balancelift.h:31`), shared by the spring, the damping and every
    /// impulse.
    pub swing_velocity: Vec3,
    /// `mSwingQuat` (`d_a_obj_balancelift.h:28`), the pendulum's lean.
    pub swing_quat: Quat,
    /// `mPlatformPos` (`d_a_obj_balancelift.h:32`), the pan's origin, which the collider is centred on.
    pub platform_pos: Vec3,
    /// `mLength` (`d_a_obj_balancelift.h:33`), how far the platform hangs.
    pub length: f32,
    /// `mLengthVelocity` (`d_a_obj_balancelift.h:44`), which `calc_weight` integrates and every hit
    /// arm adds to.
    pub length_velocity: f32,
    /// `mChainOffset` (`d_a_obj_balancelift.h:24`), the chain's swing offset, whose XZ square the length
    /// target pays for.
    pub chain_offset: Vec3,
    /// `mChainTarget` (`d_a_obj_balancelift.h:25`), the point the chain springs back to; a sword hit
    /// moves only its y.
    pub chain_target: Vec3,
    /// `mChainVelocity` (`d_a_obj_balancelift.h:26`).
    pub chain_velocity: Vec3,
    /// `mWeight` (`d_a_obj_balancelift.h:34`), the counter `mpWeight` (`:35`) aims at - which, with no
    /// partner to aim it at the parent's copy, is always this actor's own.
    pub weight: i16,
    /// The port's own create latch: `CreateInit` runs once, on the first tick.
    pub entered: bool,
}

impl Balancelift {
    /// `_create` (`d_a_obj_balancelift.cpp:305-325`) gets here from `Mthd_Create`'s
    /// `fopAcM_SetupActor`, and every field is one of the zeroes `CreateInit` rewrites on the first
    /// tick, since nothing in it can fail without the heap work `_create` early-returns on (`:315-319`).
    /// The parameter word is not read at all: its path number (`:242`) only feeds a lookup that cannot
    /// answer here.
    pub fn spawn() -> Self {
        Self {
            swing_pos: Vec3::ZERO,
            swing_velocity: Vec3::ZERO,
            swing_quat: Quat::IDENTITY,
            platform_pos: Vec3::ZERO,
            length: 0.0,
            length_velocity: 0.0,
            chain_offset: Vec3::ZERO,
            chain_target: Vec3::ZERO,
            chain_velocity: Vec3::ZERO,
            weight: 0,
            entered: false,
        }
    }

    /// The cull box `fopAcM_setCullSizeBox` hands the shell (`:321`), which is the swing volume
    /// rather than the pan.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `CreateInit` (`:241-302`), the state the actor runs on from its first frame, for an anchor at
    /// `actor.pos` (which is the placement, because the path half at `:245-251` cannot run).
    fn create_init(&mut self, anchor: Vec3) {
        // Both arms of the path branch (`:243`, `:259`) initialise the counter to the actor's own, with
        // `mSide = 0` (`:254`). The `mpWeight` and `mpUpdateFlags` of `:255-257` are exactly what the
        // no-parent else arm of the other branch settles on (`:274-276`), and the length at `:258` is
        // the same number as `:278`.
        self.weight = 0;
        self.length = MAX_LENGTH - 0.5 * TRAVEL; // <- :258, :278
        self.length_velocity = 0.0; // <- :280
        self.swing_pos = anchor;
        self.swing_pos.y -= self.length; // <- :282-283
        self.swing_velocity = Vec3::ZERO; // <- :284
        self.chain_velocity = Vec3::ZERO; // <- :285
        self.chain_offset = Vec3::ZERO; // <- :286
        self.chain_target = Vec3::ZERO; // <- :287
        // `ZeroQuat` is `{0, 0, 0, 1}` (`tww/src/d/d_lib.cpp:16`), i.e. the identity.
        self.swing_quat = Quat::IDENTITY; // <- :288-289
        self.entered = true;
        // `set_mtx()` (`:291`), before the first frame's physics.
        self.set_mtx(anchor);
    }

    /// `calc_weight` (`:180-218`): drive `mLength` toward the length the weight counter asks for and
    /// spring the chain back to its target, then start the moving sound.
    fn calc_weight(&mut self, anchor: Vec3, events: &mut SimEvents) {
        // The `mSide == 0` arm (`:182-190`). The side-1 arm at `:191-200` swaps the last two targets
        // for the partner scale, and `*mpUpdateFlags |= 1` (`:190`) with the `== 3` reset (`:201-204`)
        // are the pair's handshake, unreachable with one lifter, so the counter only ever grows.
        let target = if self.weight == 0 {
            MAX_LENGTH - 0.5 * TRAVEL
        } else if self.weight > 0 {
            MAX_LENGTH
        } else {
            MAX_LENGTH - TRAVEL
        };
        // `std::sqrtf(target * target - mChainOffset.abs2XZ())` (`:205`), which is a negative square
        // root whenever the chain has swung out further than the target reaches; retail leaves the NaN
        // in `mLength` and the sim does the same.
        let length = (target * target - abs2_xz(self.chain_offset)).sqrt(); // <- :205
        self.length_velocity += LENGTH_SPRING * (length - self.length); // <- :206
        self.length_velocity *= LENGTH_DAMPING; // <- :207
        self.length += self.length_velocity; // <- :208
        let mut force = self.chain_target - self.chain_offset; // <- :209
        force *= CHAIN_SPRING; // <- :210
        self.chain_velocity += force; // <- :211
        self.chain_velocity *= CHAIN_DAMPING; // <- :212
        self.chain_offset += self.chain_velocity; // <- :213
        // The decomp's own note above `fabsf` (`:214-215`) says the absolute value is taken of the
        // COMPARISON, which is `1` exactly when the velocity is over 1.0, so the test is the plain one.
        // The sound is `mDoAud_seStart` at the anchor (`:216`).
        if self.length_velocity > LENGTH_SOUND {
            events.list.push(SimEvent::SeAt(MOVE_SE, anchor));
        }
    }

    /// `calc_quat` (`:134-157`): the pendulum. `mSwingPos` is pulled onto the point `mLength` below the
    /// anchor and damped, and the swing quaternion is slerped onto the shortest arc that would carry
    /// `cXyz::BaseY` onto the vector from the swing point to the anchor.
    fn calc_quat(&mut self, anchor: Vec3) {
        let mut pos = anchor;
        pos.y -= self.length; // <- :135-136
        let mut force = pos - self.swing_pos; // <- :137
        force *= SPRING; // <- :138
        self.swing_velocity += force; // <- :139
        self.swing_velocity *= DAMPING; // <- :140
        self.swing_pos += self.swing_velocity; // <- :141
        // The anchor's vector is measured from the swing point (`:143-145`); the top's half of the
        // function (`:152-156`) writes `mPlatformQuat`, whose only reader is the model matrix.
        let mut head = anchor;
        head -= self.swing_pos;
        let mut target = quat_rot_base_y2(head);
        target = target.inverse();
        // `PSQUATInverse` (`tww/src/dolphin/mtx/quat.c:135-172`) is the true inverse, norm division
        // included, and `C_QUATSlerp` (`:193-224`) keeps its FIRST argument at t=0 and takes the
        // shortest arc - which is `Quat::inverse` and `Quat::slerp` as glam defines them.
        self.swing_quat = self.swing_quat.slerp(target, SLERP_SWING); // <- :150-151
    }

    /// `set_mtx` (`:160-177`), as far as the one point the simulation reads. The stack is
    /// `transS(current.pos)`, `quatM(&mSwingQuat)`, `transM(0, -mLength, 0)` (`:161-163`), and
    /// `multVec` is the full affine product (`tww/include/m_Do/m_Do_mtx.h:265`), translation
    /// included, which is `transform_point3`.
    fn set_mtx(&mut self, anchor: Vec3) {
        let mtx = Mat4::from_translation(anchor)
            * Mat4::from_quat(self.swing_quat)
            * Mat4::from_translation(Vec3::new(0.0, -self.length, 0.0));
        // `mTop` (`:164-165`) feeds only the dropped platform quaternion and `mChainJoint` (`:166`)
        // only the drawn chain; `mPlatformPos` is the origin of the model matrix (`:167-168`, `:174`)
        // and the centre of the collider (`:394`), so it is the read-back that survives.
        self.platform_pos = mtx.transform_point3(Vec3::ZERO); // <- :168
    }

    /// Whether `feet` are on this scale's pan. Retail asks the rider's own ground check -
    /// `dBgS_Acch::GroundCheck` (`tww/src/d/d_bg_s_acch.cpp:151-173`) finds a polygon of `pm_bgw` and
    /// `RideCallBack` (`tww/src/d/d_bg_s_acch.cpp:173`, `tww/src/d/d_bg_s.cpp:526-533`) dispatches the
    /// callback on it - and the pan's polygons are in the archive DZB this port does not have. The
    /// cylinder the actor registers for itself (`d_a_obj_balancelift.cpp:70`, centred on `mPlatformPos`
    /// at `:394`) is the one citable measure of the pan's width, so the test is that radius plus the
    /// port's standing tolerance, applied at the platform's own origin.
    fn carries(&self, feet: Vec3) -> bool {
        let delta = feet - self.platform_pos;
        abs2_xz(delta) <= CYL_RADIUS * CYL_RADIUS && delta.y.abs() <= RIDE_BAND
    }

    /// `ride_call_back` (`:117-129`) for one rider at `at`, weighed by
    /// `daObjBlift_ride_actor_check` (`:92-114`): a push on the swing toward the rider and the
    /// rider's weight onto the counter. The `mSide == 0` arm ADDS (`:124-126`); the subtracting arm at
    /// `:127` is the partner's.
    fn ride(&mut self, at: Vec3, weight: i16) {
        let mut force = at - self.swing_pos; // <- :121-122
        force *= RIDE_FORCE;
        self.swing_velocity += force; // <- :123
        self.weight += weight;
    }

    /// The `ChkTgHit` arm of `_execute` (`:359-393`): the three reactions that kick the scale, read off
    /// the hit the shared contact pass recorded against this entity's TG body last step - which is
    /// retail's own latency, since `dComIfG_Ccsp()->Set(&mCyl)` (`:395`) registers the cylinder at the
    /// END of the tick.
    fn tg_hit(
        &mut self,
        entity: Entity,
        actor: &Actor,
        actors: &Query<&Actor, Without<Balancelift>>,
        partners: &Query<&ContactBodies, Without<Balancelift>>,
        results: &ContactResults,
        events: &mut SimEvents,
    ) {
        let Some(hit) = results
            .at_tg
            .iter()
            .find(|rec| rec.tg == entity && rec.set_tg)
        else {
            return;
        };
        // `mCyl.GetTgHitAc()` (`:360`) is the single partner the object recorded, and
        // `fopAcM_GetProfName(actor) == fpcNm_BOMB_e` (`:362`) is its process name.
        let attacker = actors.get(hit.at).ok();
        if attacker.is_some_and(|other| other.name == BOMB_ACTOR) {
            // `pos = mPlatformPos; pos.y -= 50.0f; dir = pos - actor->current.pos` (`:363-365`), then
            // the `normalizeRS` guard that leaves `(0, 0, 1)` on a zero-length aim (`:366-368`).
            let mut pos = self.platform_pos;
            pos.y -= PLATFORM_DROP;
            if let Some(bomber) = attacker {
                let mut dir = pos - bomber.pos; // <- :365
                if !normalize_rs(&mut dir) {
                    dir = Vec3::Z; // <- :367
                }
                self.swing_velocity += dir * BOMB_PUSH; // <- :369
                self.length_velocity += LENGTH_GAIN * -dir.y; // <- :370
            }
            return;
        }
        // The object half (`:371`): the port reads the attacker's own AT word, since `ChkAtType` is a
        // test on the object that hit (`tww/include/SSystem/SComponent/c_cc_d.h:539`).
        let reflect = partners.get(hit.at).ok();
        let at_type = reflect
            .and_then(|bodies| bodies.objs.get(hit.at_body))
            .map_or(0, |obj| obj.set.at_type);
        let mut dir = hit.r_vec; // <- :372, GetTgRVecP
        if !normalize_rs(&mut dir) {
            dir = Vec3::Z; // <- :373-375
        }
        if at_type & AT_TYPE_WIND != 0 {
            self.swing_velocity += dir * WIND_PUSH; // <- :376-378
            self.length_velocity += LENGTH_GAIN * -dir.y;
        } else if at_type & AT_TYPE_SWORD != 0 {
            self.swing_velocity += dir * SWORD_PUSH; // <- :379-381
            self.length_velocity += LENGTH_GAIN * -dir.y;
            // `GetTgHitPosP()` (`:382`) is always the crossing point in this port, so the arm that
            // retail guards on a null pointer always runs. Both chain terms take the DISTANCE, leaving
            // x and z at whatever the chain already had (`d_a_obj_balancelift.cpp:384-387`), and the
            // distance itself is `delta.abs()` at `:385`, the plain length
            // (`tww/include/SSystem/SComponent/c_xyz.h:124`).
            let distance = (hit.cross - self.platform_pos).length(); // <- :385
            self.chain_velocity += dir * HIT_FORCE; // <- :386
            self.chain_offset.y = distance; // <- :387
            self.chain_target.y = distance; // <- :387
            // The spark is the dropped `dComIfGp_particle_set` (`:388`); the sound is
            // `mDoAud_seStart(JA_SE_OBJ_COL_SWS_NMTLP, &eyePos, 0, ...)` (`:390`) at the lifter's eye.
            events.list.push(SimEvent::SeAt(SWORD_SE, actor.eye));
        }
    }

    /// `daBalancelift_c::_execute` (`:353-397`), in the reference's order: `calc_weight`,
    /// `calc_quat`, `set_mtx`, the `pm_bgw->Move()` and `mStts.Move()` the port has no polygon or
    /// apid rotation for (`:357-358`), the hit arm, `mCyl.SetC(mPlatformPos)` (`:394`) and the
    /// re-registration at `:395`, which is `super::contact_run`'s own pass. The ride callback
    /// (`:292`, dispatched from the rider's tick at `d_bg_s_acch.cpp:173`) is run here because this
    /// port has no per-actor ground check to run it from.
    #[allow(clippy::too_many_arguments)]
    fn execute(
        &mut self,
        entity: Entity,
        actor: &Actor,
        bodies: &mut ContactBodies,
        events: &mut SimEvents,
        results: &ContactResults,
        actors: &Query<&Actor, Without<Balancelift>>,
        partners: &Query<&ContactBodies, Without<Balancelift>>,
        riders: &[(Vec3, i16)],
    ) {
        self.calc_weight(actor.pos, events); // <- :354
        self.calc_quat(actor.pos); // <- :355
        self.set_mtx(actor.pos); // <- :356
        for (feet, weight) in riders {
            if self.carries(*feet) {
                self.ride(*feet, *weight);
            }
        }
        self.tg_hit(entity, actor, actors, partners, results, events); // <- :359
        // The one body the row registers, kept on the pan rather than on `Actor::pos`, which is the
        // anchor at the top of the chain.
        bodies.follow(BODY, self.platform_pos);
    }
}

/// The `"Blift"` row's one volume, index 0 of [`super::bodies::rows_for`].
const BODY: usize = 0; // <- d_a_obj_balancelift.cpp:294

pub fn is_balancelift(name: &str) -> bool {
    BLIFT_ACTORS.contains(&name)
}

/// `_execute` for every scale in the room. The candidate riders are collected first because retail
/// runs the callback from the rider's own tick: the player, whose arm is the one that adds the
/// grabbed actor's weight (`:97-106`), and every other named process worth a unit (`:95-96`,
/// `:107-110`).
pub fn control(
    mut events: ResMut<SimEvents>,
    carry: Res<Carry>,
    results: Res<ContactResults>,
    players: Query<&Player, Without<Balancelift>>,
    actors: Query<&Actor, Without<Balancelift>>,
    partners: Query<&ContactBodies, Without<Balancelift>>,
    mut lifts: Query<(Entity, &Actor, &mut Balancelift, &mut ContactBodies), With<Balancelift>>,
) {
    let mut riders: Vec<(Vec3, i16)> = Vec::new();
    if let Some(player) = players.iter().next() {
        // The player's arm of the weight check is `++weight` plus the recursion into
        // `getGrabActorID()` (`:98-105`), which is one level deep because only the player holds
        // anything; a held process that is not itself a rider contributes its own 0 or 1.
        let held = carry
            .held
            .and_then(|entity| actors.get(entity).ok())
            .map_or(0, |actor| rider_weight(&actor.name));
        riders.push((player.pos, 1 + held));
    }
    for other in &actors {
        let weight = rider_weight(&other.name);
        if weight != 0 {
            riders.push((other.pos, weight));
        }
    }
    for (entity, actor, mut lift, mut bodies) in &mut lifts {
        if !lift.entered {
            lift.create_init(actor.pos);
        }
        lift.execute(
            entity,
            actor,
            &mut bodies,
            &mut events,
            &results,
            &actors,
            &partners,
            &riders,
        );
    }
}
