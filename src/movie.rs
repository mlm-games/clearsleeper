use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub const HEADER_LEN: usize = 0x60;
pub const MAX_FRAME: usize = 0x08;
pub const MAX_SAMPLES: usize = 0x0C;
pub const FRAME_RATE: usize = 0x10;
pub const FRAME_COUNT: usize = 0x14;
pub const FIRST_OFFSET: usize = 0x28;
pub const LAST_OFFSET: usize = 0x2C;

pub const FRAME_HEADER: usize = 0x10;
pub const AUDIO_HEADER: usize = 0x50;
pub const COEFFICIENT: usize = 0x08;
pub const SCALE: usize = 0x28;
pub const PREVIOUS: usize = 0x48;

const DEFAULT_RATE: f32 = 29.97;
const SOI: [u8; 3] = [0xFF, 0xD8, 0xFF];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovieError;

struct Frame {
    at: u64,
    image: usize,
    audio: usize,
}

pub struct Movie {
    file: File,
    frames: Vec<Frame>,
    rate: f32,
    declared: usize,
}

fn be32(data: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]])
}

fn i16_at(data: &[u8], at: usize) -> i16 {
    match data.get(at..at + 2) {
        Some(bytes) => i16::from_be_bytes([bytes[0], bytes[1]]),
        None => 0,
    }
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
        let first = be32(&header, FIRST_OFFSET) as usize;
        let last = be32(&header, LAST_OFFSET) as usize;
        let frames = index(&mut file, first, last, declared)?;
        if frames.is_empty() {
            return Err(MovieError);
        }
        Ok(Self {
            file,
            frames,
            rate,
            declared,
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

    pub fn frame(&mut self, index: usize) -> Result<(u32, u32, Vec<u8>), MovieError> {
        let at = self.frames.get(index).ok_or(MovieError)?.at;
        let span = self.read(at, self.frames[index].image.max(1))?;
        let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(&span));
        let pixels = decoder.decode().map_err(|_| MovieError)?;
        let info = decoder.info().ok_or(MovieError)?;
        if info.pixel_format != jpeg_decoder::PixelFormat::RGB24 {
            return Err(MovieError);
        }
        Ok((u32::from(info.width), u32::from(info.height), pixels))
    }

    pub fn audio(&mut self, index: usize) -> Result<AudioBlock, MovieError> {
        let frame = self.frames.get(index).ok_or(MovieError)?;
        let (at, len) = (frame.at + frame.image as u64, frame.audio);
        if len <= AUDIO_HEADER {
            return Err(MovieError);
        }
        let span = self.read(at, len)?;
        let channel_bytes = be32(&span, 0) as usize;
        if AUDIO_HEADER + 2 * channel_bytes != len {
            return Err(MovieError);
        }
        let split = AUDIO_HEADER + channel_bytes;
        Ok(AudioBlock {
            samples: be32(&span, 4) as usize,
            coefficient: table(&span, COEFFICIENT),
            scale: table(&span, SCALE),
            previous: [
                [i16_at(&span, PREVIOUS), i16_at(&span, PREVIOUS + 2)],
                [i16_at(&span, PREVIOUS + 4), i16_at(&span, PREVIOUS + 6)],
            ],
            channels: [span[AUDIO_HEADER..split].to_vec(), span[split..].to_vec()],
        })
    }

    fn read(&mut self, at: u64, len: usize) -> Result<Vec<u8>, MovieError> {
        let mut buf = vec![0_u8; len];
        self.file
            .seek(SeekFrom::Start(at))
            .map_err(|_| MovieError)?;
        self.file.read_exact(&mut buf).map_err(|_| MovieError)?;
        Ok(buf)
    }
}

pub struct AudioBlock {
    pub samples: usize,
    pub coefficient: [i16; 16],
    pub scale: [i16; 16],
    pub previous: [[i16; 2]; 2],
    pub channels: [Vec<u8>; 2],
}

fn index(file: &mut File, first: usize, last: usize, declared: usize) -> Result<Vec<Frame>, MovieError> {
    let mut marks = Vec::with_capacity(declared.min(8192));
    let mut at = first as u64;
    let end = (last as u64).max(at);
    let mut window = vec![0_u8; 1 << 16];
    while marks.len() < declared && at < end {
        let want = usize::try_from(end - at).unwrap_or(window.len()).min(window.len());
        file.seek(SeekFrom::Start(at)).map_err(|_| MovieError)?;
        let read = file.read(&mut window[..want]).map_err(|_| MovieError)?;
        if read < SOI.len() {
            break;
        }
        match find(&window[..read], 0, &SOI) {
            Some(hit) => {
                marks.push(at + hit as u64);
                at += hit as u64 + SOI.len() as u64;
            }
            None => at += (read - SOI.len() + 1) as u64,
        }
    }

    let mut header = [0_u8; FRAME_HEADER];
    let mut frames = Vec::with_capacity(marks.len());
    for (index, at) in marks.iter().enumerate() {
        let bound = marks
            .get(index + 1)
            .copied()
            .unwrap_or(last as u64)
            .min(last as u64);
        if *at < first as u64 + FRAME_HEADER as u64 || *at + FRAME_HEADER as u64 > bound {
            continue;
        }
        file.seek(SeekFrom::Start(at - FRAME_HEADER as u64))
            .map_err(|_| MovieError)?;
        file.read_exact(&mut header).map_err(|_| MovieError)?;
        let image = be32(&header, 8) as usize;
        let audio = be32(&header, 12) as usize;
        if image == 0 || *at + (image + audio) as u64 > bound {
            continue;
        }
        frames.push(Frame {
            at: *at,
            image,
            audio,
        });
    }
    Ok(frames)
}

fn table(span: &[u8], at: usize) -> [i16; 16] {
    std::array::from_fn(|index| i16_at(span, at + 2 * index))
}

pub fn title_path() -> PathBuf {
    crate::assets::asset_root().join("files/thpdemo/title_loop.thp")
}

pub fn to_rgba(rgb: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(rgb.len() / 3 * 4);
    for pixel in rgb.chunks_exact(3) {
        out.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 0xFF]);
    }
    out
}