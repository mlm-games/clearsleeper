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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovieError;

pub struct Movie {
    file: File,
    frames: Vec<u64>,
    rate: f32,
    declared: usize,
    last: usize,
    max_frame: usize,
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

    /// Decode one frame to packed 8-bit RGB.
    pub fn frame(&mut self, index: usize) -> Result<Vec<u8>, MovieError> {
        let start = *self.frames.get(index).ok_or(MovieError)?;
        let data = self.read_at(start, self.frame_span(index))?;
        let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(&data));
        decoder.decode().map_err(|_| MovieError)?;
        let image = decoder
            .get_image()
            .ok_or(MovieError)?
            .map_err(|_| MovieError)?;
        Ok(image.to_vec())
    }

    /// Raw bytes carried between this frame's image and the next frame.
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
        self.read_at(start, (end - start) as usize)
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
    pub samples: usize,
    pub bytes: usize,
}

/// The audio block is the tail of a frame's bytes; find where it starts.
pub fn audio_block(span: &[u8]) -> Option<AudioBlock> {
    for skip in 0..64_usize.min(span.len()) {
        let Some(at) = span.get(skip..) else {
            break;
        };
        if at.len() < AUDIO_HEADER {
            break;
        }
        let bytes = be32(at, 0) as usize;
        let samples = be32(at, 4) as usize;
        if bytes >= 32 && bytes + AUDIO_HEADER <= at.len() && samples > 0 && samples < 0x10000 {
            return Some(AudioBlock { samples, bytes });
        }
    }
    None
}

#[derive(Debug, Clone, Default)]
pub struct Adpcm {
    predictor: [i32; 2],
    scale: [i32; 2],
}

/// Decode one DSP-ADPCM block, appending interleaved stereo samples.
pub fn adpcm_block(span: &[u8], state: &mut Adpcm, out: &mut Vec<i16>) {
    let Some(block) = audio_block(span) else {
        return;
    };
    let Some(base) = block_base(span, block.bytes) else {
        return;
    };
    let Some(data) = span.get(base + AUDIO_HEADER..base + AUDIO_HEADER + block.bytes) else {
        return;
    };
    for byte in data {
        for (channel, nibble) in [(0_usize, byte >> 4), (1_usize, byte & 0x0F)] {
            let ss = coefficient(span, base, TABLE_ONE, nibble);
            let delta = coefficient(span, base, TABLE_TWO, nibble);
            state.scale[channel] = (state.scale[channel] + delta).clamp(SCALE_MIN, SCALE_MAX);
            state.predictor[channel] =
                (state.predictor[channel] + ss * state.scale[channel]).clamp(SCALE_MIN, SCALE_MAX);
            out.push(((state.predictor[channel] * state.scale[channel]) >> 4) as i16);
        }
    }
}

fn block_base(span: &[u8], bytes: usize) -> Option<usize> {
    (0..64_usize.min(span.len())).find(|skip| {
        let at = skip + AUDIO_HEADER;
        at + bytes <= span.len()
    })
}

fn coefficient(span: &[u8], base: usize, table: usize, index: usize) -> i32 {
    let at = base + table + 2 * usize::from(index);
    match span.get(at..at + 2) {
        Some(bytes) => i16::from_be_bytes([bytes[0], bytes[1]]) as i32,
        None => 0,
    }
}

/// Title loop shipped alongside the stage data.
pub fn title_path() -> std::path::PathBuf {
    assets::asset_root().join("files/thpdemo/title_loop.thp")
}

/// Expand packed RGB to the RGBA order the renderer uploads.
pub fn to_rgba(rgb: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(rgb.len() / 3 * 4);
    for pixel in rgb.chunks_exact(3) {
        out.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 0xFF]);
    }
    out
}