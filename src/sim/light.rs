//! `daObjLight::Act_c` (`tww/src/d/actor/d_a_obj_light.cpp`): the lighthouse at the Tower of the
//! Gods. One tower model and two lamp models hung off it, a s16 rotation that walks the lamps
//! around the tower once the light is lit, a save bit that says whether the light has ever been
//! lit, and an eighty-frame script that runs the ignition, the event order, the bit's four
//! toggles and the treasure switch that opens the chest at the tower's foot. Archive `"Skanran"`
//! (`d_a_obj_light.cpp:27`), process `fpcNm_Obj_Light_e`, stage name `"Stoudai"`
//! (`tww/src/d/d_stage.cpp:1118`).
//!
//! What survives here is every branch of the state machine. [`Light::create_init`]
//! (`d_a_obj_light.cpp:116-144`) seeds the angle and the lit latch, [`Light::set_collision`]
//! (`:96-113`) is the fire-arrow target test that starts the whole thing, [`Light::renew_angle`]
//! (`:292-305`) turns the lamp, [`Light::control_light`] (`:308-329`) runs the ignition script,
//! [`Light::exe_event`] (`:241-259`) holds the `light_start` event, and
//! [`Light::control_treasure`] (`:332-338`) writes switch 93. The five functions run in
//! `_execute`'s order (`:341-351`), the two dropped calls aside.
//!
//! The angle is shared state, and the sharing is the point of [`LightGlobal`]: four `M_S_`
//! statics live in `tww/src/d/d_com_static.cpp:147-150` and the two accessors that arbitrate
//! between the lighthouse actor and the LOD group are `:170-172` and `:187-196`. The arbitration
//! is frame-numbered: whichever writer's frame differs from the other's last write owns the
//! angle, and the loser reads the winner's value back instead of its own. [`Light::renew_angle`]
//! runs the actor half of that argument, [`Light::create_init`] the day-time seed at
//! `d_a_obj_light.cpp:124-126`, and the statics keep their zero-fill across a stage change
//! because the resource is inserted once, with the sim.
//!
//! ## Not carried
//!
//! * **The LOD partner**, which is the other half of the angle argument. `renew_light_angle`
//!   (`tww/src/d/d_com_static.cpp:153-162`), `get_light_angle` (`:165-167`) and
//!   `set_light_dif_angle_LOD` (`:175-184`) have exactly one caller each, in
//!   `tww/src/d/actor/d_a_lod_bg.cpp:120`, `:123`, `:464`, `:467`, and that actor is not ported.
//!   So `M_S_pre_set_frame_LOD` never moves off its zero, `set_light_dif_angle_FRRS`' first
//!   condition (`g_Counter.mCounter0 != M_S_pre_set_frame_LOD`, `d_com_static.cpp:188`) is a
//!   test against a value nothing writes, and [`LightGlobal::set_light_dif_angle_frrs`] carries
//!   it as written rather than folding it away. `M_S_lod_access` (`d_com_static.cpp:154`) is a
//!   draw-gate flag written by that same unported caller and read at `d_a_obj_light.cpp:148` and
//!   `:362`, both in dropped work, so it is not carried at all.
//! * **The fire-arrow target test's own collision object.** `init_collision`
//!   (`d_a_obj_light.cpp:87-93`) builds `mCyl` from `M_cyl_src` (`:28-32`), a cylinder of radius
//!   80 and height 130 with `SrcObjTg Type` `AT_TYPE_FIRE_ARROW` and `SrcObjTg SPrm`
//!   `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsOther_e` and no attack and no CO half at all, and then
//!   adds `OnTgNoHitMark` (`tww/include/d/d_cc_d.h:419`). [`super::bodies`] has no `"Stoudai"`
//!   row, for two reasons: the volume's only consumer in the reference is `mCyl.ChkTgHit()`
//!   (`d_a_obj_light.cpp:101`), and the pass that would answer it can never see an attacker this
//!   cylinder accepts - no port actor registers a `bodies` AT body whose type includes
//!   `AT_TYPE_FIRE_ARROW`, and the arrow that does carry that type
//!   (`tww/src/d/actor/d_a_arrow.cpp:22`, ported as `super::arrow`'s own table at
//!   `src/sim/arrow.rs:65-72`) sweeps it itself against the actors with health
//!   (`src/sim/arrow.rs:235-251`) and never enters the shared pass. So
//!   [`Light::set_collision`] surveys `Query<&Arrow>` geometrically instead, which is the
//!   reduction `super::hmlif` already uses for the same question (`src/sim/hmlif.rs:133-145`).
//!   The two filters `cCcS::ChkNoHitAtTg` (`tww/src/SSystem/SComponent/c_cc_s.cpp:101-112`)
//!   would apply are settled facts here rather than per-arrow tests: the type test is the fire
//!   arrow's `AT_TYPE_FIRE_ARROW` against the plate's own mask (`src/sim/barricade.rs:52` gives
//!   the bit as `1 << 18`), and the group test is the arrow's
//!   `cCcD_AtSPrm_VsEnemy_e | cCcD_AtSPrm_VsOther_e` against `cCcD_TgSPrm_IsOther_e`, which meet
//!   on the "other" group (`src/sim/contact_hit.rs:22-27` is the arrow's own mask).
//! * **`mFireCallback`, the follow emitter.** `set_fire` (`d_a_obj_light.cpp:160-175`) asks
//!   `dComIfGp_particle_set(dPa_name::ID_AK_JN_TORCH, &current.pos, NULL, &particleScale, 0xFF,
//!   &mFireCallback)` for the torch (`:163`, and the id is
//!   `tww/include/d/d_particle_name.h:121`), and every other call in the file reads the handle
//!   back - `exe_fire` and `draw_fire` guard on `mFireCallback.getEmitter() != NULL`
//!   (`:200`, `:179`), `delete_fire` removes it (`:221`) and `set_fire`'s second arm removes it
//!   too (`:172`). No actor in this sim can attach a follow emitter: `super::jpa`'s
//!   `Follow` callbacks are private to that module's own registry, and the one particle channel
//!   an actor does have (`super::damage_fx`) carries an id, a point, an angle and a scale and no
//!   handle back. So [`Light::fire`] is the latch alone, and the two arms of `set_fire` -
//!   "create and answer TRUE" and "remove and answer FALSE" - are ported on it. The second arm
//!   is unreachable in one lifetime, and is ported anyway: `mLit` goes to 1 at `:104` or `:134`
//!   and nothing ever clears it, so `control_light` cannot pass through the `mLightTimer == 10`
//!   branch twice.
//! * **`mBgw`, the tower's own background.** `create_heap` builds it from the archive's DZB
//!   (`:74-81`), `_create` registers it with `dComIfG_Bgsp()->Regist` and
//!   `SetCrrFunc(dBgS_MoveBGProc_Typical)` (`:131-132`), `_execute` moves it (`:347`) and
//!   `_delete` releases it (`:152`). There is no per-actor DZB for a model archive here, the
//!   same omission `super::balancelift` documents for its `pm_bgw`; the tower is a static wall
//!   either way and the typical mover would move it by nothing.
//! * **Renderer and matrix work**, each dropped at the line it is read at in `d_a_obj_light.cpp`:
//!   `create_heap` (`:40-84`) and the `cPhs_ERROR_e` its failure produces through `fopAcM_entrySolidHeap`
//!   (`:120`, `:140`), so [`Light::spawn`] never refuses; `fopAcM_SetMtx` (`:128`) and
//!   `fopAcM_setCullSizeFar` (`:130`); `set_mtx` (`:262-289`), which is the lamp's visible turn -
//!   three models, the two lamps offset `z = +377.74f` and `-377.74f` (`:265`) with the second
//!   one's `pos.z += 700.0f` and `+ 0x8000` yaw (`:281`, `:283`), and the copy into
//!   `mBgMtx` (`:272`) is for the background that is not carried either; `exe_fire` (`:199-214`) and
//!   `draw_fire` (`:178-196`) together, the flame's `cM_ssin` bob on `mFirePhase` (`:201`, `:212`),
//!   its `mFireAlpha = 140 + (int)(10.0f * wave)` and `mFireScale = 0.45f + 0.05f * wave`
//!   (`:204-205`), its `mFireAngle += 720` spin (`:206`), the `mDoMtx_stack_c` chain that composes
//!   them (`:207-210`) and the `dComIfGd_setAlphaModel(1, mFireMtx, mFireAlpha)` that consumes the
//!   result (`:194`). `mFirePhase`, `mFireAngle`, `mFireAlpha`, `mFireScale` and `mFireMtx`
//!   (`tww/include/d/actor/d_a_obj_light.h`, the `mFire` group) have no reader left in the port, so they are
//!   not stored (RULE 7) - which is also why the two dropped calls hold their place in
//!   [`Light::execute`] as comments and nothing more.
//! * `eventInfo.onCondition(2)` (`d_a_obj_light.cpp:247`), a per-actor event condition with no
//!   modelled reader - the same omission `super::ferris` notes for the identical call
//!   (`src/sim/ferris.rs:394-395`).
//! * `L_attr` (`d_a_obj_light.cpp:18-23`), the anonymous-namespace `Attr_c {0x80, 2, 20}`, which
//!   the translation unit never reads: its three numbers reappear as literals at `:302`
//!   ([`ANGLE_STEP_DARK`]), `:168` ([`SHOCK_PATTERN`]) and `:255` ([`TREASURE_DELAY`]). The struct
//!   is therefore not carried, and the three constants are cited to the lines that use them.
//! * RULE 0, the NTSC-U target: `#if VERSION == VERSION_DEMO` bodies are excluded, which takes
//!   `create_heap`'s first arm (`:44-60`), `_delete`'s unguarded `ChkUsed` release (`:149-153`),
//!   `draw_fire`'s `dComIfGd_setAlphaModelColor` (`:180-187`) and `delete_fire`'s unconditional
//!   `remove` (`:218-222`). The opposite form of the same conditional - `#if VERSION > VERSION_DEMO`
//!   around `set_collision`'s night test (`:97-99`) - is the release build and IS kept.
//!
//! Two store facts worth stating, because the tree has confused them before. The save bit
//! `dSv_event_flag_c::UNK_1C02` (`d_a_obj_light.cpp:122`, `:134`, `:299`, and the four
//! `dComIfGs_onEventBit` / `offEventBit` of `:318-324`) is a `dSv_event_flag_c` id, so it lives in
//! [`super::flags::StoryBits`] - the sim's save-bit store, aliased as `EventReg` at
//! `src/sim/flags.rs:482` - and NOT in `super::flags::EventFlags`, which is the event scripts'
//! `dEvDtFlag_c` store and cannot address a `0x1C02` at all. And `dComIfGs_onSwitch(93,
//! fopAcM_GetHomeRoomNo(this))` (`:335`) is a room switch, which [`super::flags::Switches`] keeps
//! without the room argument; 93 is under [`super::flags::Switches::SWITCH_BITS`], so RULE 11b
//! does not bite here.
//!
//! `_delete` (`:147-157`) and the profile's `fopAcM_CULLBOX_CUSTOM_e` aside, the actor is a
//! permanent resident: `IsDelete` returns `TRUE` unconditionally (`:395-397`) so it accepts a
//! delete request and never makes one, and its status is `fopAcStts_NOCULLEXEC_e |
//! fopAcStts_CULL_e | fopAcStts_UNK40000_e` (`:421`) - culled, and yet executed even while
//! culled, which the port does unconditionally, so the lamp keeps turning off-camera and the fire
//! arrow can light it from a room the camera is not in.
use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::arrow::{Arrow, CAPSULE_RADIUS, CAPSULE_STRETCH, Phase, TYPE_FIRE};
use super::contact;
use super::daynight::{DayNight, TIME_NIGHT};
use super::event::{ActiveEvent, EventList, Stage};
use super::events::{SimEvent, SimEvents};
use super::flags::{StoryBits, Switches};
use super::frame::CurrentFrame;
use super::shape::Shape;
use super::vibration::Vibration;

