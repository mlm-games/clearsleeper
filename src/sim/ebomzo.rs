use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::bombflower::Blasts;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::stream;
use super::walk;

/// The stage string is `Ebomzo` (`d_stage.cpp:1118`, `fpcNm_Obj_Ebomzo_e`) and it is also the
/// `_arcname` (`d_a_obj_ebomzo.cpp:20`, `:53`, `:88`, `:90`, `:104`).
pub const EBOMZO_ACTORS: [&str; 1] = ["Ebomzo"];

/// `daObj::PrmAbstract(this, PRM_SWSAVE_W = 8, PRM_SWSAVE_S = 0)` (`d_a_obj_ebomzo.h:17`,
/// `d_a_obj.h:14`) - `((1 << 8) - 1) & (param >> 0)`.
const SWSAVE_MASK: u32 = 0xFF;
/// `mXRotRate = 0` (`d_a_obj_ebomzo.cpp:70`).
const SPIN_INIT: i16 = 0;
/// `sph_check_src` `cM3dGSphS.Radius` (`d_a_obj_ebomzo.cpp:47`).
const SPH_RADIUS: f32 = 10.0;
/// `offset.x = cM_ssin(current.angle.y) * 50.0f` (`d_a_obj_ebomzo.cpp:178`) and
/// `offset.z = cM_scos(current.angle.y) * 50.0f` (`:180`).
const OFFSET_RADIUS: f32 = 50.0;
/// `offset.y = 320.0f` (`d_a_obj_ebomzo.cpp:179`).
const OFFSET_Y: f32 = 320.0;
/// `if (diffVec.abs() < 100.0f)` (`d_a_obj_ebomzo.cpp:129`) - `cXyz::abs()` is the vector length
/// (`c_xyz.h:124`).
const HIT_RANGE: f32 = 100.0;
/// `mXRotRate = 8` (`d_a_obj_ebomzo.cpp:130`).
const SPIN_START: i16 = 8;
/// `current.angle.x = 0x4000` (`d_a_obj_ebomzo.cpp:75`) and the `demo` cap (`:155-157`).
const SPIN_CAP: i16 = 0x4000;
/// `mXRotRate += mXRotRate / 8` (`d_a_obj_ebomzo.cpp:152`).
const SPIN_DIV: i16 = 8;
/// `fopAcM_setCullSizeBox(this, -1000.0f, -100.0f, -1000.0f, ...)` (`d_a_obj_ebomzo.cpp:64`).
const CULL_MIN: [f32; 3] = [-1000.0, -100.0, -1000.0];
/// `..., 1000.0f, 1000.0f, 1000.0f)` (`d_a_obj_ebomzo.cpp:64`).
const CULL_MAX: [f32; 3] = [1000.0, 1000.0, 1000.0];
/// `JA_SE_OBJ_BOMB_DN_ST_S` (`JAZelAudio_SE.h:1950`).
const SE_BOMB_DN_ST_S: u32 = 0x69AC;
/// `JA_SE_OBJ_BOMB_DN_ST_E` (`JAZelAudio_SE.h:1951`).
const SE_BOMB_DN_ST_E: u32 = 0x69AD;

/// `typedef enum { Ebomzo_Mode_Check = 0, Ebomzo_Mode_Demo, Ebomzo_Mode_Fall }`
/// (`d_a_obj_ebomzo.cpp:13-17`), the `switch` at `:185-189`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// `Ebomzo_Mode_Check` (`d_a_obj_ebomzo.cpp:14`).
    Check,
    /// `Ebomzo_Mode_Demo` (`d_a_obj_ebomzo.cpp:15`).
    Demo,
    /// `Ebomzo_Mode_Fall` (`d_a_obj_ebomzo.cpp:16`).
    Fall,
}

/// The bombable bird statues outside the Dragon Roost Cavern entrance. `sph_check_src`
/// (`d_a_obj_ebomzo.cpp:22-49`) has `SrcObjAt Type = 0` (`:27`) and `Flags = 0` (`:26`), so the
/// statue deals no damage and has no attack collider; its `SrcObjTg Type = AT_TYPE_BOMB` (`:29`)
/// is a one-way trigger that makes the statue switch itself on, not a health path.
/// `daObjEbomzo::Act_c` (`d_a_obj_ebomzo.h:10-48`) has no health field and the profile `Group` is
/// `fopAc_ACTOR_e` (`d_a_obj_ebomzo.cpp:259`), so neither `combat::HitPoints` nor `enemy::Hurt` is
/// attached.
///
/// `Mthd_Execute` is `MoveBGExecute()` (`:223`), which forwards to the virtual `Execute(Mtx**)`
/// (`:175`) - so `Execute` is the real per-tick body and `shape_angle = current.angle` (`:191`),
/// `set_mtx()` (`:192`) and `*matrix = &M_tmp_mtx` (`:193`) are renderer and `dBgW` work.
/// `Mthd_Create` (`:85-94`) can only return `cPhs_ERROR_e` from `MoveBGCreate` on a heap or
/// `dComIfG_Bgsp()` registration failure (`d_bg_s_movebg_actor.cpp:60-66`), never from its
/// parameters, and `Create` (`:60-82`) ends on an unconditional TRUE, so there is no
/// spawn-then-despawn path. `MoveBGIsDelete` is the bare `TRUE` of
/// `dBgS_MoveBgActor::Delete()` (`d_bg_s_movebg_actor.cpp:77`, `d_a_obj_ebomzo.cpp:97-99`) - an
/// accepted delete request, not a self-destruct.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ebomzo {
    /// `prm_get_swSave()` (`d_a_obj_ebomzo.cpp:73`, `:136`), `param & 0xFF`.
    pub switch_no: u8,
    /// `mMode` (`d_a_obj_ebomzo.h:44`).
    pub mode: Mode,
    /// `mXRotRate` (`d_a_obj_ebomzo.h:45`), seeded to 0 by `Create` (`:70`).
    pub spin_rate: i16,
    /// `current.angle.x = 0x4000` (`d_a_obj_ebomzo.cpp:75`), a `Create`-time write to
    /// `current.angle.x`. Carried to the first tick because `spawn` cannot mutate `Actor`.
    pending_cap: bool,
}

