//! The GameCube disc image read in memory: the boot header and the file system table.
//!
//! A disc image carries its whole directory tree in one table. The boot header names the table's
//! offset at `0x424` and its size at `0x428`; the table is a depth first list of 12 byte entries
//! whose names live in a string table after the last entry. [`DiscImage`] speaks the same
//! relative paths an extracted `CS_ASSETS` tree holds, so the asset layer above reads
//! `files/res/Stage/sea/Room44.arc` either way, and nothing above this module knows which of the
//! two it is talking to. A disc's table does not carry that `files/` component: its root is the
//! data root the extraction nests under `files/`, and `fst_path` translates one onto the other.
//! The extraction's `sys/` tree is the disc's preamble, which the table does not describe; the
//! port reads no `sys/` path, so that boundary is stated in `read` rather than worked around.
//!
//! The layout, as Dolphin's `DiscIO/FileSystemGCWii.cpp` and `nod`'s `disc/fst.rs` read it (nod
//! is the reader dusklight mounts discs with, and the one this module uses for RVZ containers):
//!
//! - `0x0000`: the game id, six bytes (`GZLE01`), then the disc number at `0x0006` and the disc
//!   version at `0x0007`.
//! - `0x0018`: the Wii magic `0x5D1C9EA3`, zero on a GameCube disc.
//! - `0x001C`: the GameCube magic `0xC2339F3D`.
//! - `0x0020`: the title, NUL terminated; the field is longer than the `0x40` bytes the readers
//!   that model it hand out.
//! - `0x0424`: the file table's offset from the start of the image, big endian.
//! - `0x0428`: the file table's size, the entries plus the names.
//!
//! An entry is three big endian words. The first word's low 24 bits are the name's offset in the
//! string table; its top byte is the kind, `0` for a file and `1` for a directory. A file's
//! second and third words are its offset in the image and its byte size. A directory's second
//! word is the index of its parent entry, its own for the root, and its third word is the index
//! one past its subtree, so the children of the directory at `i` are the entries between `i + 1`
//! and `size`. The root's `size` is the entry count, which is also where the names start:
//! `12 * count` bytes into the table.
//!
//! An RVZ container is not an image: it stores the disc's sectors compressed, so it is unwrapped
//! through `nod` and its table is handed to the same parser here. The port reads the four
//! GameCube discs the decompilation targets (GZLE01, GZLP01, GZLJ01, D44J01, the versions the
//! `tww` spec's README lists); any other image is a typed error rather than a session without
//! assets.

use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use nod::{PartitionBase, PartitionKind};

/// The web build's wasm memory ceiling. `.cargo/config.toml` hands this number to the linker as
/// `--max-memory`, which the threaded build must declare: a shared memory needs a maximum, and
/// wasm32 cannot address beyond 4 GiB. `u64` because the ceiling itself does not fit wasm32's
/// `usize`; `tests/disc.rs` keeps the two in step.
pub const WASM_MEMORY_CEILING: u64 = 4 << 30;

/// Room the session keeps inside that ceiling for the assets it decodes out of the image, so a
/// full disc's working set is not competing with the image itself.
pub const SESSION_HEADROOM: u64 = 2 << 30;

/// The largest image this build accepts. Both containers are read whole: an ISO is sliced in
/// place and an RVZ is handed to `nod` as one `Arc<Vec<u8>>`. An image over the limit is refused
/// here rather than left to fail inside the allocation that copies it in.
pub const MAX_IMAGE_BYTES: usize = (WASM_MEMORY_CEILING - SESSION_HEADROOM) as usize;

/// Whether an image of this length fits. [`DiscImage::open`] asks before it reads anything, and
/// the web entry hands the same limit to the page, which checks the picked file against it before
/// the bytes are copied into wasm memory.
pub fn image_len_ok(len: usize) -> Result<(), DiscError> {
    if len > MAX_IMAGE_BYTES {
        Err(DiscError::TooLarge {
            len,
            limit: MAX_IMAGE_BYTES,
        })
    } else {
        Ok(())
    }
}

