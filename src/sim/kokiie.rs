use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::flags::{Collect, Switches};
use super::inventory::{self, Equipped, ITEM_BOOMERANG};
use super::walk;

/// `OBJNAME("Kokiie", fpcNm_KOKIIE_e, 255, 0)` (`d_stage.cpp:888`) is the only row bound to
/// `fpcNm_KOKIIE_e` (`d_a_kokiie.cpp:469`). The class is `kokiie_class` (`d_a_kokiie.h:8`), the
/// profile is `g_profile_KOKIIE` (`:465`) and the resource is the literal `"Kokiie"` (`:379`), so
/// all four spellings agree here.
pub const KOKIIE_ACTORS: [&str; 1] = ["Kokiie"];

/// `fopAcM_SetMin(a_this, a_this->scale.x * -1000.0f, -1000.0f, a_this->scale.z * -1000.0f)` and
/// `fopAcM_SetMax(a_this, a_this->scale.x * 1000.0f, 1000.0f, a_this->scale.z * 1000.0f)`
/// (`d_a_kokiie.cpp:433-434`). Retail is `VERSION_NTSC_U > VERSION_JPN`, so the shipping
/// `VERSION_SELECT` argument of `:433` is the fourth one, `-1000.0f`. The profile's
/// `fopAc_CULLBOX_CUSTOM_e` (`:479`) resolves here, so this IS the box - and it is scaled by the
/// actor's own `scale`, which `Create` stamps from `m2A0`.
pub const CULL_HALF_XZ: f32 = 1000.0; // <- d_a_kokiie.cpp:433, :434
pub const CULL_MIN_Y: f32 = -1000.0; // <- d_a_kokiie.cpp:433
pub const CULL_MAX_Y: f32 = 1000.0; // <- d_a_kokiie.cpp:434

/// ```
/// switch (i_this->m2A0) {
/// case 1: a_this->scale.x = 0.9f; a_this->scale.z = 0.9f; break;
/// case 2: a_this->scale.x = 0.8f; a_this->scale.z = 0.8f; break;
/// case 3: a_this->scale.x = 0.7f; a_this->scale.z = 0.7f; break;
/// default: a_this->scale.z = 1.0f; a_this->scale.x = 1.0f; break;
/// }
/// a_this->scale.y = 1.0f;
/// ```
/// (`d_a_kokiie.cpp:408-430`). `m2A0` is a `u8` with the `0xff` wildcard folded to zero
/// (`:388-391`), so index 0 is the "unspecified" case and lands on the 1.0 default.
pub const SCALE_Y: f32 = 1.0; // <- d_a_kokiie.cpp:430
pub const SCALE_XZ: [f32; 4] = [1.0, 0.9, 0.8, 0.7]; // <- d_a_kokiie.cpp:410-425
pub const SCALE_COUNT: usize = 4; // <- d_a_kokiie.cpp:408 (labels 1, 2, 3 plus default)
/// `if (i_this->m2A0 == 0xff) { i_this->m2A0 = 0; }` (`:389-391`).
pub const PARAM_WILDCARD: u8 = 0xFF; // <- d_a_kokiie.cpp:389
/// `i_this->m2A2 = fopAcM_GetParam(a_this) >> 0x18;` (`:393`) - the "already fell" switch, tested
/// as `(m2A2 != 0xff) && fopAcM_isSwitch(a_this, m2A2)` at `:173` and `:438`.
pub const SWITCH_SHIFT: u32 = 0x18; // <- d_a_kokiie.cpp:393
pub const SWITCH_NONE: u8 = 0xFF; // <- d_a_kokiie.cpp:438