impl Ebomzo {
    /// `daObjEbomzo::Act_c::Create` (`d_a_obj_ebomzo.cpp:60-82`), minus `fopAcM_SetMtx` (`:61`),
    /// `init_mtx` (`:62`), `fopAcM_setCullSizeBox` (`:64`), the `mStts`/`mCollider` registration
    /// (`:66-68`) and `mpParticleEmitter = NULL` (`:71`). The `fopAcM_isSwitch` branch (`:73-79`)
    /// sets `mMode`, `current.angle.x` and nothing else.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = (parameters & SWSAVE_MASK) as u8;
        let fallen = switches.test(switch_no);
        Self {
            switch_no,
            mode: if fallen { Mode::Fall } else { Mode::Check },
            spin_rate: SPIN_INIT,
            pending_cap: fallen,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -1000.0f, -100.0f, -1000.0f, 1000.0f, 1000.0f, 1000.0f)`
    /// (`d_a_obj_ebomzo.cpp:64`) - a real custom cull box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `mCollider.SetC(current.pos + offset)` (`d_a_obj_ebomzo.cpp:182`) with
    /// `offset = (cM_ssin(current.angle.y) * 50.0f, 320.0f, cM_scos(current.angle.y) * 50.0f)`
    /// (`:176-180`). Retail then registers the sphere with `dComIfG_Ccsp()->Set(&mCollider)`
    /// (`:183`); the centre is what `check` measures from.
    fn collider_centre(pos: Vec3, yaw: i16) -> Vec3 {
        pos + Vec3::new(
            stream::sin_s(yaw) * OFFSET_RADIUS,
            OFFSET_Y,
            stream::cos_s(yaw) * OFFSET_RADIUS,
        )
    }

    /// `mCollider.ChkTgHit()` against a `SrcObjTg Type = AT_TYPE_BOMB` sphere
    /// (`d_a_obj_ebomzo.cpp:29`, `:47`, `:125`). Blast origins come from `bombflower::Blasts`,
    /// the shared AT_BOMB channel, whose entry is the exploding bomb's `Actor.pos`
    /// (`bomb.rs:166`) - that is exactly the `acActor->current.pos` of `:128`.
    fn bomb_hit(&self, centre: Vec3, blasts: &Blasts) -> Option<Vec3> {
        blasts
            .list
            .iter()
            .copied()
            .find(|origin| walk::within(*origin, centre, SPH_RADIUS))
    }

    /// `check` (`d_a_obj_ebomzo.cpp:124-148`), minus the `mpParticleEmitter` block (`:138-144`).
    fn check(
        &mut self,
        centre: Vec3,
        angle: &mut [i16; 3],
        blasts: &Blasts,
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        let Some(blast) = self.bomb_hit(centre, blasts) else {
            return;
        };
        if (centre - blast).length() >= HIT_RANGE {
            return;
        }
        self.spin_rate = SPIN_START;
        angle[0] = angle[0].wrapping_add(SPIN_START);
        events.list.push(SimEvent::Se(SE_BOMB_DN_ST_S));
        self.mode = Mode::Demo;
        switches.set(self.switch_no);
    }

    /// `demo` (`d_a_obj_ebomzo.cpp:151-169`), minus the `StartShock` rumble (`:159`) and both
    /// particle calls (`:162-167`).
    fn demo(&mut self, angle: &mut [i16; 3], events: &mut SimEvents) {
        self.spin_rate = self.spin_rate.wrapping_add(self.spin_rate / SPIN_DIV);
        angle[0] = angle[0].wrapping_add(self.spin_rate);
        if angle[0] >= SPIN_CAP {
            angle[0] = SPIN_CAP;
            self.mode = Mode::Fall;
            events.list.push(SimEvent::Se(SE_BOMB_DN_ST_E));
        }
    }

    /// `Execute`'s `switch (this->mMode)` (`d_a_obj_ebomzo.cpp:185-189`). `fall` (`:172`) is an
    /// empty body in the decomp.
    fn step(
        &mut self,
        pos: Vec3,
        angle: &mut [i16; 3],
        blasts: &Blasts,
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        match self.mode {
            Mode::Check => {
                let centre = Self::collider_centre(pos, angle[1]);
                self.check(centre, angle, blasts, switches, events);
            }
            Mode::Demo => self.demo(angle, events),
            Mode::Fall => {}
        }
    }
}

pub fn is_ebomzo(name: &str) -> bool {
    EBOMZO_ACTORS.contains(&name)
}

pub fn control(
    blasts: Res<Blasts>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut statues: Query<(&mut Actor, &mut Ebomzo)>,
) {
    for (mut actor, mut statue) in &mut statues {
        if statue.pending_cap {
            statue.pending_cap = false;
            actor.angle[0] = SPIN_CAP;
        }
        let pos = actor.pos;
        statue.step(pos, &mut actor.angle, &blasts, &mut switches, &mut events);
    }
}