//! `daGrid_c` (`d_a_grid.h:50-87`) - the King's Sail cloth, behind the stage string `Grid`.
//!
//! `NONMATCHING` count: 0 in the `/* Nonmatching */` sense that `gate.py` counts - there is no
//! un-decompiled function body in the file. The one marker present is
//! `// NONMATCHING - some small reg alloc` on `ho_move` (`:358`), which is a register-allocation
//! difference, not a missing body: every table, constant and branch in `ho_move` below is real.
//! So this actor is fully portable and is transcribed rather than declined.
//!
//! `OBJNAME("Grid", fpcNm_GRID_e, 255, 0)` (`d_stage.cpp:1251`) is the only row for
//! `g_profile_GRID` (`d_a_grid.cpp:853-868`). Nothing else in `src/sim/` claims `Grid`. The two
//! object archives are `"Cloth"` (`:657`) and `"Ship"` (`:662`), neither of which is the stage
//! string; `fpcNm_SAIL_e`'s stage string is `"Psail"` (`d_stage.cpp:1119`, `src/sim/sail.rs`) and
//! that is the OUTSET PIRATES' sail, a different actor in a different translation unit.
//!
//! CULL: CONFIRMED. The profile says `fopAc_CULLBOX_0_e` (`:867`), which is a REAL profile, not
//! CUSTOM, and it resolves through `l_cullSizeBox[0]` at `f_op_actor_mng.cpp:558` to
//! `fopAc_cullSizeBox(cXyz(-40.0f, 0.0f, -40.0f), cXyz(40.0f, 125.0f, 40.0f))`. This file calls
//! neither `fopAcM_setCullSizeBox` nor `fopAcM_SetMin` / `SetMax`, which is correct and expected -
//! a non-CUSTOM profile supplies the box itself. `cull()` therefore stays, and
//! `src/lib.rs:1524-1525` already calls it.
//!
//! WHAT IS PORTED: `_execute` (`:784-819`) minus its `ho_move` call, i.e. the whole camera-distance
//! alpha ramp. That is the actor's only per-tick behaviour whose inputs all exist here - the HIO
//! defaults are FILE-SCOPE literals (`static daHo_HIO_c l_HIO` at `:89`, a plain static, not a
//! `g_regHIO` child; only `mNo` is handed to `mDoHIO_createChild` at `:669`), and the camera eye is
//! `super::camera::Camera`'s `view.lookat.eye`, which `camera::track` maintains from the player.
//! That is the sim's real camera eye - NOT `super::kytag00::CameraEye`, which the brief bans because
//! it is write-never and permanently `Vec3::ZERO`.
//!
//! WHAT IS OMITTED - `ho_move` (`:359-607`), and with it most of the file. Every input of the
//! 85-vertex cloth solve is absent:
//! * `cXyz* windVec = dKyw_get_wind_vec();` (`:386`) and `f32 windPow = dKyw_get_wind_pow();`
//!   (`:387`) feed `windAngle = cM_atan2s(windVec->x, windVec->z)` (`:395`), the sway increment
//!   `2500.0f + 9000.0f * (0.8f * windPow)` (`:431`) and the per-vertex gains
//!   `x_value = 0.25f + 0.75f * windPow` (`:485`) and `z_value = 0.5f + 0.25f * windPow` (`:488`).
//!   Both are weather-HIO state - they return `g_env_light.mWind.mWindVec` / `mWindPower`
//!   (`d_kankyo_wether.cpp:1088`, `:1093`), which only the per-stage weather tables populate. The
//!   brief is explicit that folding them is invented data, so they are UNKNOWN, not proven zero.
//! * `l_ship->getSailAngle()` (`:393`, `:412`, `:833`) is `daShip_c::mSailAngle`
//!   (`d_a_ship.h:165`) and `l_ship->getSailOn()` (`:450`) is `checkStateFlg(daSFLG_SAIL_ON_e)`
//!   (`d_a_ship.h:101`). `super::ship::Ship` carries only `speed`, `tiller`, `riding`, `warp_y`
//!   and `held` (`src/sim/ship.rs:44-48`).
//! * `l_pos[85]`, `l_texCoord`, `l_DL` and `l_matDL` live in `assets/*__d_a_grid.h` (included at
//!   `:215-218`) and are not in this checkout. `field_0x1b54[85]` is derived from `l_pos` by
//!   `_create` (`:674-759`) and multiplies every displacement in the solve loop (`:496-500`).
//! * `daHo_packet_c::setNrmVtx` / `setTopNrmVtx` / `setBackNrm` (`:121-213`) and all of `draw`
//!   (`:220-350`) are `J3D` / GX work - a legitimate omission under RULE 3b.
//! * `DCStoreRangeNoSync` (`:601`, `:604-605`) is cache maintenance for the same arrays.
//!
//! GRID INDEXING (recorded so a later pass that gets the mesh cannot get it wrong). There is NO
//! room/position index table in this actor. The vertex grid is a fixed 7-WIDE DOUBLE BUFFER:
//! `daHo_packet_c::mPos[2][85]` / `mNrm[2][85]` / `mBackNrm[2][85]` (`d_a_grid.h:41`, `:42`,
//! `:43`), ping-ponged by `changeCurrentPos() { field_0x18a2 ^= 1; }` (`d_a_grid.h:30`, called at
//! `:388`), and `85 = 12 * 7 + 1`. Inside `ho_move` the column index `var_r23` walks 0..=12:
//! ```c
//! var_r23 = var_r24 < 6 ? var_r23 : var_r23 + 1;   // :560
//! var_r24 = var_r24 < 6 ? var_r24 + 1 : 0;         // :561
//! ```
//! (`:560-561`), which is why all three tables it reads are THIRTEEN entries wide, not seven -
//! `z_rate_tbl` (`:364-367`), `z_rate_tbl2` (`:369-372`, byte-identical to `l_HIO.field_0x70[13]`
//! set at `:33-45`), `x_rate_tbl` (`:374-377`) and `l_HIO.field_0x3c[13]` (`:47-59`). RULE 10's
//! index guard is `var_r23` in 0..=12. The vertex loop then indexes `l_pos[var_r23 * 7]` (`:523`),
//! and `_create` walks `l_pos[var_r29]` with `var_r29` either 6 or in 56..=62, plus `l_pos[84]`
//! (`:686`, `:715`, `:743`). `field_0x1b48 = fopAcM_GetParam(this)` (`:655`, retail `>= VERSION_JPN`)
//! has NO reader anywhere in the translation unit, so the profile's `Parameters` field being `0`
//! (`:861`) costs nothing and no parameter decode is stored.
//!
//! `daGrid_IsDelete` (`:614-617`) is the bare "accepts a delete request" form of RULE 6b - it is
//! NOT a despawn. `_create` (`:630-769`) returns a non-complete `cPhs_State` only from the two
//! `dComIfG_resLoad` calls (`:657-665`), which is archive loading, so `spawn` never refuses and
//! there is no `refused` flag.
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE. `daGrid_c` registers no `dCcD` collider at
//! all - there is no `mStts`, no `dCcD_Cyl` / `dCcD_Sph` member in `d_a_grid.h:65-86`, no
//! `dComIfG_Ccsp()->Set(...)` call anywhere in the translation unit and no `ChkTgHit`. The profile
//! `Group` is `fopAc_ACTOR_e` (`:866`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.

