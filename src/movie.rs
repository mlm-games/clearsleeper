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
pub const SAMPLE_RATE: u32 = 32000;
const MAX_BLOCK_SAMPLES: usize = 0x10000;

const DEFAULT_RATE: f32 = 29.97;
const EOI: [u8; 2] = [0xFF, 0xD9];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovieError;

impl std::fmt::Display for MovieError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid or unreadable THP movie")
    }
}

impl std::error::Error for MovieError {}

struct Frame {
    at: u64,
    image: usize,
    audio: usize,
}

pub struct Movie {
    source: Box<dyn crate::disc::AssetStream>,
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

impl Movie {
    pub fn open(path: &Path) -> Result<Self, MovieError> {
        Self::parse(Box::new(File::open(path).map_err(|_| MovieError)?))
    }

    /// A movie over bytes already in memory, which a disc image has and a file path does not.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, MovieError> {
        Self::parse(Box::new(Cursor::new(bytes)))
    }

    pub(crate) fn parse(mut source: Box<dyn crate::disc::AssetStream>) -> Result<Self, MovieError> {
        let mut header = [0_u8; HEADER_LEN];
        source.read_exact(&mut header).map_err(|_| MovieError)?;
        if &header[..4] != b"THP\0" || be32(&header, 4) != 0x11000 {
            return Err(MovieError);
        }
        let rate = f32::from_bits(be32(&header, FRAME_RATE));
        let declared = be32(&header, FRAME_COUNT) as usize;
        let first = u64::from(be32(&header, FIRST_OFFSET));
        let last = u64::from(be32(&header, LAST_OFFSET));
        let end = source.seek(SeekFrom::End(0)).map_err(|_| MovieError)?;
        // tww/src/d/actor/d_a_movie_player.cpp:3482-3526: the retail reels have video then stereo audio.
        let mut components = [0; 6];
        source
            .seek(SeekFrom::Start(u64::from(be32(&header, 0x20))))
            .map_err(|_| MovieError)?;
        source.read_exact(&mut components).map_err(|_| MovieError)?;
        if components != [0, 0, 0, 2, 0, 1] || !rate.is_finite() || rate <= 0.0 || declared == 0 {
            return Err(MovieError);
        }
        let frames = index(&mut source, first, last, end, declared, be32(&header, 0x18))?;
        Ok(Self {
            source,
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
        if self.rate > 0.0 {
            self.rate
        } else {
            DEFAULT_RATE
        }
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
        let samples = be32(&span, 4) as usize;
        if samples == 0 || samples > MAX_BLOCK_SAMPLES {
            return Err(MovieError);
        }
        let split = AUDIO_HEADER + channel_bytes as usize;
        Ok(AudioBlock {
            samples,
            left: channel(&span, AUDIO_HEADER, LEFT_FILTER, LEFT_PREVIOUS, split),
            right: (channel_bytes > 0)
                .then(|| channel(&span, split, RIGHT_FILTER, RIGHT_PREVIOUS, len)),
        })
    }

    pub fn soundtrack(&mut self) -> Result<Vec<u8>, MovieError> {
        let mut samples = Vec::with_capacity(self.len() * 2048);
        for index in 0..self.len() {
            if let Ok(block) = self.audio(index) {
                block.decode(&mut samples);
            }
        }
        Ok(wave(&samples))
    }

    fn read(&mut self, at: u64, len: usize) -> Result<Vec<u8>, MovieError> {
        let mut buf = vec![0_u8; len];
        self.source
            .seek(SeekFrom::Start(at))
            .map_err(|_| MovieError)?;
        self.source.read_exact(&mut buf).map_err(|_| MovieError)?;
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

    fn next(&mut self) -> i16 {
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
        value
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
        let Some(length) = frame
            .get(at..at + 2)
            .map(|head| usize::from(u16::from_be_bytes([head[0], head[1]])))
        else {
            break;
        };
        let Some(segment) = frame.get(at..at + length) else {
            break;
        };
        out.extend_from_slice(segment);
        at += length;
        if marker == 0xDA {
            // tww/src/d/actor/d_a_movie_player.cpp:377-385: stuffing applies to the scan, not the end marker.
            let end = frame
                .iter()
                .rposition(|&b| b != 0 && b != 0xff)
                .unwrap_or(0);
            let scan_end = if end > at && frame.get(end - 1..=end) == Some(EOI.as_slice()) {
                end - 1
            } else {
                frame.len()
            };
            for byte in &frame[at..scan_end] {
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

fn index<R: Read + Seek>(
    file: &mut R,
    mut at: u64,
    last: u64,
    end: u64,
    declared: usize,
    mut size: u32,
) -> Result<Vec<Frame>, MovieError> {
    // tww/src/d/actor/d_a_movie_player.cpp:2719-2729,2788-2795: the final offset is a frame start.
    let mut header = [0_u8; FRAME_HEADER];
    let mut frames = Vec::with_capacity(declared.min(8192));
    for n in 0..declared {
        let next = at + u64::from(size);
        if size < FRAME_HEADER as u32
            || next > end
            || at > last
            || (n + 1 == declared) != (at == last)
        {
            return Err(MovieError);
        }
        file.seek(SeekFrom::Start(at)).map_err(|_| MovieError)?;
        file.read_exact(&mut header).map_err(|_| MovieError)?;
        let image = u64::from(be32(&header, 8));
        let audio = u64::from(be32(&header, 12));
        if image == 0 || FRAME_HEADER as u64 + image + audio > u64::from(size) {
            return Err(MovieError);
        }
        frames.push(Frame {
            at: at + FRAME_HEADER as u64,
            image: image as usize,
            audio: audio as usize,
        });
        at = next;
        size = be32(&header, 0);
    }
    Ok(frames)
}

/// The title movie's path under an extracted `CS_ASSETS` tree.
///
/// `daMP_c_Init` names `/thpdemo/title_loop.thp` for the title scene and
/// `/thpdemo/end_st_epilogue.thp` for the ending scene (`d_a_movie_player.cpp:4225-4228`),
/// and the title scene creates its movie player with the index that picks the first of them
/// (`d_s_title.cpp:122`). The reel is the attract movie of that scene, which hands the game
/// back to the title screen on a button press or when it runs out (`d_s_title.cpp:86-89`);
/// it is not the title screen itself, which is the 3D title scene with the J2D logo over it
/// (`d_a_title.cpp:626-631`).
///
/// A disc image has no path to hand a file handle, so the app reads the same entry through
/// [`crate::assets::AssetRoot::read`] and builds the movie with [`Movie::from_bytes`].
pub const TITLE_PATH: &str = "files/thpdemo/title_loop.thp";
// tww/src/d/actor/d_a_movie_player.cpp:4225-4228
pub const ENDING_PATH: &str = "files/thpdemo/end_st_epilogue.thp";

pub fn title_path() -> Option<PathBuf> {
    Some(
        crate::assets::AssetRoot::from_env()
            .ok()?
            .fs_root()?
            .join(TITLE_PATH),
    )
}

pub(crate) fn wave(samples: &[i16]) -> Vec<u8> {
    let data = u32::try_from(samples.len() * 2).unwrap_or(u32::MAX);
    let mut out = Vec::with_capacity(44 + data as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16_u32.to_le_bytes());
    out.extend_from_slice(&1_u16.to_le_bytes());
    out.extend_from_slice(&2_u16.to_le_bytes());
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 4).to_le_bytes());
    out.extend_from_slice(&4_u16.to_le_bytes());
    out.extend_from_slice(&16_u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    for sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

pub fn to_rgba(rgb: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(rgb.len() / 3 * 4);
    for pixel in rgb.chunks_exact(3) {
        out.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 0xFF]);
    }
    out
}
