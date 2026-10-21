//! The Dolphin FIFO-log container: one `.dff` per recorded video, holding the stream of
//! graphics commands a frame ran plus the GPU memory those commands replayed against.
//!
//! The layout is a 128-byte header naming each section, the four register and memory images,
//! a frame list of 64-byte entries and, behind every entry, that frame's command stream and
//! its out-of-band memory updates. Offsets are absolute from the start of the file, so this
//! reads the header and then jumps section to section, which is how
//! `gecko/crates/dff/src/lib.rs:197` walks it.
//!
//! Every length and offset is checked before use: a damaged file returns an error rather than
//! reading out of bounds, and a table count is validated against the bytes that are actually
//! left so a corrupt header cannot ask for an unbounded walk.
//!
//! This is a data reader and sits below the game the way [`crate::archive`] does, so nothing
//! here may name anything from [`crate::sim`], [`crate::ui`] or [`crate::assets`].

use std::fmt;
use std::io;
use std::path::Path;

/// Container magic, `gecko/crates/dff/src/lib.rs:4`.
pub const FILE_ID: u32 = 0x0d01_f1f0;
/// Version this loader reads and writes, `gecko/crates/dff/src/lib.rs:5`.
pub const VERSION_NUMBER: u32 = 6;
/// Oldest loader a written file stays readable by, `gecko/crates/dff/src/lib.rs:6`.
pub const MIN_LOADER_VERSION: u32 = 1;
/// Blitting processor registers, `gecko/crates/dff/src/lib.rs:8`.
pub const BP_MEM_SIZE: usize = 256;
/// Command processor registers, `gecko/crates/dff/src/lib.rs:9`.
pub const CP_MEM_SIZE: usize = 256;
/// Transform unit memory, `gecko/crates/dff/src/lib.rs:10`.
pub const XF_MEM_SIZE: usize = 4096;
/// Transform unit registers as Dolphin holds them, `gecko/crates/dff/src/lib.rs:11`.
pub const XF_REGS_SIZE: usize = 88;
/// Embedded texture memory, `gecko/crates/dff/src/lib.rs:12`.
pub const TEX_MEM_SIZE: usize = 1024 * 1024;
/// Main memory of a retail GameCube, `gecko/crates/dff/src/lib.rs:14`.
pub const MEM1_SIZE_RETAIL: u32 = 0x0180_0000;
/// Extra memory, `gecko/crates/dff/src/lib.rs:15`.
pub const MEM2_SIZE_RETAIL: u32 = 0x0400_0000;

const FLAG_IS_WII: u32 = 1;
const HEADER_SIZE: usize = 128;
const FRAME_INFO_SIZE: usize = 64;
const MEMORY_UPDATE_SIZE: usize = 24;
const GAME_ID_LEN: usize = 8;
const DEFAULT_GAME_ID: &str = "00000000";
const TEX_MEM_SECTION_VERSION: u32 = 4;
const MEM_SIZE_VERSION: u32 = 5;
const GAME_ID_VERSION: u32 = 6;

/// What an out-of-band memory update carries, `gecko/crates/dff/src/lib.rs:23`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryUpdateType {
    /// A texture map, the payload a command stream would not carry itself.
    TextureMap = 0x01,
    /// Transform data for the vertices that follow.
    XfData = 0x02,
    /// A vertex stream loaded outside the command stream.
    VertexStream = 0x04,
    /// Texture memory.
    Tmem = 0x08,
}

impl MemoryUpdateType {
    /// The value on disk. An unnamed byte reads as [`Self::TextureMap`], which is what
    /// `gecko/crates/dff/src/lib.rs:32` does with a kind it does not know.
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            0x02 => Self::XfData,
            0x04 => Self::VertexStream,
            0x08 => Self::Tmem,
            _ => Self::TextureMap,
        }
    }
}

/// One memory write a frame needed, `gecko/crates/dff/src/lib.rs:42`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryUpdate {
    /// Where in the frame's command stream the write happens.
    pub fifo_position: u32,
    /// Address the write lands at.
    pub address: u32,
    /// What is written.
    pub kind: MemoryUpdateType,
    /// The payload itself.
    pub data: Vec<u8>,
}