/// `s16 m2D4[5]` (`d_a_kokiie.h:56`), `u8 m2E8[5]` (`:58`) and `u8 m32C[5]` (`:74`) - the five rope
/// slots, looped `for (i = 0; i < ARRAY_SSIZE(i_this->m32C); i++)` (`:71`) and again over `m2F0`
/// (`:317`).
pub const HIMO_COUNT: usize = 5; // <- d_a_kokiie.cpp:71
/// `static u8 himo_off_check[] = {1, 2, 4, 8, 16};` (`:52`) - the five rope-state bits OR-ed into the
/// table index.
const HIMO_CHECK: [u32; HIMO_COUNT] = [1, 2, 4, 8, 16]; // <- d_a_kokiie.cpp:52
/// `static s16 himo_off_ya[]` (`:53-56`) - the rope yaw target, indexed by the five-bit rope mask.
const HIMO_YAW: [i16; 32] = [
    0x1, 0x0, 0x2EE0, 0x1D4C, 0x5FB4, 0x0, 0x4E20, 0x30D4, 0x987C, 0x0, 0x0, 0x1964, 0x7FFF, 0x7FFF,
    0x6784, 0x5014, 0xCF2C, 0xE2B4, 0x0, 0x0, 0x0, 0xE4A8, 0x4C2C, 0x1D4C, 0xB1E0, 0xCF2C, 0xB3D4,
    0xE2B4, 0x987C, 0xB3D4, 0x8000, 0x1,
]; // <- d_a_kokiie.cpp:53-56
/// `static s16 himo_off_xa[]` (`:57-60`) - the rope tilt target.
const HIMO_TILT: [i16; 32] = [
    0x0, 0x7D0, 0x7D0, 0xBB8, 0x7D0, 0x0, 0xBB8, 0xFA0, 0x7D0, 0x0, 0x0, 0x7D0, 0xBB8, 0x7D0, 0xFA0,
    0xFA0, 0x3E8, 0x7D0, 0x0, 0xFA0, 0x0, 0x7D0, 0x7D0, 0xFA0, 0xBB8, 0xFA0, 0xBB8, 0xFA0, 0xFA0,
    0xFA0, 0xFA0, 0xFA0,
]; // <- d_a_kokiie.cpp:57-60
/// `static s16 himo_off_yp[]` (`:61-64`) - the rope drop, NEGATED at `:88`.
const HIMO_DROP: [i16; 32] = [
    0x0, 0x96, 0x96, 0x12C, 0x96, 0x190, 0x190, 0x1F4, 0x96, 0x12C, 0x12C, 0x1F4, 0x12C, 0x1F4,
    0x1F4, 0x2BC, 0x96, 0x12C, 0x12C, 0x1F4, 0x12C, 0x1F4, 0x1F4, 0x2BC, 0x12C, 0x1F4, 0x1F4, 0x2BC,
    0x1F4, 0x2BC, 0x2BC, 0x0,
]; // <- d_a_kokiie.cpp:61-64
/// `(uVar6 == 0x1f)` (`:89`) - all five ropes out, the gate on the boomerang drop.
const ROPE_ALL: u32 = 0x1F; // <- d_a_kokiie.cpp:89