/// `OBJNAME("Stoudai", fpcNm_Obj_Light_e, 255, 0)` - the stage name, whose `255` argument is
/// `dStage_objectNameInf::argument` and never reaches the actor
/// (`tww/src/d/d_stage.cpp:1118`, `:1301-1310`).
pub const LIGHT_ACTORS: [&str; 1] = ["Stoudai"];

/// `fopAcM_setCullSizeBox(this, -300.0f, -300.0f, -10000.0f, 300.0f, 300.0f, 10000.0f)` - the box
/// that reaches ten thousand units fore and aft, which is the tower and its lamp rather than the
/// housing.
const CULL_MIN: [f32; 3] = [-300.0, -300.0, -10000.0]; // <- d_a_obj_light.cpp:129
const CULL_MAX: [f32; 3] = [300.0, 300.0, 10000.0]; // <- d_a_obj_light.cpp:129

/// `M_cyl_src`'s `cM3dGCylS`, the target volume the fire arrow has to reach (`:31`) under the
/// `/* Radius */ 80.0f` and `/* Height */ 130.0f` it carries.
const CYL_RADIUS: f32 = 80.0; // <- d_a_obj_light.cpp:31
const CYL_HEIGHT: f32 = 130.0; // <- d_a_obj_light.cpp:31