/// One recorded frame, `gecko/crates/dff/src/lib.rs:50`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frame {
    /// The command stream, raw FIFO bytes.
    pub fifo_data: Vec<u8>,
    /// Where the frame's write pointer starts.
    pub fifo_start: u32,
    /// Where the frame's write pointer ends.
    pub fifo_end: u32,
    /// Memory writes the stream itself did not carry.
    pub memory_updates: Vec<MemoryUpdate>,
}

/// A whole recording: the memory images plus every frame that ran against them,
/// `gecko/crates/dff/src/lib.rs:58`.
#[derive(Debug, Clone)]
pub struct DffFile {
    /// Blitting processor registers.
    pub bp_mem: Vec<u32>,
    /// Command processor registers.
    pub cp_mem: Vec<u32>,
    /// Transform unit memory.
    pub xf_mem: Vec<u32>,
    /// Transform unit registers.
    pub xf_regs: Vec<u32>,
    /// Embedded texture memory.
    pub tex_mem: Vec<u8>,
    /// Whether the recording came from a Wii.
    pub is_wii: bool,
    /// Size of main memory, or the retail size when the version carries none.
    pub mem1_size: u32,
    /// Size of the extra memory, or the retail size when the version carries none.
    pub mem2_size: u32,
    /// The game's id, `00000000` when the version carries none.
    pub game_id: String,
    /// Every recorded frame, in order.
    pub frames: Vec<Frame>,
}

impl Default for DffFile {
    fn default() -> Self {
        Self {
            bp_mem: vec![0; BP_MEM_SIZE],
            cp_mem: vec![0; CP_MEM_SIZE],
            xf_mem: vec![0; XF_MEM_SIZE],
            xf_regs: vec![0; XF_REGS_SIZE],
            tex_mem: vec![0; TEX_MEM_SIZE],
            is_wii: false,
            mem1_size: MEM1_SIZE_RETAIL,
            mem2_size: MEM2_SIZE_RETAIL,
            game_id: DEFAULT_GAME_ID.to_string(),
            frames: Vec::new(),
        }
    }
}

/// Why a recording could not be read, `gecko/crates/dff/src/lib.rs:91`.
#[derive(Debug)]
pub enum DffError {
    /// The file could not be read from disk.
    Io(io::Error),
    /// The magic is not `FILE_ID`.
    BadMagic(u32),
    /// The file needs a loader newer than `VERSION_NUMBER`.
    UnsupportedVersion(u32),
    /// A section runs past the end of the file.
    Truncated,
}

impl fmt::Display for DffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "dff io error: {e}"),
            Self::BadMagic(magic) => {
                write!(f, "bad dff magic {magic:#010x}, expected {FILE_ID:#010x}")
            }
            Self::UnsupportedVersion(version) => write!(
                f,
                "dff min loader version {version} exceeds supported {VERSION_NUMBER}"
            ),
            Self::Truncated => write!(f, "dff file truncated"),
        }
    }
}

impl std::error::Error for DffError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for DffError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl DffFile {
    /// Write a recording to disk, in the layout [`Self::from_bytes`] reads.
    pub fn save(&self, path: &Path) -> Result<(), DffError> {
        std::fs::write(path, self.to_bytes())?;
        Ok(())
    }