/// `cLib_addCalc2(&i_this->m2D0, REG8_F(4) - himo_off_yp[uVar6], 0.1f, i_this->m338 * 50.0f)`
/// (`:88`). RULE 2b: `REG8_F(4)` is `g_regHIO.mChild[8].mFloatRegs[4]`, zeroed by the constructor
/// (`d_s_play.cpp:80-82`) for all 22 children (`d_s_play.h:163`), so it is a PROVEN `0.0f` and the
/// target folds to `-himo_off_yp[uVar6]`.
pub const DROP_SCALE: f32 = 0.1; // <- d_a_kokiie.cpp:88
pub const DROP_MAX_SCALE: f32 = 50.0; // <- d_a_kokiie.cpp:88 (m338 * 50.0f)
/// `maxSpeed = i_this->m338 * 10000.0f;` (`:95`) - an `f32` truncated into the `s16` max-step
/// parameter of `cLib_addCalcAngleS2`.
pub const TILT_MAX_SCALE: f32 = 10000.0; // <- d_a_kokiie.cpp:95
/// `cLib_addCalcAngleS2(&i_this->m334, target, 4, maxSpeed)` (`:96`).
pub const TILT_STEP: i16 = 4; // <- d_a_kokiie.cpp:96
/// `cLib_addCalcAngleS2(&i_this->m332, sVar1, 2, maxSpeed)` (`:99`).
pub const YAW_STEP: i16 = 2; // <- d_a_kokiie.cpp:99
/// `cLib_addCalc2(&i_this->m338, 1.0f, 1.0f, REG8_F(14) + 0.001f)` (`:102`) - RULE 2b folds
/// `REG8_F(14)` to a proven `0.0f`.
pub const ROPE_SPEED_TARGET: f32 = 1.0; // <- d_a_kokiie.cpp:102
pub const ROPE_SPEED_SCALE: f32 = 1.0; // <- d_a_kokiie.cpp:102
pub const ROPE_SPEED_MAX: f32 = 0.001; // <- d_a_kokiie.cpp:102
/// `cLib_addCalcAngleS2(&actor->current.angle.x, 0, 10, 0x200)` and the `.z` twin (`:103-104`) - the
/// hanging house levels itself out, because the rope tilt lives in `m334` / `m332` and not here.
pub const LEVEL_STEP: i16 = 10; // <- d_a_kokiie.cpp:103
pub const LEVEL_MAX: i16 = 0x200; // <- d_a_kokiie.cpp:103
/// `cLib_addCalcAngleS2(&actor->shape_angle.x, 0, 10, 0x800)` and the `.z` twin (`:125-126`) - the
/// falling house's stronger level-out.
pub const FALL_LEVEL_MAX: i16 = 0x800; // <- d_a_kokiie.cpp:125
/// `cLib_addCalcAngleS2(&i_this->m334, 0, 4, 0x200);` (`:127`) - the falling rope tilt target.
pub const FALL_TILT_MAX: i16 = 0x200; // <- d_a_kokiie.cpp:127
/// `i_this->m2C8.x = i_this->m2BC * cM_ssin(i_this->m298 * 900);` (`:106`) and the `.z` twin at
/// `:107` with `700`. `m2C8` is a `csXyz`, i.e. three `s16` (`c_sxyz.h:18-20`), so each `f32` product
/// is truncated by the `csXyz(s16, s16, s16)` constructor.
pub const SWAY_PERIOD_X: i32 = 900; // <- d_a_kokiie.cpp:106
pub const SWAY_PERIOD_Z: i32 = 700; // <- d_a_kokiie.cpp:107
/// `cLib_addCalc2(&i_this->m2BC, REG14_F(9) + 300.0f, 1.0f, REG14_F(3) + 20.0f)` and the `.m2C4`
/// twin (`:109-110`) - RULE 2b folds both `REG14_F` reads to proven `0.0f`s.
pub const SWAY_REACH: f32 = 300.0; // <- d_a_kokiie.cpp:109
pub const SWAY_REACH_SCALE: f32 = 1.0; // <- d_a_kokiie.cpp:109
pub const SWAY_REACH_MAX: f32 = 20.0; // <- d_a_kokiie.cpp:109
/// `i_this->m2A4.x = i_this->m2B0 * cM_ssin(i_this->m298 * 0x2ee);` (`:112`) and the `.z` twin at
/// `:113` with `900`. `m2A4` is a `cXyz`, so these stay `f32`.
pub const DRIFT_PERIOD_X: i32 = 0x2ee; // <- d_a_kokiie.cpp:112 (750)
pub const DRIFT_PERIOD_Z: i32 = 900; // <- d_a_kokiie.cpp:113
/// `cLib_addCalc0(&i_this->m2B0, 1.0f, 0.25f)` and the `.m2B8` twin (`:115-116`).
pub const DRIFT_GROW_SCALE: f32 = 1.0; // <- d_a_kokiie.cpp:115
pub const DRIFT_GROW_MAX: f32 = 0.25; // <- d_a_kokiie.cpp:115
/// `cLib_addCalc0(&i_this->m2A4.y, 0.05f, REG0_F(7) + 2.0f)` (`:121`) - RULE 2b folds `REG0_F(7)`.
pub const DRIFT_RISE_SCALE: f32 = 0.05; // <- d_a_kokiie.cpp:121
pub const DRIFT_RISE_MAX: f32 = 2.0; // <- d_a_kokiie.cpp:121
/// `actor->speed.y -= REG12_F(14) + 3.0f;` (`:130`) - RULE 2b folds `REG12_F(14)`.
pub const FALL_GRAVITY: f32 = 3.0; // <- d_a_kokiie.cpp:130
/// `i_this->m374 = (REG0_F(9) + 5180.0f) - 3800.0f;` (`:150`) - RULE 2b folds `REG0_F(9)` to a
/// proven `0.0f`, so the resting floor is `5180.0 - 3800.0 = 1380.0`.
pub const FLOOR: f32 = 1380.0; // <- d_a_kokiie.cpp:150
/// `a_this->current.pos.y = i_this->m374 + 50.0f + 720.0f + REG0_F(17);` (`:439`) - RULE 2b folds
/// `REG0_F(17)`. `m374` is read BEFORE any `kokiie_move` has run, so it is still the ctor's `0.0f`
/// on a fresh actor even though `Create` then calls `daKokiie_Execute` twice (`:450-452`); the sum
/// of the surviving terms is therefore `0.0 + 50.0 + 720.0`.
pub const FALLEN_SPAWN_LIFT: f32 = 770.0; // <- d_a_kokiie.cpp:439
/// `cLib_addCalcAngleS2(&i_this->m334, 0, 2, 0x2000);` (`:159`) - the impact settle.
pub const LAND_SETTLE_STEP: i16 = 2; // <- d_a_kokiie.cpp:159
pub const LAND_SETTLE_MAX: i16 = 0x2000; // <- d_a_kokiie.cpp:159
/// `kokiie_move`'s `switch (i_this->m29A)` labels (`:68-70`, `:124`) - `0` hanging, `1` falling.
/// Nothing else ever writes `m29A`.
pub const STATE_HANG: u8 = 0; // <- d_a_kokiie.cpp:69
pub const STATE_FALL: u8 = 1; // <- d_a_kokiie.cpp:124

