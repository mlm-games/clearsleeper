use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

pub const HEADER_LEN: usize = 0x60;
pub const MAX_FRAME: usize = 0x08;
pub const MAX_SAMPLES: usize = 0x0C;
pub const FRAME_RATE: usize = 0x10;
pub const FRAME_COUNT: usize = 0x14;
pub const FIRST_OFFSET: usize = 0x28;
pub const LAST_OFFSET: usize = 0x2C;

pub const SAMPLE_RATE: u32 = 32000;
pub const AUDIO_HEADER: usize = 0x50;
pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;
const TABLE_ONE: usize = 8;
const TABLE_TWO: usize = 40;
const SCALE_MIN: i32 = -2048;
const SCALE_MAX: i32 = 2047;
const DEFAULT_RATE: f32 = 29.97;

const SOI: [u8; 3] = [0xFF, 0xD8, 0xFF];
const EOI: [u8; 2] = [0xFF, 0xD9];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovieError;

pub struct Movie {
    file: File,
    frames: Vec<u64>,
    rate: f32,
    declared: usize,
    last: usize,
    max_frame: usize,
    max_samples: usize,
}

fn be32(data: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]])
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

impl Movie {
    pub fn open(path: &Path) -> Result<Self, MovieError> {
        let mut file = File::open(path).map_err(|_| MovieError)?;
        let mut header = [0_u8; HEADER_LEN];
        file.read_exact(&mut header).map_err(|_| MovieError)?;
        if &header[..4] != b"THP\0" {
            return Err(MovieError);
        }
        let rate = f32::from_bits(be32(&header, FRAME_RATE));
        let declared = be32(&header, FRAME_COUNT) as usize;
        let last = be32(&header, LAST_OFFSET) as usize;
        let first = be32(&header, FIRST_OFFSET) as usize;
        let max_frame = be32(&header, MAX_FRAME) as usize;
        let max_samples = be32(&header, MAX_SAMPLES) as usize;
        let frames = scan(&mut file, first, declared, last)?;
        if frames.is_empty() {
            return Err(MovieError);
        }
        Ok(Self {
            file,
            frames,
            rate,
            declared,
            last,
            max_frame,
            max_samples,
        })
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn declared_frames(&self) -> usize {
        self.declared
    }

    pub fn rate(&self) -> f32 {
        if self.rate > 0.0 { self.rate } else { DEFAULT_RATE }
    }

    pub fn interval(&self) -> f32 {
        1.0 / self.rate()
    }

    pub fn max_frame_bytes(&self) -> usize {
        self.max_frame
    }

    pub fn max_samples(&self) -> usize {
        self.max_samples
    }

    /// Decode one frame to packed 8-bit RGB with its pixel dimensions.
    pub fn frame(&mut self, index: usize) -> Result<(u32, u32, Vec<u8>), MovieError> {
        let start = *self.frames.get(index).ok_or(MovieError)?;
        let data = self.read_at(start, self.frame_span(index))?;
        let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(&data));
        let pixels = decoder.decode().map_err(|_| MovieError)?;
        let info = decoder.info().ok_or(MovieError)?;
        if info.pixel_format != jpeg_decoder::PixelFormat::RGB24 {
            return Err(MovieError);
        }
        Ok((u32::from(info.width), u32::from(info.height), pixels))
    }

    /// Bytes trailing this frame's image, starting at the audio block header.
    pub fn audio(&mut self, index: usize) -> Result<Vec<u8>, MovieError> {
        let start = *self.frames.get(index).ok_or(MovieError)?;
        let end = self
            .frames
            .get(index + 1)
            .copied()
            .unwrap_or(self.last as u64)
            .min(self.last as u64);
        if end <= start {
            return Err(MovieError);
        }
        let span = self.read_at(start, (end - start) as usize)?;
        let tail = find(&span, 0, &EOI).map_or(span.len(), |at| at + EOI.len());
        Ok(span[tail..].to_vec())
    }

    fn frame_span(&self, index: usize) -> usize {
        let start = self.frames[index] as usize;
        let end = self
            .frames
            .get(index + 1)
            .map_or(self.last, |next| *next as usize)
            .min(self.last);
        end.saturating_sub(start).max(1024)
    }

