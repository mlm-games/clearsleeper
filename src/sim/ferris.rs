//! `daObjFerris::Act_c` (`d_a_obj_ferris.h:9-72`) - the Windfall Island Ferris wheel behind the
//! stage string `Skanran`. `Nonmatching` count: 0, so every token below is transcribed. The file
//! header calls it "Object - Windfall Island - Ferris wheel" (`d_a_obj_ferris.cpp:3`).
//!
//! Nothing else in `src/sim/` claims `Skanran`: `OBJNAME("Skanran", fpcNm_Obj_Ferris_e, 255, 0)`
//! (`d_stage.cpp:1110`) is the only row, and it agrees with `M_arcname[8] = "Skanran"`
//! (`d_a_obj_ferris.cpp:81`).

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, Stage};
use super::flags::{EventReg, Switches};

/// `OBJNAME("Skanran", fpcNm_Obj_Ferris_e, 255, 0)` (`d_stage.cpp:1110`). The archive name agrees:
/// `const char daObjFerris::Act_c::M_arcname[8] = "Skanran";` (`d_a_obj_ferris.cpp:81`).
pub const FERRIS_ACTORS: [&str; 1] = ["Skanran"];

/// ```
/// static const Attr_c L_attr = { 45, 0x01, 50.0f, 195.0f, { 0.0f, -100.0f, -135.0f }, 1820.0f,
///                                0.0f, 30.0f };
/// ```
/// (`d_a_obj_ferris.cpp:17`) against `struct Attr_c` (`d_a_obj_ferris.h:11-19`). Read in this order:
/// * `field_0x00 = 45` - the running wheel's terminal speed (`:421-425`, `:185`).
/// * `field_0x02 = 0x01` - UNREAD: `field_0x02` appears nowhere in the translation unit.
/// * `mCylR = 50.0f` - the gondola cylinder radius, re-set at `:439`.
/// * `mCylH = 195.0f` - the gondola cylinder height, re-set at `:440`.
/// * `mOffset = { 0.0f, -100.0f, -135.0f }` - the first cylinder's local centre, re-set at `:441`.
/// * `field_0x18 = 1820.0f` - the height gate: `if (pt.y <= 1820.0f)` at `:438` and `:477`.
/// * `field_0x1c = 0.0f` - UNREAD.
/// * `m20 = 30.0f` - the ride sphere radius, re-set at `:479`.
const SPEED_FULL: i16 = 45; // <- d_a_obj_ferris.cpp:17 (Attr_c::field_0x00)
const CYL_R: f32 = 50.0; // <- d_a_obj_ferris.cpp:17 (Attr_c::mCylR)
const CYL_H: f32 = 195.0; // <- d_a_obj_ferris.cpp:17 (Attr_c::mCylH)
const CYL_H_2: f32 = 148.0; // <- d_a_obj_ferris.cpp:447
const CYL_OFFSET_1: [f32; 3] = [0.0, -100.0, -135.0]; // <- d_a_obj_ferris.cpp:17 (Attr_c::mOffset)
const CYL_OFFSET_2: [f32; 3] = [0.0, -56.0, 127.0]; // <- d_a_obj_ferris.cpp:448
const SPH_R: f32 = 30.0; // <- d_a_obj_ferris.cpp:17 (Attr_c::m20)
const RIDE_GATE_Y: f32 = 1820.0; // <- d_a_obj_ferris.cpp:17 (Attr_c::field_0x18)
/// `static cXyz offset[6] = { ... }` (`d_a_obj_ferris.cpp:250-257`) - the five gondola anchors in
/// wheel-local space plus the base at the origin. Index 5 is the wheel BASE, whose `set_mtx` arm
/// (`:268-276`) translates by `offset[5] == (0,0,0)` and does NOT undo the roll.
const GONDOLA_OFFSETS: [[f32; 3]; 6] = [
    [0.56, 1078.13, 162.68],    // <- d_a_obj_ferris.cpp:251
    [1026.17, 332.98, 162.68],  // <- d_a_obj_ferris.cpp:252
    [634.42, -872.69, 162.68],  // <- d_a_obj_ferris.cpp:253
    [-633.3, -872.69, 162.68],  // <- d_a_obj_ferris.cpp:254
    [-1025.04, 332.98, 162.68], // <- d_a_obj_ferris.cpp:255
    [0.0, 0.0, 0.0],            // <- d_a_obj_ferris.cpp:256
];
/// ```
/// cXyz offsets[5] = { cXyz(-993.54f, -325.78f, 37.63f), cXyz(-616.86f, 844.24f, 37.63f),
///                    cXyz(612.3f, 847.55f, 37.63f), cXyz(995.28f, -320.43f, 37.63f),
///                    cXyz(2.82f, -1045.59f, 37.63f) };
/// ```
/// (`d_a_obj_ferris.cpp:455-461`) - the RIDE-SURFACE anchors, a DIFFERENT table from
/// [`GONDOLA_OFFSETS``: these feed `mRideWaveAngle` (`:470`), not `set_mtx`.
const RIDE_OFFSETS: [[f32; 3]; 5] = [
    [-993.54, -325.78, 37.63], // <- d_a_obj_ferris.cpp:456
    [-616.86, 844.24, 37.63],  // <- d_a_obj_ferris.cpp:457
    [612.3, 847.55, 37.63],    // <- d_a_obj_ferris.cpp:458
    [995.28, -320.43, 37.63],  // <- d_a_obj_ferris.cpp:459
    [2.82, -1045.59, 37.63],   // <- d_a_obj_ferris.cpp:460
];
/// `mRotAngle = 0x1800;` (`:194`, `:335`) - the wheel's rest pitch, 45 degrees in 0x10000 units.
const REST_ANGLE: i16 = 0x1800; // <- d_a_obj_ferris.cpp:194
/// `mRotSpeed = 45; mRotState = 1;` (`:185-186`) when the stage switch is already on.
const BOOT_SPEED: i16 = 45; // <- d_a_obj_ferris.cpp:185
/// ```
/// if (mRotTimer < 11) { mRotSpeed = 4; }
/// else { switch (mRotState) { case 9: case 5: if (!(mFrameTimer & 3)) mRotSpeed--; break;
///                              default: mRotSpeed--; break; } }
/// ```
/// (`:370-383`) - the spin-up rate and the every-fourth-frame decay.
const SPIN_UP: i16 = 4; // <- d_a_obj_ferris.cpp:371
const DECAY_MASK: i16 = 3; // <- d_a_obj_ferris.cpp:376
/// `if (mRotTimer > 20) { mRotTimer = 0; mRotState++; }` (`:359-362`) - the dwell between states.
const STATE_DWELL: i16 = 20; // <- d_a_obj_ferris.cpp:359
/// ```
/// case 1: dComIfGs_onEventBit(dSv_event_flag_c::UNK_2104);
///        if (!(mFrameTimer & 3)) mRotSpeed++;
///        if (now_event(mEventIdx_kanban_start)) { if (mRotSpeed > 67) mRotSpeed = 67; }
///        else { if (mRotSpeed > 45) mRotSpeed = 45; }
/// ```
/// (`:415-428`) - the running wheel: a story bit, the same every-fourth-frame accel, and a speed
/// cap that rises from `Attr_c::field_0x00` to 67 while the start cut plays.
const RUN_MAX: i16 = 67; // <- d_a_obj_ferris.cpp:421
const RUN_FLAG: u16 = 0x2104;
/// ```
/// case 10: mRotSpeed = -5;
///        if (mRotAngle < 0x1800 || !now_event(mEventIdx_kanban_vive)) { mRotState = 0; mRotSpeed = 0; }
/// ```
/// (`:408-414`) - the wind-down.
const SPIN_DOWN: i16 = -5; // <- d_a_obj_ferris.cpp:409
/// `mGondolaWaveTimer += 500; mGondolaWaveAngle = cM_ssin(mGondolaWaveTimer) * 380.0f;`
/// (`:326-327`) - the gondolas' idle sway, in 0x10000 units.
const WAVE_STEP: i16 = 500; // <- d_a_obj_ferris.cpp:326
const WAVE_AMP: f32 = 380.0; // <- d_a_obj_ferris.cpp:327
/// ```
/// mDoMtx_multVec(mMtx[i], &offs1, &pt1);   // offs1 = (0.0f, -100.0f, -135.0f)
/// delta.set(pt1.x - pt0.x, 0.0f, pt1.z - pt0.z);
/// delta.normalizeRS();
/// ...
/// f32 h2 = h / 162.0f;
/// mRideWaveTarget[i] = (temp < 0.0f) ? h2 * 550.0f : h2 * -550.0f;
/// ```
/// (`:494-511`) - the pendulum's suspension arm length and its lean limit.
const LEAN_ARM: f32 = 162.0; // <- d_a_obj_ferris.cpp:506
const LEAN_LIMIT: f32 = 550.0; // <- d_a_obj_ferris.cpp:508, :510
/// `cLib_addCalcAngleS2(&mRideWaveAngle[i], mRideWaveTarget[i], 4, 0x1000);` (`:516`) - the
/// suspension's chase rate.
const LEAN_SCALE: i16 = 4; // <- d_a_obj_ferris.cpp:516
const LEAN_MAX: i16 = 0x1000; // <- d_a_obj_ferris.cpp:516
/// ```
/// mEventIdx_kanban_vive = dComIfGp_evmng_getEventIdx("kanran_vive");
/// mEventIdx_kanban_start = dComIfGp_evmng_getEventIdx("kanran_start");
/// ```
/// (`:192-193`) - note the stage file spells the stage `Skanran` but the cut names use `kanran`.
const EV_KANBAN_VIVE: &str = "kanran_vive"; // <- d_a_obj_ferris.cpp:192
const EV_KANBAN_START: &str = "kanran_start"; // <- d_a_obj_ferris.cpp:193
/// ```
/// fopAcM_setCullSizeBox(this, -1400.0f, -1400.0f, -500.f, 1400.0f, 1400.0f, 800.0f);
/// fopAcM_setCullSizeFar(this, 10.0f);
/// ```
/// (`:182-183`) - the actor's one real custom cull box, matching the profile's
/// `fopAc_CULLBOX_CUSTOM_e` (`:598`). The far plane is a renderer override, not a box.
const CULL_MIN: [f32; 3] = [-1400.0, -1400.0, -500.0]; // <- d_a_obj_ferris.cpp:182
const CULL_MAX: [f32; 3] = [1400.0, 1400.0, 800.0]; // <- d_a_obj_ferris.cpp:182
/// ```
/// const dCcD_SrcCyl M_cyl_src = { ... SrcObjAt Type 0, SrcObjTg Type
///     ~(AT_TYPE_LIGHT | AT_TYPE_UNK400000 | AT_TYPE_WIND | AT_TYPE_UNK20000 | AT_TYPE_WATER), ... };
/// ```
/// (`:21-49`). `SrcObjAt Type = 0` is the HOUSE NOTE's "the actor deals no damage" case, and
/// `SrcObjTg Type` excludes five bits - so the gondola cylinders are collision geometry only.
/// `Radius 100.0f` / `Height 200.0f` (`:46-47`) are the values `Create` installs; `set_collision`
/// overwrites them every tick with `CYL_R` / `CYL_H` (`:439-440`) and `CYL_H_2` (`:447`).
const CYL_DEFAULT_R: f32 = 100.0; // <- d_a_obj_ferris.cpp:46
const CYL_DEFAULT_H: f32 = 200.0; // <- d_a_obj_ferris.cpp:47
/// ```
/// const dCcD_SrcSph M_sph_src = { ... SrcObjAt Type 0, SrcObjTg Type 0 ... Radius 35.0f };
/// ```
/// (`:51-78`). `SrcObjTg Type = 0` (`:29` of the struct, `:55` in the literal) means this sphere is
/// never a target: the ride spheres are pure collision.
const SPH_DEFAULT_R: f32 = 35.0; // <- d_a_obj_ferris.cpp:77
/// `daObjFerris::Act_c` (`d_a_obj_ferris.h:34-72`).
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `M_cyl_src` sets `SrcObjAt Type = 0`
/// (`:25`) and `M_sph_src` sets `SrcObjAt Type = 0` (`:55`) - the HOUSE NOTE's first case: the
/// Ferris registers NO attack collider, so it damages nothing. `SrcGObjAt HitMark = 0` (`:32`, `:62`)
/// and the `OnTgNoHitMark()` calls (`:202`, `:208`, `:214`) mean even a hit leaves no mark, and
/// neither `tg_hitCB` nor `co_hitCB` exists in the class (`d_a_obj_ferris.h:43-62`), so there is no
/// `c_damagereaction` hookup and no `health` field on the actor at all. `SrcObjTg Type` on
/// `M_cyl_src` does accept most Tg types (`:28`), but that is COLLISION - standing on a gondola -
/// not damage. The profile `Group` is `fopAc_ACTOR_e` (`:597`). Neither `combat::HitPoints` nor
/// `enemy::Hurt` is attached; `combat::control` would despawn the Ferris wheel, which retail cannot
/// damage.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Ferris {
    /// `daObjFerris::Act_c::param_get_swSave()` (`d_a_obj_ferris.h:40`) - the parameter's low byte,
    /// `PRM_SWSAVE_W = 0x08`, `PRM_SWSAVE_S = 0x00` (`d_a_obj_ferris.h:37-38`). Read by
    /// `is_switch()` (`:39`) in `rot_mng`'s case 0 (`:336`) and cleared by `off_switch()` (`:351`).
    pub switch_bit: u8,
    /// `mRotState` (`d_a_obj_ferris.h:50`) - the wheel's state-machine index, `0..=10`.
    pub state: i16,
    /// `mRotSpeed` (`:51`) - the current spin rate, clamped to `0..=67`.
    pub speed: i16,
    /// `mRotAngle` (`:49`) - the wheel's roll, chased by `speed` each tick (`:523`).
    pub angle: i16,
    /// `mRotTimer` (`:48`) - the dwell counter inside the current state.
    pub timer: i16,
    /// `mGondolaWaveAngle` (`:54`) - the gondolas' idle sway offset.
    pub wave_angle: i16,
    /// `mGondolaWaveTimer` (`:55`) - the sway's phase, advanced by `WAVE_STEP` per tick.
    pub wave_timer: i16,
    /// `mFrameTimer` (`:53`) - the frame counter that gates the every-fourth-frame speed changes.
    pub frame_timer: i16,
    /// `mRideWaveAngle[5]` (`:56`) - each gondola's suspension lean, in 0x10000 units.
    pub ride_angle: [i16; 5],
    /// `mRideWaveTarget[5]` (`:57`) - each gondola's lean TARGET, from `make_lean` (`:495-517`).
    pub ride_target: [i16; 5],
    /// `mRideState[6]` (`d_a_obj_ferris.h:32`) - which gondola has a rider. `ride_call_back`
    /// (`:155-164`) sets index `i` to 1 and records `mRidePos`; `make_lean` clears every entry each
    /// tick (`:517`), so the sim samples this once per tick rather than waiting for the callback,
    /// which needs the `dBgW` ride system (RULE 3b).
    pub ridden: [bool; 6],
    /// `mRidePos` (`:33`) - the rider's position, written by `ride_call_back` (`:159`).
    pub ride_pos: Vec3,
    /// Whether each gondola's `field_0x1238[i]` ride sphere passes `if (pt.y <= 1820.0f)`
    /// (`:477`) - i.e. whether it is registered this tick.
    pub live: [bool; 5],
    /// The world centre each `field_0x1238[i]` would take, from `set_collision` (`:478`).
    pub surfaces: [[f32; 3]; 5],
    /// `mEventState` (`:62`) - `0` idle, `1` ordering, `2` running (`now_event` / `set_event`
    /// `:289-301`, `exe_event` `:304-322`).
    pub event_state: i16,
    /// `mEventIdx` (`:61`) - the slot `set_event` is holding.
    pub event_slot: Option<usize>,
    /// `mEventIdx_kanban_start` (`:50`) from `getEventIdx("kanran_start")` (`:193`).
    pub start_slot: Option<usize>,
    /// `mEventIdx_kanban_vive` (`:49`) from `getEventIdx("kanran_vive")` (`:192`).
    pub vive_slot: Option<usize>,
    /// `mMtx[6]` (`d_a_obj_ferris.h:66`) - each gondola's world matrix. `set_collision` reads it via
    /// `mDoMtx_multVec` (`:437`, `:442`, `:496-497`), so the yaw / roll chain is modelled as a
    /// [`Gondola`] rather than a matrix.
    pub gondolas: [Gondola; 6],
    /// One-shot guard so `_create`'s switch read and the event lookups run exactly once.
    pub entered: bool,
}

