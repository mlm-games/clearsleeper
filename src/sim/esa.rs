use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::event::{ActiveEvent, EventList};
use super::ground;
use super::sea::Sea;
use super::walk;

/// The stage string is `Esa` (`d_stage.cpp:1240`, `fpcNm_ESA_e`). It is NOT the archive name -
/// `daEsa_CreateHeap` loads the model from `"Link"`, the player's own archive
/// (`d_a_esa.cpp:230`) - but the OBJNAME row is the stage name, so this array carries `Esa`.
pub const ESA_ACTORS: [&str; 1] = ["Esa"];

/// `i_this->mActionState` (`d_a_esa.h:26`, an `s8`) - the `switch (i_this->mActionState)` of
/// `esa_1_move` (`d_a_esa.cpp:93`). The five case labels of that switch are the only place the
/// values appear; note `case 0` FALLS THROUGH into `case 1` (there is no `break` at `:104`), so
/// the launch impulse and the first integration happen on the same tick.
const ACT_LAUNCH: i8 = 0; // <- d_a_esa.cpp:94
const ACT_BOUNCE: i8 = 1; // <- d_a_esa.cpp:105
const ACT_SPENT: i8 = 2; // <- d_a_esa.cpp:150
const ACT_FLOAT: i8 = 3; // <- d_a_esa.cpp:156
const ACT_SINK: i8 = 4; // <- d_a_esa.cpp:183

/// `i_this->mState` (`d_a_esa.h:27`, an `s8`), written only by `bg_check`: `s8 state = 1;`
/// (`d_a_esa.cpp:37`) is the default, `state = 2` on sea (`:42`) or room water (`:54`), and the
/// field itself takes `state` (`:59`) or `0` (`:62`).
const STATE_AIR: i8 = 0; // <- d_a_esa.cpp:62
const STATE_GROUND: i8 = 1; // <- d_a_esa.cpp:37
const STATE_WATER: i8 = 2; // <- d_a_esa.cpp:42, :54

/// `i_this->mTimer[0] = cM_rndF(50.0f) + 200.0f` (`d_a_esa.cpp:108` and `:136`) - the two ends of
/// `cM_rndF`'s range, into the `s16 mTimer[2]` (`d_a_esa.h:38`). This is `cM_rndF(range) + low`,
/// NOT `cLib_getRndValue(min, range)` - see the note on `rnd` below.
const FLOAT_TIMER_LOW: f32 = 200.0; // <- d_a_esa.cpp:108
const FLOAT_TIMER_RANGE: f32 = 50.0; // <- d_a_esa.cpp:108

/// `i_this->mTimer[0] = 10000` (`d_a_esa.cpp:158`), guarded by
/// `dComIfGp_evmng_startCheck("SO_ESA_XY")` (`:157`). `mTimer` is an `s16` (`d_a_esa.h:38`), so
/// 10000 fits and ticks down one per `Execute` (`:194-198`).
const EVENT_TIMER: i16 = 10000; // <- d_a_esa.cpp:158

/// `dComIfGp_evmng_startCheck("SO_ESA_XY")` (`d_a_esa.cpp:157`). The two-argument
/// `dComIfGp_evmng_startCheck` defaults `evNo` to `-1` (`d_com_inf_game.h:3611`), i.e. `0xFF`, so
/// this is a pure name lookup, exactly like the `getEventIdx` it pairs with.
const ESA_XY_EVENT: &str = "SO_ESA_XY"; // <- d_a_esa.cpp:157

/// `i_this->mTimer[0] = 10; i_this->mActionState = 4;` (`d_a_esa.cpp:165-166`).
const SINK_TIMER: i16 = 10; // <- d_a_esa.cpp:165

/// `temp.y += 100.0f` before `gndChk.SetPos(&temp)` (`d_a_esa.cpp:33-34`) and the same lift on the
/// `dBgS_GetWaterHeight` probe `cXyz sp54` (`:49-51`).
const GROUND_PROBE_LIFT: f32 = 100.0; // <- d_a_esa.cpp:33

