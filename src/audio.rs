//! Output device staging: cue bootstrap plus sim-event playback.

use repame_audio::{Audio, CueDef, synth_sine_wav};

pub const FOOTSTEP_CUE: &str = "footstep";

const FOOTSTEP_HZ: f32 = 140.0;
const FOOTSTEP_SECS: f32 = 0.09;
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
    let footstep = synth_sine_wav(FOOTSTEP_HZ, FOOTSTEP_SECS, rate);
    audio.load_cue(FOOTSTEP_CUE, footstep_cue(), &[&footstep])
}
