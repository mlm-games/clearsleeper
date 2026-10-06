use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::walk;

/// `OBJNAME("Hpbot1", fpcNm_SW_PROPELLER_e, 255, 0)` and `OBJNAME("Vpbot", fpcNm_SW_PROPELLER_e,
/// 255, 0)` (`d_stage.cpp:621-622`). `daSwProp_c::m_arcname[2] = { "Hpbot1", "Vpbot_00" }`
/// (`d_a_swpropeller.cpp:21`) are the ARCHIVE names, so the second stage string is `Vpbot` and not
/// `Vpbot_00` - the OBJNAME table is the authority, exactly the `Lwood`/`lwood` trap.
pub const SWPROPELLER_ACTORS: [&str; 2] = ["Hpbot1", "Vpbot"];

/// `daSwProp_prm::getType(i_this) { return (fopAcM_GetParam(i_this) >> 8) & 0x0F; }`
/// (`d_a_swpropeller.cpp:16`) over the whole u32 (`f_op_actor_mng.h:127-129`).
const TYPE_WIDTH: u32 = 0x0F; // <- d_a_swpropeller.cpp:16
const TYPE_SHIFT: u32 = 8; // <- d_a_swpropeller.cpp:16

/// `fopAcM_setCullSizeBox(this, -150.0f, -100.0f, -150.0f, 150.0f, 150.0f, 150.0f)`
/// (`d_a_swpropeller.cpp:82`) - note the box is NOT symmetric: min y is -100 while max y is 150.
/// `fopAcM_setCullSizeFar(this, 1.0f)` (`:83`) scales it by exactly 1, so the literal stands.
/// The profile's `fopAc_CULLBOX_CUSTOM_e` (`:272`) resolves to it.
const CULL_MIN: [f32; 3] = [-150.0, -100.0, -150.0]; // <- d_a_swpropeller.cpp:82
const CULL_MAX: [f32; 3] = [150.0, 150.0, 150.0]; // <- d_a_swpropeller.cpp:82

/// `mRotYVelTarget = mRotYVelTarget * -0.6f` (`d_a_swpropeller.cpp:193`) - the decay shrinks the
/// flip each time the speed catches the target.
const DECAY_FLIP: f32 = -0.6; // <- d_a_swpropeller.cpp:193
/// `if (abs(mRotYVelTarget) < 32)` (`:194`) - below this the target is zeroed and `mSpinDecay` clears.
const DECAY_SETTLE: i32 = 32; // <- d_a_swpropeller.cpp:194
/// `s16 angle = 30;` (`:187`) - `cLib_addCalcAngleS`'s `scale` while free-spinning.
const CHASE_SCALE: i16 = 30; // <- d_a_swpropeller.cpp:187
/// `if (mSpinDecay != 0) { angle = 10; }` (`:188-190`) - the `scale` while decaying.
const CHASE_SCALE_DECAY: i16 = 10; // <- d_a_swpropeller.cpp:189
/// `cLib_addCalcAngleS(&mRotYVel, mRotYVelTarget, angle, 100, 10)` (`:191`) - `maxStep`, `minStep`.
const CHASE_MAX_STEP: i16 = 100; // <- d_a_swpropeller.cpp:191
const CHASE_MIN_STEP: i16 = 10; // <- d_a_swpropeller.cpp:191
/// `if (mType == 1) mDoAud_seStart(JA_SE_OBJ_KM_WINDMILL, &current.pos,
/// mRotYVel / (f32)r30 * 100.0f, 0);` (`d_a_swpropeller.cpp:200-202`). The third argument is the
/// sound's PITCH, not a volume, and `SimEvent::Se` carries no pitch channel, so only the id
/// survives. `JA_SE_OBJ_KM_WINDMILL = 0x61B1` (`JAZelAudio_SE.h:1700`).
const SE_KM_WINDMILL: u32 = 0x61B1; // <- JAZelAudio_SE.h:1700
/// `if (mType == 1)` (`d_a_swpropeller.cpp:200`) - the only surviving reader of `mType`.
const TYPE_WINDMILL: u8 = 1; // <- d_a_swpropeller.cpp:200