/// `pos.y -= 75.0f`, the drop that puts the volume's base under the actor - half the height, so
/// the un-hit cylinder is centred on the placement.
const CYL_DROP: f32 = 75.0; // <- d_a_obj_light.cpp:108

/// `mLightAngle = 0x4000`, the quarter turn the daylight and the create seed both park the lamps
/// at (`d_a_obj_light.cpp:124`, `:304`).
const ANGLE_DAY: i16 = 0x4000;
/// The two steps `renew_angle` turns by: `0x40` for a lit lamp and for the slow sweep the event
/// runs, `0x80` for an unlit one at night (`:295`, `:300`, `:302`), all `s16` and wrapping.
const ANGLE_STEP_LIT: i16 = 0x40;
const ANGLE_STEP_DARK: i16 = 0x80;

/// `dSv_event_flag_c::UNK_1C02` (`tww/include/d/d_save_event_flag.inc:215`), the save bit that
/// says the light has been lit. Read at `d_a_obj_light.cpp:122`, `:134` and `:299`, written by
/// the four toggles of `:318-324`.
const EVT_LIT: u16 = 0x1C02;

/// `dComIfGp_evmng_getEventIdx("light_start")`, the event the ignition orders.
const LIGHT_START: &str = "light_start"; // <- d_a_obj_light.cpp:136

/// The seven `mLightTimer` marks the ignition script stops on: `1` orders the event (`:312`), the
/// `default` arm's `++` then walks 2 upward and the chain at `:316-326` answers 10, 75, 76, 77, 79
/// and 80.
const TIMER_ORDER: i16 = 1; // <- d_a_obj_light.cpp:311
const TIMER_FIRE: i16 = 10; // <- d_a_obj_light.cpp:316
const TIMER_BIT_ON: i16 = 75; // <- d_a_obj_light.cpp:317
const TIMER_BIT_OFF: i16 = 76; // <- d_a_obj_light.cpp:320
const TIMER_BIT_ON2: i16 = 77; // <- d_a_obj_light.cpp:321
const TIMER_BIT_OFF2: i16 = 79; // <- d_a_obj_light.cpp:322
const TIMER_DONE: i16 = 80; // <- d_a_obj_light.cpp:323

/// `dComIfGs_onSwitch(93, fopAcM_GetHomeRoomNo(this))`, the treasure room switch the script's
/// tail writes.
const TREASURE_SWITCH: u8 = 93; // <- d_a_obj_light.cpp:335
/// `mTreasureTimer = 20`, set when the event ends and counted down to 1, where the switch goes up.
const TREASURE_DELAY: i16 = 20; // <- d_a_obj_light.cpp:255

/// `mDoAud_seStart(JA_SE_OBJ_L_HOUSE_ROUND, &current.pos)`, the lamp's turn cue, which BOTH night
/// arms play (`:296`, `:303`).
const SE_ROUND: u32 = 0x303B; // <- JAZelAudio_SE.h:624
/// `mDoAud_seStart(JA_SE_OBJ_L_HOUSE_FIRE, &current.pos)`, the ignition's fire cue (`:167`).
const SE_FIRE: u32 = 0x6A03; // <- JAZelAudio_SE.h:2026
/// `mDoAud_seStart(JA_SE_OBJ_L_HOUSE_LIGHT, &current.pos)`, the cue the 75-frame mark plays with
/// the first bit set (`:319`).
const SE_LIGHT: u32 = 0x6A04; // <- JAZelAudio_SE.h:2027