const BOOT_HEADER_LEN: usize = 0x440;
const WII_MAGIC_AT: usize = 0x18;
const GAME_MAGIC_AT: usize = 0x1C;
const TITLE_AT: usize = 0x20;
const TITLE_LEN: usize = 0x40;
const FST_OFFSET_AT: usize = 0x424;
const FST_SIZE_AT: usize = 0x428;
const FST_ENTRY_LEN: usize = 12;
const NAME_MASK: u32 = 0x00FF_FFFF;
const KIND_DIRECTORY: u8 = 1;
const GAME_ID_LEN: usize = 6;

const GAME_MAGIC: [u8; 4] = [0xC2, 0x33, 0x9F, 0x3D];
const WII_MAGIC: [u8; 4] = [0x5D, 0x1C, 0x9E, 0xA3];
const RVZ_MAGIC: [u8; 3] = *b"RVZ";
const WIA_MAGIC: [u8; 4] = *b"WIA\x01";
const CISO_MAGIC: [u8; 4] = *b"CISO";
const GCZ_MAGIC: [u8; 4] = [0x01, 0xC0, 0x0B, 0xB1];
const WBFS_MAGIC: [u8; 4] = *b"WBFS";
const TGC_MAGIC: [u8; 4] = [0xAE, 0x0F, 0x38, 0xA2];
const NFS_MAGIC: [u8; 4] = *b"EGGS";

/// The GameCube versions of The Wind Waker, from the `tww` spec's README: USA rev 0 and rev 48,
/// PAL, Japan, and the Japanese kiosk demo.
const WIND_WAKER_IDS: [&str; 4] = ["GZLE01", "GZLP01", "GZLJ01", "D44J01"];

#[derive(Debug)]
pub enum DiscError {
    /// The image is shorter than the boot header it would have to carry.
    Truncated { len: usize },
    /// The image is larger than this build holds in memory ([`MAX_IMAGE_BYTES`]).
    TooLarge { len: usize, limit: usize },
    /// The magic at `0x1C` is not the GameCube one.
    BadMagic { actual: [u8; 4] },
    /// The image is a Wii disc, which stores its files under a partition this port does not read.
    WiiDisc { game_id: String },
    /// A container this build does not unwrap.
    UnsupportedContainer { format: &'static str },
    /// An RVZ container whose version byte is not the one `nod` reads.
    RvzVersion { version: u8 },
    /// A readable image that is not one of the discs the port has assets for.
    NotWindWaker { game_id: String },
    /// A malformed file table, with the byte offset the reader gave up at.
    BadFst { at: usize, detail: &'static str },
    /// The table holds no such path, or the path names a directory.
    MissingFile { path: String },
    /// `nod` refused the compressed container or a block of it.
    Compressed { detail: String },
}

impl std::fmt::Display for DiscError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Truncated { len } => write!(
                f,
                "the image is {len} bytes, too short for a GameCube boot header"
            ),
            Self::TooLarge { len, limit } => write!(
                f,
                "the image is {len} bytes; this build holds images up to {limit} bytes in memory"
            ),
            Self::BadMagic { actual } => write!(
                f,
                "no GameCube boot magic at 0x1C (found {:02X?}); this is not a GameCube disc",
                actual
            ),
            Self::WiiDisc { game_id } => write!(
                f,
                "the image is the Wii disc {game_id}; this port reads GameCube discs"
            ),
            Self::UnsupportedContainer { format } => write!(
                f,
                "{format} images are not supported; drop an ISO/GCM dump or an RVZ"
            ),
            Self::RvzVersion { version } => write!(
                f,
                "the RVZ container is version {version}; only version 1 is read"
            ),
            Self::NotWindWaker { game_id } => write!(
                f,
                "the disc image {game_id} is not The Wind Waker ({})",
                WIND_WAKER_IDS.join(", ")
            ),
            Self::BadFst { at, detail } => {
                write!(f, "the disc's file table is malformed at {at:#x}: {detail}")
            }
            Self::MissingFile { path } => write!(f, "the disc image holds no {path}"),
            Self::Compressed { detail } => {
                write!(f, "the compressed image could not be read: {detail}")
            }
        }
    }
}

impl std::error::Error for DiscError {}

