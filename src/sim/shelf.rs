//! The wooden shelf - `fpcNm_Obj_Shelf_e`, `d_a_obj_shelf.cpp`, stage strings `Otana` and
//! `OtanaG` (`d_stage.cpp:862-863`).
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE. `daObjShelf::Act_c` derives from
//! `dBgS_MoveBgActor` and its class body (`d_a_obj_shelf.h:8-54`) holds no `dCcD` member; the
//! profile `Group` is `fopAc_ACTOR_e` (`d_a_obj_shelf.cpp:314`). Neither `combat::HitPoints` nor
//! `enemy::Hurt` is attached. A toppled shelf blocks the player as a `dBgW`, and moving
//! backgrounds are one of the inputs RULE 3b lists as absent.
//!
//! Both mode transitions out of `mode_wait` and the one out of `mode_fell` are gated on
//! `dComIfGp_getDetect().chk_quake(&current.pos)` (`d_a_obj_shelf.cpp:113` and `:219`).
//! `dDetect_c::chk_quake` (`d_detect.cpp:59-79`) is TRUE when the global `mTimer` is positive,
//! when the player is mid front-roll-crash inside `chk_quake_area(pos)` (`d_detect.cpp:98-103`),
//! or when `mPlace[0].mTimer` is positive - and the only writer of those three in the whole
//! decomp is `dDetect_c::set_quake` (`d_detect.cpp:82-95`), called once, at
//! `d_a_npc_ji1.cpp:746`. No `dDetect_c` model exists here, and
//! `vibration::Vibration::check_quake()` is a different subsystem (it reads the rumble channel's
//! quake pattern), so `control` feeds the gate `false`. The mode machine is transcribed verbatim
//! so that arm lights up as soon as a detector lands; see NEEDS INSERT in the report.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::EventFlags;

/// `d_stage.cpp:862` (`Otana`) and `:863` (`OtanaG`) both bind to `fpcNm_Obj_Shelf_e`, the
/// profile of `d_a_obj_shelf.cpp:303-318`. The archive is `M_arcname = "Otana"`
/// (`d_a_obj_shelf.cpp:55`) for both.
pub const SHELF_ACTORS: [&str; 2] = ["Otana", "OtanaG"];

/// `daObj::PrmAbstract<Prm_e>(this, PRM_GROUNDMA_W = 0x01, PRM_GROUNDMA_S = 0x00)`
/// (`d_a_obj_shelf.h:13-17`) is `((1 << 1) - 1) & (param >> 0)`, i.e. `param & 1`.
const GROUNDMA_MASK: u32 = (1 << 1) - 1; // <- d_a_obj.h:14 via d_a_obj_shelf.h:13
const GROUNDMA_SHIFT: u32 = 0x00; // <- d_a_obj_shelf.h:14 (PRM_GROUNDMA_S)

/// `daObjShelf::L_attr` (`d_a_obj_shelf.cpp:34-49`).
const VIB_DURATION: i16 = 4; // <- d_a_obj_shelf.cpp:35 (mVibDuration)
const VIB_Y_SPEED: i16 = 30000; // <- d_a_obj_shelf.cpp:36 (mVibYSpeed)
const VIB_Y_MAGNITUDE: f32 = 4.0; // <- d_a_obj_shelf.cpp:37 (mVibYMagnitude)
const VIB_X_SPEED: i16 = 30500; // <- d_a_obj_shelf.cpp:38 (mVibXSpeed)
const VIB_X_MAGNITUDE: f32 = 200.0; // <- d_a_obj_shelf.cpp:40 (mVibXMagnitude)
const ROT_ACCEL: f32 = 600.0; // <- d_a_obj_shelf.cpp:42 (mRotAccel)
const ROT_DECAY: f32 = 0.02; // <- d_a_obj_shelf.cpp:43 (mRotDecay)
const BOUNCE_FACTOR: f32 = 0.5; // <- d_a_obj_shelf.cpp:44 (mBounceFactor)
const BOUNCE_NUM: i16 = 5; // <- d_a_obj_shelf.cpp:45 (mBounceNum)
const ROT_WAIT: i16 = 0; // <- d_a_obj_shelf.cpp:46 (mRotWaitDuration)
const INIT_SPEED_2: f32 = -3500.0; // <- d_a_obj_shelf.cpp:47 (mInitSpeed2)
const INIT_SPEED_3: f32 = -2500.0; // <- d_a_obj_shelf.cpp:48 (mInitSpeed3)

