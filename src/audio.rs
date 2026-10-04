//! Output device staging: cue bootstrap plus sim-event playback.

use repame_audio::{Audio, CueDef, synth_sine_wav};

pub const FOOTSTEP_CUE: &str = "footstep";
pub const SLASH_CUE: &str = "slash";
pub const HIT_CUE: &str = "hit";
pub const BOOM_CUE: &str = "boom";
pub const PICKUP_CUE: &str = "pickup";
pub const HURT_CUE: &str = "hurt";
pub const HAMMER_CUE: &str = "hammer";
pub const HAMMER_SWING_CUE: &str = "hammer-swing";

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
const FALLBACK_RATE: u32 = 44100;

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

pub fn boot_cues() -> [(CueDef, f32, f32); 8] {
    [
        (footstep_cue(), FOOTSTEP_HZ, FOOTSTEP_SECS),
        (cue(0.42, 40, 0.22), SLASH_HZ, SLASH_SECS),
        (cue(0.62, 30, 0.18), HIT_HZ, HIT_SECS),
        (cue(0.85, 80, 0.12), BOOM_HZ, BOOM_SECS),
        (cue(0.5, 40, 0.05), PICKUP_HZ, PICKUP_SECS),
        (cue(0.66, 60, 0.14), HURT_HZ, HURT_SECS),
        (cue(0.9, 90, 0.1), HAMMER_HZ, HAMMER_SECS),
        (cue(0.6, 50, 0.2), HAMMER_SWING_HZ, HAMMER_SWING_SECS),
    ]
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
    let rate = match audio.sample_rate() {
        0 => FALLBACK_RATE,
        r => r,
    };
    let names = [
        FOOTSTEP_CUE,
        SLASH_CUE,
        HIT_CUE,
        BOOM_CUE,
        PICKUP_CUE,
        HURT_CUE,
        HAMMER_CUE,
        HAMMER_SWING_CUE,
    ];
    for (index, (def, hz, secs)) in boot_cues().into_iter().enumerate() {
        let wave = synth_sine_wav(hz, secs, rate);
        audio.load_cue(names[index], def, &[&wave])?;
    }
    Ok(())
}
