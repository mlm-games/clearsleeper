use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::ground;
use super::walk;

pub const ROHAT_ACTORS: [&str; 1] = ["RoHat"];

const STATE_NORMAL: u8 = 0; // <- d_a_obj_hat.cpp:54 (mState = 0), :141-143 (moveProc[0])
const GRAVITY: f32 = -5.0; // <- d_a_obj_hat.cpp:116
const MAX_FALL_SPEED: f32 = -100.0; // <- f_op_actor.cpp:286 (maxFallSpeed = -100.0f)
const GROUND_SCALE: f32 = 0.3; // <- d_a_obj_hat.cpp:164 (cLib_addCalc scale)
const GROUND_MAX: f32 = 1000.0; // <- d_a_obj_hat.cpp:164 (cLib_addCalc maxStep)
const GROUND_MIN: f32 = 1.0; // <- d_a_obj_hat.cpp:164 (cLib_addCalc minStep)

pub const CULL_MIN: [f32; 3] = [-50.0, 0.0, -50.0]; // <- d_a_obj_hat.cpp:115
pub const CULL_MAX: [f32; 3] = [50.0, 200.0, 50.0]; // <- d_a_obj_hat.cpp:115

/// The wind-borne travelling-merchant hats. The stage string is `RoHat` (`d_stage.cpp:1183`) and
/// the archive is `Ro` (`d_a_obj_hat.cpp:66`, `:80`).
///
/// `l_cyl_src` (`:18-46`) is a `dCcD_SrcCyl` of radius 70 and height 80 whose Tg type is
/// `AT_TYPE_WIND` (`:25`) with `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsEnemy_e` (`:26`). No retail
/// actor registers an `AT_TYPE_WIND` AT collider, so `mCyl.ChkTgHit()` (`:169`) never fires and
/// the `setSpeed(*mCyl.GetTgRVecP())` of `:170` is unreachable. The other `setSpeed` call,
/// `createInit`'s parent wind vector (`:118-121`), needs the linked parent actor, which `Actor`
/// does not carry, and `dKy` wind vectors are confirmed-absent. `mMoveNorm` therefore stays zero,
/// `setSpeed` (`:198-211`) is never entered, and the hat simply falls.
///
/// `mStts.Init(2, 0xff, this)` (`:111`) gives the Co collider `cCcD_AtSPrm_GrpAll_e`; the profile
/// is `fopAc_CULLBOX_CUSTOM_e` with a real `fopAcM_setCullSizeBox` (`:115`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct RoHat {
    /// `mMoveNorm` (`d_a_obj_hat.h:43`), written only by `setSpeed` (`:203`).
    pub move_norm: Vec3,
    /// `fopAc_ac_c::speedF`, written by `setSpeed` (`:204`) and by `_execute` (`:161`, `:164`).
    pub speed_f: f32,
    /// `fopAc_ac_c::speed`, written by `setSpeed` (`:210`) and `_execute` (`:156`).
    pub speed: Vec3,
    /// `fopAc_ac_c::gravity`, set once by `createInit` (`:116`).
    pub gravity: f32,
    /// `mState` (`d_a_obj_hat.h:44`), the `moveProc` index (`:148`).
    pub state: u8,
}

impl RoHat {
    /// `daObjHat_c` (`:51-55`) plus `createInit` (`:110-125`), minus `mStts.Init`, `mCyl.Set`,
    /// `mCyl.SetStts`, `fopAcM_SetMtx`, `fopAcM_setCullSizeBox` and `setMtx`. `mHatNo` is dropped
    /// under RULE 7: `getPrmHatNo` (`:181-183`) is its only writer and its only readers are the
    /// model/BCK selectors `l_bmd_ix_tbl` (`:15`) and `l_bck_ix_tbl` (`:16-17`) inside
    /// `createHeap` (`:80-87`), so the whole u32 parameter is unread.
    pub fn spawn() -> Self {
        Self {
            move_norm: Vec3::ZERO,
            speed_f: 0.0,
            speed: Vec3::ZERO,
            gravity: GRAVITY,
            state: STATE_NORMAL,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `_execute` (`d_a_obj_hat.cpp:147-175`), minus `setMtx` (`:173`) and the
    /// `mCyl.SetC`/`dComIfG_Ccsp()->Set(&mCyl)`/`setSpeed` pair (`:167-171`).
    ///
    /// `fopAcM_posMove(this, mStts.GetCCMoveP())` (`:158`) adds `speed` plus the Co collider's
    /// corrective move, which is `Collision::slide` here; `mAcch.CrrPos(*dComIfG_Bgsp())` (`:159`)
    /// is the reduced merged-DZB surface snap. The retail acch cylinder is
    /// `mAcchCir.SetWall(30.0f, 30.0f)` (`:94`); `Collision::slide` uses the crate-wide
    /// `ACTOR_HEIGHT`, the same reduction `kanban.rs` and `iball.rs` already make.
    fn execute(&mut self, actor: &mut Actor, solid: Option<&Collision>) {
        // `(this->*moveProc[mState])()` (`d_a_obj_hat.cpp:148`).
        match self.state {
            STATE_NORMAL => self.execute_normal(),
            _ => {}
        }

        let xspeed = self.move_norm.x * self.speed_f;
        let mut yspeed = self.speed.y + self.gravity;
        let zspeed = self.move_norm.z * self.speed_f;
        if yspeed < MAX_FALL_SPEED {
            yspeed = MAX_FALL_SPEED;
        }
        self.speed = Vec3::new(xspeed, yspeed, zspeed);

        let from = actor.pos;
        let to = actor.pos + self.speed;
        let flat = match solid {
            Some(solid) => solid.slide(from, to),
            None => to,
        };
        let wall_hit = flat != to;
        actor.pos = flat;
        let ground_hit = ground::grounded(solid, actor.pos);
        if ground_hit
            && let Some(level) = ground::surface(solid, actor.pos)
        {
            actor.pos.y = level;
        }

        if wall_hit {
            self.speed_f = 0.0;
        }
        if ground_hit {
            walk::add_calc(&mut self.speed_f, 0.0, GROUND_SCALE, GROUND_MAX, GROUND_MIN);
        }
    }

    /// `executeNormal` (`d_a_obj_hat.cpp:178`): the sole entry of `moveProc[]` (`:141-143`) has an
    /// empty body.
    fn execute_normal(&self) {}
}

pub fn is_rohat(name: &str) -> bool {
    ROHAT_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    mut hats: Query<(&mut Actor, &mut RoHat)>,
) {
    for (mut actor, mut hat) in &mut hats {
        let actor = &mut *actor;
        hat.execute(actor, collision.as_deref());
    }
}