/// `mTargetAngle = 0x4000` in `mode_rot_init` / `mode_rot_init2` (`d_a_obj_shelf.cpp:159`,
/// `:169`) - a quarter turn.
const TARGET_FALL: i16 = 0x4000; // <- d_a_obj_shelf.cpp:159, :169
/// `mTargetAngle = 0` in `mode_rot_init3` (`d_a_obj_shelf.cpp:179`).
const TARGET_UPRIGHT: i16 = 0; // <- d_a_obj_shelf.cpp:179

const SE_RACK_BREAK: u32 = 0x696D; // <- d_a_obj_shelf.cpp:206 (JA_SE_OBJ_RACK_BREAK)

/// `fopAcM_CULLBOX_CUSTOM_e` (`d_a_obj_shelf.cpp:317`) is filled by the explicit
/// `fopAcM_setCullSizeBox(this, -110.0f, -55.0f, -1.0f, 110.0f, 20.0f, 110.0f)` at `:69`.
const CULL_MIN: [f32; 3] = [-110.0, -55.0, -1.0]; // <- d_a_obj_shelf.cpp:69
const CULL_MAX: [f32; 3] = [110.0, 20.0, 110.0]; // <- d_a_obj_shelf.cpp:69

/// `mode_proc[]` (`d_a_obj_shelf.cpp:241-246`) indexes `mMode` at `:247`.
const MODE_WAIT: u8 = 0; // <- d_a_obj_shelf.cpp:242
const MODE_VIB: u8 = 1; // <- d_a_obj_shelf.cpp:243
const MODE_ROT: u8 = 2; // <- d_a_obj_shelf.cpp:244
const MODE_FELL: u8 = 3; // <- d_a_obj_shelf.cpp:245

/// `!dComIfGs_isEventBit(dSv_event_flag_c::UNK_0001)` (`d_a_obj_shelf.cpp:115`). Retail indexes
/// `dSv_event_c::mFlags[0x100]` BYTE (`d_save.h:722`, `d_save.cpp:1187-1204`), so per RULE 11c
/// the enum value goes straight into `EventFlags`.
const EVENT_QUAKE_USED: u32 = 0x0001; // <- d_a_obj_shelf.cpp:115

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Shelf {
    /// `mMode` (`d_a_obj_shelf.h:45`), armed to 0 by `mode_wait_init` (`:108`).
    mode: u8,
    /// `mRotSpeed` (`d_a_obj_shelf.h:46`).
    rot_speed: f32,
    /// `mTargetAngle` (`d_a_obj_shelf.h:47`).
    target: i16,
    /// `mTimer` (`d_a_obj_shelf.h:48`).
    timer: i16,
    /// `mVibY` (`d_a_obj_shelf.h:49`), the vertical-shake phase.
    vib_y: i16,
    /// `mVibX` (`d_a_obj_shelf.h:50`), the roll-shake phase. `mVibZ` (`d_a_obj_shelf.h:51`) is
    /// omitted: it only reaches `shape_angle.z` (`:150`), a renderer-only field the sim does not
    /// carry.
    vib_x: i16,
    /// `mCurBounce` (`d_a_obj_shelf.h:52`).
    bounce: i16,
    /// `m2e7` (`d_a_obj_shelf.h:53`), the "settles back upright" flag `mode_rot_init3` sets.
    upright: bool,
    /// `shape_angle.x`. `Actor` carries only `current.angle`, and the shelf's tilt IS
    /// `shape_angle.x` (`d_a_obj_shelf.cpp:141`, `:149`, `:192`), so it lives here.
    /// `fopAcM_ct` (`:76`) zeroes it, so it does NOT start at `home.angle.x`.
    tilt: i16,
    /// `home.pos.y` and `home.angle.x` (`d_a_obj_shelf.cpp:140-141`). `home.angle.z` only feeds
    /// the dropped `shape_angle.z`, so it is not stored.
    home_y: f32,
    home_x: i16,
    /// `prm_get_groundma()` (`d_a_obj_shelf.h:17`), one of the three terms of the `mode_wait`
    /// gate (`:115`).
    groundma: bool,
}