use bevy_ecs::prelude::*;

use super::actor::Actor;

/// `OBJNAME("Grid", fpcNm_GRID_e, 255, 0)` (`d_stage.cpp:1251`) - the only row for `g_profile_GRID`
/// (`d_a_grid.cpp:853-868`).
pub const GRID_ACTORS: [&str; 1] = ["Grid"];

/// `daHo_HIO_c() { ... field_0x30 = 0xFF; ... }` (`:27`) - the fully OPAQUE target of the alpha ramp,
/// used when the sail is further from the camera than `ALPHA_FADE_RANGE` (`:795`) and whenever an
/// event is running (`:802`).
const ALPHA_FAR: u8 = 0xFF; // <- d_a_grid.cpp:27 (daHo_HIO_c::field_0x30)
/// `daHo_HIO_c() { ... field_0x31 = 50; ... }` (`:28`) - the nearly TRANSPARENT target, reached as the
/// camera closes to the sail (`:798`).
const ALPHA_NEAR: u8 = 50; // <- d_a_grid.cpp:28 (daHo_HIO_c::field_0x31)
/// `daHo_HIO_c() { ... field_0x34 = 900.0f; ... }` (`:29`) - the camera distance over which the sail
/// fades in, and the divisor of `rate` (`:794`, `:797`).
const ALPHA_FADE_RANGE: f32 = 900.0; // <- d_a_grid.cpp:29 (daHo_HIO_c::field_0x34)
/// ```c
/// if (target_alpha > alpha + 5) { mPacket.setAlpha(alpha + 5); }
/// else if (target_alpha < alpha - 5) { mPacket.setAlpha(alpha - 5); }
/// else { mPacket.setAlpha(target_alpha); }
/// ```
/// (`:805-810`) - the per-tick ramp limit in both directions.
const ALPHA_STEP: i32 = 5; // <- d_a_grid.cpp:805, :807
/// `daHo_packet_c() { ... mAlpha = 0xFF; ... }` (`d_a_grid.h:15`) - the sail starts opaque; the ramp
/// only ever pulls it down when the camera comes close.
const ALPHA_INIT: u8 = 0xFF; // <- d_a_grid.h:15 (daHo_packet_c::mAlpha)
/// ```c
/// if (scale.y < 0.06f) { return true; }
/// ```
/// (`:813-815` in `_execute`, `:823-825` in `_draw`) - below this vertical squash the cloth is not
/// updated and not drawn. RULE 2: `dStage_actorInit` never writes `fopAcM_prm_class::scale`, so a
/// stage-placed actor has scale exactly `1.0` and this gate never fires; it is transcribed anyway
/// because `Actor::scale` is real state.
const MIN_SCALE_Y: f32 = 0.06; // <- d_a_grid.cpp:813, :823