/// The wind-swept propeller - `daSwProp_c` (`d_a_swpropeller.h:8-38`), the actor behind the stage
/// strings `Hpbot1` and `Vpbot`.
///
/// It is NOT a hazard, despite the name. `l_cyl_src` (`d_a_swpropeller.cpp:23-51`) sets
/// `SrcObjAt Type = 0` (`:27`), `SrcObjAt Atp = 0` (`:28`), `SrcObjAt SPrm = 0` (`:29`) and
/// `SrcGObjAt Se = 0` (`:33`), so it registers no attack collider at all: the only live field is
/// `SrcObjTg Type = AT_TYPE_ALL & ~AT_TYPE_BOOMERANG & ~AT_TYPE_FAN_SWING` (`:30`), a TARGET cylinder
/// that senses the wind and Link's weapons. The profile `Group` is `fopAc_ACTOR_e` (`:271`).
///
/// (a) inbound damage: NONE - with `SrcObjAt` all zero, nothing can hit it.
/// (b) outbound damage: NONE - `mCyl` is a `dCcD` Tg sensor, never an AT emitter; it reads other
///   actors' AT flags and writes only `mRotYVel` / `mRotYVelTarget` / `mSpinDecay`. Neither
///   `combat::HitPoints` nor `enemy::Hurt` is attached, and attaching them would let
///   `combat::control` despawn a propeller retail cannot damage.
///
/// Every input that could move the blades is the `cCcD` actor-collision registry, which RULE 3b
/// lists among the absent inputs. This port therefore keeps the physics and its zero-input result
/// rather than substituting a driver:
///   * `mAcch.CrrPos(*dComIfG_Bgsp())` (`:155`) is a `dBgS_ObjAcch` ground correct - RULE 3b.
///   * `if (mCyl.ChkTgHit()) { ... }` (`:156-179`) reads `GetTgHitObj()` and `ChkAtType(...)`, so
///     `u8 windHit = 0` (`:154`) keeps its initialiser every tick.
///   * `dComIfGp_getReverb(fopAcM_GetRoomNo(this))` (`:162`) has no modelled reverb channel.
///   * `AT_TYPE_WIND` collider semantics are themselves RULE 3b absent - five retail emitters, none
///     ported as targets - so the wind arm is unreachable twice over.
///
/// So `mRotYVel`, `mRotYVelTarget` and `mSpinDecay` keep the `fopAcM_ct` zero-fill, and the
/// transcribed chase is a genuine no-op: `cLib_addCalcAngleS` (`c_lib.cpp:160-189`) short-circuits
/// on `*pValue == target`. The one behaviour that does NOT depend on the registry is `mType == 1`'s
/// looping windmill voice (`:200-202`), which fires every tick at a pitch of
/// `0 / 4096.0 * 100.0` - the resting blade. `s16 r30 = 0x1000` (`:153`) is that divisor.
///
/// `daSwProp_IsDelete` (`:246-248`) is the bare "accepts a delete request" form of RULE 6b.
/// `_create` (`:129-141`) returns `cPhs_ERROR_e` only from `fopAcM_entrySolidHeap` (`:135`), which is
/// heap work, so `spawn` never refuses. There is no BCK frame read anywhere in this file, so no
/// frame count is invented.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwPropeller {
    /// `mType` (`d_a_swpropeller.h:28`, a `u8`) from `daSwProp_prm::getType` (`:16`). It also indexes
    /// `m_bdlidx`, `m_heapsize` and `m_arcname` (`:19-21`), all of which have exactly two entries
    /// while the decode takes FOUR bits (`:16`); those three are archive and heap work and are not
    /// transcribed, so nothing here can read past a table.
    pub kind: u8,
    /// `mRotYVel` (`d_a_swpropeller.h:26`, an `s16`) - the current blade speed, chased toward
    /// [`SwPropeller::vel_target`] and the numerator of the windmill sound's pitch.
    pub vel: i16,
    /// `mRotYVelTarget` (`d_a_swpropeller.h:27`, an `s16`) - the speed being chased toward.
    pub vel_target: i16,
    /// `mSpinDecay` (`d_a_swpropeller.h:29`, a `u8`) - picks `CHASE_SCALE_DECAY` over `CHASE_SCALE`
    /// (`:188-190`) and arms the settle test at `:192-198`.
    pub spin_decay: u8,
}