/// One gondola's transform, `mMtx[idx]` (`d_a_obj_ferris.h:66`) rebuilt from `set_mtx`
/// (`d_a_obj_ferris.cpp:249-277`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Gondola {
    /// The wheel-local anchor, `offset[idx]` (`:250-257`).
    pub offset: [f32; 3],
    /// `mGondolaWaveAngle` (`:265`) - the idle sway added after the roll is undone.
    pub wave: i16,
    /// `mRideWaveAngle[idx]` (`:470`) - the suspension lean, which for a RIDE SPHERE replaces the
    /// idle `wave` in the `ZrotM`.
    pub ride: i16,
}

impl Ferris {
    /// `daObjFerris::Act_c::_create` (`:167-222`)'s parameter decode and state seed.
    ///
    /// `daObjFerris::Act_c::param_get_swSave()` is
    /// `daObj::PrmAbstract<Param_e>(this, PRM_SWSAVE_W = 0x08, PRM_SWSAVE_S = 0x00)`
    /// (`d_a_obj_ferris.h:37-41`), i.e. `((1 << 8) - 1) & (param >> 0)` - the parameter's low byte.
    /// The profile's `Parameters` field is `0` (`:593`), so the whole `u32` is available.
    ///
    /// ```
    /// if (is_switch() == 1) { mRotSpeed = 45; mRotState = 1; } else { mRotSpeed = 0; mRotState = 0; }
    /// mEventIdx_kanban_vive = dComIfGp_evmng_getEventIdx("kanran_vive");
    /// mEventIdx_kanban_start = dComIfGp_evmng_getEventIdx("kanran_start");
    /// mRotAngle = 0x1800;
    /// mGondolaWaveAngle = 0;
    /// ```
    /// (`:184-195`)
    ///
    /// `is_switch()` (`:39`) is `fopAcM_isSwitch(this, param_get_swSave())`, which is
    /// `dComIfGs_isSwitch(sw, fopAcM_GetHomeRoomNo(pActor))` (`f_op_actor_mng.h`) - the house
    /// reduction `Switches::test`. RULE 11b: a `swSave` at or above 0x80 reads false here even though
    /// retail routes it to the per-zone `dSv_zoneBit_c` arrays; the REAL index is passed through.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            // `daObj::PrmAbstract<PRM_SWSAVE_W = 8, PRM_SWSAVE_S = 0>(this, ...)`
            // (`d_a_obj_ferris.h:40`) -> `((1 << 8) - 1) & (param >> 0)`, the parameter's low byte.
            // `rot_mng`'s `is_switch()` (`:336`) and `off_switch()` (`:351`) both read it.
            switch_bit: parameters as u8,
            // `mRotState` is sampled in `create_init` (`:184-190`), where `Switches` is available;
            // `mRotSpeed` there is overwritten by `rot_mng` on the very next line, so its seed
            // carries no information.
            state: 0,
            speed: 0,
            // `mRotAngle = 0x1800;` (:194)
            angle: REST_ANGLE,
            // `mGondolaWaveAngle = 0;` (:195)
            wave_angle: 0,
            // `mGondolaWaveTimer` takes the zero fill (`f_pc_base.cpp:97-98`).
            wave_timer: 0,
            // `mRotTimer`, `mFrameTimer`, `mRideWaveAngle[5]`, `mRideWaveTarget[5]`,
            // `mRideState[6]`, `mEventIdx`, `mEventState` likewise.
            timer: 0,
            frame_timer: 0,
            ride_angle: [0; 5],
            ride_target: [0; 5],
            ridden: [false; 6],
            // `mRidePos` likewise.
            ride_pos: Vec3::ZERO,
            // `field_0x1238[5]` holds no stored centre until `set_collision` (`:433-483`) first
            // runs, so both arrays start empty.
            live: [false; 5],
            surfaces: [[0.0; 3]; 5],
            // `mEventState` is 0 until `set_event` claims it (:296-299).
            event_state: 0,
            event_slot: None,
            start_slot: None,
            vive_slot: None,
            // `mMtx[6]` is built by `init_mtx` (`:280-286`) from `set_mtx` (`:249-277`).
            gondolas: build_gondolas(),
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -1400.0f, -1400.0f, -500.f, 1400.0f, 1400.0f, 800.0f)`
    /// (`d_a_obj_ferris.cpp:182`) - the actor's one real custom cull box, matching the profile's
    /// `fopAc_CULLBOX_CUSTOM_e` (`:598`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// The gondola cylinder, re-registered every tick by `set_collision` (`:439-440`).
    pub fn cylinder(&self) -> (f32, f32) {
        (CYL_R, CYL_H)
    }

    /// The second gondola cylinder's height (`:447`) - the class writes `CYL_H_2`, not
    /// `Attr_c::mCylH`, into it.
    pub fn cylinder_tall(&self) -> (f32, f32) {
        (CYL_R, CYL_H_2)
    }

    /// The ride sphere's radius (`:479`), distinct from `M_sph_src`'s `Radius 35.0f` (`:77`).
    pub fn sphere(&self) -> f32 {
        SPH_R
    }

    /// `daObjFerris::Act_c::_create` (`:167-222`) first tick only: the switch read that seeds
    /// `mRotState` (`:184-190`) and the two event-index lookups (`:192-193`).
    ///
    /// DROPPED: `dComIfG_resLoad` (`:170`), `fopAcM_entrySolidHeap` / `solidHeapCB` /
    /// `create_heap` (`:174`) - heap and archive work; `fopAcM_SetMtx` (`:175`); the six
    /// `dComIfG_Bgsp()->Regist` / `SetCrrFunc` / `SetRideCallback` calls (`:177-181`) - `dBgW`
    /// registration and the ride callback, RULE 3b; `init_mtx` (`:105`) - model and matrix work;
    /// `dComIfG_setListBG` / `modelUpdateDL` (`:117-118`, `:118`, `:123`) - renderer.
    fn create_init(&mut self, list: &EventList, switches: &Switches) {
        // `if (is_switch() == 1) { mRotSpeed = 45; mRotState = 1; } else { mRotSpeed = 0;
        // mRotState = 0; }` (:184-190). RULE 11b: a `swSave` at or above 0x80 reads false here
        // even though retail routes it to the per-zone `dSv_zoneBit_c` arrays.
        if switches.test(self.switch_no()) {
            self.speed = BOOT_SPEED;
            self.state = 1;
        } else {
            self.speed = 0;
            self.state = 0;
        }
        // `mEventIdx_kanban_vive = dComIfGp_evmng_getEventIdx("kanran_vive");` (:192)
        self.vive_slot = list.event_index(EV_KANBAN_VIVE);
        // `mEventIdx_kanban_start = dComIfGp_evmng_getEventIdx("kanran_start");` (:193)
        self.start_slot = list.event_index(EV_KANBAN_START);
        self.entered = true;
    }

    /// The switch bit, `daObjFerris::Act_c::param_get_swSave()`
    /// (`d_a_obj_ferris.h:40`) = the parameter's low byte. Stored as a field because
    /// `off_switch()` (`:41`) writes it and `rot_mng` calls that at `:351`.
    fn switch_no(&self) -> u8 {
        self.switch_bit
    }

    /// `daObjFerris::Act_c::now_event(s16 p1)` (`:289-291`): `mEventState != 0 && mEventIdx == p1`.
    fn now_event(&self, slot: Option<usize>) -> bool {
        self.event_state != 0 && self.event_slot == slot && slot.is_some()
    }

    /// `daObjFerris::Act_c::set_event(s16 p1)` (`:294-301`): claims the event slot only from idle.
    fn set_event(&mut self, slot: Option<usize>) -> bool {
        if self.event_state == 0 {
            self.event_slot = slot;
            self.event_state = 1;
            return true;
        }
        false
    }

    /// `daObjFerris::Act_c::exe_event` (`:304-322`).
    ///
    /// ```
    /// switch (mEventState) {
    /// case 1:
    ///     if (eventInfo.checkCommandDemoAccrpt()) { mEventState = 2; }
    ///     else { fopAcM_orderOtherEventId(this, mEventIdx);
    ///            eventInfo.onCondition(dEvtCnd_UNK2_e); }
    ///     break;
    /// case 2:
    ///     if (dComIfGp_evmng_endCheck(mEventIdx)) {
    ///         dComIfGp_event_reset();
    ///         mEventIdx = -1;
    ///         mEventState = 0;
    ///     }
    ///     break;
    /// }
    /// ```
    ///
    /// `eventInfo.checkCommandDemoAccrpt()` is the per-actor `dEvtCmd_INDEMO_e` latch the sim does
    /// not carry; the house reduction is `Stage::Play | Stage::Close` on this actor's own slot
    /// (`hbrf1.rs:258`, `hole.rs:293`).
    fn exe_event(&mut self, active: &mut ActiveEvent, entity: Entity, list: &EventList) {
        match self.event_state {
            // `case 1:` (:306-313)
            1 => {
                let Some(slot) = self.event_slot else {
                    return;
                };
                if demo_running(active, slot) {
                    self.event_state = 2;
                } else {
                    // `fopAcM_orderOtherEventId(this, mEventIdx);` (:310). RULE 5: only order when
                    // this actor does not already own the slot, or the demo livelocks.
                    if active.stage_of(slot).is_none() {
                        active.order(list, slot, entity);
                    }
                    // `eventInfo.onCondition(dEvtCnd_UNK2_e);` (:311) - a per-actor condition bit
                    // with no modelled reader (the `dai.rs:351-355` omission).
                }
            }
            // `case 2:` (:314-321)
            2 => {
                let Some(slot) = self.event_slot else {
                    return;
                };
                if active.end_check(slot) {
                    // `dComIfGp_event_reset();` (:316) - `ActiveEvent::end`.
                    active.end(slot);
                    // `mEventIdx = -1; mEventState = 0;` (:317-318)
                    self.event_slot = None;
                    self.event_state = 0;
                }
            }
            // No default; `mEventState` only ever holds 0, 1 or 2.
            _ => {}
        }
    }

    /// `daObjFerris::Act_c::angle_mng` (`:326-328`).
    ///
    /// `mGondolaWaveTimer += 500; mGondolaWaveAngle = cM_ssin(mGondolaWaveTimer) * 380.0f;` -
    /// RULE 5b: `super::stream::sin_s` delegates to `super::walk::from_s16`, the crate's only
    /// radians path, and `0x10000 == one full turn`.
    fn angle_mng(&mut self) {
        self.wave_timer = self.wave_timer.wrapping_add(WAVE_STEP);
        self.wave_angle = (super::stream::sin_s(self.wave_timer) * WAVE_AMP) as i16;
    }

    /// `daObjFerris::Act_c::rot_mng` (`:331-430`), the wheel's state machine.
    ///
    /// DROPPED: the `case 0` wind test's inner body. `if (is_switch()) { cXyz* wind =
    /// dKyw_get_wind_vec(); s16 windAngle = cM_atan2s(wind->x, wind->z); ... }` (`:336-353`) reads
    /// `dKyw_get_wind_vec`, which the brief lists as a config symbol with NO value in this checkout -
    /// UNKNOWN, not proven zero - so the whole branch is unavailable and no wind direction is
    /// invented. That leaves the wheel in state 0 forever, which is the correct outcome for the sim.
    ///
    /// DROPPED: `dCam_getBody()` (absent), `mDoAud_seStart(JA_SE_READ_RIDDLE_1)` (`:344`).
    fn rot_mng(&mut self, flags: &mut EventReg) {
        match self.state {
            // ```
            // case 0: mRotSpeed = 0; mRotAngle = 0x1800;
            //         if (is_switch()) { ...wind... }
            // ```
            // (:333-355)
            0 => {
                self.speed = 0;
                self.angle = REST_ANGLE;
            }
            // ```
            // case 2: case 6:
            //     mRotTimer++;
            //     if (mRotTimer > 20) { mRotTimer = 0; mRotState++; }
            // ```
            // (:356-363)
            2 | 6 => {
                self.timer += 1;
                if self.timer > STATE_DWELL {
                    self.timer = 0;
                    self.state += 1;
                }
            }
            // ```
            // case 3: case 4: case 5: case 7: case 8: case 9:
            //     if (mRotTimer < 11) { mRotSpeed = 4; }
            //     else {
            //         switch (mRotState) {
            //         case 9: case 5: if (!(mFrameTimer & 3)) mRotSpeed--; break;
            //         default: mRotSpeed--; break;
            //         }
            //     }
            //     mRotTimer++;
            //     if (mRotSpeed < 0) {
            //         mRotSpeed = 0; mRotTimer = 0;
            //         switch (mRotState) {
            //         case 6: break;
            //         case 3: case 4: case 7: case 8: mRotState++; break;
            //         case 5: mRotState = 10; break;
            //         case 9: mRotState = 1; break;
            //         }
            //     }
            // ```
            // (:364-407)
            3 | 4 | 5 | 7 | 8 | 9 => {
                if self.timer < 11 {
                    self.speed = SPIN_UP;
                } else if self.state == 9 || self.state == 5 {
                    // `case 9: case 5: if (!(mFrameTimer & 3)) mRotSpeed--;` (:374-378)
                    if self.frame_timer & DECAY_MASK == 0 {
                        self.speed -= 1;
                    }
                } else {
                    // `default: mRotSpeed--;` (:379-382)
                    self.speed -= 1;
                }
                self.timer += 1;
                if self.speed < 0 {
                    self.speed = 0;
                    self.timer = 0;
                    match self.state {
                        // `case 6: break;` (:391-392) - stays put.
                        6 => {}
                        // `case 3: case 4: case 7: case 8: mRotState++;` (:393-398)
                        3 | 4 | 7 | 8 => self.state += 1,
                        // `case 5: mRotState = 10;` (:399-400)
                        5 => self.state = 10,
                        // `case 9: mRotState = 1;` (:402-403)
                        9 => self.state = 1,
                        // No default; the arm list is exactly these six states.
                        _ => {}
                    }
                }
            }
            // ```
            // case 10:
            //     mRotSpeed = -5;
            //     if (mRotAngle < 0x1800 || !now_event(mEventIdx_kanban_vive)) {
            //         mRotState = 0; mRotSpeed = 0;
            //     }
            // ```
            // (:408-414) - `mRotAngle` is written by `_execute` at :523, i.e. BEFORE this runs.
            10 => {
                self.speed = SPIN_DOWN;
                if self.angle < REST_ANGLE || !self.now_event(self.vive_slot) {
                    self.state = 0;
                    self.speed = 0;
                }
            }
            // ```
            // case 1:
            //     dComIfGs_onEventBit(dSv_event_flag_c::UNK_2104);
            //     if (!(mFrameTimer & 3)) mRotSpeed++;
            //     if (now_event(mEventIdx_kanban_start)) { if (mRotSpeed > 67) mRotSpeed = 67; }
            //     else { if (mRotSpeed > 45) mRotSpeed = 45; }
            //     angle_mng();
            // ```
            // (:415-428)
            1 => {
                // `dComIfGs_onEventBit(dSv_event_flag_c::UNK_2104);` (:416) - RULE 11c: the retail id
                // goes straight into the byte-indexed `EventFlags`.
                flags.set(RUN_FLAG);
                if self.frame_timer & DECAY_MASK == 0 {
                    self.speed += 1;
                }
                if self.now_event(self.start_slot) {
                    if self.speed > RUN_MAX {
                        self.speed = RUN_MAX;
                    }
                } else if self.speed > SPEED_FULL {
                    self.speed = SPEED_FULL;
                }
                // `angle_mng();` (:427)
                self.angle_mng();
            }
            // No default; `mRotState` is only ever written 0..=10 by the arms above and by `_create`
            // (`:186`, `:189`).
            _ => {}
        }
    }

    /// `daObjFerris::Act_c::make_lean` (`:486-519`), the gondolas' pendulum.
    ///
    /// ```
    /// for (i = 0; i < 5; i++) {
    ///     if (mRideState[i] == 1) {
    ///         mDoMtx_multVec(mMtx[i], &offs0, &pt0);
    ///         mDoMtx_multVec(mMtx[i], &offs1, &pt1);
    ///         delta.set(pt1.x - pt0.x, 0.0f, pt1.z - pt0.z);
    ///         delta2.set(mRidePos.x - pt0.x, 0.0f, mRidePos.z - pt0.z);
    ///         delta.normalizeRS();
    ///         f32 temp2 = -delta.z;
    ///         f32 h = std::fabsf(-(temp2 * pt0.x + delta.x * pt0.z) + (temp2 * mRidePos.x + delta.x * mRidePos.z));
    ///         f32 temp = delta.x * delta2.z - delta.z * delta2.x;
    ///         f32 h2 = h / 162.0f;
    ///         if (temp < 0.0f) { mRideWaveTarget[i] = h2 * 550.0f; }
    ///         else { mRideWaveTarget[i] = h2 * -550.0f; }
    ///     } else { mRideWaveTarget[i] = 0; }
    ///     cLib_addCalcAngleS2(&mRideWaveAngle[i], mRideWaveTarget[i], 4, 0x1000);
    ///     mRideState[i] = 0;
    /// }
    /// ```
    ///
    /// `offs1` is `cXyz(0.0f, -100.0f, -135.0f)` (`:489`) - [`CYL_OFFSET_1`] - and `offs0` is the
    /// origin (`:490`). The chase is `cLib_addCalcAngleS2`, transcribed locally rather than reusing
    /// `stream::add_calc_angle_s2`, which the brief's joint-turning rule forbids introducing.
    ///
    /// NOTE `mRideState[0]` is used at `:495` even though `ride_call_back` (`:157-160`) writes it,
    /// so the reader and the writer use the same index; the sim samples the whole array and clears
    /// it here (`:517`).
    fn make_lean(&mut self) {
        for index in 0..5 {
            let gondola = &self.gondolas[index];
            let (sin_a, cos_a) = super::stream::sin_cos(self.angle);
            let (sin_w, cos_w) = super::stream::sin_cos(gondola.ride);
            // `mDoMtx_multVec(mMtx[i], &offs0, &pt0)` / `(..., &offs1, &pt1)` (:496-497) with
            // `offs0 = (0,0,0)` and `offs1 = CYL_OFFSET_1`.
            let pt0 = rotate(gondola.offset, self.angle, gondola.wave, gondola.ride);
            let pt1 = rotate(
                [
                    gondola.offset[0] + CYL_OFFSET_1[0],
                    gondola.offset[1] + CYL_OFFSET_1[1],
                    gondola.offset[2] + CYL_OFFSET_1[2],
                ],
                self.angle,
                gondola.wave,
                gondola.ride,
            );
            let target = if self.ridden[index] {
                // `delta.set(pt1.x - pt0.x, 0.0f, pt1.z - pt0.z);` (:499)
                let mut delta = Vec3::new(pt1.x - pt0.x, 0.0, pt1.z - pt0.z);
                // `delta.normalizeRS();` (:501) - the in-plane normalised direction.
                let length = delta.x * delta.x + delta.z * delta.z;
                if length > 0.0 {
                    let inv = 1.0 / length.sqrt();
                    delta.x *= inv;
                    delta.z *= inv;
                }
                // `delta2.set(mRidePos.x - pt0.x, 0.0f, mRidePos.z - pt0.z);` (:500)
                let delta2 = Vec3::new(self.ride_pos.x - pt0.x, 0.0, self.ride_pos.z - pt0.z);
                // ```
                // f32 temp2 = -delta.z;
                // f32 h = std::fabsf(-(temp2 * pt0.x + delta.x * pt0.z)
                //                    + (temp2 * mRidePos.x + delta.x * mRidePos.z));
                // ```
                // (:503-504)
                let temp2 = -delta.z;
                let h = (-(temp2 * pt0.x + delta.x * pt0.z)
                    + (temp2 * self.ride_pos.x + delta.x * self.ride_pos.z))
                    .abs();
                // `f32 temp = delta.x * delta2.z - delta.z * delta2.x;` (:505)
                let temp = delta.x * delta2.z - delta.z * delta2.x;
                // `f32 h2 = h / 162.0f;` (:506)
                let h2 = h / LEAN_ARM;
                // ```
                // if (temp < 0.0f) { mRideWaveTarget[i] = h2 * 550.0f; }
                // else { mRideWaveTarget[i] = h2 * -550.0f; }
                // ```
                // (:507-511)
                if temp < 0.0 {
                    (h2 * LEAN_LIMIT) as i16
                } else {
                    (h2 * -LEAN_LIMIT) as i16
                }
            } else {
                // `} else { mRideWaveTarget[i] = 0; }` (:513)
                0
            };
            self.ride_target[index] = target;
            // `cLib_addCalcAngleS2(&mRideWaveAngle[i], mRideWaveTarget[i], 4, 0x1000);` (:516)
            self.ride_angle[index] =
                chase_angle_s2(self.ride_angle[index], target, LEAN_SCALE, LEAN_MAX);
            // `mRideState[i] = 0;` (:517) - the ride callback is a one-tick pulse.
            self.ridden[index] = false;
            // `sin_a` / `cos_a` are consumed by `rotate`; keeping the bindings named here documents
            // that `set_mtx` reads the wheel's angle in 0x10000 units (RULE 5b).
            let _ = (sin_a, cos_a, sin_w, cos_w);
        }
    }

    /// `daObjFerris::Act_c::set_collision` (`:433-483`), reduced to which gondolas are live and
    /// where their ride surfaces are.
    ///
    /// For each gondola it transforms the local origins through `mMtx[i]` and gates every
    /// registration on `pt.y <= 1820.0f` (`:438`, `:477`):
    /// * `field_0x0400[i]`: `SetR(50.0f)`, `SetH(195.0f)`, centre from `cXyz(0.0f, -100.0f,
    ///   -135.0f)` (`:439-443`).
    /// * `field_0x0b1c[i]`: `SetR(50.0f)`, `SetH(148.0f)`, centre from `cXyz(0.0f, -56.0f, 127.0f)`
    ///   (`:446-450`).
    /// * `field_0x1238[i]`: `SetC(pt)`, `SetR(30.0f)`, from the RIDE offsets (`:463-481`).
    ///
    /// `dComIfG_Ccsp()->Set(...)` (`:444`, `:451`, `:480`) is the `cCcD` collider registry, which
    /// RULE 3b lists as absent, so nothing is registered; the world points are returned so the gate
    /// and the geometry are still observable.
    fn set_collision(&self, origin: Vec3, yaw: i16) -> [[f32; 3]; 5] {
        let (sin_y, cos_y) = super::stream::sin_cos(yaw);
        let mut points = [[0.0; 3]; 5];
        for (index, slot) in points.iter_mut().enumerate() {
            let gondola = self.gondolas[index];
            // ```
            // mDoMtx_stack_c::transS(current.pos); YrotM(shape_angle.y); ZrotM(mRotAngle);
            // transM(offsets[i]); ZrotM(-mRotAngle); ZrotM(mGondolaWaveAngle + mRideWaveAngle[i]);
            // ```
            // (:465-470) - the Y rotation by `shape_angle.y` then the Z roll by `mRotAngle`.
            let rolled = rotate(RIDE_OFFSETS[index], self.angle, gondola.wave, gondola.ride);
            // ```
            // mDoMtx_multVec(now, &zero_offset, &pt);
            // if (pt.y <= 1820.0f) { field_0x1238[i].SetC(pt); field_0x1238[i].SetR(30.0f);
            //                         dComIfG_Ccsp()->Set(&field_0x1238[i]); }
            // ```
            // (:474-481) - the height gate, `Attr_c::field_0x18`.
            *slot = [
                origin.x + rolled.x * cos_y + rolled.z * sin_y,
                origin.y + rolled.y,
                origin.z - rolled.x * sin_y + rolled.z * cos_y,
            ];
        }
        points
    }

    /// `if (pt.y <= 1820.0f)` (`:438`, `:477`) - `Attr_c::field_0x18` is the height above which a
    /// gondola's collider is NOT registered.
    fn reachable(world_y: f32) -> bool {
        world_y <= RIDE_GATE_Y
    }

    /// `daObjFerris::Act_c::_execute` (`:522-538`).
    ///
    /// ```
    /// mRotAngle += mRotSpeed;
    /// f32 volume = (mRotSpeed / 45.0f) * 100.0f;
    /// volume = volume > 100.0f ? 100.0f : volume;
    /// mDoAud_seStart(JA_SE_OBJ_FER_WHEEL_ROUND, &current.pos, volume);
    /// rot_mng();
    /// exe_event();
    /// make_lean();
    /// for (i = 0; i < 6; i++) { set_mtx(i); if (i < 6) mpBgW[i]->Move(); }
    /// set_collision();
    /// mFrameTimer++;
    /// ```
    ///
    /// DROPPED: the positional `JA_SE_OBJ_FER_WHEEL_ROUND` volume ramp (`:524-526`) - audio with no
    /// modelled channel; the six `mpBgW[i]->Move()` calls (`:533`) - `dBgW` moving backgrounds,
    /// RULE 3b; `set_mtx`'s `setBaseTRMtx` and `mDoMtx_copy` (`:266-267`, `:274-275`) - model
    /// matrix writes, whose RESULT is modelled as a [`Gondola`] transform because `set_collision`
    /// reads it back (`:437`, `:442`, `:496`).
    fn execute(
        &mut self,
        active: &mut ActiveEvent,
        flags: &mut EventReg,
        entity: Entity,
        list: &EventList,
        origin: Vec3,
        yaw: i16,
    ) {
        // `mRotAngle += mRotSpeed;` (:523)
        self.angle = self.angle.wrapping_add(self.speed);
        // `rot_mng();` (:527)
        self.rot_mng(flags);
        // `exe_event();` (:528)
        self.exe_event(active, entity, list);
        // `make_lean();` (:529)
        self.make_lean();
        // ```
        // for (i = 0; i < 6; i++) { set_mtx(i); if (i < 6) mpBgW[i]->Move(); }
        // set_collision();
        // ```
        // (:530-535) - `set_mtx` writes each `mpModel[i]` / `mMtx[i]`; `set_collision` reads `mMtx`
        // back through `mDoMtx_multVec`, so the transform is modelled as [`Ferris::set_collision`].
        for (index, point) in self.set_collision(origin, yaw).iter().enumerate() {
            // ```
            // if (pt.y <= 1820.0f) { field_0x1238[i].SetC(pt); field_0x1238[i].SetR(30.0f);
            //                         dComIfG_Ccsp()->Set(&field_0x1238[i]); }
            // ```
            // (:477-481) - the gate. The `cCcD` registration is dropped (RULE 3b); the live flag and
            // the surface point are kept, because they are what `ride_call_back` (`:155-164`) and
            // `make_lean` (`:495`) key off.
            self.live[index] = Self::reachable(point[1]);
            self.surfaces[index] = *point;
        }
        // `mFrameTimer++;` (:536)
        self.frame_timer = self.frame_timer.wrapping_add(1);
    }
}

