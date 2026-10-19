//! Output device staging: cue bootstrap plus sim-event playback.

use repame_audio::{AmplitudeEnvelope, Audio, AudioChannel, CueDef, CueSpec, Waveform};

use crate::sim::{canon, damage_resolve as hit};

pub const FOOTSTEP_CUE: &str = "footstep";
pub const SLASH_CUE: &str = "slash";
pub const HIT_CUE: &str = "hit";
pub const BOOM_CUE: &str = "boom";
pub const PICKUP_CUE: &str = "pickup";
pub const HURT_CUE: &str = "hurt";
pub const HAMMER_CUE: &str = "hammer";
pub const HAMMER_SWING_CUE: &str = "hammer-swing";
pub const MOVIE_CUE: &str = "movie";
pub const SWORD_HIT_CUE: &str = "sword-hit";
pub const SWORD_CRITICAL_HIT_CUE: &str = "sword-critical-hit";
pub const WOOD_HIT_CUE: &str = "wood-hit";
pub const WOOD_CRITICAL_HIT_CUE: &str = "wood-critical-hit";
pub const METAL_HIT_CUE: &str = "metal-hit";
pub const METAL_CRITICAL_HIT_CUE: &str = "metal-critical-hit";
pub const ARROW_HIT_CUE: &str = "arrow-hit";
pub const HOOKSHOT_HIT_CUE: &str = "hookshot-hit";
pub const HAMMER_HIT_CUE: &str = "hammer-hit";
pub const LAST_HIT_CUE: &str = "last-hit";
pub const BALL_HIT_CUE: &str = "ball-hit";
/// Stand-in for the battle-music hit accent. The function that owns the hook has no recovered
/// body, so its five selector values collapse to this one tone. See the drain site for why.
pub const BGM_HIT_CUE: &str = "bgm-hit";

const FOOTSTEP_HZ: f32 = 140.0;
const FOOTSTEP_SECS: f32 = 0.09;
const SLASH_HZ: f32 = 520.0;
const SLASH_SECS: f32 = 0.07;
const HIT_HZ: f32 = 900.0;
const HIT_SECS: f32 = 0.06;
const BOOM_HZ: f32 = 70.0;
const BOOM_SECS: f32 = 0.36;
const PICKUP_HZ: f32 = 1180.0;
const PICKUP_SECS: f32 = 0.12;
const HURT_HZ: f32 = 220.0;
const HURT_SECS: f32 = 0.16;
const HAMMER_HZ: f32 = 96.0;
const HAMMER_SECS: f32 = 0.42;
const HAMMER_SWING_HZ: f32 = 300.0;
const HAMMER_SWING_SECS: f32 = 0.2;
const SWORD_HIT_HZ: f32 = 880.0;
const SWORD_HIT_SECS: f32 = 0.08;
const SWORD_CRITICAL_HIT_HZ: f32 = 1240.0;
const SWORD_CRITICAL_HIT_SECS: f32 = 0.11;
const WOOD_HIT_HZ: f32 = 320.0;
const WOOD_HIT_SECS: f32 = 0.07;
const WOOD_CRITICAL_HIT_HZ: f32 = 430.0;
const WOOD_CRITICAL_HIT_SECS: f32 = 0.1;
const METAL_HIT_HZ: f32 = 1700.0;
const METAL_HIT_SECS: f32 = 0.09;
const METAL_CRITICAL_HIT_HZ: f32 = 2100.0;
const METAL_CRITICAL_HIT_SECS: f32 = 0.12;
const ARROW_HIT_HZ: f32 = 640.0;
const ARROW_HIT_SECS: f32 = 0.07;
const HOOKSHOT_HIT_HZ: f32 = 520.0;
const HOOKSHOT_HIT_SECS: f32 = 0.06;
const HAMMER_HIT_HZ: f32 = 110.0;
const HAMMER_HIT_SECS: f32 = 0.3;
const LAST_HIT_HZ: f32 = 260.0;
const LAST_HIT_SECS: f32 = 0.25;
const BALL_HIT_HZ: f32 = 760.0;
const BALL_HIT_SECS: f32 = 0.1;
const BGM_HIT_HZ: f32 = 1040.0;
const BGM_HIT_SECS: f32 = 0.05;
const FALLBACK_RATE: u32 = 44100;

/// The bank cue as `synth_sine_wav` wrote it: a sine with a `1/30` s
/// exponential decay time constant at half gain, at [`FALLBACK_RATE`]; a bank
/// with a device open renders it at the device rate.
fn cue_spec(hz: f32, secs: f32) -> CueSpec {
    CueSpec::new(Waveform::Sine, hz, secs)
        .with_amplitude(AmplitudeEnvelope::Adsr {
            attack_s: 0.0,
            decay_s: 1.0 / 30.0,
            sustain: 0.0,
            release_s: 0.0,
        })
        .with_gain(0.5)
        .with_sample_rate(FALLBACK_RATE)
}

pub fn footstep_cue() -> CueDef {
    CueDef {
        gain: 0.5,
        pitch_wobble: 0.1,
        cooldown_ms: 60,
        max_voices: 4,
        ..CueDef::default()
    }
}

fn cue(gain: f32, cooldown_ms: u64, wobble: f32) -> CueDef {
    CueDef {
        gain,
        pitch_wobble: wobble,
        cooldown_ms,
        max_voices: 4,
        ..CueDef::default()
    }
}