/// `dComIfGp_getVibration().StartShock(2, -17, cXyz(0.0f, 1.0f, 0.0f))`, the rumble only the
/// `effect == 1` arm of `set_fire` makes - which is the script's call, never `create_init`'s.
const SHOCK_PATTERN: i32 = 2; // <- d_a_obj_light.cpp:168
const SHOCK_FLAGS: i32 = -17; // <- d_a_obj_light.cpp:168
const SHOCK_UP: [f32; 3] = [0.0, 1.0, 0.0]; // <- d_a_obj_light.cpp:168

/// `d_a_obj_light.cpp:126` and `tww/src/d/d_com_static.cpp:147-150`, the four `M_S_` statics the
/// lighthouse and the LOD group share. Only three are carried: `M_S_lod_access` (`:150`) belongs
/// to the dropped draw gate and to the unported LOD caller, and `get_light_angle` (`:165-167`) -
/// which is `M_S_light_angle` plus `0xF6C2` - has no caller but that same unported one.
///
/// `M_S_pre_set_frame_LOD` (`:148`) is written only by `set_light_dif_angle_LOD` (`:175-184`),
/// which `renew_light_angle` (`:153-162`) is the only caller of, so it keeps the zero the retail
/// `.bss` gives it here too; the FRRS accessor's first condition still tests against it.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct LightGlobal {
    /// `M_S_light_angle` (`d_com_static.cpp:147`), the angle both writers share.
    pub angle: i16,
    /// `M_S_pre_set_frame_LOD` (`d_com_static.cpp:148`), the LOD writer's stamp.
    pub pre_set_frame_lod: u32,
    /// `M_S_pre_set_frame_FRRS` (`d_com_static.cpp:149`), the lighthouse writer's stamp.
    pub pre_set_frame_frrs: u32,
}

impl LightGlobal {
    /// `daObjLight::Act_c::get_light_dif_angle` (`d_com_static.cpp:170-172`): the stored angle,
    /// which is what `_create` seeds from at `d_a_obj_light.cpp:122` and what `renew_angle` falls
    /// back to at `:301` when it loses the frame argument.
    pub fn get_light_dif_angle(&self) -> i16 {
        self.angle
    }

    /// `daObjLight::Act_c::set_light_dif_angle_FRRS(s16 r3)` (`d_com_static.cpp:187-196`), the
    /// lighthouse's half of the arbitration. `g_Counter.mCounter0` is
    /// [`super::frame::CurrentFrame`]'s own count.
    ///
    /// ```c
    /// if (g_Counter.mCounter0 != M_S_pre_set_frame_LOD && g_Counter.mCounter0-1 == M_S_pre_set_frame_FRRS) {
    ///     M_S_light_angle = r3;
    ///     M_S_pre_set_frame_FRRS = g_Counter.mCounter0;
    ///     return true;
    /// } else {
    ///     M_S_pre_set_frame_FRRS = g_Counter.mCounter0;
    ///     return false;
    /// }
    /// ```
    ///
    /// The second condition is the one that matters: a writer that already stamped this frame
    /// fails, so two lighthouses in one stage cannot both push their own angle, and the loser's
    /// `mCounter0-1` no longer equals the winner's stamp either. Both arms write the stamp, so a
    /// refusal is not a no-op - it re-files this writer's frame.
    pub fn set_light_dif_angle_frrs(&mut self, angle: i16, counter0: u32) -> bool {
        if counter0 != self.pre_set_frame_lod && counter0.wrapping_sub(1) == self.pre_set_frame_frrs
        {
            self.angle = angle; // <- d_com_static.cpp:189
            self.pre_set_frame_frrs = counter0; // <- :190
            return true;
        }
        self.pre_set_frame_frrs = counter0; // <- :193
        false
    }
}

/// The lighthouse itself - `daObjLight::Act_c` (`tww/include/d/actor/d_a_obj_light.h:16-58`), the
/// `"Stoudai"` tower that a fire arrow lights, plays an event while it does, and then turns a
/// lamp for the rest of the night.
///
/// (a) inbound damage: NONE as damage, ALL of it as a target. The cylinder's `SrcObjTg Type` is
/// `AT_TYPE_FIRE_ARROW` and nothing else (`d_a_obj_light.cpp:29`), so the only contact the actor
/// answers is a fire arrow's, and the answer is [`Light::set_collision`]'s latch, not health: no
/// `super::combat::HitPoints` and no `super::enemy::Hurt` is attached. (b) outbound damage: NONE.
/// `M_cyl_src`'s `SrcObjAt Type` / `Atp` / `SPrm` are all zero (`:29-30`), so the cylinder never
/// attacks, and `SrcObjCo SPrm` is zero too, so it corrects nobody's position and is corrected by
/// nobody.
///
/// `mStts.Init(0xFF, 0xFF, this)` (`:88`) is the collision weight, which [`super::weight`]
/// tabulates for the stage name. The `OnTgNoHitMark` of `:92` is the general TG bit
/// `dCcG_TgSPrm_NoHitMark_e` (`tww/include/d/d_cc_d.h:59`), which suppresses the hit mark a
/// `dCcS::SetAtTgGObjInf` would otherwise publish; it belongs to the volume this module does not
/// register, so it is recorded here and nowhere else.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Light {
    /// `mLightAngle` (`d_a_obj_light.h:2D8`), the lamp's yaw. `s16`, wrapping, and shared through
    /// [`LightGlobal::angle`] whenever the night arm's frame argument succeeds.
    pub angle: i16,
    /// `mLightTimer` (`:2DA`), the ignition script's own counter - `0` idle, `1` waiting for the
    /// event slot, `2..` the walk through the six marks.
    pub light_timer: i16,
    /// `mLit` (`:2DC`), the latch that says the arrow has reached the tower. Set from the save bit
    /// at `:134` and from the target test at `:104`, and never cleared.
    pub lit: i16,
    /// `mTreasureTimer` (`:2DE`), the twenty-frame count from the event's end to switch 93.
    pub treasure_timer: i16,
    /// `mFireCallback` (`:44C`) as the latch that the torch emitter exists - see the module docs
    /// for why the handle itself cannot be carried.
    pub fire: bool,
    /// `mEventIdx` (`:49C`) as the slot this actor holds, [`super::ferris`]'s `Option<usize>`
    /// reading of the same field: `None` is retail's `-1`.
    pub event_slot: Option<usize>,
    /// `mEventState` (`:49E`) - `0` idle, `1` ordering, `2` running.
    pub event_state: i16,
    /// `mLightEvent` (`:4A0`), the slot `dComIfGp_evmng_getEventIdx("light_start")` answered, or
    /// `None` for retail's `-1` when the stage's event list has no such script.
    pub light_event: Option<usize>,
    /// The port's own create latch: `create_init` runs once, on the first tick.
    pub entered: bool,
}