/// `fopAc_CULLBOX_0_e` (`:867`) resolves through `l_cullSizeBox[0]`
/// (`f_op_actor_mng.cpp:558`): `fopAc_cullSizeBox(cXyz(-40.0f, 0.0f, -40.0f),
/// cXyz(40.0f, 125.0f, 40.0f))`. This is a REAL profile, so the box comes from the table and the
/// file's own `fopAcM_setCullSizeBox` / `fopAcM_SetMin` / `SetMax` silence is expected.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// `daGrid_c` (`d_a_grid.h:50-87`), narrowed to the one field a `control` can read.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid {
    /// `daHo_packet_c::mAlpha` (`d_a_grid.h:47`, a `u8`) - the sail's opacity, ramped toward the
    /// camera-distance target every tick by `_execute` (`:805-810`). Read by `_create` at `:786`
    /// and written only by `setAlpha` (`:806`, `:808`, `:810`).
    pub alpha: u8,
}

impl Grid {
    /// `daGrid_c::_create` (`:630-769`) with the archive phase (`:657-665`), `fopAcM_ct` (`:652`),
    /// `field_0x1b48 = fopAcM_GetParam(this)` (`:655`, unread in this TU), the
    /// `mDoHIO_createChild("船の帆", &l_HIO)` (`:668-670`), the `field_0x1b54[85]` solve from
    /// `l_pos` (`:674-759`), the `fopAcM_SearchByName(fpcNm_SHIP_e)` (`:761`) and the first
    /// `ho_move(this)` (`:762`) all dropped - see the module docs.
    pub fn spawn(_parameters: u32) -> Self {
        Self {
            // `mAlpha = 0xFF;` (d_a_grid.h:15)
            alpha: ALPHA_INIT,
        }
    }

    /// `fopAc_CULLBOX_0_e` (`:867`) -> `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_grid(name: &str) -> bool {
    GRID_ACTORS.contains(&name)
}

