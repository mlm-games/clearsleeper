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
pub const LEFT_FILTER: usize = 0x08;
pub const RIGHT_FILTER: usize = 0x28;
pub const LEFT_PREVIOUS: usize = 0x48;
pub const RIGHT_PREVIOUS: usize = 0x4C;

const DEFAULT_RATE: f32 = 29.97;
const SOI: [u8; 3] = [0xFF, 0xD8, 0xFF];
const EOI: [u8; 2] = [0xFF, 0xD9];

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
        let span = restuff(&span);
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
        let channel_bytes = u64::from(be32(&span, 0));
        if u64::try_from(AUDIO_HEADER).unwrap_or(0) + 2 * channel_bytes != len as u64 {
            return Err(MovieError);
        }
        let split = AUDIO_HEADER + channel_bytes as usize;
        Ok(AudioBlock {
            samples: be32(&span, 4) as usize,
            left: channel(&span, AUDIO_HEADER, LEFT_FILTER, LEFT_PREVIOUS, split),
            right: (channel_bytes > 0)
                .then(|| channel(&span, split, RIGHT_FILTER, RIGHT_PREVIOUS, len)),
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
    pub left: Channel,
    pub right: Option<Channel>,
}

pub struct Channel {
    filter: [[i16; 2]; 8],
    previous: [i16; 2],
    encoded: Vec<u8>,
}

struct Nibbles<'a> {
    data: &'a [u8],
    at: usize,
    offset: u32,
    predictor: usize,
    scale: u32,
}

struct Voice<'a> {
    filter: [[i16; 2]; 8],
    previous: [i16; 2],
    nibbles: Nibbles<'a>,
}

fn sign_extend(value: u8) -> i32 {
    let value = i32::from(value & 0x0F);
    value - if value & 0x08 != 0 { 0x10 } else { 0 }
}

impl<'a> Voice<'a> {
    fn new(channel: &'a Channel) -> Self {
        let first = channel.encoded.first().copied().unwrap_or(0);
        Self {
            filter: channel.filter,
            previous: channel.previous,
            nibbles: Nibbles {
                data: &channel.encoded,
                at: 1,
                offset: 2,
                predictor: usize::from((first & 0x70) >> 4),
                scale: u32::from(first & 0x0F),
            },
        }
    }

    fn next(&mut self) -> i32 {
        if self.nibbles.offset & 0x0F == 0 {
            let byte = self.nibbles.data.get(self.nibbles.at).copied().unwrap_or(0);
            self.nibbles.predictor = usize::from((byte & 0x70) >> 4);
            self.nibbles.scale = u32::from(byte & 0x0F);
            self.nibbles.at += 1;
            self.nibbles.offset += 2;
        }
        let byte = self.nibbles.data.get(self.nibbles.at).copied().unwrap_or(0);
        let delta = if self.nibbles.offset & 1 == 1 {
            self.nibbles.at += 1;
            sign_extend(byte)
        } else {
            sign_extend(byte >> 4)
        };
        self.nibbles.offset += 1;

        let pair = self.filter[self.nibbles.predictor & 7];
        let mut yn = i64::from(pair[1]) * i64::from(self.previous[1]);
        yn += i64::from(pair[0]) * i64::from(self.previous[0]);
        yn += (i64::from(delta) << self.nibbles.scale) << 11;
        yn <<= 5;
        let half = yn & 0xFFFF;
        if half > 0x8000 || (half == 0x8000 && yn & 0x10000 != 0) {
            yn += 0x10000;
        }
        let value = yn.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) >> 16;
        let value = value as i16;
        self.previous = [value, self.previous[0]];
        i32::from(value)
    }
}

impl AudioBlock {
    pub fn decode(&self, out: &mut Vec<i16>) {
        let mut left = Voice::new(&self.left);
        let mut right = self.right.as_ref().map(Voice::new);
        out.reserve(self.samples * 2);
        for _ in 0..self.samples {
            let first = left.next();
            match right.as_mut() {
                Some(voice) => {
                    out.push(first);
                    out.push(voice.next());
                }
                None => {
                    out.push(first);
                    out.push(first);
                }
            }
        }
    }
}

fn restuff(frame: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(frame.len() + frame.len() / 16);
    let mut at = 0;
    while at < frame.len() {
        if frame[at] != 0xFF {
            break;
        }
        while at < frame.len() && frame[at] == 0xFF {
            at += 1;
        }
        let Some(&marker) = frame.get(at) else {
            break;
        };
        out.extend_from_slice(&[0xFF, marker]);
        at += 1;
        if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) {
            continue;
        }
        let Some(length) = frame.get(at..at + 2).map(|head| {
            usize::from(u16::from_be_bytes([head[0], head[1]]))
        }) else {
            break;
        };
        let Some(segment) = frame.get(at..at + length) else {
            break;
        };
        out.extend_from_slice(segment);
        at += length;
        if marker == 0xDA {
            for byte in &frame[at..] {
                out.push(*byte);
                if *byte == 0xFF {
                    out.push(0x00);
                }
            }
            out.extend_from_slice(&EOI);
            return out;
        }
    }
    out
}

fn channel(span: &[u8], from: usize, filter: usize, previous: usize, to: usize) -> Channel {
    Channel {
        filter: std::array::from_fn(|pair| {
            [
                i16_at(span, filter + 4 * pair),
                i16_at(span, filter + 4 * pair + 2),
            ]
        }),
        previous: [i16_at(span, previous), i16_at(span, previous + 2)],
        encoded: span.get(from..to).unwrap_or_default().to_vec(),
    }
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
        if *at < FRAME_HEADER as u64 || *at + FRAME_HEADER as u64 > bound {
            continue;
        }
        file.seek(SeekFrom::Start(at - FRAME_HEADER as u64))
            .map_err(|_| MovieError)?;
        file.read_exact(&mut header).map_err(|_| MovieError)?;
        let image = u64::from(be32(&header, 8));
        let audio = u64::from(be32(&header, 12));
        if image == 0 || *at + image + audio > bound {
            continue;
        }
        frames.push(Frame {
            at: *at,
            image: image as usize,
            audio: audio as usize,
        });
    }
    Ok(frames)
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