impl Light {
    /// `_create` (`:116-144`) reaches here through `Mthd_Create`'s `fopAcM_SetupActor` (`:117`).
    /// The parameter word is not read at all - the profile's `/* Parameters */ 0` (`:417`) means
    /// `fopAcM_GetParam` would answer the room's default, and nothing in the file asks. Every
    /// field is one of the values `create_init` rewrites on the first tick, so all of them start
    /// where retail's `fopAcM_ct` zero-fill leaves them.
    pub fn spawn() -> Self {
        Self {
            angle: 0,
            light_timer: 0,
            lit: 0,
            treasure_timer: 0,
            fire: false,
            event_slot: None,
            event_state: 0,
            light_event: None,
            entered: false,
        }
    }

    /// The cull box `fopAcM_setCullSizeBox` hands the shell (`:129`), which the profile's
    /// `fopAc_CULLBOX_CUSTOM_e` (`:423`) says is the actor's own rather than a sphere off the model.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// The state half of `_create` (`d_a_obj_light.cpp:121-138`), for a lamp that stands where the
    /// room placed it.
    ///
    /// The day arm seeds the shared angle too, so that a lighthouse created in daylight hands the
    /// LOD group's reader a quarter turn rather than a value some other stage's night left behind
    /// (`:125-126`); the night arm only READS it, and only when the bit is already up (`:122`).
    ///
    /// DROPPED with the lines each is read at: `dComIfG_resLoad` (`:118`) and
    /// `fopAcM_entrySolidHeap` / `solidHeapCB` / `create_heap` (`:120`), all heap and archive work,
    /// which is why the `cPhs_ERROR_e` at `:140` never happens; `fopAcM_SetMtx` (`:128`);
    /// `fopAcM_setCullSizeFar` (`:130`), whose `10.0f` the draw layer would have folded into the
    /// box [`Light::cull`] already returns; `dComIfG_Bgsp()->Regist` and `SetCrrFunc` (`:131-132`),
    /// the `dBgW` registration; and `init_collision` (`:133`), whose volume this module surveys
    /// instead of registering - see the module docs.
    fn create_init(&mut self, ctx: &mut Context<'_, '_, '_, '_>) {
        // ```
        // if (dKy_daynight_check() == 1) {
        //     if (dComIfGs_isEventBit(dSv_event_flag_c::UNK_1C02)) mLightAngle = get_light_dif_angle();
        // }
        // ```
        // (`d_a_obj_light.cpp:121-122`) - an unlit night leaves the angle at the zero the setup
        // left, so the first `renew_angle` turn starts from `0` and the `+= 0x80` arm walks up
        // from there.
        if ctx.night {
            if ctx.bits.test(EVT_LIT) {
                self.angle = ctx.global.get_light_dif_angle(); // <- :122
            }
        } else {
            self.angle = ANGLE_DAY; // <- :124
            ctx.global.angle = self.angle; // <- :125
            ctx.global.pre_set_frame_frrs = ctx.counter0; // <- :126
        }
        // `mLit = !!dComIfGs_isEventBit(...)` (`d_a_obj_light.cpp:134`), the save bit's second
        // read, after the angle's conditional one.
        self.lit = ctx.bits.test(EVT_LIT) as i16;
        // `if (dKy_daynight_check() == 1 && mLit == 1) set_fire(0);` (`:135`) - a night load of an
        // already-lit tower starts the torch with `effect` 0, so with no fire cue and no rumble.
        if ctx.night && self.lit == 1 {
            self.set_fire(0, ctx);
        }
        // `mLightEvent = dComIfGp_evmng_getEventIdx("light_start");` (`:136`)
        self.light_event = ctx.list.event_index(LIGHT_START);
        self.light_timer = 0; // <- :137
        self.treasure_timer = 0; // <- :138
        self.entered = true;
    }

    /// `daObjLight::Act_c::now_event(s16 event)` (`:226-228`): `mEventState != 0 && mEventIdx == event`.
    ///
    /// Note the absence of a guard `super::ferris` puts on its own copy of this function
    /// (`src/sim/ferris.rs:343-345`, `&& slot.is_some()`). Here it would be a behaviour change:
    /// when the stage's event list has no `light_start`, retail holds `mLightEvent` at `-1`, and
    /// once the copy in `set_event` puts it in `mEventIdx`, the `now_event` test of
    /// `d_a_obj_light.cpp:293` answers true for the rest of the actor's life, which parks the lamp
    /// at `mLightAngle = 0` on every frame the script is counting. Both `None`s comparing equal is
    /// what that is, and it is kept.
    fn now_event(&self, slot: Option<usize>) -> bool {
        self.event_state != 0 && self.event_slot == slot
    }

