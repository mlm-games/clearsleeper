use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::player::Player;
use super::walk;

/// `OBJNAME("Olift", fpcNm_LEAF_LIFT_e, 255, 0)` (`d_stage.cpp:617`) is `fpcNm_LEAF_LIFT_e` =
/// `d_a_leaflift.cpp` - the only such row in the table. The enum is spelled `LEAF_LIFT`, not
/// `LEAFLIFT`. The archive name agrees: `const char daLlift_c::m_arcname[6] = "Olift"`
/// (`d_a_leaflift.cpp:47`). No other `*_ACTORS` array or `pub const &str` in `src/sim/` claims
/// `Olift`.
pub const OLIFT_ACTORS: [&str; 1] = ["Olift"];

/// `fopAcM_setCullSizeBox(this, -300.0f, -600.0f, -300.0f, 300.0f, 100.0f, 300.0f)`
/// (`d_a_leaflift.cpp:111`). The profile's `fopAc_CULLBOX_CUSTOM_e` (`:378`) resolves to it.
/// `fopAcM_setCullSizeFar(this, 1.0f)` (`:112`) overrides only the far plane and creates no box.
const CULL_MIN: [f32; 3] = [-300.0, -600.0, -300.0]; // <- d_a_leaflift.cpp:111
const CULL_MAX: [f32; 3] = [300.0, 100.0, 300.0]; // <- d_a_leaflift.cpp:111

/// `daLlift_c::m_height = 560.0f` (`d_a_leaflift_static.cpp:11`) - the raised height, used by
/// `CreateInit` (`:139`) and as the ride cylinder's downward offset (`:270`).
const HEIGHT: f32 = 560.0; // <- d_a_leaflift_static.cpp:11
/// `daLlift_c::m_max_speed = 10.0f` (`d_a_leaflift_static.cpp:12`) - `MoveDownLift`'s `maxSpeed`
/// (`:299`, `:304`).
const MAX_SPEED: f32 = 10.0; // <- d_a_leaflift_static.cpp:12
/// `daLlift_c::m_min_speed = 5.0f` (`d_a_leaflift_static.cpp:13`) - `MoveDownLift`'s `minSpeed`.
const MIN_SPEED: f32 = 5.0; // <- d_a_leaflift_static.cpp:13

/// `daLlift_prm::getPos(item) = fopAcM_GetParam(item) & 0x0F` (`d_a_leaflift.h:64`) over the WHOLE
/// u32 (`f_op_actor_mng.h:127-129`).
const POS_WIDTH: u32 = 0x0F; // <- d_a_leaflift.h:64
/// `if ((s32)daLlift_prm::getPos(this) == 1) { current.pos.y = home.pos.y + m_height; }`
/// (`d_a_leaflift.cpp:138-140`).
const POS_RAISED: u32 = 1; // <- d_a_leaflift.cpp:138

/// `home.pos.y += 15.0f; current.pos.y = home.pos.y;` (`d_a_leaflift.cpp:127-128`) - `CreateInit`
/// raises the parked height by 15 before anything reads it.
const SPAWN_LIFT: f32 = 15.0; // <- d_a_leaflift.cpp:127
/// `m43F = 0x1e;` (`d_a_leaflift.cpp:146`) - the `u8` frame counter (`:249-254`) after which
/// `m43E` flips TRUE and a descent becomes eligible.
const RIDE_DELAY: u8 = 0x1e; // <- d_a_leaflift.cpp:146
/// `f32 f2 = 270.0f;` (`d_a_leaflift.cpp:247`) - past this XZ distance the lift descends.
const DESCEND_DIST: f32 = 270.0; // <- d_a_leaflift.cpp:247
/// `cLib_addCalc(&current.pos.y, home.pos.y, 0.1f, maxSpeed, minSpeed)` (`d_a_leaflift.cpp:304`).
const MOVE_SCALE: f32 = 0.1; // <- d_a_leaflift.cpp:304
/// `fopAcM_seStart(this, JA_SE_OBJ_LOTUS_LIFT_DOWN, 0)` (`d_a_leaflift.cpp:314`) -
/// `JA_SE_OBJ_LOTUS_LIFT_DOWN = 0x6980` (`JAZelAudio_SE.h:1909`), once per descent.
const SE_LOTUS_LIFT_DOWN: u32 = 0x6980; // <- JAZelAudio_SE.h:1909
/// `l_cyl_src`'s ride cylinder - `Radius 280.0f` / `Height 510.0f` (`d_a_leaflift.cpp:40-41`),
/// registered at `current.pos` minus [`HEIGHT`] each tick (`:269-272`).
const CYL_RADIUS: f32 = 280.0; // <- d_a_leaflift.cpp:40
const CYL_HEIGHT: f32 = 510.0; // <- d_a_leaflift.cpp:41