impl Shelf {
    /// `Mthd_Create` (`:75-84`) and `Create` (`:66-72`). `Create` stores nothing but the cull box
    /// and `mode_wait_init` (`:107-109`), which is the `mMode = 0` here. `fopAcM_ct` (`:76`)
    /// zeroes the rest, which is where every other field starts.
    pub fn spawn(parameters: u32, pos: Vec3, angle: [i16; 3]) -> Self {
        Self {
            mode: MODE_WAIT,
            rot_speed: 0.0,
            target: TARGET_UPRIGHT,
            timer: 0,
            vib_y: 0,
            vib_x: 0,
            bounce: 0,
            upright: false,
            tilt: 0,
            home_y: pos.y,
            home_x: angle[0],
            groundma: ((parameters >> GROUNDMA_SHIFT) & GROUNDMA_MASK) != 0,
        }
    }

    /// `fopAcM_setCullSizeBox` at `d_a_obj_shelf.cpp:69`.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `mode_wait_init` (`:107-109`).
    fn mode_wait_init(&mut self) {
        self.mode = MODE_WAIT;
    }

    /// `mode_vib_init` (`:129-135`).
    fn mode_vib_init(&mut self) {
        self.timer = VIB_DURATION;
        self.vib_y = 0;
        self.vib_x = 0;
        self.mode = MODE_VIB;
    }

    /// `mode_fell_init` (`:213-215`).
    fn mode_fell_init(&mut self) {
        self.mode = MODE_FELL;
    }

    /// `mode_rot_init` (`:155-162`): the shake decays and the shelf settles onto 0x4000.
    fn mode_rot_init(&mut self) {
        self.bounce = BOUNCE_NUM + 1;
        self.rot_speed = 0.0;
        self.timer = ROT_WAIT;
        self.target = TARGET_FALL;
        self.upright = false;
        self.mode = MODE_ROT;
    }

    /// `mode_rot_init2` (`:165-172`): a re-quake while already fallen starts at -3500 and still
    /// settles onto 0x4000.
    fn mode_rot_init2(&mut self) {
        self.bounce = BOUNCE_NUM + 1;
        self.rot_speed = INIT_SPEED_2;
        self.timer = 0;
        self.target = TARGET_FALL;
        self.upright = false;
        self.mode = MODE_ROT;
    }

    /// `mode_rot_init3` (`:175-182`): the first shake starts at -2500 and settles back at 0.
    fn mode_rot_init3(&mut self) {
        self.bounce = BOUNCE_NUM + 1;
        self.rot_speed = INIT_SPEED_3;
        self.timer = 0;
        self.target = TARGET_UPRIGHT;
        self.upright = true;
        self.mode = MODE_ROT;
    }

    /// `mode_wait` (`:112-126`). `hold_event()` (`:99-104`) is
    /// `fopAcM_SearchByName(fpcNm_NPC_BA1_e)` followed by `daNpc_Ba1_c::holdEventOn()`,
    /// which is `sim::ba1::Ba1::hold_event_on`; the call sits inside the
    /// `quake && groundma && !UNK_0001` arm, and `control` feeds `quake` false because
    /// no `dDetect_c` exists (see the module header), so it is unreachable here and the
    /// fetch is left to whoever lands the detector. The `quake = false; event = true;`
    /// that follows (`:117-118`) is kept.
    fn mode_wait(&mut self, quake: bool, quake_used: bool) {
        let mut quake = quake;
        let mut event = false;
        if quake && self.groundma && !quake_used {
            quake = false;
            event = true;
        }
        if quake {
            self.mode_vib_init();
        } else if event {
            self.mode_rot_init3();
        }
    }

