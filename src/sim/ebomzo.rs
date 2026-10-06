use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::bombflower::Blasts;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::walk;

pub const EBOMZO_ACTORS: [&str; 1] = ["Ebomzo"];

const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_ebomzo.h:13 (PRM_SWSAVE_W = 8, PRM_SWSAVE_S = 0)
const XROT_INIT: i16 = 0; // <- d_a_obj_ebomzo.cpp:70 (mXRotRate = 0)
const XROT_RATE: i16 = 8; // <- d_a_obj_ebomzo.cpp:130 (mXRotRate = 8)
const XROT_DIV: i16 = 8; // <- d_a_obj_ebomzo.cpp:152 (mXRotRate / 8)
const SPH_RADIUS: f32 = 10.0; // <- d_a_obj_ebomzo.cpp:47 (dCcD_SrcSph Radius)
const OFFSET_YAW: f32 = 50.0; // <- d_a_obj_ebomzo.cpp:178, :180 (cM_ssin/cos(angle.y) * 50.0f)
const OFFSET_HEIGHT: f32 = 320.0; // <- d_a_obj_ebomzo.cpp:179 (offset.y = 320.0f)
const DIFF_LIMIT: f32 = 100.0; // <- d_a_obj_ebomzo.cpp:129 (diffVec.abs() < 100.0f)
const SE_OBJ_BOMB_DN_ST_S: u32 = 0x69AC; // <- JAZelAudio_SE.h:1950
const SE_OBJ_BOMB_DN_ST_E: u32 = 0x69AD; // <- JAZelAudio_SE.h:1951

pub const FALL_ANGLE: i16 = 0x4000; // <- d_a_obj_ebomzo.cpp:75, :155, :156
pub const CULL_MIN: [f32; 3] = [-1000.0, -100.0, -1000.0]; // <- d_a_obj_ebomzo.cpp:64
pub const CULL_MAX: [f32; 3] = [1000.0, 1000.0, 1000.0]; // <- d_a_obj_ebomzo.cpp:64

/// `Ebomzo_Mode_e` (`d_a_obj_ebomzo.cpp:13-17`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Check,
    Demo,
    Fall,
}

/// The bombable bird statues outside the Dragon Roost Cavern entrance. The stage string is
/// `Ebomzo` (`d_stage.cpp:1118`) and the archive is `Ebomzo` (`d_a_obj_ebomzo.cpp:20`).
///
/// `sph_check_src` (`:22-49`) is a zero-`Center` `dCcD_SrcSph` of radius 10 whose Tg side is
/// `AT_TYPE_BOMB` (`:29`, `d_cc_d.h:15` = `super::barricade::AT_BOMB`) and whose Tg `SPrm` is
/// `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsOther_e` (`:30`), so `check` (`:124-148`) is a genuine
/// inbound bomb path - the only one of the seven actors with a bomb-driven mode change.
/// `mStts.Init(0xff, 0xff, this)` (`:66`) accepts every AT and Tg type, and the profile carries
/// `fopAcStts_NOCULLEXEC_e` (`:258`), so retail `_execute` runs even while culled.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ebomzo {
    /// `prm_get_swSave` (`d_a_obj_ebomzo.h:17`), the switch `Create` tests (`:73`) and `check`
    /// sets (`:136`).
    pub switch_no: u8,
    pub mode: Mode,
    /// `mXRotRate` (`d_a_obj_ebomzo.h:45`).
    pub x_rot_rate: i16,
}