/// What an installed image is, for the page that handed its bytes over.
#[derive(Debug)]
pub struct DiscSummary {
    pub game_id: String,
    /// The boot header's title, absent when it does not decode as UTF-8 (the Japanese discs
    /// carry Shift-JIS there).
    pub title: Option<String>,
    pub files: usize,
}

impl std::fmt::Display for DiscSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.title {
            Some(title) => write!(f, "{} ({title}), {} files", self.game_id, self.files),
            None => write!(f, "{}, {} files", self.game_id, self.files),
        }
    }
}

/// One file table entry. Directories carry the parent's index in `offset` and the index one past
/// their subtree in `size`, which is what the walk in [`Fst::find`] relies on.
struct Entry {
    name: Vec<u8>,
    directory: bool,
    offset: u32,
    size: u32,
}

struct Fst {
    entries: Vec<Entry>,
}

impl Fst {
    /// Parse the table an image's boot header points at.
    ///
    /// `bound` is the byte length the file offsets index into, which a raw image knows and a
    /// compressed container does not: an RVZ's offsets address the partition `nod` reads, so the
    /// container checks them instead.
    fn parse(raw: &[u8], bound: Option<u64>) -> Result<Self, DiscError> {
        let word = |at: usize| -> u32 {
            let bytes = [raw[at], raw[at + 1], raw[at + 2], raw[at + 3]];
            u32::from_be_bytes(bytes)
        };
        if raw.len() < FST_ENTRY_LEN {
            return Err(DiscError::BadFst {
                at: 0,
                detail: "the file table is shorter than one entry",
            });
        }
        let count = word(8) as usize;
        let entries_len = match count.checked_mul(FST_ENTRY_LEN) {
            Some(len) if count > 0 && len <= raw.len() => len,
            _ => {
                return Err(DiscError::BadFst {
                    at: 8,
                    detail: "the entry count does not fit the table",
                });
            }
        };
        let names = &raw[entries_len..];
        if names.last() != Some(&0) {
            return Err(DiscError::BadFst {
                at: entries_len,
                detail: "the name table does not end in a terminator",
            });
        }
        if (word(0) >> 24) as u8 != KIND_DIRECTORY {
            return Err(DiscError::BadFst {
                at: 0,
                detail: "the root entry is not a directory",
            });
        }
        let mut entries: Vec<Entry> = Vec::with_capacity(count);
        for index in 0..count {
            let at = index * FST_ENTRY_LEN;
            let kind = (word(at) >> 24) as u8;
            let directory = match kind {
                0 => false,
                KIND_DIRECTORY => true,
                _ => {
                    return Err(DiscError::BadFst {
                        at,
                        detail: "an entry is neither a file nor a directory",
                    });
                }
            };
            let name_at = (word(at) & NAME_MASK) as usize;
            let name = match names.get(name_at..) {
                Some(rest) => match rest.iter().position(|byte| *byte == 0) {
                    Some(end) => rest[..end].to_vec(),
                    None => {
                        return Err(DiscError::BadFst {
                            at,
                            detail: "a name is not terminated",
                        });
                    }
                },
                None => {
                    return Err(DiscError::BadFst {
                        at,
                        detail: "a name offset is past the name table",
                    });
                }
            };
            let offset = word(at + 4);
            let size = word(at + 8);
            if directory {
                if index == 0 {
                    if offset != 0 {
                        return Err(DiscError::BadFst {
                            at,
                            detail: "the root entry names a parent",
                        });
                    }
                } else if offset as usize >= index || !entries[offset as usize].directory {
                    return Err(DiscError::BadFst {
                        at,
                        detail: "a directory does not point at its parent",
                    });
                }
                let end = size as usize;
                if end <= index || end > count {
                    return Err(DiscError::BadFst {
                        at,
                        detail: "a directory's subtree runs past the table",
                    });
                }
                if index != 0 && end > entries[offset as usize].size as usize {
                    return Err(DiscError::BadFst {
                        at,
                        detail: "a directory runs past its parent",
                    });
                }
            } else if let Some(bound) = bound {
                if u64::from(offset) + u64::from(size) > bound {
                    return Err(DiscError::BadFst {
                        at,
                        detail: "a file runs past the end of the image",
                    });
                }
            }
            entries.push(Entry {
                name,
                directory,
                offset,
                size,
            });
        }
        Ok(Self { entries })
    }

