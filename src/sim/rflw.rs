use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::bombflower::Blasts;
use super::events::{SimEvent, SimEvents};
use super::stream;

pub const RFLW_ACTORS: [&str; 1] = ["Rflw"];

const SE_TREE_SWING: u32 = 0x69E6; // <- JAZelAudio_SE.h:1998 (JA_SE_OBJ_TREE_SWING_S)
const CYL_RADIUS: f32 = 30.0; // <- d_a_obj_rflw.cpp:38
const CYL_HEIGHT: f32 = 200.0; // <- d_a_obj_rflw.cpp:39
const CULL_MIN: [f32; 3] = [-600.0, 0.0, -600.0]; // <- d_a_obj_rflw.cpp:76
const CULL_MAX: [f32; 3] = [600.0, 900.0, 600.0]; // <- d_a_obj_rflw.cpp:76
const SWING_GATE: i16 = 0x38; // <- d_a_obj_rflw.cpp:145 (m40C > 0x38)
const SWING_AIM: i16 = -0x4000; // <- d_a_obj_rflw.cpp:149 (dir_angle + -0x4000)
const FAST_TICKS: i16 = 8; // <- d_a_obj_rflw.cpp:158 (temp4 < 8)
const FAST_GAIN: f32 = 4.0; // <- d_a_obj_rflw.cpp:159
const FAST_STEP: i16 = 0x1000; // <- d_a_obj_rflw.cpp:161
const SLOW_GAIN: f32 = 2.0; // <- d_a_obj_rflw.cpp:163
const SLOW_STEP: i16 = 0x800; // <- d_a_obj_rflw.cpp:165
const SWING_SPAN: i16 = 0x100; // <- d_a_obj_rflw.cpp:167 (m40C > 0x100)
const SWING_ORIGIN: i32 = 0x100; // <- d_a_obj_rflw.cpp:159, :163 (0x100 - temp4)

/// The pond the reflection swells across - a bomb-blastable trigger whose only job is to
/// rotate the `joint2` pan once and then settle. The stage string is `Rflw` (`d_stage.cpp:989`)
/// and the archive is also `"Rflw"` (`d_a_obj_rflw.cpp:52`), so `_arcname` and the STAGE string
/// agree.
///
/// `l_cyl_src` (`:13-41`) registers a 30/200 cylinder at the origin with
/// `SrcObjTg Type = AT_TYPE_BOMB` (`:20`, `d_cc_d.h:18`) and
/// `SrcObjTg SPrm = cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsOther_e` (`:21`): the pool is a *target*
/// for bomb blasts, so `super::bombflower::Blasts` is this sim's AT_BOMB source. `SrcObjAt Type`
/// is 0 (`:18`) - the pool has no attack arm of its own and takes no damage.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rflw {
    /// `m408` (`d_a_obj_rflw.h:23`), the Y rotation `nodeCallBack` writes into the joint.
    pub bend: i16,
    /// `m40A` (`d_a_obj_rflw.h:24`), the phase fed to `cM_ssin`.
    pub sweep: i16,
    /// `m40C` (`d_a_obj_rflw.h:25`), the tick counter.
    pub count: i16,
    /// `m40E` (`d_a_obj_rflw.h:26`), the angle the pool turned towards the hitter.
    pub aim: i16,
    /// `m410` (`d_a_obj_rflw.h:27`).
    pub swinging: bool,
}

impl Rflw {
    /// `CreateInit` (`:74-83`) writes `m408 = 0` (`:81`) and `fopAcM_ct` (`:114`) zeroes the
    /// rest; `_create` (`:124`) then clears `m410`.
    pub fn spawn() -> Self {
        Self {
            bend: 0,
            sweep: 0,
            count: 0,
            aim: 0,
            swinging: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `mCyl.SetC(current.pos)` + `dComIfG_Ccsp()->Set(&mCyl)` + `ChkCoHit()`/`GetCoHitAc()`
    /// (`:143-146`). `cM3d_Cross_CylCyl` (`c_m3d.cpp:1571-1583`) tests
    /// `dx^2 + dz^2 <= (r1 + r2)^2` and then `c.y + h >= other.y && c.y <= other.y + h`, so
    /// the tube stands on `current.pos` and reaches `CYL_HEIGHT` upward, not around it.
    fn co_hit(&self, actor: &Actor, blasts: &Blasts) -> Option<Vec3> {
        blasts.list.iter().copied().find(|origin| {
            let dx = origin.x - actor.pos.x;
            let dz = origin.z - actor.pos.z;
            dx * dx + dz * dz <= CYL_RADIUS * CYL_RADIUS
                && origin.y >= actor.pos.y
                && origin.y <= actor.pos.y + CYL_HEIGHT
        })
    }

    /// `:148-153`: aim the joint at whatever hit, reset the counters and swing. `m40E =
    /// dir_angle + -0x4000` promotes to `int` before the `s16` store, so the add wraps.
    fn face_hitter(&mut self, actor: &Actor, origin: Vec3, events: &mut SimEvents) {
        let to_object = actor.pos - origin;
        let dir_angle = stream::angle_s(to_object.x.atan2(to_object.z));
        self.aim = dir_angle.wrapping_add(SWING_AIM);
        self.swinging = true;
        self.count = 0;
        self.sweep = 0;
        events.list.push(SimEvent::Se(SE_TREE_SWING));
    }

    /// `:155-172`: `m40C++`, then the fast arm below `FAST_TICKS` (gain `FAST_GAIN`, phase
    /// `FAST_STEP`) or the slow arm, released once the counter passes `SWING_SPAN`. The
    /// `(s32)((float)(0x100 - temp4) * cM_ssin(m40A) * gain)` products are `int` arithmetic on
    /// an `s16` sine, so both the phase add and the store truncate.
    fn sweep_tick(&mut self) {
        self.count = self.count.wrapping_add(1);
        let count = self.count;
        let (gain, step) = if count < FAST_TICKS {
            (FAST_GAIN, FAST_STEP)
        } else {
            (SLOW_GAIN, SLOW_STEP)
        };
        let span = SWING_ORIGIN - i32::from(count);
        self.bend = (span as f32 * stream::sin_s(self.sweep) * gain) as i32 as i16;
        self.sweep = self.sweep.wrapping_add(step);
        if count > SWING_SPAN {
            self.swinging = false;
            self.count = 0;
            self.bend = 0;
        }
    }

    /// `_execute` (`:142-175`) minus `set_mtx` (`:105-110`), which is model scale and position,
    /// and minus `nodeCallBack` (`:86-102`), which is the joint-matrix write the renderer reads.
    fn step(&mut self, actor: &Actor, blasts: &Blasts, events: &mut SimEvents) {
        if self.count > SWING_GATE || !self.swinging {
            if let Some(origin) = self.co_hit(actor, blasts) {
                self.face_hitter(actor, origin, events);
            }
        }
        if self.swinging {
            self.sweep_tick();
        }
    }
}

pub fn is_rflw(name: &str) -> bool {
    RFLW_ACTORS.contains(&name)
}

pub fn control(
    blasts: Res<Blasts>,
    mut events: ResMut<SimEvents>,
    mut pools: Query<(&Actor, &mut Rflw)>,
) {
    for (actor, mut pool) in &mut pools {
        pool.step(actor, &blasts, &mut events);
    }
}