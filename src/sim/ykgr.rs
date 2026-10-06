use bevy_ecs::prelude::*;

use super::area::StageName;
use super::flags::{Collect, SYMBOL_DIN};

/// The two stage strings (`d_stage.cpp:859-860`), both `fpcNm_Ykgr_e`. The actor owns no archive -
/// it drives the shared `ID_AK_SP_MTDRAGONKAGEROU` projection emitter (`d_a_ykgr.cpp:144`) - so both
/// placed variants map here.
pub const YKGR_ACTORS: [&str; 2] = ["YkgrON", "YkgrOFF"];

/// `u8 uVar4 = fopAcM_GetParam(this) >> 0x14 & 0xF` (`d_a_ykgr.cpp:126`).
const VAR_SHIFT: u32 = 0x14; // <- d_a_ykgr.cpp:126
const VAR_WIDTH: u32 = 4; // <- d_a_ykgr.cpp:126
/// `u8 pathIndex = fopAcM_GetParam(this) >> 8 & 0xFF` (`d_a_ykgr.cpp:130`), whose `0xff` sentinel
/// skips `dPath_GetRoomPath` (`:131`).
const PATH_SHIFT: u32 = 8; // <- d_a_ykgr.cpp:130
const PATH_WIDTH: u32 = 8; // <- d_a_ykgr.cpp:130
const PATH_NONE: u8 = 0xFF; // <- d_a_ykgr.cpp:131
/// `strcmp(dComIfGp_getStartStageName(), "Adanmae") == 0` (`d_a_ykgr.cpp:137`).
const STAGE_ADANMAE: &str = "Adanmae"; // <- d_a_ykgr.cpp:137
/// `dSymbol_DIN_e` (`d_com_inf_game.h:1691`, enumerator 1) - `dComIfGs_isSymbol`'s argument
/// (`d_a_ykgr.cpp:137`).
const SYMBOL_DIN_FLAG: u8 = SYMBOL_DIN; // <- d_com_inf_game.h:1691 (d_a_ykgr.cpp:137)
/// `m0C = 3` in `daYkgr_HIO_c`'s constructor (`d_a_ykgr.cpp:29`) - the per-frame alpha step of
/// `:203-217`.
const HIO_ALPHA_STEP: u8 = 3; // <- d_a_ykgr.cpp:29
/// `m14 = -5.0f` (`d_a_ykgr.cpp:31`), the far end of the `YkgrCB.setParam` blend at `:200`.
const HIO_M14: f32 = -5.0; // <- d_a_ykgr.cpp:31
/// `m18 = -3.0f` (`d_a_ykgr.cpp:32`), the near end of the same blend.
const HIO_M18: f32 = -3.0; // <- d_a_ykgr.cpp:32
/// `m1C = 0.0f` (`d_a_ykgr.cpp:33`) - the target of the second `cLib_addCalc2` at `:196`.
const HIO_M1C: f32 = 0.0; // <- d_a_ykgr.cpp:33
/// `setAlpha(0xFF)` and `setAlpha(0)` (`d_a_ykgr.cpp:155`, `:161`).
const ALPHA_ON: u8 = 0xFF; // <- d_a_ykgr.cpp:155
const ALPHA_OFF: u8 = 0x00; // <- d_a_ykgr.cpp:161
/// `m2CC = 0.5f` / `m2D0 = 0.0f` in `_create` (`d_a_ykgr.cpp:157-158`, `:163-164`) - the
/// `d_aYkgr_c` instance fields at `d_a_ykgr.h:70-71`.
const RATE_INIT: f32 = 0.5; // <- d_a_ykgr.cpp:157
const DIST_INIT: f32 = 0.0; // <- d_a_ykgr.cpp:158
/// `cLib_addCalc2(&m2CC, m_aim_rate, 0.25f, 0.05f)` and its two siblings (`d_a_ykgr.cpp:195-197`);
/// `cLib_addCalc2(f32* pValue, f32 target, f32 scale, f32 maxStep)` is a symmetric rate limiter
/// with no minimum step (`c_lib.cpp:56-66`).
const ADD_CALC_SCALE: f32 = 0.25; // <- d_a_ykgr.cpp:195
const ADD_CALC_MAX_STEP: f32 = 0.05; // <- d_a_ykgr.cpp:195
/// `f32 fVar1 = m2CC * 0.5f + m2D0 * 0.5f` (`d_a_ykgr.cpp:199`).
const BLEND_HALF: f32 = 0.5; // <- d_a_ykgr.cpp:199
/// `(1.0f - fVar1)` (`d_a_ykgr.cpp:200`).
const BLEND_ONE: f32 = 1.0; // <- d_a_ykgr.cpp:200