    /// The entry a disc-relative path names, by splitting it on `/` and walking the table, the
    /// way the extracted tree does. The extraction's `files/` component is translated away by
    /// `fst_path` before this is called. Names are matched exactly: the paths the port reads are
    /// the disc's own names, and the reference's case insensitive lookup is a property of the
    /// game's DVD calls, not of the asset root.
    fn find(&self, rel: &str) -> Option<usize> {
        let mut parts = rel
            .split('/')
            .filter(|part| !part.is_empty() && *part != ".");
        let mut wanted = parts.next()?;
        let mut index = 0_usize;
        loop {
            let end = self.entries[index].size as usize;
            let mut cursor = index + 1;
            let mut found = None;
            while cursor < end {
                let entry = &self.entries[cursor];
                if entry.name == wanted.as_bytes() {
                    found = Some(cursor);
                    break;
                }
                cursor = if entry.directory {
                    entry.size as usize
                } else {
                    cursor + 1
                };
            }
            let found = found?;
            match parts.next() {
                Some(next) => {
                    if !self.entries[found].directory {
                        return None;
                    }
                    index = found;
                    wanted = next;
                }
                None => return Some(found),
            }
        }
    }

    fn file(&self, index: usize) -> Option<(u64, u64)> {
        let entry = self.entries.get(index)?;
        (!entry.directory).then(|| (u64::from(entry.offset), u64::from(entry.size)))
    }

    fn files(&self) -> usize {
        self.entries.iter().filter(|entry| !entry.directory).count()
    }
}

/// An image stream for `nod`: the byte range `Arc` keeps the image in one place, so the clones
/// `nod` makes of its readers share it instead of copying a gigabyte per partition.
#[derive(Clone)]
struct ImageStream {
    bytes: Arc<Vec<u8>>,
    pos: u64,
}

impl ImageStream {
    fn new(bytes: Arc<Vec<u8>>) -> Self {
        Self { bytes, pos: 0 }
    }
}

impl Read for ImageStream {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        let start = match usize::try_from(self.pos) {
            Ok(start) if start < self.bytes.len() => start,
            _ => return Ok(0),
        };
        let end = start.saturating_add(out.len()).min(self.bytes.len());
        let len = end - start;
        out[..len].copy_from_slice(&self.bytes[start..end]);
        self.pos += len as u64;
        Ok(len)
    }
}

impl Seek for ImageStream {
    fn seek(&mut self, from: SeekFrom) -> std::io::Result<u64> {
        let target = match from {
            SeekFrom::Start(at) => at as i128,
            SeekFrom::End(at) => self.bytes.len() as i128 + at as i128,
            SeekFrom::Current(at) => self.pos as i128 + at as i128,
        };
        if target < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek before the start of the image",
            ));
        }
        self.pos = target as u64;
        Ok(self.pos)
    }
}

enum ImageData {
    /// The image itself, as it was handed over.
    Raw { bytes: Arc<Vec<u8>> },
    /// An RVZ container: the partition `nod` decodes and its own node table, whose indices match
    /// the entries parsed here.
    Rvz {
        partition: Mutex<Box<dyn PartitionBase>>,
        nodes: Vec<nod::Node>,
    },
}

pub struct DiscImage {
    game_id: String,
    title: Option<String>,
    fst: Fst,
    data: ImageData,
}

impl DiscImage {
    /// Read a disc image: an ISO/GCM as it is, or an RVZ through `nod`.
    pub fn open(bytes: Vec<u8>) -> Result<Self, DiscError> {
        image_len_ok(bytes.len())?;
        if bytes.len() < WII_MAGIC_AT + 4 {
            return Err(DiscError::Truncated { len: bytes.len() });
        }
        let magic = [bytes[0], bytes[1], bytes[2], bytes[3]];
        if magic[..RVZ_MAGIC.len()] == RVZ_MAGIC {
            return match magic[3] {
                1 => Self::open_rvz(bytes),
                version => Err(DiscError::RvzVersion { version }),
            };
        }
        match magic {
            WIA_MAGIC => Self::open_rvz(bytes),
            CISO_MAGIC => Err(DiscError::UnsupportedContainer { format: "CISO" }),
            GCZ_MAGIC => Err(DiscError::UnsupportedContainer { format: "GCZ" }),
            WBFS_MAGIC => Err(DiscError::UnsupportedContainer { format: "WBFS" }),
            TGC_MAGIC => Err(DiscError::UnsupportedContainer { format: "TGC" }),
            NFS_MAGIC => Err(DiscError::UnsupportedContainer { format: "NFS" }),
            _ => Self::open_raw(bytes),
        }
    }