/// `i_this->mGroundHeight = dComIfG_Bgsp()->GroundCross(&gndChk) + 5.0f;` (`d_a_esa.cpp:36`).
const GROUND_LIFT: f32 = 5.0; // <- d_a_esa.cpp:36

/// `f32 daSea_packet_c::BASE_HEIGHT = 1.0f` (`d_a_sea.cpp:23`) - what
/// `daSea_calcWave` returns when `!daSea_ChkArea(x, z)` (`d_a_sea.cpp:350-352`), and the same
/// `1.0` the sim's own `super::sea` uses (`src/sim/sea.rs:12`).
const SEA_BASE_HEIGHT: f32 = 1.0; // <- d_a_sea.cpp:23

/// `i_this->speed.y -= 3.0f;` (`d_a_esa.cpp:146`) - the only gravity this actor applies; it never
/// calls `fopAcM_SetGravity` or `fopAcM_posMoveF`.
const GRAVITY: f32 = 3.0; // <- d_a_esa.cpp:146

/// `sp24.y = cM_rndF(8.0f) + 15.0f` (`d_a_esa.cpp:98`) and
/// `sp24.z = cM_rndF(5.0f) + 10.0f` (`:99`) in `case 0`.
const LAUNCH_LIFT_LOW: f32 = 15.0; // <- d_a_esa.cpp:98
const LAUNCH_LIFT_RANGE: f32 = 8.0; // <- d_a_esa.cpp:98
const LAUNCH_FWD_LOW: f32 = 10.0; // <- d_a_esa.cpp:99
const LAUNCH_FWD_RANGE: f32 = 5.0; // <- d_a_esa.cpp:99

/// `if (i_this->speedF < 0.1f)` (`d_a_esa.cpp:134`) - the `case 1` stall threshold.
const STALL_SPEED: f32 = 0.1; // <- d_a_esa.cpp:134

/// `if (i_this->speed.y < 5.0f) { i_this->speed.y *= -(cM_rndF(0.05f) + 0.15f);
/// i_this->current.angle.z = cM_rndFX(0x8000); }` (`d_a_esa.cpp:119-122`) - the bounce test and the
/// restitution range. The multiplier is NEGATED at `:120`.
const BOUNCE_TEST: f32 = 5.0; // <- d_a_esa.cpp:119
const BOUNCE_GAIN_LOW: f32 = 0.15; // <- d_a_esa.cpp:120
const BOUNCE_GAIN_RANGE: f32 = 0.05; // <- d_a_esa.cpp:120

/// `i_this->current.angle.y += (s16)cM_rndFX(4000.0f)` (`d_a_esa.cpp:95`) and
/// `i_this->current.angle.y += (s16)cM_rndFX(8000.0f)` (`:124`) - the launch and the bounce spin.
const SPIN_LAUNCH: f32 = 4000.0; // <- d_a_esa.cpp:95
const SPIN_BOUNCE: f32 = 8000.0; // <- d_a_esa.cpp:124

/// `i_this->current.angle.z = cM_rndFX(0x8000)` (`d_a_esa.cpp:96` and `:121`) - the two pitch draws.
/// `0x8000` is half a turn and the assignment truncates the `f32` into the `s16 angle.z`.
const PITCH_SPIN: f32 = 0x8000.0; // <- d_a_esa.cpp:96

/// `i_this->speedF *= cM_rndF(0.3f) + 0.3f;` (`d_a_esa.cpp:125`) - the `case 1` roll decay.
const ROLL_LOW: f32 = 0.3; // <- d_a_esa.cpp:125
const ROLL_RANGE: f32 = 0.3; // <- d_a_esa.cpp:125

/// `cXyz sp48 = i_this->old.pos + (i_this->current.pos - i_this->old.pos) * 1.5f;`
/// (`d_a_esa.cpp:66`) - the swept line check's far end.
const LINE_SCALE: f32 = 1.5; // <- d_a_esa.cpp:66