    fn read_at(&mut self, at: u64, len: usize) -> Result<Vec<u8>, MovieError> {
        let mut buf = vec![0_u8; len];
        self.file
            .seek(SeekFrom::Start(at))
            .map_err(|_| MovieError)?;
        self.file.read_exact(&mut buf).map_err(|_| MovieError)?;
        Ok(buf)
    }
}

fn scan(file: &mut File, first: usize, count: usize, last: usize) -> Result<Vec<u64>, MovieError> {
    let mut out = Vec::with_capacity(count.min(8192));
    let mut at = first as u64;
    let end = (last as u64).max(at);
    let mut window = vec![0_u8; 1 << 16];
    while out.len() < count && at < end {
        let want = ((end - at) as usize).min(window.len());
        file.seek(SeekFrom::Start(at)).map_err(|_| MovieError)?;
        let read = file.read(&mut window[..want]).map_err(|_| MovieError)?;
        if read < 3 {
            break;
        }
        let chunk = &window[..read];
        let Some(hit) = find(chunk, 0, &SOI) else {
            at += (read - 2) as u64;
            continue;
        };
        out.push(at + hit as u64);
        at += hit as u64 + 3;
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioBlock {
    pub at: usize,
    pub samples: usize,
    pub bytes: usize,
}

/// One DSP-ADPCM block header: channel byte count, sample count, two
/// 16-entry coefficient tables, then the samples as 4-bit deltas.
pub fn audio_block(span: &[u8], max_samples: usize) -> Option<AudioBlock> {
    (0..16_usize.min(span.len())).find_map(|at| {
        let block = span.get(at..)?;
        if block.len() < AUDIO_HEADER {
            return None;
        }
        let bytes = be32(block, 0) as usize;
        let samples = be32(block, 4) as usize;
        let cap = if max_samples == 0 { 1 << 16 } else { max_samples };
        if bytes < 32 || AUDIO_HEADER + bytes > block.len() || samples == 0 || samples > cap {
            return None;
        }
        if (0..16).any(|index| read_i16(block, TABLE_ONE + 2 * index).abs() > SCALE_MAX) {
            return None;
        }
        Some(AudioBlock { at, samples, bytes })
    })
}

fn read_i16(data: &[u8], at: usize) -> i32 {
    match data.get(at..at + 2) {
        Some(bytes) => i16::from_be_bytes([bytes[0], bytes[1]]) as i32,
        None => 0,
    }
}

#[derive(Debug, Clone, Default)]
pub struct Adpcm {
    predictor: [i32; 2],
    scale: [i32; 2],
}

/// Decode one DSP-ADPCM block, appending interleaved stereo samples.
pub fn adpcm_block(span: &[u8], max_samples: usize, state: &mut Adpcm, out: &mut Vec<i16>) {
    let Some(block) = audio_block(span, max_samples) else {
        return;
    };
    let Some(data) = span.get(
        block.at + AUDIO_HEADER..block.at + AUDIO_HEADER + block.bytes,
    ) else {
        return;
    };
    for byte in data {
        for (channel, nibble) in [(0_usize, byte >> 4), (1_usize, byte & 0x0F)] {
            let ss = read_i16(span, block.at + TABLE_ONE + 2 * usize::from(nibble));
            let delta = read_i16(span, block.at + TABLE_TWO + 2 * usize::from(nibble));
            state.scale[channel] = (state.scale[channel] + delta).clamp(SCALE_MIN, SCALE_MAX);
            state.predictor[channel] =
                (state.predictor[channel] + ss * state.scale[channel]).clamp(SCALE_MIN, SCALE_MAX);
            out.push(((state.predictor[channel] * state.scale[channel]) >> 4) as i16);
        }
    }
}

/// Title loop shipped alongside the stage data.
pub fn title_path() -> std::path::PathBuf {
    crate::assets::asset_root().join("files/thpdemo/title_loop.thp")
}

/// Expand packed RGB to the RGBA order the renderer uploads.
pub fn to_rgba(rgb: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(rgb.len() / 3 * 4);
    for pixel in rgb.chunks_exact(3) {
        out.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 0xFF]);
    }
    out
}