pub fn movie_cue() -> CueDef {
    CueDef {
        bus: AudioChannel::Music,
        gain: 1.0,
        cooldown_ms: 0,
        max_voices: 1,
        ..CueDef::default()
    }
}

/// Every cue the bank holds, in the order [`boot_cues`] returns its voices.
pub const CUE_NAMES: [&str; 20] = [
    FOOTSTEP_CUE,
    SLASH_CUE,
    HIT_CUE,
    BOOM_CUE,
    PICKUP_CUE,
    HURT_CUE,
    HAMMER_CUE,
    HAMMER_SWING_CUE,
    SWORD_HIT_CUE,
    SWORD_CRITICAL_HIT_CUE,
    WOOD_HIT_CUE,
    WOOD_CRITICAL_HIT_CUE,
    METAL_HIT_CUE,
    METAL_CRITICAL_HIT_CUE,
    ARROW_HIT_CUE,
    HOOKSHOT_HIT_CUE,
    HAMMER_HIT_CUE,
    LAST_HIT_CUE,
    BALL_HIT_CUE,
    BGM_HIT_CUE,
];

pub fn boot_cues() -> [(CueDef, f32, f32); 20] {
    [
        (footstep_cue(), FOOTSTEP_HZ, FOOTSTEP_SECS),
        (cue(0.42, 40, 0.22), SLASH_HZ, SLASH_SECS),
        (cue(0.62, 30, 0.18), HIT_HZ, HIT_SECS),
        (cue(0.85, 80, 0.12), BOOM_HZ, BOOM_SECS),
        (cue(0.5, 40, 0.05), PICKUP_HZ, PICKUP_SECS),
        (cue(0.66, 60, 0.14), HURT_HZ, HURT_SECS),
        (cue(0.9, 90, 0.1), HAMMER_HZ, HAMMER_SECS),
        (cue(0.6, 50, 0.2), HAMMER_SWING_HZ, HAMMER_SWING_SECS),
        (cue(0.6, 40, 0.15), SWORD_HIT_HZ, SWORD_HIT_SECS),
        (
            cue(0.7, 40, 0.15),
            SWORD_CRITICAL_HIT_HZ,
            SWORD_CRITICAL_HIT_SECS,
        ),
        (cue(0.5, 40, 0.12), WOOD_HIT_HZ, WOOD_HIT_SECS),
        (
            cue(0.6, 40, 0.12),
            WOOD_CRITICAL_HIT_HZ,
            WOOD_CRITICAL_HIT_SECS,
        ),
        (cue(0.55, 40, 0.1), METAL_HIT_HZ, METAL_HIT_SECS),
        (
            cue(0.65, 40, 0.1),
            METAL_CRITICAL_HIT_HZ,
            METAL_CRITICAL_HIT_SECS,
        ),
        (cue(0.5, 40, 0.14), ARROW_HIT_HZ, ARROW_HIT_SECS),
        (cue(0.45, 40, 0.14), HOOKSHOT_HIT_HZ, HOOKSHOT_HIT_SECS),
        (cue(0.9, 60, 0.1), HAMMER_HIT_HZ, HAMMER_HIT_SECS),
        (cue(0.8, 50, 0.1), LAST_HIT_HZ, LAST_HIT_SECS),
        (cue(0.6, 40, 0.12), BALL_HIT_HZ, BALL_HIT_SECS),
        (cue(0.35, 40, 0.2), BGM_HIT_HZ, BGM_HIT_SECS),
    ]
}

/// The cue a hit sound id plays: the two `d_cc_uty` ladders' ids, the last-hit sound of the
/// death branch, and the `fpcNm_MT_e` ball damage. `None` for an id with no ported cue.
pub fn hit_cue(id: u32) -> Option<&'static str> {
    match id {
        hit::SE_LK_SW_HIT_S => Some(SWORD_HIT_CUE),
        hit::SE_LK_SW_CRT_HIT => Some(SWORD_CRITICAL_HIT_CUE),
        hit::SE_LK_W_WEP_HIT => Some(WOOD_HIT_CUE),
        hit::SE_LK_W_WEP_CRT_HIT => Some(WOOD_CRITICAL_HIT_CUE),
        hit::SE_LK_MS_WEP_HIT => Some(METAL_HIT_CUE),
        hit::SE_LK_MS_WEP_CRT_HIT => Some(METAL_CRITICAL_HIT_CUE),
        hit::SE_LK_ARROW_HIT => Some(ARROW_HIT_CUE),
        hit::SE_LK_HS_SPIKE => Some(HOOKSHOT_HIT_CUE),
        hit::SE_LK_HAMMER_HIT => Some(HAMMER_HIT_CUE),
        hit::SE_OBJ_MG_BALL_DMG => Some(BALL_HIT_CUE),
        canon::SE_LAST_HIT => Some(LAST_HIT_CUE),
        _ => None,
    }
}

/// Open the output device, falling back to silence when none is available.
pub fn open() -> Audio {
    match Audio::try_init() {
        Ok(audio) => audio,
        Err(e) => {
            log::warn!("clearsleeper: no audio device ({e}); running silent");
            Audio::noop()
        }
    }
}

/// Register every game cue on the bank; works with or without a device.
pub fn load_cues(audio: &mut Audio) -> anyhow::Result<()> {
    for (name, (def, hz, secs)) in CUE_NAMES.iter().zip(boot_cues()) {
        audio.load_synth(name, def, &[cue_spec(hz, secs)])?;
    }
    Ok(())
}