    /// Read a recording from disk.
    pub fn load(path: &Path) -> Result<Self, DffError> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes)
    }

    /// Read a recording from a buffer already in memory.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DffError> {
        let header = Header::parse(bytes)?;
        let frame_list_at =
            usize::try_from(header.frame_list_offset).map_err(|_| DffError::Truncated)?;
        let frame_count = table_count(bytes, frame_list_at, header.frame_count, FRAME_INFO_SIZE)?;

        let mut file = DffFile {
            is_wii: header.flags & FLAG_IS_WII != 0,
            ..DffFile::default()
        };

        if header.version >= MEM_SIZE_VERSION {
            file.mem1_size = header.mem1_size;
            file.mem2_size = header.mem2_size;
        }
        if header.version >= GAME_ID_VERSION {
            file.game_id = game_id(&header.game_id);
        }

        file.bp_mem = read_words(bytes, header.bp_mem, BP_MEM_SIZE)?;
        file.cp_mem = read_words(bytes, header.cp_mem, CP_MEM_SIZE)?;
        file.xf_mem = read_words(bytes, header.xf_mem, XF_MEM_SIZE)?;
        file.xf_regs = read_words(bytes, header.xf_regs, XF_REGS_SIZE)?;

        if header.version >= TEX_MEM_SECTION_VERSION {
            let len = u64::from(header.tex_mem.size).min(TEX_MEM_SIZE as u64);
            let source = slice_at(bytes, header.tex_mem.offset, len)?;
            file.tex_mem[..source.len()].copy_from_slice(source);
        }

        let mut frames = Vec::new();
        for index in 0..frame_count {
            let entry = frame_list_at + index * FRAME_INFO_SIZE;
            frames.push(read_frame(bytes, entry)?);
        }
        file.frames = frames;
        Ok(file)
    }

    /// Serialize in the layout [`Self::from_bytes`] reads, `gecko/crates/dff/src/lib.rs:131`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let frame_list_offset = HEADER_SIZE as u64;
        let mut out = vec![0_u8; HEADER_SIZE + self.frames.len() * FRAME_INFO_SIZE];

        let bp_mem_offset = append_words(&mut out, &self.bp_mem, BP_MEM_SIZE);
        let cp_mem_offset = append_words(&mut out, &self.cp_mem, CP_MEM_SIZE);
        let xf_mem_offset = append_words(&mut out, &self.xf_mem, XF_MEM_SIZE);
        let xf_regs_offset = append_words(&mut out, &self.xf_regs, XF_REGS_SIZE);

        let tex_mem_offset = out.len() as u64;
        out.extend_from_slice(&self.tex_mem[..self.tex_mem.len().min(TEX_MEM_SIZE)]);
        out.resize(tex_mem_offset as usize + TEX_MEM_SIZE, 0);

        for (index, frame) in self.frames.iter().enumerate() {
            let fifo_data_offset = out.len() as u64;
            out.extend_from_slice(&frame.fifo_data);

            let updates_offset = out.len() as u64;
            out.resize(
                out.len() + frame.memory_updates.len() * MEMORY_UPDATE_SIZE,
                0,
            );

            for (slot, update) in frame.memory_updates.iter().enumerate() {
                let data_offset = out.len() as u64;
                out.extend_from_slice(&update.data);

                let entry = updates_offset as usize + slot * MEMORY_UPDATE_SIZE;
                out[entry..entry + 4].copy_from_slice(&update.fifo_position.to_le_bytes());
                out[entry + 4..entry + 8].copy_from_slice(&update.address.to_le_bytes());
                out[entry + 8..entry + 16].copy_from_slice(&data_offset.to_le_bytes());
                let data_len = len32(update.data.len());
                out[entry + 16..entry + 20].copy_from_slice(&data_len.to_le_bytes());
                out[entry + 20] = update.kind as u8;
            }

            let entry = frame_list_offset as usize + index * FRAME_INFO_SIZE;
            out[entry..entry + 8].copy_from_slice(&fifo_data_offset.to_le_bytes());
            let fifo_len = len32(frame.fifo_data.len());
            out[entry + 8..entry + 12].copy_from_slice(&fifo_len.to_le_bytes());
            out[entry + 12..entry + 16].copy_from_slice(&frame.fifo_start.to_le_bytes());
            out[entry + 16..entry + 20].copy_from_slice(&frame.fifo_end.to_le_bytes());
            out[entry + 20..entry + 28].copy_from_slice(&updates_offset.to_le_bytes());
            let update_count = len32(frame.memory_updates.len());
            out[entry + 28..entry + 32].copy_from_slice(&update_count.to_le_bytes());
        }

        let mut header = HeaderWriter::new(&mut out);
        header.u32(FILE_ID);
        header.u32(VERSION_NUMBER);
        header.u32(MIN_LOADER_VERSION);
        header.u64(bp_mem_offset);
        header.u32(len32(BP_MEM_SIZE));
        header.u64(cp_mem_offset);
        header.u32(len32(CP_MEM_SIZE));
        header.u64(xf_mem_offset);
        header.u32(len32(XF_MEM_SIZE));
        header.u64(xf_regs_offset);
        header.u32(len32(XF_REGS_SIZE));
        header.u64(frame_list_offset);
        header.u32(len32(self.frames.len()));
        header.u32(if self.is_wii { FLAG_IS_WII } else { 0 });
        header.u64(tex_mem_offset);
        header.u32(len32(TEX_MEM_SIZE));
        header.u32(self.mem1_size);
        header.u32(self.mem2_size);
        header.game_id(&self.game_id);

        out
    }
}