/// The Forbidden Woods hanging flower house.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `kokiie_class` registers no `dCcD` collider
/// anywhere in the translation unit - no `dCcD_Stts`, no `dCcD_Sph`, no entry collider - and its
/// profile `Group` is `fopAc_ACTOR_e` (`:477`). `a_this->health = 1` (`:437`) and the two
/// `health = 0` writes (`:91`, `:441`) are a private word with no collider behind it, so nothing can
/// act on it. Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `daKokiie_IsDelete` (`:336-338`) is the bare "accepts a delete request" form of RULE 6b, and this
/// translation unit contains no `fopAcM_delete`. `daKokiie_Create` returns `cPhs_ERROR_e` from the
/// `entrySolidHeap` else arm (`:401`) and from the `dBgW` registration arm (`:405`); the first is
/// heap work and the second is a moving background, confirmed-absent under RULE 3b, so `spawn`
/// never refuses and needs no `refused` flag.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Kokiie {
    /// `m29A` (`d_a_kokiie.h:26`) - the `kokiie_move` switch subject (`:68`).
    pub state: u8,
    /// `m2A0` (`d_a_kokiie.h:28`) - the parameter byte with the `0xff` wildcard folded to zero
    /// (`:388-391`); the scale and cull-box selector.
    pub kind: u8,
    /// `m2A2` (`d_a_kokiie.h:30`) - `fopAcM_GetParam(a_this) >> 0x18` (`:393`), the "already fell"
    /// switch.
    pub switch_no: u8,
    /// `m298` (`d_a_kokiie.h:22`) - the `s16` tick counter that drives every sine period.
    pub ticks: i16,
    /// `m32C[i]` (`d_a_kokiie.h:74`) - the five rope states, written by `fpcNm_SHAND_e` through the
    /// `field_314` pointer handed over at `d_a_kokiie.cpp:198`. `fpcNm_SHAND_e` has NO row in
    /// `d_stage.cpp`'s OBJNAME table and no ported actor, so every slot keeps the ctor zero-fill;
    /// the array is stored because `kokiie_move` reads and clears it every tick (`:71-84`).
    pub ropes: [u8; HIMO_COUNT],
    /// `m338` (`d_a_kokiie.h:72`) - the rope speed multiplier, scaling both `addCalcAngleS2` max
    /// steps (`:95`) and the drop `addCalc2` max step (`:88`).
    pub rope_speed: f32,
    /// `m334` (`d_a_kokiie.h:70`) - the rope tilt angle, chased to `himo_off_xa[uVar6]` (`:96`).
    pub tilt: i16,
    /// `m332` (`d_a_kokiie.h:68`) - the rope yaw angle, chased to `himo_off_ya[uVar6]` and only
    /// while `himo_off_xa[uVar6] != 0` (`:98-100`).
    pub yaw: i16,
    /// `m2D0` (`d_a_kokiie.h:50`) - the rope drop added to `current.pos.y` (`:120`).
    pub drop: f32,
    /// `m2BC` / `m2C4` (`d_a_kokiie.h:58` / `:62`) - the sway reach along x and z, chased to
    /// [`SWAY_REACH`] (`:109-110`).
    pub sway: [f32; 2],
    /// `m2B0` / `m2B8` (`d_a_kokiie.h:54` / `:56`) - the drift amplitudes, grown by `cLib_addCalc0`
    /// toward 1.0 (`:115-116`).
    pub drift_amp: [f32; 2],
    /// `m2A4` (`d_a_kokiie.h:48`) - the drift offset added to `home.pos` (`:119`).
    pub drift: Vec3,
    /// `m2C8` (`d_a_kokiie.h:68`) - the sway ANGLES, a `csXyz` of three `s16` added to
    /// `shape_angle` (`:118`). `.y` is never written, so it adds a zero.
    pub sway_angle: [i16; 3],
    /// `m374` (`d_a_kokiie.h:76`) - the resting floor: [`FLOOR`] after the first `kokiie_move` in
    /// [`Kokiie::create_init`], and the ctor's `0.0f` before it (which is what `:439` reads).
    pub floor: f32,
    /// `home.pos` (`fopAc_ac_c`), which has no `Actor` field: the spawn-time position the drift is
    /// measured from (`:119`). Captured once in `Kokiie::create_init`.
    pub home: Vec3,
    /// One-shot guard so the create-time scale stamp (`:408-430`), the already-fallen placement
    /// (`:438-448`) and the two `daKokiie_Execute` calls retail performs at the end of `Create`
    /// (`:450-452`) happen exactly once.
    pub entered: bool,
}