/// `cXyz temp5 = i_this->old.pos - sp48; if (temp5.abs() > 1.0f)` (`d_a_esa.cpp:68-69`).
/// `cXyz::abs()` is the magnitude, `sqrtf(abs2())` (`c_xyz.h:124`).
const LINE_MIN: f32 = 1.0; // <- d_a_esa.cpp:69

/// `i_this->speedF *= 0.5f;` (`d_a_esa.cpp:74`) and
/// `i_this->current.angle.y -= 0x8000;` (`:75`) - the wall-bounce response. `0x10000` is one full
/// turn (RULE 5b), so the negative form is the house spelling of `- 0x8000`.
const WALL_SPEED_KEEP: f32 = 0.5; // <- d_a_esa.cpp:74
const WALL_TURN: i16 = -0x8000; // <- d_a_esa.cpp:75

/// `fopAc_CULLBOX_0_e` (`d_a_esa.cpp:307`) resolves through `l_cullSizeBox[0]`
/// (`f_op_actor_mng.cpp:558`) - a real box, so `cull()` is derivable. `daEsa_Create` never calls
/// `fopAcM_setCullSizeBox`.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// `cM_rndF`'s three lanes, copied from `cM_rnd` (`c_math.cpp:175-183`): the multipliers
/// (`:176-178`), the moduli (`:176-178`) and the divisors (`:176-178`), seeded by
/// `cM_randInit(0)` (`:160-164`).
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA]; // <- c_math.cpp:176
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673]; // <- c_math.cpp:176
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0]; // <- c_math.cpp:176
const RND_SEED: i32 = 100; // <- c_math.cpp:160 (cM_randInit(0))

/// The all-purpose bait - `esa_class` (`d_a_esa.h:10-40`), the actor behind the stage string `Esa`.
///
/// This is NOT an enemy, despite "Item". (a) inbound damage: NONE. (b) outbound damage: NONE.
/// `esa_class` derives from a bare `fopAc_ac_c` and registers no `dCcD` collider of any kind - it
/// holds only the two timers, a mode byte, a state byte, a ripple emitter callback and two
/// parameter bytes (`d_a_esa.h:25-38`) - and the profile `Group` is `fopAc_ACTOR_e`
/// (`d_a_esa.cpp:306`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached: attaching them
/// would let `combat::control` despawn bait retail cannot damage.
///
/// `daEsa_IsDelete` (`:214-217`) is the bare "accepts a delete request" form of RULE 6b. The real
/// destruction is the actor's own: `fopAcM_delete(i_this)` in `case 2` (`:152`) and `case 4`
/// (`:185`), both gated on `mTimer[0] == 0`.
///
/// There is no `old_pos` field even though `bg_check` reads `i_this->old.pos` (`:66-73`):
/// `fopAc_Execute` does `actor->old = actor->current;` immediately before dispatching the leaf
/// `Execute` (`f_op_actor.cpp:196`), and `fopAc_Init` seeds `actor->old = actor->home` (`:283`),
/// so `old.pos` is exactly the position at the start of the current tick - which `execute` already
/// has as a local.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Esa {
    /// `mActionState` (`d_a_esa.h:26`), the `switch` index of `esa_1_move` (`:93`).
    pub action: i8,
    /// `mState` (`d_a_esa.h:27`), written only by `bg_check` (`:59`, `:62`) and read only by
    /// `esa_1_move`'s `case 1` (`:106-107`).
    pub state: i8,
    /// `fopAc_ac_c::speed` - the `cXyz` that `esa_1_move` integrates into `current.pos`
    /// (`:143-145`) and that `bg_check` tests at `:57`, scales at `:120` and rewrites at
    /// `:81-82` / `:131-133`.
    pub speed: Vec3,
    /// `fopAc_ac_c::speedF` - the scalar whose horizontal projection becomes `speed.x` / `speed.z`
    /// (`:74-82`, `:100`, `:128-133`).
    pub speed_f: f32,
    /// `mTimer[0]` (`d_a_esa.h:38`). Only index 0 is carried: `mTimer[1]` is drained by
    /// `daEsa_Execute`'s `for (int i = 0; i < 2; i++)` (`:194-198`) but has no writer and no reader
    /// anywhere in the decomp, so carrying it would be a dead field.
    pub timer: i16,
    /// The `dComIfGp_evmng_startCheck("SO_ESA_XY")` slot (`:157`); `None` is that function's `-1`.
    pub event_slot: Option<usize>,
    /// `cM_randInit(0)`'s three-lane `cM_rnd` state, so the per-actor draws at `:95-99`, `:108`,
    /// `:120-125` and `:136` are reproducible in the house style (`boko.rs`, `bwds.rs`).
    pub seed: [i32; 3],
    /// One-shot guard so the `SO_ESA_XY` lookup runs once; the slot cannot change after `Create`.
    pub entered: bool,
}

