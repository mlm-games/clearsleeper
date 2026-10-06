use bevy_ecs::prelude::*;

pub const IKARI_ACTORS: [&str; 1] = ["Ikari"];

const ENV_MASK: u32 = 0xFF; // <- d_a_ikari.cpp:69 (mEnvType = param, stored in the u8 at d_a_ikari.h:0x29C)
const ENV_SCALE: u8 = 0x01; // <- d_a_ikari.cpp:79
const WIND_SCALE: f32 = 1.27; // <- d_a_ikari.cpp:80-82
const CULL_MIN_X: f32 = -160.0; // <- d_a_ikari.cpp:142-143
const CULL_MIN_Y: f32 = -2500.0; // <- d_a_ikari.cpp:143
const CULL_MIN_Z: f32 = -600.0; // <- d_a_ikari.cpp:143
const CULL_MAX_X: f32 = 160.0; // <- d_a_ikari.cpp:144
const CULL_MAX_Y: f32 = 100.0; // <- d_a_ikari.cpp:144
const CULL_MAX_Z: f32 = 600.0; // <- d_a_ikari.cpp:144

/// Forsaken Fortress anchor. The stage string is `Ikari` (`d_stage.cpp:830`) and the archive is
/// `Ikari` (`d_a_ikari.cpp:15`), with `S_IKARI2..4` (`d_a_ikari.cpp:45-49`).
///
/// `_execute` (`:87-104`) is driven entirely by `dKyw_get_wind_vec()` and
/// `dKyw_get_wind_power()` (`:90-92`), which are weather config with no home in this sim (the
/// only wind state here is `windmill::Wind::strength`, the Wind's-Requiem event strength
/// written by `wind::control`, which is a different quantity). `mTimer` (`:88`, `:99`) is
/// seeded from `cM_rndF(0x8000)` (`:147`) - the sim has no global `cM_rndF`, only the per-actor
/// LCGs in `ajav.rs`/`agbsw0.rs` - and is read nowhere but the omitted wind expression, so no
/// field survives. `l_HIO.mWindPowerScale` is 0.1f (`d_a_ikari.h:9`), inside the same omitted
/// expression.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ikari {
    scale_x: f32,
}

impl Ikari {
    /// `getArg` (`d_a_ikari.cpp:65-84`). `mEnvType` is the whole parameter truncated into the
    /// `u8` field at `d_a_ikari.h:0x29C`; `0xff` forces scale 1 (`:75-78`) and `0x01` scales it
    /// by 1.27 (`:79-83`), every other value leaves the `fopAcM_ct` default of 1.0. `_create`
    /// reads `scale.x` straight back (`:139`) to size the cull box, so that is the only part of
    /// the scale this component keeps - `setMtx` (`:23-33`) is the sole other consumer.
    pub fn spawn(parameters: u32) -> Self {
        let env = (parameters & ENV_MASK) as u8;
        Self {
            scale_x: if env == ENV_SCALE { WIND_SCALE } else { 1.0 },
        }
    }

    /// `fopAcM_setCullSizeBox` (`d_a_ikari.cpp:142-144`) scaled by `scaleX` from `:139`;
    /// `fopAcM_setCullSizeFar(this, 10.0f)` (`:145`) is a cull distance, not a box, so it does
    /// not appear here.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let scale = self.scale_x;
        (
            [
                CULL_MIN_X * scale,
                CULL_MIN_Y * scale,
                CULL_MIN_Z * scale,
            ],
            [CULL_MAX_X * scale, CULL_MAX_Y * scale, CULL_MAX_Z * scale],
        )
    }
}

pub fn is_ikari(name: &str) -> bool {
    IKARI_ACTORS.contains(&name)
}

pub fn control(_anchors: Query<&Ikari>) {}