impl Kokiie {
    /// `daKokiie_Create` (`:374-455`) minus the resource load, `himo_create`, `CallbackCreateHeap`
    /// and the `dBgW` registration.
    pub fn spawn(parameters: u32) -> Self {
        // ```
        // i_this->m2A0 = fopAcM_GetParam(a_this);
        // if (i_this->m2A0 == 0xff) { i_this->m2A0 = 0; }
        // i_this->m2A2 = fopAcM_GetParam(a_this) >> 0x18;
        // ```
        // (:388-393). Both fields are `u8` (`d_a_kokiie.h:28`, `:30`), so the high byte is `>> 0x18`
        // with no mask and the low byte is `& 0xFF`.
        let raw = (parameters & 0xFF) as u8;
        Self {
            // `m29A` takes the ctor zero-fill (`f_pc_base.cpp:98`), which is `kokiie_move`'s
            // hanging state.
            state: STATE_HANG,
            kind: if raw == PARAM_WILDCARD { 0 } else { raw },
            switch_no: (parameters >> SWITCH_SHIFT) as u8,
            ticks: 0,
            ropes: [0; HIMO_COUNT],
            rope_speed: 0.0,
            tilt: 0,
            yaw: 0,
            drop: 0.0,
            sway: [0.0; 2],
            drift_amp: [0.0; 2],
            drift: Vec3::ZERO,
            sway_angle: [0; 3],
            // `m374` (`d_a_kokiie.h:76`) is never written by `Create`, so it is the ctor's `0.0f` on
            // the tick `:439` reads it.
            floor: 0.0,
            home: Vec3::ZERO,
            entered: false,
        }
    }