impl SwPropeller {
    /// `daSwProp_c::_create` (`:129-141`) plus `CreateInit` (`:80-107`), minus `fopAcM_ct`,
    /// `dComIfG_resLoad`, `fopAcM_entrySolidHeap`, `fopAcM_SetMtx` (`:81`), the cull box (`:82-83`,
    /// carried by [`SwPropeller::cull`]), the whole `mAcch` / `mStts` / `mCyl` setup (`:85-94`),
    /// `set_mtx` (`:97`), the `kaiten` joint-name scan (`:99-105`) and `mpModel->calc()` (`:106`) -
    /// heap, `dBgS_ObjAcch`, `dCcD`, model and joint work.
    ///
    /// RULE 2: `set_mtx` (`:144-149`) reads `scale` only for `setBaseScale`, and the stage-placed
    /// scale is exactly 1.0, so nothing is lost.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: ((parameters >> TYPE_SHIFT) & ((1 << TYPE_WIDTH) - 1)) as u8,
            // `mRotYVel` / `mRotYVelTarget` (`d_a_swpropeller.h:26-27`) and `mSpinDecay` (`:29`) have
            // no initialiser in `_create` or `CreateInit`, so `fopAcM_ct`'s clear-area zero-fill
            // (`f_pc_base.cpp:98`) makes them 0.
            vel: 0,
            vel_target: 0,
            spin_decay: 0,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -150.0f, -100.0f, -150.0f, 150.0f, 150.0f, 150.0f)`
    /// (`d_a_swpropeller.cpp:82`), followed by `fopAcM_setCullSizeFar(this, 1.0f)` (`:83`) - a
    /// multiplier of exactly 1, so the box is unchanged.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daSwProp_c::_execute` (`:152-211`), minus `mAcch.CrrPos` (`:155`), the whole `ChkTgHit` arm
    /// (`:156-179`), the `windHit` latch and `fopAcM_revSwitch` (`:180-185`), `set_mtx()` (`:205`)
    /// and the `mCyl.SetC` / `dComIfG_Ccsp()->Set` tail (`:206-209`) - all absent-input or
    /// renderer / `dCcD` work.
    fn execute(&mut self, events: &mut SimEvents) {
        // `s16 angle = 30; if (mSpinDecay != 0) { angle = 10; }` (:187-190)
        let scale = if self.spin_decay != 0 {
            CHASE_SCALE_DECAY
        } else {
            CHASE_SCALE
        };
        // `angle = cLib_addCalcAngleS(&mRotYVel, mRotYVelTarget, angle, 100, 10);` (:191). The decomp
        // BOTH mutates `*pValue` and returns `target - *pValue` (`c_lib.cpp:188`) - a signed
        // residual over the POST-update value, not its magnitude - so the new speed and the residual
        // are two separate reads here.
        let vel = walk::add_calc_angle_s(
            self.vel,
            self.vel_target,
            scale,
            CHASE_MAX_STEP,
            CHASE_MIN_STEP,
        );
        let angle = self.vel_target.wrapping_sub(vel);
        self.vel = vel;
        // `if (mSpinDecay != 0 && angle == 0) {` (:192)
        if self.spin_decay != 0 && angle == 0 {
            // `mRotYVelTarget = mRotYVelTarget * -0.6f;` (:193)
            let decayed = (f32::from(self.vel_target) * DECAY_FLIP) as i16;
            // `if (abs(mRotYVelTarget) < 32) { mRotYVelTarget = 0; mSpinDecay = 0; }` (:194-197).
            // The decomp's `abs` promotes to `int`, so RULE 10 wants `unsigned_abs` on the `i16`.
            if i32::from(decayed.unsigned_abs()) < DECAY_SETTLE {
                self.vel_target = 0;
                self.spin_decay = 0;
            } else {
                self.vel_target = decayed;
            }
        }
        // `if (mType == 1) mDoAud_seStart(JA_SE_OBJ_KM_WINDMILL, &current.pos,
        // mRotYVel / (f32)r30 * 100.0f, 0);` (:200-202) - a per-tick loop, so the id is pushed on
        // every tick for the windmill type.
        if self.kind == TYPE_WINDMILL {
            events.list.push(SimEvent::Se(SE_KM_WINDMILL));
        }
    }
}

pub fn is_swpropeller(name: &str) -> bool {
    SWPROPELLER_ACTORS.contains(&name)
}

pub fn control(mut events: ResMut<SimEvents>, mut props: Query<&mut SwPropeller>) {
    for mut prop in &mut props {
        prop.execute(&mut events);
    }
}