    fn open_raw(bytes: Vec<u8>) -> Result<Self, DiscError> {
        if bytes.len() < BOOT_HEADER_LEN {
            return Err(DiscError::Truncated { len: bytes.len() });
        }
        if bytes[WII_MAGIC_AT..WII_MAGIC_AT + 4] == WII_MAGIC {
            return Err(DiscError::WiiDisc {
                game_id: game_id(&bytes[..GAME_ID_LEN]),
            });
        }
        if bytes[GAME_MAGIC_AT..GAME_MAGIC_AT + 4] != GAME_MAGIC {
            return Err(DiscError::BadMagic {
                actual: [
                    bytes[GAME_MAGIC_AT],
                    bytes[GAME_MAGIC_AT + 1],
                    bytes[GAME_MAGIC_AT + 2],
                    bytes[GAME_MAGIC_AT + 3],
                ],
            });
        }
        let id = game_id(&bytes[..GAME_ID_LEN]);
        wind_waker(&id)?;
        let title = title(&bytes[TITLE_AT..TITLE_AT + TITLE_LEN]);
        let offset = word_at(&bytes, FST_OFFSET_AT) as usize;
        let size = word_at(&bytes, FST_SIZE_AT) as usize;
        let end = offset
            .checked_add(size)
            .filter(|end| *end <= bytes.len())
            .ok_or(DiscError::BadFst {
                at: FST_OFFSET_AT,
                detail: "the file table runs past the end of the image",
            })?;
        let fst = Fst::parse(&bytes[offset..end], Some(bytes.len() as u64))?;
        Ok(Self {
            game_id: id,
            title,
            fst,
            data: ImageData::Raw {
                bytes: Arc::new(bytes),
            },
        })
    }

    fn open_rvz(bytes: Vec<u8>) -> Result<Self, DiscError> {
        let disc = nod::Disc::new_stream(Box::new(ImageStream::new(Arc::new(bytes))))
            .map_err(container_error)?;
        let header = disc.header();
        let id = game_id(&header.game_id);
        if header.is_wii() {
            return Err(DiscError::WiiDisc { game_id: id });
        }
        if !header.is_gamecube() {
            return Err(DiscError::BadMagic {
                actual: header.gcn_magic,
            });
        }
        wind_waker(&id)?;
        let title = title(&header.game_title);
        // The partition's table is the disc's own, so the parser here owns the path semantics for
        // both containers; `nod`'s node values carry the offsets `open_file` seeks to.
        let mut partition = disc
            .open_partition_kind(PartitionKind::Data)
            .map_err(container_error)?;
        let meta = partition.meta().map_err(container_error)?;
        let nodes = meta
            .fst()
            .map_err(|detail| DiscError::BadFst { at: 0, detail })?
            .nodes
            .to_vec();
        let fst = Fst::parse(&meta.raw_fst, None)?;
        if nodes.len() < fst.entries.len() {
            return Err(DiscError::BadFst {
                at: 0,
                detail: "the container's node table is shorter than the file table",
            });
        }
        Ok(Self {
            game_id: id,
            title,
            fst,
            data: ImageData::Rvz {
                partition: Mutex::new(partition),
                nodes,
            },
        })
    }

    pub fn game_id(&self) -> &str {
        &self.game_id
    }

    /// The label a path carries when it came out of this image rather than a directory, which the
    /// errors report instead of a filesystem path: `GZLE01:files/res/Stage/sea/Room44.arc`.
    pub fn path(&self, rel: &str) -> PathBuf {
        PathBuf::from(format!("{}:{rel}", self.game_id))
    }

    pub fn summary(&self) -> DiscSummary {
        DiscSummary {
            game_id: self.game_id.clone(),
            title: self.title.clone(),
            files: self.fst.files(),
        }
    }