/// A named run of bytes in the file.
#[derive(Debug, Clone, Copy)]
struct Section {
    offset: u64,
    size: u32,
}

/// The header as it sits on disk, in the order `gecko/crates/dff/src/lib.rs:200` reads it.
#[derive(Debug, Clone, Copy)]
struct Header {
    version: u32,
    bp_mem: Section,
    cp_mem: Section,
    xf_mem: Section,
    xf_regs: Section,
    /// Offset only: the format has no size field here, the count is separate.
    frame_list_offset: u64,
    frame_count: u32,
    flags: u32,
    tex_mem: Section,
    mem1_size: u32,
    mem2_size: u32,
    game_id: [u8; GAME_ID_LEN],
}

impl Header {
    fn parse(bytes: &[u8]) -> Result<Self, DffError> {
        if bytes.len() < HEADER_SIZE {
            return Err(DffError::Truncated);
        }
        let mut reader = Reader { bytes, pos: 0 };

        let magic = reader.u32()?;
        if magic != FILE_ID {
            return Err(DffError::BadMagic(magic));
        }

        let version = reader.u32()?;
        let min_loader_version = reader.u32()?;
        if min_loader_version > VERSION_NUMBER {
            return Err(DffError::UnsupportedVersion(min_loader_version));
        }

        Ok(Self {
            version,
            bp_mem: Section {
                offset: reader.u64()?,
                size: reader.u32()?,
            },
            cp_mem: Section {
                offset: reader.u64()?,
                size: reader.u32()?,
            },
            xf_mem: Section {
                offset: reader.u64()?,
                size: reader.u32()?,
            },
            xf_regs: Section {
                offset: reader.u64()?,
                size: reader.u32()?,
            },
            frame_list_offset: reader.u64()?,
            frame_count: reader.u32()?,
            flags: reader.u32()?,
            tex_mem: Section {
                offset: reader.u64()?,
                size: reader.u32()?,
            },
            mem1_size: reader.u32()?,
            mem2_size: reader.u32()?,
            game_id: game_id_bytes(reader.take(GAME_ID_LEN)?),
        })
    }
}

fn read_frame(bytes: &[u8], entry: usize) -> Result<Frame, DffError> {
    let mut reader = Reader::at(bytes, entry);
    let fifo_data_offset = reader.u64()?;
    let fifo_len = u64::from(reader.u32()?);
    let fifo_start = reader.u32()?;
    let fifo_end = reader.u32()?;
    let updates_offset = reader.u64()?;
    let update_count = reader.u32()?;

    let updates_at = usize::try_from(updates_offset).map_err(|_| DffError::Truncated)?;
    let update_count = table_count(bytes, updates_at, update_count, MEMORY_UPDATE_SIZE)?;

    let mut frame = Frame {
        fifo_data: slice_at(bytes, fifo_data_offset, fifo_len)?.to_vec(),
        fifo_start,
        fifo_end,
        memory_updates: Vec::new(),
    };
    for index in 0..update_count {
        let entry = updates_at + index * MEMORY_UPDATE_SIZE;
        frame.memory_updates.push(read_update(bytes, entry)?);
    }
    Ok(frame)
}

fn read_update(bytes: &[u8], entry: usize) -> Result<MemoryUpdate, DffError> {
    let mut reader = Reader::at(bytes, entry);
    let fifo_position = reader.u32()?;
    let address = reader.u32()?;
    let data_offset = reader.u64()?;
    let data_len = u64::from(reader.u32()?);
    let kind = MemoryUpdateType::from_raw(reader.u8()?);

    Ok(MemoryUpdate {
        fifo_position,
        address,
        kind,
        data: slice_at(bytes, data_offset, data_len)?.to_vec(),
    })
}