impl Esa {
    /// `daEsa_Create` (`d_a_esa.cpp:242-283`), minus `daPy_getPlayerActorClass()` (`:243`),
    /// `fopAcM_ct` (`:245`), the child-spawn block (`:259-273`, whose
    /// `player->getLeftHandPos()` is a joint-matrix value - RULE 3b),
    /// `fopAcM_entrySolidHeap` / `daEsa_CreateHeap` (`:255`, `:227-239`), the `REG0_*` scale draw
    /// (`:275-279`), `fopAcM_SetMtx` (`:280`) and the two parameter bytes (`:249-253`), whose only
    /// readers are inside the omitted child-spawn block. `dComIfG_resLoad` is never called - the
    /// model comes from `"Link"`, the player's own archive (`:230`).
    ///
    /// RULE 2b folds both register reads away: `g_regHIO.mChild[0].mShortRegs[0]` and
    /// `mFloatRegs[5]` / `mFloatRegs[6]` are zero-initialised at `d_s_play.cpp:81-84`, so
    /// `if (REG0_S(0) != 0)` (`:251`) is a proven false and `scaleF` is
    /// `0.65 + cM_rndF(1.0 - 0.65)` - and it only feeds `mpModel->setBaseScale` (`:279`).
    ///
    /// `fopAc_Init` seeds `current` and `old` from `home` (`f_op_actor.cpp:282-284`), so the bait
    /// starts at its placement position, and RULE 2 applies: `dStage_actorInit` uses
    /// `fopAcM_CreateAppend`, whose default `scale` is `(10, 10, 10)` (`:84-86`), divided by ten at
    /// `f_op_actor.cpp:280`, i.e. exactly 1.0.
    pub fn spawn(_parameters: u32) -> Self {
        // `i_this->field_0x2B9 = fopAcM_GetParam(i_this) & 0xFF` (:249) and
        // `i_this->field_0x2BA = fopAcM_GetParam(i_this) >> 8 & 0xFF` (:250) are transcribed in the
        // doc comment and deliberately NOT decoded into fields: their only reader is the omitted
        // child-spawn loop (`:259`, `:261-262`), and an unread field is dead code (RULE 7 /
        // RULE 12b). The parameter is the whole u32 (`fopAcM_SetParam(actor, prm->base.parameters)`,
        // `f_op_actor.cpp:271`), so the shifts and masks were copied exactly before being dropped.
        Self {
            // `mActionState` has no initialiser in `daEsa_Create`; the value is the
            // `sBs_ClearArea` zero-fill (`f_pc_base.cpp:98`), which is `ACT_LAUNCH`.
            action: ACT_LAUNCH,
            state: STATE_AIR,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            // `mTimer[0]` is likewise the zero-fill, so `case 0` -> `case 1` sees 0.
            timer: 0,
            event_slot: None,
            seed: [RND_SEED; 3],
            entered: false,
        }
    }