    /// The scaled CUSTOM cull box of `:433-434`.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let scale = self.xz_scale();
        (
            [scale * -CULL_HALF_XZ, CULL_MIN_Y, scale * -CULL_HALF_XZ],
            [scale * CULL_HALF_XZ, CULL_MAX_Y, scale * CULL_HALF_XZ],
        )
    }

    /// The `switch` on `m2A0` at `:408-428` as a single x/z scale. `m2A0` is a `u8`, so only indices
    /// 0..3 of [`SCALE_XZ`] are selected; RULE 10 requires the runtime index to be guarded.
    pub fn xz_scale(&self) -> f32 {
        SCALE_XZ[usize::from(self.kind) % SCALE_COUNT]
    }

    /// The create-time work `Create` does before its two trailing `daKokiie_Execute` calls: the
    /// scale stamp (`:408-430`), the already-fallen placement (`:438-448`) and those two calls
    /// (`:450-452`).
    ///
    /// The two executes are not cosmetic: `Create` runs `kokiie_move` twice before returning, so a
    /// fresh actor has already ticked twice by the time the scheduler reaches it. Transcribing only
    /// one would leave `m374` at `0.0f` and `current.pos.y` at [`FALLEN_SPAWN_LIFT`] instead of the
    /// resting [`FLOOR`] that the two ticks produce.
    pub fn create_init(
        &mut self,
        actor: &mut Actor,
        switches: &Switches,
        collect: &Collect,
        equipped: &Equipped,
    ) {
        // `a_this->scale.{x,y,z}` from the `m2A0` switch (`:408-430`); `scale.y` is 1.0 in every arm.
        let xz = self.xz_scale();
        actor.scale = Vec3::new(xz, SCALE_Y, xz);
        // ```
        // a_this->health = 1;
        // if ((m2A2 != 0xff) && fopAcM_isSwitch(a_this, m2A2)) {
        //     a_this->current.pos.y = m374 + 50.0f + 720.0f + REG0_F(17);
        //     i_this->m29A = 1;
        //     a_this->health = 0;
        // }
        // ```
        // (:437-448) - `m374` is still `0.0f` here. `a_this->health` is dropped (see the struct
        // note); the `m29A = 1` write is what puts the actor straight into the falling state.
        self.home = actor.pos;
        if self.switch_no != SWITCH_NONE && switches.test(self.switch_no) {
            actor.pos.y = self.floor + FALLEN_SPAWN_LIFT;
            self.state = STATE_FALL;
        }
        self.entered = true;
        // `for (s32 i = 0; i < 2; i++) { daKokiie_Execute(i_this); }` (:450-452). `demo_camera`
        // inside each call is omitted in full (see the report), so this is two `kokiie_move` steps.
        self.move_house(actor, has_boomerang(equipped, collect));
        self.move_house(actor, has_boomerang(equipped, collect));
    }

    /// `kokiie_move` (`d_a_kokiie.cpp:44-163`).
    fn move_house(&mut self, actor: &mut Actor, boomerang: bool) {
        // `i_this->m298++;` (:66) - an `s16`, so it wraps rather than saturating.
        self.ticks = self.ticks.wrapping_add(1);
        match self.state {
            STATE_HANG => self.hang(actor, boomerang),
            _ => self.fall(actor),
        }
    }

    /// `kokiie_move`'s `case 0:` (`:69-122`) - the hanging house.
    fn hang(&mut self, actor: &mut Actor, boomerang: bool) {
        // ```
        // uVar6 = 0;
        // for (i = 0; i < 5; i++) {
        //     if (m32C[i] == 2) { m32C[i] = 0; m338 = 0.0f; }
        //     if (m32C[i] != 0) {
        //         if (m32C[i] == 1) { m338 = 1.0f; m32C[i] = 5; }
        //         uVar6 |= himo_off_check[i];
        //     }
        // }
        // ```
        // (:70-84). The reset at `:73-74` clears the slot AND zeroes the shared speed, so a rope
        // snapping mid-tick drops every other rope's max step to zero for that tick as well. The
        // test at `:77` sees the ALREADY-cleared slot, which is why the `== 0` case falls through
        // to the bit test only when the slot is genuinely non-zero.
        let mut mask = 0u32;
        for slot in 0..HIMO_COUNT {
            if self.ropes[slot] == 2 {
                self.ropes[slot] = 0;
                self.rope_speed = 0.0;
            }
            if self.ropes[slot] != 0 {
                if self.ropes[slot] == 1 {
                    self.rope_speed = ROPE_SPEED_TARGET;
                    self.ropes[slot] = 5;
                }
                mask |= HIMO_CHECK[slot];
            }
        }
        // RULE 10: `mask` is the OR of five bits drawn from `HIMO_CHECK`, so it cannot exceed 31,
        // but the three tables are indexed with a guarded value regardless.
        let slot = (mask as usize) % HIMO_YAW.len();
        let yaw_target = HIMO_YAW[slot];
        let tilt_target = HIMO_TILT[slot];
        // ```
        // sVar1 = himo_off_ya[uVar6];
        // target = himo_off_xa[uVar6];
        // cLib_addCalc2(&m2D0, REG8_F(4) - himo_off_yp[uVar6], 0.1f, m338 * 50.0f);
        // ```
        // (:86-88) - `REG8_F(4)` folds to a proven `0.0f`, so the target is `-himo_off_yp`.
        let drop_target = -(HIMO_DROP[slot] as f32);
        add_calc2(
            &mut self.drop,
            drop_target,
            DROP_SCALE,
            self.rope_speed * DROP_MAX_SCALE,
        );
        // ```
        // if ((uVar6 == 0x1f) && dComIfGs_checkGetItem(dItemNo_BOOMERANG_e)) {
        //     m29A = 1; actor->health = 0; m378 = 1;
        // }
        // ```
        // (:89-93) - `0x1f` is all five ropes out. `dItemNo_BOOMERANG_e = 0x2D` (`d_item_data.h:52`).
        // `actor->health` and `m378` (the camera state) are omitted; the `m29A = 1` write is the
        // whole modelled effect, and it `return`s from the decomp's arm, so the sway tail is skipped
        // on the drop tick.
        if mask == ROPE_ALL && boomerang {
            self.state = STATE_FALL;
            return;
        }
        // ```
        // maxSpeed = m338 * 10000.0f;
        // cLib_addCalcAngleS2(&m334, target, 4, maxSpeed);
        // if (target != 0) { cLib_addCalcAngleS2(&m332, sVar1, 2, maxSpeed); }
        // cLib_addCalc2(&m338, 1.0f, 1.0f, REG8_F(14) + 0.001f);
        // ```
        // (:95-102). `maxSpeed` is an `f32` truncated into the `s16` parameter.
        let max_speed = (self.rope_speed * TILT_MAX_SCALE) as i32 as i16;
        self.tilt = add_calc_angle_s2(self.tilt, tilt_target, TILT_STEP, max_speed);
        if tilt_target != 0 {
            self.yaw = add_calc_angle_s2(self.yaw, yaw_target, YAW_STEP, max_speed);
        }
        add_calc2(
            &mut self.rope_speed,
            ROPE_SPEED_TARGET,
            ROPE_SPEED_SCALE,
            ROPE_SPEED_MAX,
        );
        // ```
        // cLib_addCalcAngleS2(&actor->current.angle.x, 0, 10, 0x200);
        // cLib_addCalcAngleS2(&actor->current.angle.z, 0, 10, 0x200);
        // ```
        // (:103-104)
        actor.angle[0] = add_calc_angle_s2(actor.angle[0], 0, LEVEL_STEP, LEVEL_MAX);
        actor.angle[2] = add_calc_angle_s2(actor.angle[2], 0, LEVEL_STEP, LEVEL_MAX);
        // ```
        // m2C8.x = m2BC * cM_ssin(m298 * 900);
        // m2C8.z = m2C4 * cM_ssin(m298 * 700);
        // cLib_addCalc2(&m2BC, REG14_F(9) + 300.0f, 1.0f, REG14_F(3) + 20.0f);
        // cLib_addCalc2(&m2C4, REG14_F(9) + 300.0f, 1.0f, REG14_F(3) + 20.0f);
        // ```
        // (:106-110). Both sines are computed BEFORE the two reach chases, so this tick's sway uses
        // the previous reach.
        let sway_x = (self.sway[0] * ssin(tick(self.ticks, SWAY_PERIOD_X))) as i32 as i16;
        let sway_z = (self.sway[1] * ssin(tick(self.ticks, SWAY_PERIOD_Z))) as i32 as i16;
        add_calc2(&mut self.sway[0], SWAY_REACH, SWAY_REACH_SCALE, SWAY_REACH_MAX);
        add_calc2(&mut self.sway[1], SWAY_REACH, SWAY_REACH_SCALE, SWAY_REACH_MAX);
        // ```
        // m2A4.x = m2B0 * cM_ssin(m298 * 0x2ee);
        // m2A4.z = m2B8 * cM_ssin(m298 * 900);
        // cLib_addCalc0(&m2B0, 1.0f, 0.25f);
        // cLib_addCalc0(&m2B8, 1.0f, 0.25f);
        // ```
        // (:112-116)
        let drift_x = self.drift_amp[0] * ssin(tick(self.ticks, DRIFT_PERIOD_X));
        let drift_z = self.drift_amp[1] * ssin(tick(self.ticks, DRIFT_PERIOD_Z));
        add_calc0(&mut self.drift_amp[0], DRIFT_GROW_SCALE, DRIFT_GROW_MAX);
        add_calc0(&mut self.drift_amp[1], DRIFT_GROW_SCALE, DRIFT_GROW_MAX);
        // ```
        // actor->shape_angle = actor->current.angle + i_this->m2C8;
        // actor->current.pos = actor->home.pos + i_this->m2A4;
        // actor->current.pos.y += i_this->m2D0;
        // cLib_addCalc0(&i_this->m2A4.y, 0.05f, REG0_F(7) + 2.0f);
        // ```
        // (:118-121). `csXyz::operator+` (`c_sxyz.cpp:19-21`) is a componentwise `s16` add and
        // `m2C8.y` was never written, so the shape yaw is unchanged. The `m2A4.y` decay runs AFTER
        // the position has already consumed `m2A4`, so it only affects the NEXT tick - and since
        // `m2A4.y` is never assigned anywhere, it decays from `0.0f` and stays there.
        self.sway_angle = [sway_x, 0, sway_z];
        for component in 0..3 {
            actor.shape_angle[component] =
                actor.angle[component].wrapping_add(self.sway_angle[component]);
        }
        actor.pos = self.home + Vec3::new(drift_x, self.drift.y, drift_z);
        actor.pos.y += self.drop;
        add_calc0(&mut self.drift.y, DRIFT_RISE_SCALE, DRIFT_RISE_MAX);
    }

    /// `kokiie_move`'s `case 1:` (`:124-161`) - the house falling out of the tree.
    fn fall(&mut self, actor: &mut Actor) {
        // ```
        // cLib_addCalcAngleS2(&actor->shape_angle.x, 0, 10, 0x800);
        // cLib_addCalcAngleS2(&actor->shape_angle.z, 0, 10, 0x800);
        // cLib_addCalcAngleS2(&i_this->m334, 0, 4, 0x200);
        // ```
        // (:125-127)
        actor.shape_angle[0] = add_calc_angle_s2(actor.shape_angle[0], 0, LEVEL_STEP, FALL_LEVEL_MAX);
        actor.shape_angle[2] = add_calc_angle_s2(actor.shape_angle[2], 0, LEVEL_STEP, FALL_LEVEL_MAX);
        self.tilt = add_calc_angle_s2(self.tilt, 0, TILT_STEP, FALL_TILT_MAX);
        // ```
        // actor->current.pos.y += actor->speed.y;
        // actor->speed.y -= REG12_F(14) + 3.0f;
        // ```
        // (:129-130) - RULE 2b folds `REG12_F(14)`. The position write comes first, so the gravity
        // applied on this tick only bites on the NEXT one.
        actor.pos.y += actor.speed.y;
        actor.speed.y -= FALL_GRAVITY;
        // ```
        // if (m37A == (s16)(REG0_S(4) + 0x1b)) { if (m2A2 != 0xff) { onSwitch(m2A2); }
        //     seStart(JA_SE_OBJ_KOKIRI_H_CRASH, 0); StartShock(REG0_S(2) + 4, -0x21, (0,1,0)); }
        // if (m37A == (s16)(REG0_S(5) + 0x40)) { ...four dPa_name emitters... }
        // ```
        // (:132-148) - OMITTED. `m37A` is written in exactly two places, `:250` (`= 0`) and `:294`
        // (`= m37A + 1`), and BOTH are inside `demo_camera`, which is omitted in full, so it keeps
        // the ctor's `0` and neither arm can fire. RULE 2b folds `REG0_S(4)` and `REG0_S(5)`
        // anyway. The consequence - no crash sound, no splash particles, and no
        // `dComIfGs_onSwitch(m2A2)` from `:134` - is stated in the report.
        //
        // ```
        // m374 = (REG0_F(9) + 5180.0f) - 3800.0f;
        // if (actor->current.pos.y <= m374) {
        //     actor->current.pos.y = m374;
        //     if (actor->speed.y < -20.0f) { StartShock(REG6_S(2) + 7, -0x21, (0,1,0));
        //         seStart(JA_SE_OBJ_KOKIRI_H_LANDING, 0); m39C = REG12_F(10) + 40.0f + 50.0f; }
        //     actor->speed.y = 0.0f;
        //     cLib_addCalcAngleS2(&m334, 0, 2, 0x2000);
        // }
        // ```
        // (:150-160) - the clamp, the rest and the settle ARE transcribed. The `speed.y < -20.0f`
        // body is omitted with `m37A` above: its three effects are two rumble calls and one sound,
        // and `m39C` feeds only `demo_camera`. Note the clamp is re-armed EVERY falling tick, so
        // the floor is written before the speed test on each of them.
        self.floor = FLOOR;
        if actor.pos.y <= self.floor {
            actor.pos.y = self.floor;
            actor.speed.y = 0.0;
            self.tilt = add_calc_angle_s2(self.tilt, 0, LAND_SETTLE_STEP, LAND_SETTLE_MAX);
        }
    }
}