/// `static cXyz offset[6]` (`d_a_obj_ferris.cpp:250-257`) as the per-gondola table `set_mtx` reads.
/// `mRideWaveAngle[idx]` and `mGondolaWaveAngle` both take the zero fill (`f_pc_base.cpp:97-98`).
fn build_gondolas() -> [Gondola; 6] {
    GONDOLA_OFFSETS.map(|offset| Gondola {
        offset,
        // `mGondolaWaveAngle = 0;` (:195)
        wave: 0,
        // `mRideWaveAngle[5]` zero fill.
        ride: 0,
    })
}

/// `mDoMtx_stack_c::transS(pos) / YrotM(roll) / transM(offset) / ZrotM(-roll) / ZrotM(wave)` then
/// `mDoMtx_multVec(mMtx[i], &vec, &out)` (`d_a_obj_ferris.cpp:260-267`, `:496-497`) reduced to a
/// direct transform in 0x10000 units (RULE 5b via `super::stream::sin_cos`).
fn rotate(offset: [f32; 3], angle: i16, wave: i16, ride: i16) -> glam::Vec3 {
    let (sin_a, cos_a) = super::stream::sin_cos(angle);
    let (sin_w, cos_w) = super::stream::sin_cos(wave.wrapping_add(ride));
    // Z rot by `angle`: x' = x cos - y sin, y' = x sin + y cos.
    let x = offset[0] * cos_a - offset[1] * sin_a;
    let y = offset[0] * sin_a + offset[1] * cos_a;
    // Z rot by `wave + ride`.
    glam::Vec3::new(x * cos_w - y * sin_w, x * sin_w + y * cos_w, offset[2])
}