    /// `fopAc_CULLBOX_0_e` (`d_a_esa.cpp:307`) -> `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `cM_rnd()` (`c_math.cpp:175-183`) folded into `cM_rndF(f) = cM_rnd() * f` (`:185-187`),
    /// which is a value in `[0, f)`.
    ///
    /// NOTE the convention: `d_a_esa.cpp` writes every random draw as `cM_rndF(range) + low`
    /// (`:98-99`, `:108`, `:120`, `:125`, `:136`), which is the same arithmetic as
    /// `cLib_getRndValue(low, range) = low + cM_rndF(range)` (`c_lib.h:79-81`). No line of
    /// `d_a_esa.cpp` calls `cLib_getRndValue`, and none of these draws is a constant - every one
    /// has `cM_rndF` with a non-zero range, including `getRndValue(x, x)`-shaped pairs.
    fn rnd(&mut self, range: f32) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = (self.seed[0]) as f32 / RND_SCALE[0]
            + (self.seed[1]) as f32 / RND_SCALE[1]
            + (self.seed[2]) as f32 / RND_SCALE[2];
        range * (sum % 1.0).abs()
    }

    /// `cM_rndFX(f) = f * (cM_rnd() - 0.5f) * 2.0f` (`c_math.cpp:190-192`) - a value in `(-f, f)`.
    /// This is the OTHER convention: not `low + cM_rndF(range)`.
    fn rnd_fx(&mut self, scale: f32) -> f32 {
        scale * (self.rnd(1.0) - 0.5) * 2.0
    }

    /// `cMtx_YrotS(*calc_mtx, i_this->current.angle.y); MtxPosition(&src, &dst);`
    /// (`d_a_esa.cpp:101-102`, and the identical pairs at `:79-80` and `:130-132`). A Y rotation
    /// gives `x' = x * cos + z * sin`, `z' = z * cos - x * sin`, and every one of the three
    /// sources is built with `x` literally `0.0f` (`:96`, `:76-78`, `:126-128`) - which is exactly
    /// how `bg_check` reads the result back at `:81-82` (`speed.x = sp60.x; speed.z = sp60.z`).
    /// `from_s16` is the single angle conversion path (RULE 5b).
    fn yrot(y: f32, z: f32, yaw: i16) -> Vec3 {
        let (s, c) = walk::from_s16(yaw).sin_cos();
        Vec3::new(z * s, y, z * c)
    }

    /// `bg_check` (`d_a_esa.cpp:27-88`), with `old` standing in for `i_this->old.pos`, which
    /// `fopAc_Execute` refreshed to `current.pos` immediately before this tick
    /// (`f_op_actor.cpp:196`).
    ///
    /// * `dComIfG_Bgsp()->GroundCross(&gndChk)` at `pos.y + 100` (`:33-36`) is
    ///   `super::ground::surface`, the sim's reduction of `GroundCross`, plus `GROUND_LIFT`. A
    ///   `None` there - a collision resource with no plane under (x, z) - is read as a flat 0.0
    ///   floor, the same default `ground::ground_check` uses (`src/sim/ground.rs:12`).
    /// * `daSea_ChkArea(x, z)` (`:38`) is `Sea::draw_contains` and `daSea_calcWave(x, z)` (`:39`)
    ///   is `Sea::height_at` - the same pair `ritem.rs:320-321` uses.
    /// * `dBgS_GetWaterHeight(sp54)` (`:51-52`) is the per-position ROOM water height. The sim's
    ///   `super::water::Water` is a single player probe written by `water::water_probe`
    ///   (`src/sim/water.rs`), not a room query, so this arm is omitted. Retail's own guard is
    ///   `waterHeight != -G_CM3D_F_INF && mGroundHeight <= waterHeight` (`:52-53`), i.e. an
    ///   explicit "no water here" sentinel; see the report.
    /// * `if (i_this->speed.y <= 0.0f && i_this->current.pos.y <= i_this->mGroundHeight)` (`:57`)
    ///   snaps to the surface and latches `mState` to the local `state`, else `0`.
    /// * The `dComIfG_Bgsp()->LineCross(&linChk)` wall bounce (`:65-84`) uses the same
    ///   `Collision::slide(from, to) != to` reduction `kamome.rs:322-327` uses for the identical
    ///   call. `linChk.Set(&old.pos, &sp48, this)` (`:70`) excludes the bait itself, which
    ///   `Collision::slide` cannot express - the house reduction.
    /// * `cLib_offBit<u32>(i_this->actor_status, fopAcStts_UNK4000_e)` (`:86`) in the `else` is a
    ///   per-actor status bit, which RULE 3b lists as absent.
    fn bg_check(&mut self, actor: &mut Actor, old: Vec3, ctx: &Context<'_>) {
        let probe = actor.pos + Vec3::new(0.0, GROUND_PROBE_LIFT, 0.0);
        let mut height = ground::surface(ctx.collision, probe).map_or(0.0, |y| y + GROUND_LIFT);
        // `s8 state = 1;` (:37)
        let mut state = STATE_GROUND;
        if ctx.sea.is_some_and(|sea| sea.draw_contains(actor.pos.x, actor.pos.z)) {
            let wave = ctx.sea.map_or(0.0, |sea| sea.height_at(actor.pos.x, actor.pos.z));
            if height <= wave {
                height = wave;
                state = STATE_WATER;
            }
        }
        // `if (i_this->speed.y <= 0.0f && i_this->current.pos.y <= i_this->mGroundHeight)` (:57)
        if self.speed.y <= 0.0 && actor.pos.y <= height {
            actor.pos.y = height;
            self.state = state;
        } else {
            self.state = STATE_AIR;
        }

        // `cXyz sp48 = old.pos + (current.pos - old.pos) * 1.5f; sp48.y = old.pos.y;` (:66-67)
        let grown = old + (actor.pos - old) * LINE_SCALE;
        let far = Vec3::new(grown.x, old.y, grown.z);
        // `cXyz temp5 = old.pos - sp48; if (temp5.abs() > 1.0f)` (:68-69). After the y flattening
        // `temp5` is `(0, 0, -0.5 * horizontal travel)`, so its magnitude is half the travel.
        if (old - far).length() > LINE_MIN && ctx.line_crossed(old, far) {
            actor.pos.x = old.x;
            actor.pos.z = old.z;
            self.speed_f *= WALL_SPEED_KEEP;
            actor.angle[1] = actor.angle[1].wrapping_add(WALL_TURN);
            // `cMtx_YrotS(*calc_mtx, current.angle.y); MtxPosition(&sp6C, &sp60);
            // speed.x = sp60.x; speed.z = sp60.z;` (:79-82) with `sp6C = (0, 0, speedF)`.
            let turned = Self::yrot(0.0, self.speed_f, actor.angle[1]);
            self.speed.x = turned.x;
            self.speed.z = turned.z;
        }
    }

    /// `esa_1_move` `case 0` (`d_a_esa.cpp:94-104`) - the launch, which FALLS THROUGH into
    /// `case 1` because there is no `break` at `:104`.
    fn act_launch(&mut self, actor: &mut Actor) {
        actor.angle[1] = actor.angle[1].wrapping_add(self.rnd_fx(SPIN_LAUNCH) as i16);
        actor.angle[2] = self.rnd_fx(PITCH_SPIN) as i16;
        let lift = self.rnd(LAUNCH_LIFT_RANGE) + LAUNCH_LIFT_LOW;
        let ahead = self.rnd(LAUNCH_FWD_RANGE) + LAUNCH_FWD_LOW;
        self.speed_f = ahead;
        self.speed = Self::yrot(lift, ahead, actor.angle[1]);
        self.action = ACT_BOUNCE;
    }

    /// `esa_1_move` `case 1` (`d_a_esa.cpp:105-149`) - the bounce / roll state. The `case 0` label
    /// falls into it, so this also runs on the launch tick.
    ///
    /// The `break` at `:138` leaves `case 1` early, skipping the integration at `:143-145` and the
    /// `bg_check` at `:147`; that is modelled by returning before them.
    fn act_bounce(&mut self, actor: &mut Actor, old: Vec3, ctx: &Context<'_>) {
        if self.state != STATE_AIR {
            if self.state == STATE_WATER {
                // `mTimer[0] = cM_rndF(50.0f) + 200.0f; mActionState = 3;` (:108-109)
                self.timer = (self.rnd(FLOAT_TIMER_RANGE) + FLOAT_TIMER_LOW) as i16;
                self.action = ACT_FLOAT;
            } else {
                // `if (i_this->speed.y < 5.0f)` (:119)
                if self.speed.y < BOUNCE_TEST {
                    // `speed.y *= -(cM_rndF(0.05f) + 0.15f); angle.z = cM_rndFX(0x8000);` (:120-121)
                    let gain = self.rnd(BOUNCE_GAIN_RANGE) + BOUNCE_GAIN_LOW;
                    self.speed.y *= -gain;
                    actor.angle[2] = self.rnd_fx(PITCH_SPIN) as i16;
                }
                // `angle.y += (s16)cM_rndFX(8000.0f); speedF *= cM_rndF(0.3f) + 0.3f;` (:124-125)
                actor.angle[1] = actor.angle[1].wrapping_add(self.rnd_fx(SPIN_BOUNCE) as i16);
                let roll = self.rnd(ROLL_RANGE) + ROLL_LOW;
                self.speed_f *= roll;
                // `sp24 = (0, 0, speedF); cMtx_YrotS(...); MtxPosition(&sp24, &sp18);
                // speed.x = sp18.x; speed.z = sp18.z;` (:126-133)
                let ahead = Self::yrot(0.0, self.speed_f, actor.angle[1]);
                self.speed.x = ahead.x;
                self.speed.z = ahead.z;
                if self.speed_f < STALL_SPEED {
                    // `mActionState = 2; mTimer[0] = cM_rndF(50.0f) + 200.0f; break;` (:135-138)
                    self.action = ACT_SPENT;
                    self.timer = (self.rnd(FLOAT_TIMER_RANGE) + FLOAT_TIMER_LOW) as i16;
                    return;
                }
            }
        }
        actor.pos.x += self.speed.x;
        actor.pos.y += self.speed.y;
        actor.pos.z += self.speed.z;
        self.speed.y -= GRAVITY;
        self.bg_check(actor, old, ctx);
    }

    /// `esa_1_move` `case 2` (`d_a_esa.cpp:150-155`):
    /// `if (i_this->mTimer[0] == 0) { fopAcM_delete(i_this); }`. True means the actor asked to be
    /// deleted.
    fn act_spent(&self) -> bool {
        self.timer == 0
    }

    /// `esa_1_move` `case 3` (`d_a_esa.cpp:156-182`) - the float.
    ///
    /// `if (dComIfGp_evmng_startCheck("SO_ESA_XY")) { i_this->mTimer[0] = 10000; }` (`:157-159`)
    /// re-arms the sink timer whenever the event starts. `field_0x2A4.remove()` (`:161`) is the
    /// ripple emitter (particles) and `i_this->current.pos.y = daSea_calcWave(x, z)` (`:162`)
    /// rides the wave - `daSea_calcWave` returns `BASE_HEIGHT` outside the sea area
    /// (`d_a_sea.cpp:350-352`), which is what the `map_or` supplies when there is no sea chart.
    ///
    /// The `if (i_this->field_0x298)` arm (`:169-180`) is DEAD and is not transcribed:
    /// `field_0x298` is the `u8` at `d_a_esa.h:29`, has no initialiser in `daEsa_Create`, so it
    /// takes the `sBs_ClearArea` zero-fill (`f_pc_base.cpp:98`), and nothing anywhere in the
    /// decomp writes it. The test is therefore always false and `:169-180` is unreachable.
    fn act_float(&mut self, actor: &mut Actor, ctx: &Context<'_>) {
        if ctx.event_started(self.event_slot) {
            self.timer = EVENT_TIMER;
        }
        actor.pos.y = ctx.sea.map_or(SEA_BASE_HEIGHT, |sea| {
            sea.height_at(actor.pos.x, actor.pos.z)
        });
        if self.timer == 0 {
            self.timer = SINK_TIMER;
            self.action = ACT_SINK;
        }
    }

    /// `esa_1_move` `case 4` (`d_a_esa.cpp:183-188`):
    /// `if (i_this->mTimer[0] == 0) { fopAcM_delete(i_this); }`.
    fn act_sink(&self) -> bool {
        self.timer == 0
    }

    /// `esa_1_move` (`d_a_esa.cpp:91-190`). Returns true when the actor asked to be deleted.
    fn esa_1_move(&mut self, actor: &mut Actor, old: Vec3, ctx: &Context<'_>) -> bool {
        match self.action {
            ACT_LAUNCH => {
                self.act_launch(actor);
                self.act_bounce(actor, old, ctx);
                false
            }
            ACT_BOUNCE => {
                self.act_bounce(actor, old, ctx);
                false
            }
            ACT_SPENT => self.act_spent(),
            ACT_FLOAT => {
                self.act_float(actor, ctx);
                false
            }
            ACT_SINK => self.act_sink(),
            _ => false,
        }
    }

    /// `daEsa_Execute` (`d_a_esa.cpp:193-211`) minus the renderer tail - `MtxTrans` /
    /// `cMtx_YrotM` / `cMtx_XrotM` / `cMtx_ZrotM` / `mpModel->setBaseTRMtx` (`:202-206`) and
    /// `g_env_light.settingTevStruct` (`:208`) are model and tev-strategy work. What survives is
    /// the timer drain (`:194-198`), the `SO_ESA_XY` lookup (`:157`) and `esa_1_move` (`:200`).
    fn execute(&mut self, actor: &mut Actor, ctx: &Context<'_>) -> bool {
        if !self.entered {
            self.event_slot = ctx.list.event_index(ESA_XY_EVENT);
            self.entered = true;
        }
        // `for (int i = 0; i < 2; i++) { if (i_this->mTimer[i] != 0) i_this->mTimer[i]--; }`
        // (:194-198), restricted to index 0.
        if self.timer != 0 {
            self.timer -= 1;
        }
        // `fopAc_Execute` already ran `actor->old = actor->current;` (`f_op_actor.cpp:196`), so the
        // pre-tick position is what is in `actor.pos` right now.
        let old = actor.pos;
        self.esa_1_move(actor, old, ctx)
    }
}