    /// `daObjLight::Act_c::set_event(s16 event)` (`:231-238`): claims the slot only from idle, and
    /// answers TRUE when it does.
    fn set_event(&mut self, slot: Option<usize>) -> bool {
        if self.event_state == 0 {
            self.event_slot = slot; // <- :233
            self.event_state = 1; // <- :234
            return true;
        }
        false
    }

    /// `daObjLight::Act_c::exe_event` (`:241-259`), the same two-arm machine `super::ferris` ports
    /// (`src/sim/ferris.rs:357-414`), with one extra write: the end arm also starts the treasure
    /// countdown (`:255`).
    ///
    /// `eventInfo.checkCommandDemoAccrpt()` (`:244`) is the house reduction to
    /// `Stage::Play | Stage::Close` on this actor's own slot (`src/sim/ferris.rs:811-819`), and
    /// `dComIfGp_event_reset()` (`:252`) is [`ActiveEvent::end`].
    fn exe_event(&mut self, ctx: &mut Context<'_, '_, '_, '_>) {
        match self.event_state {
            // `case 1:` (`d_a_obj_light.cpp:243-249`)
            1 => {
                let Some(slot) = self.event_slot else {
                    return;
                };
                if demo_running(ctx.active, slot) {
                    self.event_state = 2; // <- :244
                } else {
                    // `fopAcM_orderOtherEventId(this, mEventIdx);` (`:246`). RULE 5: order only
                    // while this actor does not already own the slot, or the demo livelocks.
                    if ctx.active.stage_of(slot).is_none() {
                        ctx.active.order(ctx.list, slot, ctx.entity);
                    }
                    // `eventInfo.onCondition(2);` (`:247`) - dropped, see the module docs.
                }
            }
            // `case 2:` (`:250-257`)
            2 => {
                let Some(slot) = self.event_slot else {
                    return;
                };
                if ctx.active.end_check(slot) {
                    ctx.active.end(slot); // <- :252
                    self.event_slot = None; // <- :253, the `-1`
                    self.event_state = 0; // <- :254
                    self.treasure_timer = TREASURE_DELAY; // <- :255
                }
            }
            // No default arm; `mEventState` holds only 0, 1 and 2.
            _ => {}
        }
    }

    /// `daObjLight::Act_c::renew_angle` (`:292-305`), the lamp's turn, in the reference's own
    /// four arms.
    ///
    /// ```c
    /// if (now_event(mLightEvent)) {
    ///     if (mLightTimer == 0) { mLightAngle += 0x40; ...SE... } else mLightAngle = 0;
    /// } else if (dKy_daynight_check() == 1) {
    ///     if (dComIfGs_isEventBit(dSv_event_flag_c::UNK_1C02)) {
    ///         mLightAngle += 0x40;
    ///         if (!set_light_dif_angle_FRRS(mLightAngle)) mLightAngle = get_light_dif_angle();
    ///     } else mLightAngle += 0x80;
    ///     ...SE...
    /// } else mLightAngle = 0x4000;
    /// ```
    ///
    /// The event arm is the one that stops the lamp: while the ignition script counts - and it
    /// counts from 2 to 80 without a pause - the angle is written to 0 every frame, and only on
    /// the frame the script is idle does the slow `0x40` step run. Both night arms play the round
    /// cue, the unlit one every frame (`:303`).
    fn renew_angle(&mut self, ctx: &mut Context<'_, '_, '_, '_>) {
        if self.now_event(self.light_event) {
            // `d_a_obj_light.cpp:293-298`
            if self.light_timer == 0 {
                self.angle = self.angle.wrapping_add(ANGLE_STEP_LIT); // <- :295
                ctx.round_se(); // <- :296
            } else {
                self.angle = 0; // <- :297
            }
        } else if ctx.night {
            if ctx.bits.test(EVT_LIT) {
                self.angle = self.angle.wrapping_add(ANGLE_STEP_LIT); // <- :300
                // `:301` - win the frame argument or take the winner's angle back. The store is
                // the shared one, so a second lighthouse in the room is what makes the fallback
                // bite.
                if !ctx
                    .global
                    .set_light_dif_angle_frrs(self.angle, ctx.counter0)
                {
                    self.angle = ctx.global.get_light_dif_angle();
                }
            } else {
                self.angle = self.angle.wrapping_add(ANGLE_STEP_DARK); // <- :302
            }
            ctx.round_se(); // <- :303
        } else {
            self.angle = ANGLE_DAY; // <- :304
        }
    }

    /// `daObjLight::Act_c::set_collision` (`:96-113`): the fire-arrow target test, and the volume
    /// it is made against.
    ///
    /// The whole body sits behind the release-build night test (`:97-99`), so in daylight nothing
    /// is checked and the cylinder is not even registered - which is how an arrow that reaches the
    /// tower at noon lights nothing. At night the answer is the same one `ChkTgHit` gives: a latch
    /// that fires once, because `:102`'s `mLit == 0` test cannot pass a second time.
    ///
    /// The no-hit arm (`:107-110`) re-publishes the volume at `current.pos` minus 75 on y, which is
    /// the registration [`Light::reaches`] tests against instead of one the shared pass holds.
    fn set_collision(&mut self, ctx: &mut Context<'_, '_, '_, '_>) {
        if !ctx.night {
            return;
        }
        if self.reaches(ctx) {
            if self.lit == 0 {
                self.light_timer = TIMER_ORDER; // <- :103
                self.lit = 1; // <- :104
            }
            return;
        }
        // `mCyl.SetC(pos); dComIfG_Ccsp()->Set(&mCyl);` (`:109-110`) has no registration to
        // update: the volume lives in `reaches`, which reads the same three numbers.
    }