/// `cLib_addCalcAngleS2(s16* pValue, s16 target, s16 scale, s16 maxStep)` (`c_lib.cpp:192-202`):
///
/// ```c
/// s16 diff = target - *pValue;
/// s16 step = diff / scale;
/// if (step > maxStep)       { *pValue += maxStep; }
/// else if (step < -maxStep) { *pValue -= maxStep; }
/// else                     { *pValue += step; }
/// ```
///
/// Transcribed locally: this is NOT `super::walk::add_calc_angle_s` (the five-argument
/// `cLib_addCalcAngleS`) and NOT `super::stream::add_calc_angle_s2`, which the brief's joint-turning
/// rule forbids introducing. RULE 10: both the subtraction and the division are `s16` in retail.
fn chase_angle_s2(value: i16, target: i16, scale: i16, max_step: i16) -> i16 {
    let diff = target.wrapping_sub(value);
    let step = diff.wrapping_div(scale);
    if step > max_step {
        value.wrapping_add(max_step)
    } else if step < max_step.wrapping_neg() {
        value.wrapping_sub(max_step)
    } else {
        value.wrapping_add(step)
    }
}

/// `eventInfo.checkCommandDemoAccrpt()` as `Stage::Play | Stage::Close` - the two states
/// `dEvent_manager_c::getMyStaffId` scans (`d_event_manager.cpp:412`). The `hbrf1.rs:258` /
/// `hole.rs:293` house reduction.
fn demo_running(active: &ActiveEvent, slot: usize) -> bool {
    matches!(
        active.stage_of(slot),
        Some(Stage::Play) | Some(Stage::Close)
    )
}