/// Everything `_execute` needs beyond the component, in one bundle so the state machine has a
/// single reader - the `tagetc` / `dekuitem` shape.
struct Context<'a> {
    collision: Option<&'a Collision>,
    sea: Option<&'a Sea>,
    list: &'a EventList,
    active: &'a ActiveEvent,
}

impl Context<'_> {
    /// `dComIfG_Bgsp()->LineCross(&linChk)` (`d_a_esa.cpp:71`), reduced exactly as `kamome.rs`
    /// does it.
    fn line_crossed(&self, from: Vec3, to: Vec3) -> bool {
        match self.collision {
            Some(collision) => collision.slide(from, to) != to,
            None => false,
        }
    }

    /// `dComIfGp_evmng_startCheck("SO_ESA_XY")` (`d_a_esa.cpp:157`) = `ActiveEvent::start_check`.
    fn event_started(&self, slot: Option<usize>) -> bool {
        slot.is_some_and(|slot| self.active.start_check(slot))
    }
}

pub fn is_esa(name: &str) -> bool {
    ESA_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    active: Res<ActiveEvent>,
    collision: Option<Res<Collision>>,
    sea: Option<Res<Sea>>,
    mut baits: Query<(Entity, &mut Actor, &mut Esa)>,
    mut commands: Commands,
) {
    let mut gone: Vec<Entity> = Vec::new();
    for (entity, mut actor, mut bait) in &mut baits {
        let actor = &mut *actor;
        let bait = &mut *bait;
        let ctx = Context {
            collision: collision.as_deref(),
            sea: sea.as_deref(),
            list: &list,
            active: &active,
        };
        if bait.execute(actor, &ctx) {
            gone.push(entity);
        }
    }
    for entity in gone {
        let _ = commands.entity(entity).despawn();
    }
}