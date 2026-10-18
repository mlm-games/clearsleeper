use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::stream;

pub const OTBLE_ACTORS: [&str; 2] = ["Otble", "OtbleL"];

pub const GRAVITY: f32 = -6.5; // <- d_a_obj_otble.cpp:104
pub const CULL_MIN: [f32; 3] = [-100.0, 0.0, -100.0]; // <- d_a_obj_otble.cpp:112
pub const CULL_MAX: [f32; 3] = [100.0, 150.0, 100.0]; // <- d_a_obj_otble.cpp:112
pub const CULL_MIN_LONG: [f32; 3] = [-100.0, 0.0, -200.0]; // <- d_a_obj_otble.cpp:110
pub const CULL_MAX_LONG: [f32; 3] = [100.0, 150.0, 200.0]; // <- d_a_obj_otble.cpp:110

const PARAM_MASK: u32 = 0xFF; // <- d_a_obj_otble.cpp:121
const VARIANT_MAX: u8 = 1; // <- d_a_obj_otble.cpp:122
const VARIANT_LONG: u8 = 1; // <- d_a_obj_otble.cpp:109
/// `fopAc_ac_c::speedF` (`f_op_actor.h:301`); `daObj_Otble::Act_c` never calls
/// `fopAcM_SetSpeedF` and process memory is zeroed by `sBs_ClearArea`.
const SPEED_F: f32 = 0.0;
/// `fopAc_ac_c::maxFallSpeed` (`f_op_actor.h:303`); never set, so the same zeroing applies.
const MAX_FALL: f32 = 0.0;

/// Generic brown wooden tables. `M_attr` is `{0, 0}` (`d_a_obj_otble.cpp:13`), so both
/// `attr()->m00` in `_execute` (`:35`) and `attr()->m01` in `_draw` (`:47`) are false and
/// neither early return fires. There is no `_arcname`: `_create` (`:126`) loads the resource
/// archive `"Okmono"` and picks `OTABLE`/`OTABLEL` plus `OTBLE`/`OTBLE_L` by `m294` (`:68-69`),
/// all renderer data.
///
/// `mObjAcch.CrrPos` in `set_mtx` (`:22`) needs `dBgS` poly groups and a wall-correction pass
/// this sim's merged-DZB `Collision` cannot answer, so `CreateInit`'s `mObjAcch.Set` /
/// `SetWall(30.0f, 30.0f)` (`:98-102`), its create-time `fopAcM_posMoveF` (`:105`) and the
/// per-tick ground clamp are all omitted; only the gravity half of `fopAcM_posMoveF` remains.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Otble {
    variant: u8,
    speed: Vec3,
}

impl Otble {
    /// `m294` is `fopAcM_GetParam(this) & 0xff` clamped to 1 (`:121-124`).
    pub fn spawn(parameters: u32) -> Self {
        let raw = (parameters & PARAM_MASK) as u8;
        Self {
            variant: if raw > VARIANT_MAX { VARIANT_MAX } else { raw },
            speed: Vec3::ZERO,
        }
    }

    /// `fopAcM_posMoveF(this, NULL)` (`f_op_actor_mng.cpp:481-485`) is `fopAcM_calcSpeed`
    /// (`:453-466`) followed by `fopAcM_posMove` (`:468-479`) with a NULL `move`, which
    /// `stream::calc_speed` already is.
    fn pos_move_f(&mut self, heading: i16) -> Vec3 {
        let (speed, _) = stream::calc_speed(SPEED_F, heading, self.speed.y, GRAVITY, MAX_FALL);
        self.speed = speed;
        speed
    }

    /// `CreateInit`'s cull box (`:109-113`); `m294 == 1` is the `OtbleL` long table.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        if self.variant == VARIANT_LONG {
            (CULL_MIN_LONG, CULL_MAX_LONG)
        } else {
            (CULL_MIN, CULL_MAX)
        }
    }
}

pub fn is_otble(name: &str) -> bool {
    OTBLE_ACTORS.contains(&name)
}

pub fn control(mut tables: Query<(&mut Actor, &mut Otble)>) {
    for (mut actor, mut table) in &mut tables {
        let step = table.pos_move_f(actor.angle[1]);
        actor.pos += step;
    }
}