    /// `mode_vib` (`:138-152`). `mVibY` is an `s16` and wraps at 30000 per tick, so the phase is
    /// `wrapping_add`; `(short)(cM_ssin(mVibX) * 200.0f)` is a truncating `f32` -> `i16` cast.
    fn mode_vib(&mut self, actor: &mut Actor) {
        self.timer -= 1;
        if self.timer <= 0 {
            actor.pos.y = self.home_y;
            self.tilt = self.home_x;
            self.mode_rot_init();
            return;
        }
        self.vib_y = self.vib_y.wrapping_add(VIB_Y_SPEED);
        actor.pos.y = self.home_y + super::stream::sin_s(self.vib_y) * VIB_Y_MAGNITUDE;
        self.vib_x = self.vib_x.wrapping_add(VIB_X_SPEED);
        self.tilt = self
            .home_x
            .wrapping_add((super::stream::sin_s(self.vib_x) * VIB_X_MAGNITUDE) as i32 as i16);
    }

    /// `mode_rot` (`:185-210`). `shape_angle.x += (short)(int)mRotSpeed` truncates the float into
    /// an `s16` every tick, and both the `shape_angle.x - mTargetAngle` test and the bounce
    /// expression run through `s16` arithmetic, so all of it is `wrapping_*` here.
    fn mode_rot(&mut self) -> Option<u32> {
        if self.timer > 0 {
            self.timer -= 1;
            return None;
        }
        self.rot_speed += ROT_ACCEL;
        self.rot_speed -= self.rot_speed * ROT_DECAY;
        self.tilt = self.tilt.wrapping_add(self.rot_speed as i32 as i16);
        if self.tilt <= self.target {
            return None;
        }
        self.bounce -= 1;
        if self.bounce <= 0 {
            self.tilt = self.target;
            if self.upright {
                self.mode_wait_init();
            } else {
                self.mode_fell_init();
            }
            return None;
        }
        let overshoot = (self.tilt.wrapping_sub(self.target) as f32 * BOUNCE_FACTOR) as i32 as i16;
        self.tilt = self.target.wrapping_sub(overshoot);
        self.rot_speed *= -BOUNCE_FACTOR;
        if self.bounce == BOUNCE_NUM {
            return Some(SE_RACK_BREAK);
        }
        None
    }

    /// `mode_fell` (`:218-222`).
    fn mode_fell(&mut self, quake: bool) {
        if quake {
            self.mode_rot_init2();
        }
    }

    /// `Execute` (`:239-253`): `(this->*mode_proc[mMode])()`, then `set_mtx()`.
    fn execute(&mut self, actor: &mut Actor, quake: bool, quake_used: bool) -> Option<u32> {
        match self.mode {
            MODE_WAIT => {
                self.mode_wait(quake, quake_used);
                None
            }
            MODE_VIB => {
                self.mode_vib(actor);
                None
            }
            MODE_ROT => self.mode_rot(),
            _ => {
                self.mode_fell(quake);
                None
            }
        }
    }
}

pub fn is_shelf(name: &str) -> bool {
    SHELF_ACTORS.contains(&name)
}

pub fn control(
    flags: Res<EventFlags>,
    mut shelves: Query<(&mut Actor, &mut Shelf)>,
    mut events: ResMut<SimEvents>,
) {
    let quake_used = flags.test(EVENT_QUAKE_USED);
    for (mut actor, mut shelf) in &mut shelves {
        let actor = &mut *actor;
        let shelf = &mut *shelf;
        if let Some(se) = shelf.execute(actor, false, quake_used) {
            events.list.push(SimEvent::Se(se));
        }
    }
}