/// `daGrid_c::_execute` (`:784-819`), minus `ho_move(this)` at `:817`.
pub fn control(
    mut sails: Query<(&Actor, &mut Grid), With<Grid>>,
    camera: Res<super::camera::Camera>,
    info: Res<crate::text::msgproc::MsgGameInfo>,
) {
    // RULE 9 / RULE 12(d): the access set is `&Actor` (read, `With<Grid>`), `&mut Grid` (write,
    // `With<Grid>`), `Res<camera::Camera>` (read) and `Res<MsgGameInfo>` (read). Nothing here writes
    // `Actor`, `Camera` or `MsgGameInfo`, and no two params touch the same component, so there is no
    // intra-system ambiguity to guard.
    //
    // `dComIfGp_event_runCheck()` (`:789`) - `crate::text::msgproc::MsgGameInfo::event_run_check`
    // (`src/text/msgproc.rs:378`), the crate's existing reduction for this call.
    let event_running = info.event_run_check();
    for (actor, mut sail) in &mut sails {
        // ```
        // u8 alpha = mPacket.getAlpha();
        // ```
        // (:786)
        let alpha = sail.alpha;
        // ```
        // if (!dComIfGp_event_runCheck()) {
        //     cXyz eye = dComIfGp_getCamera(0)->mCamera.Eye();
        //     f32 cam_eye_dist = (current.pos - eye).abs();
        //     u8 target;
        //     if (cam_eye_dist > l_HIO.field_0x34) { target = l_HIO.field_0x30; }
        //     else {
        //         f32 rate = cam_eye_dist / l_HIO.field_0x34;
        //         target = l_HIO.field_0x30 * rate + l_HIO.field_0x31 * (1.0f - rate);
        //     }
        //     target_alpha = target;
        // } else {
        //     target_alpha = l_HIO.field_0x30;
        // }
        // ```
        // (:789-803)
        let target_alpha = if event_running {
            // `} else { target_alpha = l_HIO.field_0x30; }` (:801-803)
            ALPHA_FAR
        } else if let Some(view) = camera.view() {
            // ```
            // cXyz eye = dComIfGp_getCamera(0)->mCamera.Eye();
            // f32 cam_eye_dist = (current.pos - eye).abs();
            // u8 target;
            // if (cam_eye_dist > l_HIO.field_0x34) { target = l_HIO.field_0x30; }
            // else {
            //     f32 rate = cam_eye_dist / l_HIO.field_0x34;
            //     target = l_HIO.field_0x30 * rate + l_HIO.field_0x31 * (1.0f - rate);
            // }
            // target_alpha = target;
            // ```
            // (:790-800) - `cXyz::abs(const Vec&)` is `std::sqrtf(this->abs2(other))`, and `abs2` is
            // `VECSquareDistance` (`include/SSystem/SComponent/c_xyz.h:125`, `:114`), so this is the
            // EUCLIDEAN distance, not `absXZ`. Retail assigns the float blend into a `u8`, i.e.
            // truncation toward zero; the value is inside `[50, 255]`, so `as u8` reproduces it.
            let cam_eye_dist = (actor.pos - view.lookat.eye).length();
            if cam_eye_dist > ALPHA_FADE_RANGE {
                ALPHA_FAR
            } else {
                let rate = cam_eye_dist / ALPHA_FADE_RANGE;
                (f32::from(ALPHA_FAR) * rate + f32::from(ALPHA_NEAR) * (1.0 - rate)) as u8
            }
        } else {
            // `dComIfGp_getCamera(0)` is never NULL in retail. The sim's `Camera::view` is `Some`
            // from `Camera::default()` (`src/sim/camera.rs:131`) and nothing in the crate clears it,
            // so this arm is defensive rather than a modelled retail state: with no eye there is no
            // distance, so the ramp is left alone instead of invented.
            continue;
        };
        // ```
        // if (target_alpha > alpha + 5) { mPacket.setAlpha(alpha + 5); }
        // else if (target_alpha < alpha - 5) { mPacket.setAlpha(alpha - 5); }
        // else { mPacket.setAlpha(target_alpha); }
        // ```
        // (:805-810) - `mAlpha` is a `u8`, so BOTH `alpha + 5` and `alpha - 5` are formed in the
        // promoted `int` (RULE 10: `alpha - 5` would underflow a `u8`, and `bevy_ecs` is built with
        // its `debug` feature). Both stores are provably in range: the step-up branch needs
        // `target > alpha + 5` with `target <= 255`, hence `alpha <= 249`; the step-down branch needs
        // `target < alpha - 5` with `target >= 0`, hence `alpha >= 6`.
        let (alpha, target) = (i32::from(alpha), i32::from(target_alpha));
        sail.alpha = if target > alpha + ALPHA_STEP {
            (alpha + ALPHA_STEP) as u8
        } else if target < alpha - ALPHA_STEP {
            (alpha - ALPHA_STEP) as u8
        } else {
            target_alpha
        };
        // ```
        // if (scale.y < 0.06f) { return true; }
        // ho_move(this);
        // ```
        // (:813-817) - the scale gate is transcribed; `ho_move` is OMITTED (module docs), so nothing
        // runs past it in this port. RULE 2 makes the gate unreachable for a stage-placed actor.
        if actor.scale.y < MIN_SCALE_Y {
            continue;
        }
    }
}
