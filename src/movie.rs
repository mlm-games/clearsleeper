use std::io::Cursor;

pub const HEADER_LEN: usize = 0x60;
pub const MAX_FRAME: usize = 0x08;
pub const MAX_SAMPLES: usize = 0x0C;
pub const FRAME_RATE: usize = 0x10;
pub const FRAME_COUNT: usize = 0x14;
pub const FIRST_OFFSET: usize = 0x28;
pub const LAST_OFFSET: usize = 0x2C;

const SOI: [u8; 3] = [0xFF, 0xD8, 0xFF];
const DEFAULT_RATE: f32 = 29.97;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovieError;

pub struct Movie {
    data: Vec<u8>,
    frames: Vec<usize>,
    rate: f32,
}

fn be32(data: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]])
}

impl Movie {
    pub fn parse(data: Vec<u8>) -> Result<Self, MovieError> {
        if data.len() < HEADER_LEN || &data[..4] != b"THP\0" {
            return Err(MovieError);
        }
        let rate = f32::from_bits(be32(&data, FRAME_RATE));
        let count = be32(&data, FRAME_COUNT) as usize;
        let frames = scan(&data, be32(&data, FIRST_OFFSET) as usize, count);
        if frames.is_empty() {
            return Err(MovieError);
        }
        Ok(Self { data, frames, rate })
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn rate(&self) -> f32 {
        if self.rate > 0.0 { self.rate } else { DEFAULT_RATE }
    }

    pub fn interval(&self) -> f32 {
        1.0 / self.rate()
    }

    pub fn declared_frames(&self) -> usize {
        be32(&self.data, FRAME_COUNT) as usize
    }

    pub fn max_frame_bytes(&self) -> usize {
        be32(&self.data, MAX_FRAME) as usize
    }

    pub fn max_samples(&self) -> usize {
        be32(&self.data, MAX_SAMPLES) as usize
    }

    pub fn last_offset(&self) -> usize {
        be32(&self.data, LAST_OFFSET) as usize
    }

    /// Decode one frame to packed 8-bit RGB.
    pub fn frame(&self, index: usize) -> Option<Vec<u8>> {
        let start = *self.frames.get(index)?;
        decode(&self.data[start..]).ok()
    }

    /// Bytes between the end of this frame's image and the start of the next.
    pub fn audio_span(&self, index: usize) -> Option<(usize, usize)> {
        let start = *self.frames.get(index)?;
        let end = match self.frames.get(index + 1) {
            Some(next) => *next,
            None => self.last_offset().min(self.data.len()),
        };
        (start < end && end <= self.data.len()).then_some((start, end))
    }
}

fn scan(data: &[u8], first: usize, count: usize) -> Vec<usize> {
    let mut out = Vec::with_capacity(count.min(8192));
    let mut at = first.min(data.len());
    while out.len() < count {
        let Some(found) = find(data, at, &SOI) else {
            break;
        };
        out.push(found);
        at = found + 3;
    }
    out
}

fn find(data: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if from >= data.len() {
        return None;
    }
    data[from..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|at| at + from)
}

fn decode(data: &[u8]) -> Result<Vec<u8>, MovieError> {
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(data));
    decoder.decode().map_err(|_| MovieError)?;
    let image = decoder
        .get_image()
        .ok_or(MovieError)?
        .map_err(|_| MovieError)?;
    Ok(image.to_vec())
}

pub const SAMPLE_RATE: u32 = 32000;
pub const AUDIO_HEADER: usize = 0x50;
const TABLE_ONE: usize = 8;
const TABLE_TWO: usize = 40;
const SCALE_MIN: i32 = -2048;
const SCALE_MAX: i32 = 2047;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioBlock {
    pub samples: usize,
    pub bytes: usize,
}

/// Header of one audio block: channel byte count, sample count, two
/// 16-entry coefficient tables, then the four carried sample values.
pub fn audio_block(span: &[u8]) -> Option<AudioBlock> {
    if span.len() < AUDIO_HEADER {
        return None;
    }
    let bytes = be32(span, 0) as usize;
    let samples = be32(span, 4) as usize;
    if bytes == 0 || AUDIO_HEADER + bytes > span.len() {
        return None;
    }
    Some(AudioBlock { samples, bytes })
}

#[derive(Debug, Clone)]
pub struct Adpcm {
    predictor: [i32; 2],
    scale: [i32; 2],
}

impl Default for Adpcm {
    fn default() -> Self {
        Self {
            predictor: [0; 2],
            scale: [0; 2],
        }
    }
}

/// Decode one block, appending interleaved stereo samples.
pub fn adpcm_block(span: &[u8], state: &mut Adpcm, out: &mut Vec<i16>) {
    let Some(block) = audio_block(span) else {
        return;
    };
    let Some(data) = span.get(AUDIO_HEADER..AUDIO_HEADER + block.bytes) else {
        return;
    };
    for byte in data {
        for (channel, nibble) in [(0usize, byte >> 4), (1usize, byte & 0x0F)] {
            let ss = coefficient(span, TABLE_ONE, nibble);
            let delta = coefficient(span, TABLE_TWO, nibble);
            state.scale[channel] = (state.scale[channel] + delta).clamp(SCALE_MIN, SCALE_MAX);
            state.predictor[channel] =
                (state.predictor[channel] + ss * state.scale[channel]).clamp(SCALE_MIN, SCALE_MAX);
            let sample = (state.predictor[channel] * state.scale[channel]) >> 4;
            out.push(sample as i16);
        }
    }
}

fn coefficient(span: &[u8], table: usize, index: usize) -> i32 {
    let at = table + 2 * usize::from(index);
    match span.get(at..at + 2) {
        Some(bytes) => i16::from_be_bytes([bytes[0], bytes[1]]) as i32,
        None => 0,
    }
}