    /// The bytes of one path, relative to the disc root the way an extracted dump is:
    /// `files/res/Stage/sea/Room44.arc`. The extraction's `files/` component names the FST root
    /// the walk starts at, so it is translated away before the lookup; a path outside it,
    /// `sys/boot.bin` included, is looked up as given, and the table holds no such entry because
    /// the `sys/` files live in the disc preamble rather than in the table. A path the table
    /// does not hold, or one that names a directory, is [`DiscError::MissingFile`], labelled in
    /// the extraction style.
    pub fn read(&self, rel: &str) -> Result<Vec<u8>, DiscError> {
        let index = self
            .fst
            .find(fst_path(rel))
            .ok_or_else(|| DiscError::MissingFile {
                path: rel.to_owned(),
            })?;
        let (offset, size) = self.fst.file(index).ok_or_else(|| DiscError::MissingFile {
            path: rel.to_owned(),
        })?;
        match &self.data {
            ImageData::Raw { bytes } => {
                let start = usize::try_from(offset).map_err(|_| DiscError::BadFst {
                    at: index * FST_ENTRY_LEN,
                    detail: "a file offset does not fit this target",
                })?;
                let end = start
                    .checked_add(usize::try_from(size).map_err(|_| DiscError::BadFst {
                        at: index * FST_ENTRY_LEN,
                        detail: "a file size does not fit this target",
                    })?)
                    .ok_or(DiscError::BadFst {
                        at: index * FST_ENTRY_LEN,
                        detail: "a file range overflows",
                    })?;
                bytes
                    .get(start..end)
                    .map(<[u8]>::to_vec)
                    .ok_or(DiscError::BadFst {
                        at: index * FST_ENTRY_LEN,
                        detail: "a file runs past the end of the image",
                    })
            }
            ImageData::Rvz { partition, nodes } => {
                let node = *nodes.get(index).ok_or(DiscError::BadFst {
                    at: index * FST_ENTRY_LEN,
                    detail: "a file is past the container's node table",
                })?;
                let mut partition = partition.lock().map_err(|_| DiscError::Compressed {
                    detail: "the disc image lock is poisoned".to_owned(),
                })?;
                let mut stream = partition.open_file(node).map_err(container_error)?;
                let mut out = Vec::new();
                stream.read_to_end(&mut out).map_err(container_error)?;
                Ok(out)
            }
        }
    }
}

/// The path the file table is walked with, given one relative to the disc root the way an
/// extracted dump lays it out.
///
/// An extraction (Dolphin's, and the virtual root Riivolution mounts a disc under) nests the
/// disc's data root under `files/` and keeps the preamble's boot files beside it under `sys/`
/// (`boot.bin`, `bi2.bin`, `apploader.img`, `fst.bin`). A GameCube FST has neither: the table's
/// root is the data root itself, and the preamble files are not entries at all. A leading `files`
/// component therefore names the root the walk already starts at and is dropped -
/// `files/res/Stage/sea/Stage.arc` is the table's `res/Stage/sea/Stage.arc` - while anything else
/// is looked up as given. Nothing in the port reads a `sys/` path, so the missing entries there
/// are the boundary this states rather than a lookup that falls back to something on a guess.
fn fst_path(rel: &str) -> &str {
    let mut rest = rel;
    loop {
        let Some((head, tail)) = rest.split_once('/') else {
            return rest;
        };
        match head {
            "" | "." => rest = tail,
            "files" => return tail,
            _ => return rest,
        }
    }
}

fn container_error(error: impl std::fmt::Display) -> DiscError {
    DiscError::Compressed {
        detail: error.to_string(),
    }
}

fn word_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn game_id(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn title(bytes: &[u8]) -> Option<String> {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    let title = std::str::from_utf8(&bytes[..end]).ok()?.trim();
    (!title.is_empty() && !title.chars().any(char::is_control)).then(|| title.to_owned())
}

fn wind_waker(game_id: &str) -> Result<(), DiscError> {
    if WIND_WAKER_IDS.contains(&game_id) {
        Ok(())
    } else {
        Err(DiscError::NotWindWaker {
            game_id: game_id.to_owned(),
        })
    }
}
