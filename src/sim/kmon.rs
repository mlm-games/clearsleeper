use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::stream;

pub const KMON_ACTORS: [&str; 1] = ["Kmon"];

pub const CULL_MIN: [f32; 3] = [-100.0, 0.0, -100.0]; // <- f_op_actor_mng.cpp:562 (fopAc_CULLBOX_4_e)
pub const CULL_MAX: [f32; 3] = [100.0, 800.0, 100.0]; // <- f_op_actor_mng.cpp:562 (fopAc_CULLBOX_4_e)

pub const GRAVITY: f32 = -4.0; // <- d_a_kmon.cpp:61
/// `fopAc_ac_c::speedF` (`f_op_actor.h:301`); `daKmon_c` never calls `fopAcM_SetSpeedF`, and
/// process memory is zeroed by `sBs_ClearArea` (`f_pc_base.cpp:98`, `s_basic.cpp:19-21`).
const SPEED_F: f32 = 0.0;
/// `fopAc_ac_c::maxFallSpeed` (`f_op_actor.h:303`); `daKmon_c` never calls
/// `fopAcM_SetMaxFallSpeed`, so the same zeroing applies.
const MAX_FALL: f32 = 0.0;

/// The Bell. The stage string is `Kmon` (`d_stage.cpp:1136`); the model's `_arcname` is
/// `Always` (`d_a_kmon.cpp:10`). `CreateInit` (`:53-68`) arms a `dBgS_ObjAcch` pendulum with
/// `gravity = -4.0f` and a 30-unit wall circle, but the only per-tick work `_execute`
/// (`:124-132`) does is `checkTalk`, `fopAcM_posMoveF` plus `mAcch.CrrPos`, the BCK/BTK plays
/// and `set_mtx`.
///
/// `mAcch.CrrPos` (`d_bg_s_acch.cpp:209-245`) needs `dBgS` poly groups and a wall-correction
/// pass this sim's merged-DZB `Collision` cannot answer, and `checkTalk`'s two outputs are both
/// absent: `eyePos`/`attention_info` (`:76-77`) is per-actor attention state, and the
/// `eventInfo.setEventName("Ji1_KmonTalk")` (`:65`) / `onCondition(dEvtCnd_CANTALK_e)` (`:91`)
/// pair needs the stage event-info table (`dComIfGp_evmng_getEventIdx`,
/// `d_com_inf_game.h:3611`), which does not exist here.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Kmon {
    pub speed: Vec3,
}

impl Kmon {
    pub fn spawn() -> Self {
        Self { speed: Vec3::ZERO }
    }

    /// `fopAcM_posMoveF(this, NULL)` (`f_op_actor_mng.cpp:481-485`) is `fopAcM_calcSpeed`
    /// (`:453-466`) followed by `fopAcM_posMove` (`:468-479`) with a NULL `move`, which
    /// `stream::calc_speed` already is.
    fn pos_move_f(&mut self, heading: i16) -> Vec3 {
        let (speed, _) = stream::calc_speed(SPEED_F, heading, self.speed.y, GRAVITY, MAX_FALL);
        self.speed = speed;
        speed
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_kmon(name: &str) -> bool {
    KMON_ACTORS.contains(&name)
}

pub fn control(mut bells: Query<(&mut Actor, &mut Kmon)>) {
    for (mut actor, mut bell) in &mut bells {
        let step = bell.pos_move_f(actor.angle[1]);
        actor.pos += step;
    }
}