/// `cLib_addCalc2(f32* pValue, f32 target, f32 scale, f32 maxStep)` (`c_lib.cpp:56-66`) - the same
/// shape `kn.rs:100-105` carries.
fn add_calc2(value: &mut f32, target: f32) {
    if *value == target {
        return;
    }
    let step = (ADD_CALC_SCALE * (target - *value)).clamp(-ADD_CALC_MAX_STEP, ADD_CALC_MAX_STEP);
    *value += step;
}

/// The dragon's sun-haze projection over Dragon Roost - `daYkgr_c` (`d_a_ykgr.h:14-72`).
///
/// `daYkgr_c` registers no `dCcD_Src*` table: `_create` (`:125-183`) touches only `m_path`,
/// `current.pos`, `m_emitter`, `m_alpha`, `m_alpha_flag`, `m2CC` and `m2D0`, and `_execute`
/// (`:194-219`) is four `cLib_addCalc2` calls, one blend and the alpha ramp. There is no AT and no
/// Tg collider, so nothing damages it and it damages nothing, and the class has no health field.
/// The profile `Group` is `fopAc_ACTOR_e` (`:291`). The `Cull Type` is `fopAc_CULLBOX_0_e`
/// (`:292`) and the file never calls `fopAcM_setCullSizeBox`, so no `cull()` is derivable.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. Neither `combat::HitPoints` nor
/// `enemy::Hurt` is attached.
///
/// `_create` returns `cPhs_STOP_e` from the `Adanmae` + `dSymbol_DIN_e` test (`:137-139`), and
/// `cPhs_STOP_e` cancels the actor inside Create (`f_pc_create_req.cpp:107-108`), so that arm is
/// modelled as a refusal despawned on the first tick. The `m_emitter != NULL` else branch
/// (`:173-180`) returns `cPhs_STOP_e` for every actor placed after the first, because
/// `m_emitter` is a process-wide static (`d_com_static.cpp:58`) that the first `_create` installs;
/// that arm needs create-order state this module cannot see, and is reported rather than guessed.
/// `daYkgrIsDelete` (`:266-268`) is the bare "accepts a delete request" form of RULE 6b.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ykgr {
    /// `m_alpha` (`d_a_ykgr.h:31`).
    pub alpha: u8,
    /// `m_alpha_flag` (`d_a_ykgr.h:30`) - 1 fades in, 0 fades out (`:203-217`).
    pub alpha_flag: u8,
    /// `m_aim_rate` (`d_a_ykgr.h:32`), chased to `l_HIO.m1C` at `:196`.
    pub aim_rate: f32,
    /// `m2CC` (`d_a_ykgr.h:70`) - the `m_aim_rate` follower of `:195`.
    pub rate: f32,
    /// `m2D0` (`d_a_ykgr.h:71`) - the `getPosRate()` follower of `:197`.
    pub dist: f32,
    /// `uVar4 = fopAcM_GetParam(this) >> 0x14 & 0xF` (`d_a_ykgr.cpp:126`); 1 is the `ON` variant.
    pub variant: u8,
    /// `pathIndex = fopAcM_GetParam(this) >> 8 & 0xFF` (`d_a_ykgr.cpp:130`), whose only reader is
    /// the `dPath_GetRoomPath` call of `:132` - a room `PATH` block, which RULE 3b lists as absent.
    pub path_index: u8,
    /// `_create`'s `cPhs_STOP_e` arm at `:137-139`, which cancels the actor inside Create.
    pub refused: bool,
}