    /// Whether any flying fire arrow's capsule crosses the tower's target cylinder this step - the
    /// answer `mCyl.ChkTgHit()` (`d_a_obj_light.cpp:101`) reads off the contact pass, reached here
    /// by the survey the module docs justify.
    ///
    /// The two volumes are the reference's own. The cylinder is `M_cyl_src`'s radius 80 and height
    /// 130 (`:31`) with its base at `pos.y -= 75.0f` (`:107-108`); [`Shape::cylinder`] takes its
    /// point at MID height - the `core_point` clamp and `calc_aab_box`'s own "the half-height box
    /// of a cylinder measured from its centre" (`src/sim/contact.rs:634-636`, `:676-684`) both
    /// read it that way - so the base is lifted by half the height to land the volume on the span
    /// retail registers. The capsule is the arrow's: `daArrow_c::procMove` puts `mAtCps` from the
    /// shot's own position to a point `1.25f` of a step ahead
    /// (`tww/src/d/actor/d_a_arrow.cpp:866-870`), and `src/sim/arrow.rs:202-213` builds the same
    /// two ends off the pre-step position and the velocity it just integrated, which the component
    /// still holds.
    ///
    /// The boolean is [`contact::cross_at_tg`], whose own test is
    /// [`Shape::intersects`] (`src/sim/contact.rs:704`) - the same call the shared pass would have
    /// made had the arrow registered an attack object here.
    fn reaches(&self, ctx: &Context<'_, '_, '_, '_>) -> bool {
        let base = ctx.actor_pos - Vec3::Y * CYL_DROP;
        let centre = base + Vec3::Y * (CYL_HEIGHT * 0.5);
        let tg = Shape::cylinder(centre, CYL_RADIUS, CYL_HEIGHT);
        for arrow in ctx.arrows.iter() {
            // Only a live shaft has an attack capsule at all: `super::arrow` builds one in its
            // `Phase::Flying` arm alone (`src/sim/arrow.rs:201-213`), and a stuck or drowning
            // arrow registers nothing to hit anything with.
            if arrow.phase != Phase::Flying {
                continue;
            }
            // The type filter, which is the only one of the pair that discriminates here: the
            // tower's `SrcObjTg Type` is `AT_TYPE_FIRE_ARROW` (`d_a_obj_light.cpp:29`) and the
            // arrow's is its kind's (`src/sim/arrow.rs:65-72`), so three of the four kinds cannot
            // reach it.
            if arrow.kind != TYPE_FIRE {
                continue;
            }
            let from = arrow.pos - arrow.speed;
            let tip = from + arrow.speed * CAPSULE_STRETCH;
            let at = Shape::capsule((from + tip) * 0.5, CAPSULE_RADIUS, (tip - from).length());
            let mut cross = Vec3::ZERO;
            if contact::cross_at_tg(&at, &tg, &mut cross) {
                return true;
            }
        }
        false
    }

    /// `daObjLight::Act_c::set_fire(int effect)` (`:160-175`). Both arms, on the latch: the emitter
    /// this module cannot attach is what retail tests with `mFireCallback.getEmitter() == NULL`
    /// (`:161`).
    ///
    /// `exe_fire()` (`:165`) is called from the create arm and dropped with the rest of the flame's
    /// matrix work, so `mFirePhase = 0` (`:164`) goes with it; the only observable difference
    /// between the two `effect` values is the cue and the rumble at `:167-168`, and the
    /// `set_fire(0)` of `d_a_obj_light.cpp:135` is [`Light::create_init`]'s call while
    /// [`Light::control_light`] passes 1 (`:316`).
    fn set_fire(&mut self, effect: i32, ctx: &mut Context<'_, '_, '_, '_>) -> bool {
        if self.fire {
            // `mFireCallback.remove(); return FALSE;` (`:172-173`)
            self.fire = false;
            return false;
        }
        self.fire = true; // <- :163, the emitter the port cannot carry
        if effect == 1 {
            ctx.events.list.push(SimEvent::SeAt(SE_FIRE, ctx.actor_pos)); // <- :167
            ctx.vibration
                .start_shock(SHOCK_PATTERN, SHOCK_FLAGS, Vec3::from_array(SHOCK_UP)); // <- :168
        }
        true
    }

    /// `daObjLight::Act_c::control_light` (`:308-329`), the eighty-frame ignition script.
    ///
    /// `case 1` does not increment unless the event slot was claimed (`:312`), so a lighthouse
    /// whose event is already held by another actor waits; once past it, the `default` arm's
    /// `mLightTimer++` (`:315`) runs unconditionally and the six marks are `else if` arms off it,
    /// which is why 75/76/77/79/80 are four toggles of one bit rather than any logic - retail is
    /// strobing the flag while the light comes up. Frame 80 zeroes the counter, and with `mLit`
    /// still 1 (`:104`) nothing sets it again, so the script is once per actor.
    fn control_light(&mut self, ctx: &mut Context<'_, '_, '_, '_>) {
        match self.light_timer {
            0 => {} // <- :310
            TIMER_ORDER => {
                if self.set_event(self.light_event) {
                    self.light_timer += 1; // <- :312
                }
            }
            _ => {
                self.light_timer += 1; // <- :315
                if self.light_timer == TIMER_FIRE {
                    self.set_fire(1, ctx); // <- :316
                } else if self.light_timer == TIMER_BIT_ON {
                    ctx.bits.set(EVT_LIT); // <- :318
                    ctx.events
                        .list
                        .push(SimEvent::SeAt(SE_LIGHT, ctx.actor_pos)); // <- :319
                } else if self.light_timer == TIMER_BIT_OFF {
                    ctx.bits.clear(EVT_LIT); // <- :320
                } else if self.light_timer == TIMER_BIT_ON2 {
                    ctx.bits.set(EVT_LIT); // <- :321
                } else if self.light_timer == TIMER_BIT_OFF2 {
                    ctx.bits.clear(EVT_LIT); // <- :322
                } else if self.light_timer == TIMER_DONE {
                    ctx.bits.set(EVT_LIT); // <- :324
                    self.light_timer = 0; // <- :325
                }
            }
        }
    }