/// `dComIfGs_checkGetItem(dItemNo_BOOMERANG_e)` (`d_a_kokiie.cpp:89`) through the shared inventory
/// helper, which is how `src/sim/aryll.rs` and `src/sim/npc_ho.rs` spell it.
fn has_boomerang(equipped: &Equipped, collect: &Collect) -> bool {
    inventory::check_get_item(equipped, collect, ITEM_BOOMERANG)
}

/// `m298 * period` where `m298` is an `s16` and the period an `int` (`d_a_kokiie.cpp:106` etc). The
/// product is an `int` in retail and is truncated into the `s16` that `cM_ssin` takes; RULE 10 bans
/// an `i16` multiply, so both the widen and the truncation are explicit.
fn tick(ticks: i16, period: i32) -> i16 {
    (i32::from(ticks) * period) as i16
}

/// `cM_ssin(s16 x) = JMASSin(x)` (`c_math.h:39-41`), in the `0x10000 == one full turn` units
/// `super::walk::from_s16` implements (RULE 5b).
fn ssin(angle: i16) -> f32 {
    walk::from_s16(angle).sin()
}

/// `cLib_addCalc2(f32* pValue, f32 target, f32 scale, f32 maxStep)` (`c_lib.cpp:56-66`).
fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value == target {
        return;
    }
    let step = (scale * (target - *value)).clamp(-max_step, max_step);
    *value += step;
}