/// The Forest Haven lilypad lift - `daLlift_c` (`d_a_leaflift.h:12-61`), the actor behind the
/// single stage string `Olift`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `l_cyl_src` (`d_a_leaflift.cpp:15-43`) sets
/// `SrcObjAt Type` to 0 (`:20`) and `SrcObjTg Type` to 0 (`:22`), so `daLlift_c` registers no
/// attack collider and no damage target - only a `cCcD_CoSPrm_Set_e | cCcD_CoSPrm_IsOther_e |
/// cCcD_CoSPrm_VsGrpAll_e` collision cylinder that carries the player (`:24`, `:34`). The profile
/// `Group` is `fopAc_ACTOR_e` (`:377`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached;
/// attaching them would let `combat::control` despawn a lift retail cannot damage.
///
/// `daLlift_IsDelete` (`:351-354`) is the bare "accepts a delete request" form of RULE 6b, and
/// `fopAcM_delete` appears nowhere in this translation unit. `_create` (`:155-168`) returns
/// `cPhs_ERROR_e` only from the `fopAcM_entrySolidHeap` guard (`:160-162`), which is heap work, so
/// `spawn` never refuses.
///
/// Dropped as `dBgW` / model / particle work (RULE 3b): `rideCallBack` (`:189-224`),
/// `nodeCallBack` (`:171-186`), the `mCurrentRotation` / `mTargetRotation` quaternion pair, the
/// three `JPABaseEmitter*` (`:122-126`, `:278-295`), `mWaterY` (`:118`), `set_mtx` (`:227-232`),
/// `setMoveBGMtx` (`:235-240`) and `mpBgW->Move()` (`:151`, `:268`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Olift {
    /// `mbIsDescending` (`d_a_leaflift.h:51`, a `bool`) - armed by `_execute` (`:258`), consumed by
    /// `MoveDownLift` (`:301`) and cleared on arrival (`:307`).
    pub descending: bool,
    /// `mbIsAscending` (`d_a_leaflift.h:52`, a `bool`) - the one-shot latch guarding the descent
    /// sound, set at `:315`, cleared at `:306`.
    pub ascending: bool,
    /// `m43E` (`d_a_leaflift.h:43`, a `bool`) - "the ride has settled"; set by the `m43F` countdown
    /// reaching zero (`:251-253`) and gating the descent at `:255`.
    pub settled: bool,
    /// `m43F` (`d_a_leaflift.h:44`, a `u8`) - the [`RIDE_DELAY`] countdown (`:146`, `:249-250`).
    pub ride_delay: u8,
    /// `home.pos.y` - the parked height. `CreateInit` raises it by [`SPAWN_LIFT`] (`:127-128`) and
    /// `MoveDownLift` chases it (`:304`), while `current.pos.y` starts at it (`:128`) or
    /// `home.pos.y + m_height` (`:139`). The sim's `Actor` has no `home` slot, so the parked height
    /// is carried here; see [`Olift::enter`].
    pub parked_y: f32,
    /// `if ((s32)daLlift_prm::getPos(this) == 1)` (`d_a_leaflift.cpp:138`) - whether `CreateInit`
    /// starts the lift raised. `getPos` is read exactly once in the translation unit, but the flag
    /// itself must survive to the first tick, so unlike most `*_prm` reads this one is stored.
    pub raised: bool,
    /// `mActorLifetimeFrameCount` (`d_a_leaflift.h:47`, an `s16`), incremented every `_execute`
    /// (`:248`). Its only other reader is `rideCallBack`'s
    /// `(mActorLifetimeFrameCount % 32U) * (0x10000 / 32)` (`:208`), which is dropped, so it has no
    /// modelled effect and is not stored.
    ///
    /// `m43C` (`:41`) and `m43D` (`:42`) are only ever set from `rideCallBack` (`:197-198`, `:202`,
    /// `:223`) and cleared at `:264`. With that callback absent `m43C` stays FALSE forever, so
    /// `(!m43C) && (m43E)` at `:255` reduces to `m43E` - which is why the descent still works and
    /// neither flag is stored.
    ///
    /// `mTiltAngle` (`:45`) and `mEmitterTimer` (`:55`) are `rideCallBack` / emitter only.
    /// `mEmitterTimer` is never written anywhere, so `emitterCtrl`'s `== 200` test (`:279`) is dead
    /// in retail too and the whole of `emitterCtrl` (`:278-295`) is particle work.
    /// One-shot guard so `CreateInit` (`:109-152`) runs exactly once, on the first tick.
    pub entered: bool,
}