    /// `daObjLight::Act_c::control_treasure` (`:332-338`): the twenty-frame wait after the event,
    /// then switch 93 goes up and the timer is cleared so the write happens once. The `> 1` test
    /// then the `== 1` test is one decrement per frame, and a zero timer matches neither.
    fn control_treasure(&mut self, ctx: &mut Context<'_, '_, '_, '_>) {
        if self.treasure_timer > 1 {
            self.treasure_timer -= 1; // <- :333
        }
        if self.treasure_timer == 1 {
            ctx.switches.set(TREASURE_SWITCH); // <- :335
            self.treasure_timer = 0; // <- :336
        }
    }

    /// `_execute` (`:341-351`), the reference's order, with the two dropped calls left where they
    /// sat.
    fn execute(&mut self, ctx: &mut Context<'_, '_, '_, '_>) {
        // `exe_fire();` (`:342`) - the flame's matrix and alpha, dropped.
        self.exe_event(ctx); // <- :343
        self.renew_angle(ctx); // <- :344
        // `set_mtx();` (`:345`) - the three model matrices, dropped.
        self.set_collision(ctx); // <- :346
        // `M_bgw[LIGHT_LIGHT_BASE]->Move();` (`:347`) - no per-actor `dBgW` to move.
        self.control_light(ctx); // <- :348
        self.control_treasure(ctx); // <- :349
    }
}

/// Everything `execute` needs beyond the component, in one bundle so the ignition script has a
/// single reader - the `hbrf1` / `ferris` shape (`src/sim/hbrf1.rs:385-405`). The access set:
/// `switches` is written only by [`Light::control_treasure`]; `bits` is read by
/// [`Light::create_init`] and [`Light::renew_angle`] and written by only
/// [`Light::control_light`]'s four marks; `global` is written by [`Light::create_init`]'s day arm
/// and by [`LightGlobal::set_light_dif_angle_frrs`] through [`Light::renew_angle`]; `active` and
/// `list` drive [`Light::exe_event`]'s order, and `entity` is the actor
/// `fopAcM_orderOtherEventId` orders from; `events` and `vibration` are the three cues and the one
/// rumble; `arrows` answers [`Light::reaches`]; and `actor_pos` / `night` / `counter0` are the
/// `current.pos`, `dKy_daynight_check()` and `g_Counter.mCounter0` reads the reference makes
/// directly off the actor and the clock.
struct Context<'a, 'w, 's, 'q> {
    bits: &'a mut StoryBits,
    switches: &'a mut Switches,
    global: &'a mut LightGlobal,
    active: &'a mut ActiveEvent,
    list: &'a EventList,
    events: &'a mut SimEvents,
    vibration: &'a mut Vibration,
    arrows: &'a Query<'w, 's, &'q Arrow>,
    entity: Entity,
    actor_pos: Vec3,
    night: bool,
    counter0: u32,
}

impl Context<'_, '_, '_, '_> {
    /// `mDoAud_seStart(JA_SE_OBJ_L_HOUSE_ROUND, &current.pos)`
    /// (`d_a_obj_light.cpp:296`, `:303`), the cue both night arms and the event arm play.
    fn round_se(&mut self) {
        let at = self.actor_pos;
        self.events.list.push(SimEvent::SeAt(SE_ROUND, at));
    }
}

/// `eventInfo.checkCommandDemoAccrpt()` as `Stage::Play | Stage::Close`, the two states
/// `dEvent_manager_c`'s own scan reads (`tww/src/d/d_event_manager.cpp:412`) - the house reduction
/// `super::ferris` documents at `src/sim/ferris.rs:811-819`.
fn demo_running(active: &ActiveEvent, slot: usize) -> bool {
    matches!(
        active.stage_of(slot),
        Some(Stage::Play) | Some(Stage::Close)
    )
}

pub fn is_light(name: &str) -> bool {
    LIGHT_ACTORS.contains(&name)
}

/// `_execute` for every lighthouse in the room.
pub fn control(
    mut lamps: Query<(Entity, &Actor, &mut Light), With<Light>>,
    mut bits: ResMut<StoryBits>,
    mut switches: ResMut<Switches>,
    mut global: ResMut<LightGlobal>,
    mut active: ResMut<ActiveEvent>,
    list: Res<EventList>,
    mut events: ResMut<SimEvents>,
    mut vibration: ResMut<Vibration>,
    arrows: Query<&Arrow>,
    day_night: Res<DayNight>,
    frame: Res<CurrentFrame>,
) {
    let night = day_night.daynight() == TIME_NIGHT;
    let counter0 = frame.0 as u32;
    for (entity, actor, mut lamp) in &mut lamps {
        let mut ctx = Context {
            bits: &mut bits,
            switches: &mut switches,
            global: &mut global,
            active: &mut active,
            list: &list,
            events: &mut events,
            vibration: &mut vibration,
            arrows: &arrows,
            entity,
            actor_pos: actor.pos,
            night,
            counter0,
        };
        if !lamp.entered {
            // `daObjLight::Act_c::_create` (`d_a_obj_light.cpp:116-144`), before the first
            // `_execute`, exactly as `Mthd_Create` precedes `Mthd_Execute`.
            lamp.create_init(&mut ctx);
        }
        lamp.execute(&mut ctx);
    }
}