/// `cLib_addCalc0(f32* pValue, f32 scale, f32 maxStep)` (`c_lib.cpp:69-77`) - unconditional,
/// including at zero, which is what makes the never-assigned `m2A4.y` decay a no-op.
fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let step = (*value * scale).clamp(-max_step, max_step);
    *value -= step;
}

/// `cLib_addCalcAngleS2(s16* pValue, s16 target, s16 scale, s16 maxStep)` (`c_lib.cpp:191-202`).
/// Every `scale` literal at the call sites is non-zero, so neither the divide nor an
/// `i16::MIN / -1` overflow can arise.
fn add_calc_angle_s2(value: i16, target: i16, scale: i16, max_step: i16) -> i16 {
    let step = target.wrapping_sub(value) / scale;
    if step > max_step {
        value.wrapping_add(max_step)
    } else if step < -max_step {
        value.wrapping_sub(max_step)
    } else {
        value.wrapping_add(step)
    }
}

pub fn is_kokiie(name: &str) -> bool {
    KOKIIE_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    collect: Res<Collect>,
    equipped: Res<Equipped>,
    mut houses: Query<(&mut Actor, &mut Kokiie), With<Kokiie>>,
) {
    for (mut actor, mut house) in &mut houses {
        if !house.entered {
            house.create_init(&mut *actor, &switches, &collect, &equipped);
        }
        house.move_house(&mut *actor, has_boomerang(&equipped, &collect));
    }
}