pub fn is_ferris(name: &str) -> bool {
    FERRIS_ACTORS.contains(&name)
}

pub fn control(
    mut wheels: Query<(Entity, &mut Actor, &mut Ferris), With<Ferris>>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut flags: ResMut<EventReg>,
    switches: Res<Switches>,
) {
    for (entity, mut actor, mut wheel) in &mut wheels {
        if !wheel.entered {
            // `daObjFerris::Act_c::_create` (`:167-222`)
            wheel.create_init(&list, &switches);
        }
        // ```
        // if (is_switch()) { ... wind ... if (set_event(mEventIdx_kanban_start)) {
        //     mRotState = 6; mRotSpeed = 0; mRotTimer = 0; mDoAud_seStart(JA_SE_READ_RIDDLE_1); } }
        // ```
        // (:336-353) - the wind vector has no value in this checkout, so the whole body is
        // unavailable. The two `set_event` calls it makes are therefore unreachable and not
        // transcribed; `off_switch()` (`:351`) goes with them.
        // `mDoMtx_stack_c::YrotM(shape_angle.y)` (`:261`) reads `shape_angle`, which for a
        // stage-placed actor equals `current.angle` - RULE 2, since `dStage_actorInit` never writes
        // a separate shape angle and `fopAc_Init` seeds both from the placement.
        wheel.execute(
            &mut active,
            &mut flags,
            entity,
            &list,
            actor.pos,
            actor.angle[1],
        );
    }
}