impl Olift {
    /// `daLlift_c::CreateInit` (`d_a_leaflift.cpp:109-152`) minus the renderer, the `dBgW` and the
    /// particles - see the struct docs for the full list of what that covers. Only the height
    /// bookkeeping is kept: `home.pos.y` and `current.pos.y`.
    ///
    /// It runs on the first tick rather than in `spawn` so that it happens exactly once, in the
    /// decomp's order, against a live actor.
    ///
    /// RULE 2: `dStage_actorInit` never writes `fopAcM_prm_class::scale`, so the lift's scale is
    /// exactly 1.0. `mpModel->setBaseScale(scale)` (`:228`) is renderer work.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            // `mbIsDescending` and `mbIsAscending` (`:51-52`) have no initialiser, so
            // `fopAcM_ct`'s `sBs_ClearArea` zero-fill (`f_pc_base.cpp:98`) makes both FALSE - and
            // FALSE is exactly `MoveDownLift`'s early return at `:301-303`.
            descending: false,
            ascending: false,
            // `m43E` (`:43`) likewise starts FALSE, so no descent is eligible until `m43F` expires.
            settled: false,
            // `m43F = 0x1e;` (`:146`)
            ride_delay: RIDE_DELAY,
            // `home.pos.y` is written at `:127` from the actor's own y; `enter` applies the lift.
            parked_y: 0.0,
            raised: (parameters & POS_WIDTH) == POS_RAISED,
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -300.0f, -600.0f, -300.0f, 300.0f, 100.0f, 300.0f)`
    /// (`d_a_leaflift.cpp:111`) - the actor's one real custom cull box, matching the profile's
    /// `fopAc_CULLBOX_CUSTOM_e` (`:378`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `l_cyl_src`'s ride cylinder, `Radius 280.0f` / `Height 510.0f` (`d_a_leaflift.cpp:40-41`),
    /// re-registered every tick at `current.pos` minus [`HEIGHT`] (`:269-272`).
    pub fn ride(&self) -> (f32, f32) {
        (CYL_RADIUS, CYL_HEIGHT)
    }

    /// `CreateInit`'s height bookkeeping, first tick only.
    ///
    /// `cXyz waterCheckPos = current.pos; waterCheckPos.y += 200.0f; mWaterY =
    /// dBgS_ObjGndChk_Wtr_Func(waterCheckPos);` (`:116-118`) is DROPPED, together with the `200.0f`
    /// probe lift at `:117`. `mWaterY` has exactly two readers, `:119` and `:121`, and both only
    /// decide whether `mEmitter3` exists - a particle emitter, which RULE 3b lists as absent. The
    /// query itself would have no counterpart regardless: `super::water::Water` is a single player
    /// probe, not a room query, the same omission `esa.rs:261-265` records.
    fn enter(&mut self, actor: &mut Actor) {
        // `home.pos.y += 15.0f;` (`:127`)
        self.parked_y = actor.pos.y + SPAWN_LIFT;
        // `current.pos.y = home.pos.y;` (`:128`)
        actor.pos.y = self.parked_y;
        // ```
        // if ((s32)daLlift_prm::getPos(this) == 1) {
        //     current.pos.y = home.pos.y + m_height;
        // } else {
        //     if (mEmitter3) { mEmitter3->playCreateParticle(); }
        // }
        // ```
        // (:138-145) - the `else` arm is emitter-only.
        if self.raised {
            actor.pos.y = self.parked_y + HEIGHT;
        }
        // `m43F = 0x1e;` (`:146`) - already [`RIDE_DELAY`] from `spawn`, restated because
        // `CreateInit` is its only writer.
        self.ride_delay = RIDE_DELAY;
        self.entered = true;
    }

    /// `daLlift_c::_execute` (`d_a_leaflift.cpp:243-275`), minus the dropped renderer, quaternion
    /// slerp, `dBgW` move, emitter control and cylinder registration - see the struct docs.
    fn execute(&mut self, actor: &mut Actor, player: Vec3, events: &mut SimEvents) {
        // ```
        // f32 distXZ = fopAcM_searchActorDistanceXZ(this, dComIfGp_getPlayer(0));
        // f32 f2 = 270.0f;
        // mActorLifetimeFrameCount++;
        // ```
        // (:246-248) - `searchActorDistanceXZ` is `(*other_pos - *this_pos).absXZ()`
        // (`f_op_actor_mng.cpp:511-515`), the XZ-only magnitude.
        let delta = player - actor.pos;
        let dist_xz = (delta.x * delta.x + delta.z * delta.z).sqrt();
        // `if (m43F > 0) { m43F--; if (m43F == 0) { m43E = TRUE; } }` (:249-254)
        if self.ride_delay > 0 {
            self.ride_delay -= 1;
            if self.ride_delay == 0 {
                self.settled = true;
            }
        }
        // ```
        // if ((!m43C) && (m43E)) {
        //     mTargetRotation = ZeroQuat;
        //     if (distXZ > f2) { mbIsDescending = TRUE; }
        // }
        // ```
        // (:255-260). `mTargetRotation = ZeroQuat` is quaternion state whose only readers are the
        // dropped `setMoveBGMtx` (`:238`) and slerp (`:266`), so only the `mbIsDescending` arm is
        // transcribed. `!m43C` is always true here - see [`Olift`].
        if self.settled && dist_xz > DESCEND_DIST {
            self.descending = true;
        }
        self.move_down_lift(actor, events);
    }

    /// `daLlift_c::MoveDownLift` (`d_a_leaflift.cpp:298-318`).
    fn move_down_lift(&mut self, actor: &mut Actor, events: &mut SimEvents) {
        // `if (!mbIsDescending) { return TRUE; }` (:301-303)
        if !self.descending {
            return;
        }
        // ```
        // float maxSpeed = m_max_speed;
        // float minSpeed = m_min_speed;
        // float downVel = cLib_addCalc(&current.pos.y, home.pos.y, 0.1f, maxSpeed, minSpeed);
        // ```
        // (:299-304). `cLib_addCalc` returns `fabsf(target - *pValue)` (`c_lib.cpp:52`), so its
        // zero return is exactly "the lift has arrived"; `walk::add_calc` is the same arithmetic
        // and returns nothing, so the arrival test is made on the value afterwards. `home.pos.y` is
        // [`Olift::parked_y`].
        walk::add_calc(&mut actor.pos.y, self.parked_y, MOVE_SCALE, MAX_SPEED, MIN_SPEED);
        let arrived = actor.pos.y == self.parked_y;
        // ```
        // if (downVel == 0.0f) {
        //     mbIsAscending = FALSE; mbIsDescending = FALSE;
        //     if (mEmitter3) { mEmitter3->playCreateParticle(); }
        //     return TRUE;
        // }
        // ```
        // (:305-312) - the emitter call goes with the dropped particle work.
        if arrived {
            self.ascending = false;
            self.descending = false;
            return;
        }
        // ```
        // if (downVel != 0.0f && !mbIsAscending) {
        //     fopAcM_seStart(this, JA_SE_OBJ_LOTUS_LIFT_DOWN, 0);
        //     mbIsAscending = TRUE;
        // }
        // ```
        // (:313-316)
        if !self.ascending {
            events.list.push(SimEvent::Se(SE_LOTUS_LIFT_DOWN));
            self.ascending = true;
        }
    }
}

pub fn is_olift(name: &str) -> bool {
    OLIFT_ACTORS.contains(&name)
}

pub fn control(
    players: Query<&Player>,
    mut events: ResMut<SimEvents>,
    mut lifts: Query<(&mut Actor, &mut Olift)>,
) {
    // `dComIfGp_getPlayer(0)` (`d_a_leaflift.cpp:246`) feeds both `searchActorDistanceXZ` and the
    // ride cylinder, and retail dereferences it unconditionally, so a tick with no player actor has
    // no defined behaviour. The sim skips it rather than substituting a position.
    let Some(player) = players.iter().next().map(|player| player.pos) else {
        return;
    };
    for (mut actor, mut lift) in &mut lifts {
        if !lift.entered {
            lift.enter(&mut actor);
        }
        lift.execute(&mut actor, player, &mut events);
    }
}