impl Ykgr {
    /// `_create` (`:125-183`) minus `fopAcM_ct`, `dPath_GetRoomPath`, `dComIfGp_particle_setProjection`,
    /// `setParticleCallBackPtr`, `YkgrCB.setParam(-3.0f)`, the two `setGlobalAlpha` calls,
    /// `fopAcM_setStageLayer`, `show`/`stop`/`start`/`setAlpha` (particle and status work) and the
    /// DEMO-only HIO child. `current.pos = player->current.pos` (`:143`) has no reader either - the
    /// only position store, `set_mtx` (`:228`), overwrites it from the camera every draw.
    ///
    /// Both refusal inputs are stage-load state, which is exactly when retail runs `_create`:
    /// `dComIfGp_getStartStageName()` is the stage name and `dComIfGs_isSymbol(dSymbol_DIN_e)` reads
    /// the collect record.
    pub fn spawn(parameters: u32, stage: &StageName, collect: &Collect) -> Self {
        let variant = ((parameters >> VAR_SHIFT) & ((1 << VAR_WIDTH) - 1)) as u8;
        let path_index = ((parameters >> PATH_SHIFT) & ((1 << PATH_WIDTH) - 1)) as u8;
        let on = variant == 1;
        Self {
            alpha: if on { ALPHA_ON } else { ALPHA_OFF },
            alpha_flag: u8::from(on),
            aim_rate: HIO_M1C,
            rate: RATE_INIT,
            dist: DIST_INIT,
            variant,
            path_index,
            refused: stage.0 == STAGE_ADANMAE && collect.is_symbol(SYMBOL_DIN_FLAG),
        }
    }

    /// `daYkgr_c::getPosRate` (`d_a_ykgr.cpp:90-117`) returns `0.0f` outright when `m_path == NULL`
    /// (`:91-93`). `dPath_GetRoomPath(pathIndex, fopAcM_GetRoomNo(this))` (`:132`) resolves a room
    /// `PATH` block, which RULE 3b lists as absent, so `m_path` is always NULL here and the rest of
    /// `getPosRate` - the `m_points`/`m_num` walk of `:101-107`, the
    /// `l_HIO.m20` (1500.0f, `:34`) / `l_HIO.m24` (500.0f, `:35`) clamp of `:109-113` and the
    /// `(fVar4 - m24) / (m20 - m24)` normalisation of `:115` - has no input. The proven `0.0f`
    /// return is what `m2D0` chases at `:197`.
    fn pos_rate(&self) -> f32 {
        let _ = PATH_NONE;
        0.0
    }

    /// `f32 fVar1 = m2CC * 0.5f + m2D0 * 0.5f` then
    /// `YkgrCB.setParam(fVar1 * l_HIO.m18 + (1.0f - fVar1) * l_HIO.m14)`
    /// (`d_a_ykgr.cpp:199-201`). `dPa_YkgrPcallBack::setParam` (`:69-87`) writes the indirection
    /// texture's matrix and the `GX_TEVSTAGE0` alpha-in blend of `:64-66`, so the result has no
    /// reader here - the blend is transcribed so `rate` and `dist` are not dead, and its value is
    /// dropped rather than stored.
    fn blend(&self) {
        let mixed = self.rate * BLEND_HALF + self.dist * BLEND_HALF;
        let _ = mixed * HIO_M18 + (BLEND_ONE - mixed) * HIO_M14;
    }

    /// The alpha ramp of `_execute` (`d_a_ykgr.cpp:203-217`): with `m_alpha_flag == 0` the alpha
    /// steps down by `l_HIO.m0C` and clamps at 0; otherwise - and only while `m_alpha < 0xff` - it
    /// steps up by `l_HIO.m0C` and clamps at `0xff`.
    fn ramp(&mut self) {
        if self.alpha_flag == 0 {
            if self.alpha != 0 {
                self.alpha = if self.alpha > HIO_ALPHA_STEP {
                    self.alpha - HIO_ALPHA_STEP
                } else {
                    ALPHA_OFF
                };
            }
        } else if self.alpha < ALPHA_ON {
            self.alpha = if self.alpha < ALPHA_ON - HIO_ALPHA_STEP {
                self.alpha + HIO_ALPHA_STEP
            } else {
                ALPHA_ON
            };
        }
    }

    /// `_execute` (`d_a_ykgr.cpp:194-219`): the three `cLib_addCalc2` calls (`:195-197`), the
    /// `fVar1` blend and `YkgrCB.setParam(tmp2)` (`:199-201`, particle work), then the ramp.
    fn execute(&mut self) {
        let target = self.aim_rate;
        add_calc2(&mut self.rate, target);
        add_calc2(&mut self.aim_rate, HIO_M1C);
        let pos_rate = self.pos_rate();
        add_calc2(&mut self.dist, pos_rate);
        self.blend();
        self.ramp();
    }
}

pub fn is_ykgr(name: &str) -> bool {
    YKGR_ACTORS.contains(&name)
}

pub fn control(
    mut hazes: Query<(Entity, &mut Ykgr)>,
    mut commands: Commands,
) {
    for (entity, mut haze) in &mut hazes {
        if haze.refused {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        haze.execute();
    }
}