/// Read a section of words into a fixed-size register image. A section shorter than the image
/// leaves the rest zeroed and one longer than it is cut off, which is how
/// `gecko/crates/dff/src/lib.rs:311` bounds the copy.
fn read_words(bytes: &[u8], section: Section, max: usize) -> Result<Vec<u32>, DffError> {
    let mut out = vec![0_u32; max];
    let words = usize::try_from(section.size)
        .map_err(|_| DffError::Truncated)?
        .min(max);
    let source = slice_at(bytes, section.offset, (words * 4) as u64)?;
    for (slot, chunk) in out.iter_mut().zip(source.chunks_exact(4)) {
        *slot = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    Ok(out)
}

fn append_words(out: &mut Vec<u8>, values: &[u32], count: usize) -> u64 {
    let offset = out.len() as u64;
    for index in 0..count {
        let value = values.get(index).copied().unwrap_or(0);
        out.extend_from_slice(&value.to_le_bytes());
    }
    offset
}

fn game_id(bytes: &[u8; GAME_ID_LEN]) -> String {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(GAME_ID_LEN);
    String::from_utf8_lossy(&bytes[..len]).into_owned()
}

fn game_id_bytes(bytes: &[u8]) -> [u8; GAME_ID_LEN] {
    let mut out = [0_u8; GAME_ID_LEN];
    out.copy_from_slice(&bytes[..GAME_ID_LEN]);
    out
}

/// How many `stride`-byte entries fit in the file from `at` on, so a count a damaged header
/// invented is rejected here instead of walked.
fn table_count(bytes: &[u8], at: usize, count: u32, stride: usize) -> Result<usize, DffError> {
    let count = usize::try_from(count).map_err(|_| DffError::Truncated)?;
    if count > bytes.len().saturating_sub(at) / stride {
        return Err(DffError::Truncated);
    }
    Ok(count)
}

fn slice_at(bytes: &[u8], offset: u64, len: u64) -> Result<&[u8], DffError> {
    let start = usize::try_from(offset).map_err(|_| DffError::Truncated)?;
    let end = start
        .checked_add(usize::try_from(len).map_err(|_| DffError::Truncated)?)
        .ok_or(DffError::Truncated)?;
    bytes.get(start..end).ok_or(DffError::Truncated)
}

fn len32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

/// A cursor over the file, little-endian, that stops at the end instead of running past it.
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// A reader starting `at` bytes in, clamped so an offset past the end reads as truncated
    /// rather than panicking.
    fn at(bytes: &'a [u8], at: usize) -> Self {
        Self {
            bytes,
            pos: at.min(bytes.len()),
        }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], DffError> {
        let end = self.pos.checked_add(len).ok_or(DffError::Truncated)?;
        let slice = self.bytes.get(self.pos..end).ok_or(DffError::Truncated)?;
        self.pos = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, DffError> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32, DffError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn u64(&mut self) -> Result<u64, DffError> {
        let bytes = self.take(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
}

struct HeaderWriter<'a> {
    out: &'a mut [u8],
    pos: usize,
}

impl<'a> HeaderWriter<'a> {
    fn new(out: &'a mut [u8]) -> Self {
        Self { out, pos: 0 }
    }

    fn put(&mut self, bytes: &[u8]) {
        self.out[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
    }

    fn u32(&mut self, value: u32) {
        self.put(&value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.put(&value.to_le_bytes());
    }

    /// Eight bytes of game id, the rest of the slot left zero, falling back to `00000000` for
    /// an id that cannot fit, which is what `gecko/crates/dff/src/lib.rs:345` writes.
    fn game_id(&mut self, id: &str) {
        let id = if id.len() > GAME_ID_LEN || !id.is_ascii() {
            DEFAULT_GAME_ID
        } else {
            id
        };
        self.put(id.as_bytes());
        self.pos += GAME_ID_LEN;
    }
}