impl Ebomzo {
    /// `Create` (`d_a_obj_ebomzo.cpp:60-82`), minus `fopAcM_SetMtx`, `init_mtx`,
    /// `fopAcM_setCullSizeBox`, `mStts.Init`, `mCollider.Set`/`SetStts` and the
    /// `mpParticleEmitter = NULL` store.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = (parameters & SWSAVE_MASK) as u8;
        let mode = if switches.test(switch_no) {
            Mode::Fall
        } else {
            Mode::Check
        };
        Self {
            switch_no,
            mode,
            x_rot_rate: XROT_INIT,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `Execute` (`d_a_obj_ebomzo.cpp:175-196`), minus `shape_angle = current.angle` (`:191`) and
    /// `set_mtx` (`:192`) - both renderer-only - and minus the `dComIfG_Ccsp()->Set(&mCollider)`
    /// registration of `:183`, whose only remaining job is to publish the sphere `check` reads.
    fn execute(
        &mut self,
        actor: &mut Actor,
        blasts: &Blasts,
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        match self.mode {
            Mode::Check => self.check(actor, blasts, switches, events),
            Mode::Demo => self.demo(actor, events),
            Mode::Fall => self.fall(),
        }
    }

    /// The `offset` of `Execute` (`:176-180`) added to `current.pos` by `mCollider.SetC` (`:182`).
    /// `SetC` overwrites the source sphere's zero `Center` (`:46`), so this is the world-space
    /// collider centre. `cM_ssin`/`cM_scos` are the 16-bit angle tables
    /// (`JMATrigonometric.h:10-15`), i.e. `walk::from_s16`.
    fn centre(&self, actor: &Actor) -> Vec3 {
        let (sin, cos) = walk::from_s16(actor.angle[1]).sin_cos();
        actor.pos + Vec3::new(sin * OFFSET_YAW, OFFSET_HEIGHT, cos * OFFSET_YAW)
    }

    /// `check` (`d_a_obj_ebomzo.cpp:124-148`). `mCollider.ChkTgHit()` (`:125`) is the
    /// `AT_TYPE_BOMB` AT-vs-Tg overlap; the sim's bomb channel publishes the exploding bomb's
    /// `Actor.pos` - its AT sphere centre - into `Blasts` (`bomb.rs:161`). The `acActor` non-NULL
    /// guard of `:127` is folded into that test, since `GetTgHitAc()` is non-NULL exactly when the
    /// overlap occurred, and `diffVec.abs()` is `sqrtf(abs2())` (`c_xyz.h:124`), i.e. Euclidean.
    fn check(
        &mut self,
        actor: &mut Actor,
        blasts: &Blasts,
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        let centre = self.centre(actor);
        if !blasts.list.iter().any(|origin| {
            walk::within(*origin, centre, SPH_RADIUS) && (*origin - centre).length() < DIFF_LIMIT
        }) {
            return;
        }
        self.x_rot_rate = XROT_RATE;
        let turn = self.x_rot_rate;
        actor.angle[0] = actor.angle[0].wrapping_add(turn);
        events.list.push(SimEvent::Se(SE_OBJ_BOMB_DN_ST_S));
        self.mode = Mode::Demo;
        switches.set(self.switch_no);
    }

    /// `demo` (`d_a_obj_ebomzo.cpp:151-169`), minus the `StartShock` rumble (`:159`) and the
    /// emitter teardown and splash particles (`:162-167`).
    fn demo(&mut self, actor: &mut Actor, events: &mut SimEvents) {
        self.x_rot_rate = self
            .x_rot_rate
            .wrapping_add(self.x_rot_rate / XROT_DIV);
        let turn = self.x_rot_rate;
        actor.angle[0] = actor.angle[0].wrapping_add(turn);
        if actor.angle[0] >= FALL_ANGLE {
            actor.angle[0] = FALL_ANGLE;
            self.mode = Mode::Fall;
            events.list.push(SimEvent::Se(SE_OBJ_BOMB_DN_ST_E));
        }
    }

    /// `fall` (`d_a_obj_ebomzo.cpp:172`): a toppled statue stays toppled for the rest of the stage.
    fn fall(&self) {}
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
    for (actor, mut statue) in &mut statues {
        statue.execute(actor, &blasts, &mut switches, &mut events);
    }
}