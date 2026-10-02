use std::path::{Path, PathBuf};

pub const ASSETS_ENV: &str = "CS_ASSETS";

const YAZ0_MAGIC: [u8; 4] = *b"Yaz0";
const RARC_MAGIC: [u8; 4] = *b"RARC";
const STAGE_DIR: &str = "files/res/Stage/sea";
const STAGE_MAP: &str = "files/res/Stage/sea/Stage.arc";
const STAGE_SPAWN_POINT: i32 = 128;
const STAGE_START_ROOM: u8 = 44;
const ROOM_MODELS: [&str; 4] = [
    "bdl/model.bdl",
    "bdl/model1.bdl",
    "bdl/model2.bdl",
    "bdl/model3.bdl",
];
const RAD_PER_UNIT: f32 = std::f32::consts::TAU / 65536.0;

#[derive(Debug)]
pub enum AssetError {
    MissingRoot {
        root: PathBuf,
    },
    MissingFile {
        path: PathBuf,
    },
    MissingEntry {
        path: PathBuf,
        entry: String,
    },
    BadMagic {
        path: PathBuf,
        expected: &'static str,
        actual: [u8; 4],
    },
    Corrupt {
        path: PathBuf,
        detail: &'static str,
    },
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRoot { root } => write!(
                f,
                "asset root not set or not a directory: {}",
                root.display()
            ),
            Self::MissingFile { path } => write!(f, "asset file not found: {}", path.display()),
            Self::MissingEntry { path, entry } => {
                write!(f, "entry `{entry}` not found in archive {}", path.display())
            }
            Self::BadMagic {
                path,
                expected,
                actual,
            } => write!(
                f,
                "bad magic in {}: expected {expected}, found {:?}",
                path.display(),
                String::from_utf8_lossy(actual)
            ),
            Self::Corrupt { path, detail } => {
                write!(f, "corrupt asset {}: {detail}", path.display())
            }
            Self::Io { path, source } => {
                write!(f, "failed to read asset {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for AssetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub struct AssetRoot {
    root: PathBuf,
}

impl AssetRoot {
    pub fn from_env() -> Result<Self, AssetError> {
        match std::env::var_os(ASSETS_ENV) {
            Some(value) if !value.is_empty() => Ok(Self {
                root: PathBuf::from(value),
            }),
            _ => Err(AssetError::MissingRoot {
                root: PathBuf::from(format!("<unset {ASSETS_ENV}>")),
            }),
        }
    }

    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn read(&self, rel: &str) -> Result<Vec<u8>, AssetError> {
        if !self.root.is_dir() {
            return Err(AssetError::MissingRoot {
                root: self.root.clone(),
            });
        }
        let path = self.root.join(rel);
        std::fs::read(&path).map_err(|source| match source.kind() {
            std::io::ErrorKind::NotFound => AssetError::MissingFile { path },
            _ => AssetError::Io { path, source },
        })
    }

    pub fn read_archive(&self, rel: &str) -> Result<Archive, AssetError> {
        let path = self.root.join(rel);
        let bytes = self.read(rel)?;
        open_archive(bytes, path)
    }
}

fn open_archive(bytes: Vec<u8>, path: PathBuf) -> Result<Archive, AssetError> {
    if bytes.len() >= 4 && bytes[..4] == YAZ0_MAGIC {
        let raw = yaz0_decode(&bytes, &path)?;
        Archive::parse(raw, path)
    } else {
        Archive::parse(bytes, path)
    }
}

pub fn yaz0_decode(src: &[u8], path: impl AsRef<Path>) -> Result<Vec<u8>, AssetError> {
    let path = path.as_ref();
    if src.len() < 4 {
        return Err(corrupt(path, "yaz0 data shorter than magic"));
    }
    let magic = [src[0], src[1], src[2], src[3]];
    if magic != YAZ0_MAGIC {
        return Err(AssetError::BadMagic {
            path: path.to_path_buf(),
            expected: "Yaz0",
            actual: magic,
        });
    }
    if src.len() < 16 {
        return Err(corrupt(path, "yaz0 header truncated"));
    }
    let raw_size = u32::from_be_bytes([src[4], src[5], src[6], src[7]]) as usize;
    let mut out = Vec::new();
    out.try_reserve_exact(raw_size)
        .map_err(|_| corrupt(path, "yaz0 size too large"))?;
    let mut cursor = 16;
    while out.len() < raw_size {
        let flags = *src
            .get(cursor)
            .ok_or_else(|| corrupt(path, "yaz0 stream truncated"))?;
        cursor += 1;
        for bit in (0..8).rev() {
            if out.len() >= raw_size {
                break;
            }
            if flags & (1 << bit) != 0 {
                let byte = *src
                    .get(cursor)
                    .ok_or_else(|| corrupt(path, "yaz0 stream truncated"))?;
                cursor += 1;
                out.push(byte);
            } else {
                let b1 = *src
                    .get(cursor)
                    .ok_or_else(|| corrupt(path, "yaz0 stream truncated"))?;
                let b2 = *src
                    .get(cursor + 1)
                    .ok_or_else(|| corrupt(path, "yaz0 stream truncated"))?;
                cursor += 2;
                let dist = (usize::from(b1 & 0x0F) << 8) | usize::from(b2);
                let mut len = usize::from(b1 >> 4);
                if len == 0 {
                    len = usize::from(
                        *src
                            .get(cursor)
                            .ok_or_else(|| corrupt(path, "yaz0 stream truncated"))?,
                    ) + 0x12;
                    cursor += 1;
                } else {
                    len += 2;
                }
                let start = out
                    .len()
                    .checked_sub(dist + 1)
                    .ok_or_else(|| corrupt(path, "yaz0 back-reference before stream start"))?;
                for k in 0..len {
                    let byte = out
                        .get(start + k)
                        .copied()
                        .ok_or_else(|| corrupt(path, "yaz0 back-reference out of range"))?;
                    out.push(byte);
                }
            }
        }
    }
    Ok(out)
}

fn name_hash(name: &str) -> u16 {
    let mut hash = 0_u32;
    for byte in name.bytes() {
        hash = (hash * 3 + u32::from(byte)) & 0xFFFF;
    }
    hash as u16
}

fn corrupt(path: &Path, detail: &'static str) -> AssetError {
    AssetError::Corrupt {
        path: path.to_path_buf(),
        detail,
    }
}

#[derive(Debug)]
pub struct Archive {
    path: PathBuf,
    data: Vec<u8>,
    data_start: usize,
    dirs: Vec<DirNode>,
    files: Vec<ArchiveEntry>,
}

#[derive(Debug)]
struct DirNode {
    first: usize,
    count: usize,
}

#[derive(Debug)]
struct ArchiveEntry {
    name: String,
    dir: bool,
    offset: usize,
    size: usize,
}

impl Archive {
    pub fn parse(data: Vec<u8>, path: impl Into<PathBuf>) -> Result<Self, AssetError> {
        let path = path.into();
        if data.len() < 4 {
            return Err(corrupt(&path, "archive shorter than magic"));
        }
        let magic = [data[0], data[1], data[2], data[3]];
        if magic != RARC_MAGIC {
            return Err(AssetError::BadMagic {
                path: path.clone(),
                expected: "RARC",
                actual: magic,
            });
        }
        let u32_at = |at: usize| -> Result<u32, AssetError> {
            data.get(at..at + 4)
                .map(|slice| u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
                .ok_or_else(|| corrupt(&path, "rarc truncated"))
        };
        let u16_at = |at: usize| -> Result<u16, AssetError> {
            data.get(at..at + 2)
                .map(|slice| u16::from_be_bytes([slice[0], slice[1]]))
                .ok_or_else(|| corrupt(&path, "rarc truncated"))
        };
        let rel = |at: usize| -> Result<usize, AssetError> {
            0x20_usize
                .checked_add(u32_at(at)? as usize)
                .ok_or_else(|| corrupt(&path, "rarc offset overflow"))
        };
        if u32_at(0x08)? != 0x20 {
            return Err(corrupt(&path, "unexpected rarc header size"));
        }
        if u32_at(0x04)? as usize != data.len() {
            return Err(corrupt(&path, "rarc size mismatch"));
        }
        let data_start = rel(0x0C)?;
        let data_len = u32_at(0x10)? as usize;
        if data_start.checked_add(data_len) != Some(data.len()) {
            return Err(corrupt(&path, "rarc data region mismatch"));
        }
        let dir_count = u32_at(0x20)? as usize;
        let dir_off = rel(0x24)?;
        let file_count = u32_at(0x28)? as usize;
        let file_off = rel(0x2C)?;
        let str_size = u32_at(0x30)? as usize;
        let str_off = rel(0x34)?;
        if str_off.checked_add(str_size) != Some(data_start) {
            return Err(corrupt(&path, "rarc string table mismatch"));
        }
        if dir_count == 0 {
            return Err(corrupt(&path, "rarc has no root directory"));
        }
        let dir_end = dir_off
            .checked_add(
                dir_count
                    .checked_mul(0x10)
                    .ok_or_else(|| corrupt(&path, "rarc directory table overflow"))?,
            )
            .ok_or_else(|| corrupt(&path, "rarc directory table overflow"))?;
        if dir_end > data.len() {
            return Err(corrupt(&path, "rarc directory table out of range"));
        }
        let file_end = file_off
            .checked_add(
                file_count
                    .checked_mul(0x14)
                    .ok_or_else(|| corrupt(&path, "rarc entry table overflow"))?,
            )
            .ok_or_else(|| corrupt(&path, "rarc entry table overflow"))?;
        if file_end > data.len() {
            return Err(corrupt(&path, "rarc entry table out of range"));
        }
        let string_at = |at: usize| -> Result<String, AssetError> {
            let start = str_off
                .checked_add(at)
                .filter(|&start| start < data_start)
                .ok_or_else(|| corrupt(&path, "rarc name offset out of range"))?;
            let end = data[start..data_start]
                .iter()
                .position(|&byte| byte == 0)
                .map(|rel| start + rel)
                .ok_or_else(|| corrupt(&path, "rarc name not terminated"))?;
            std::str::from_utf8(&data[start..end])
                .map(str::to_owned)
                .map_err(|_| corrupt(&path, "rarc name not utf-8"))
        };
        let mut dirs = Vec::with_capacity(dir_count);
        for index in 0..dir_count {
            let at = dir_off + index * 0x10;
            let tag = [data[at], data[at + 1], data[at + 2], data[at + 3]];
            if index == 0 && tag != *b"ROOT" {
                return Err(corrupt(&path, "rarc root marker missing"));
            }
            let name = string_at(u32_at(at + 4)? as usize)?;
            let hash = u16_at(at + 8)?;
            if name_hash(&name) != hash {
                return Err(corrupt(&path, "rarc directory hash mismatch"));
            }
            let count = usize::from(u16_at(at + 10)?);
            let first = u32_at(at + 12)? as usize;
            let end = first
                .checked_add(count)
                .ok_or_else(|| corrupt(&path, "rarc directory range overflow"))?;
            if end > file_count {
                return Err(corrupt(&path, "rarc directory range out of bounds"));
            }
            dirs.push(DirNode { first, count });
        }
        let mut files = Vec::with_capacity(file_count);
        for index in 0..file_count {
            let at = file_off + index * 0x14;
            let hash = u16_at(at + 2)?;
            let kind = data[at + 4];
            let pad = data[at + 5];
            let name = string_at(usize::from(u16_at(at + 6)?))?;
            if name_hash(&name) != hash {
                return Err(corrupt(&path, "rarc entry hash mismatch"));
            }
            let offset = u32_at(at + 8)? as usize;
            let size = u32_at(at + 12)? as usize;
            if pad != 0 || u32_at(at + 16)? != 0 {
                return Err(corrupt(&path, "rarc entry padding not zero"));
            }
            let dir = match kind {
                0x02 => true,
                0x11 | 0x95 | 0xA5 => false,
                _ => return Err(corrupt(&path, "rarc entry has unknown kind")),
            };
            if dir {
                if offset != 0xFFFF_FFFF && offset >= dir_count {
                    return Err(corrupt(&path, "rarc directory index out of range"));
                }
            } else if data_start
                .checked_add(offset)
                .and_then(|start| start.checked_add(size))
                .map(|end| end <= data.len())
                != Some(true)
            {
                return Err(corrupt(&path, "rarc entry data out of range"));
            }
            files.push(ArchiveEntry {
                name,
                dir,
                offset,
                size,
            });
        }
        Ok(Self {
            path,
            data,
            data_start,
            dirs,
            files,
        })
    }

    pub fn read(&self, entry: &str) -> Result<&[u8], AssetError> {
        let mut dir_index = 0_usize;
        let mut parts = entry.split('/').peekable();
        loop {
            let part = parts
                .next()
                .ok_or_else(|| self.missing_entry(entry))?;
            let dir = &self.dirs[dir_index];
            let found = (dir.first..dir.first + dir.count)
                .map(|index| &self.files[index])
                .find(|candidate| candidate.name == part)
                .ok_or_else(|| self.missing_entry(entry))?;
            if parts.peek().is_none() {
                if found.dir {
                    return Err(self.missing_entry(entry));
                }
                let start = self.data_start + found.offset;
                return Ok(&self.data[start..start + found.size]);
            }
            if !found.dir {
                return Err(self.missing_entry(entry));
            }
            dir_index = if found.offset == 0xFFFF_FFFF {
                0
            } else {
                found.offset
            };
            if dir_index >= self.dirs.len() {
                return Err(corrupt(&self.path, "rarc directory index out of range"));
            }
        }
    }

    fn missing_entry(&self, entry: &str) -> AssetError {
        AssetError::MissingEntry {
            path: self.path.clone(),
            entry: entry.to_owned(),
        }
    }
}

pub fn load_stage() -> Option<crate::sim::collision::StageData> {
    match stage_from_assets() {
        Ok(stage) => Some(stage),
        Err(error) => {
            log::warn!("clearsleeper: stage assets unavailable ({error})");
            None
        }
    }
}

pub fn load_room_mesh() -> Option<crate::bdl::RoomMesh> {
    match room_mesh_from_assets() {
        Ok(mesh) => Some(mesh),
        Err(error) => {
            log::warn!("clearsleeper: room mesh unavailable ({error})");
            None
        }
    }
}

pub fn load_stage_tables() -> Option<crate::dzs::StageTables> {
    match stage_tables_from_assets() {
        Ok(tables) => Some(tables),
        Err(error) => {
            log::warn!("clearsleeper: stage tables unavailable ({error})");
            None
        }
    }
}

pub fn start_set(tables: &crate::dzs::StageTables) -> Vec<u8> {
    let set = tables.load_set(STAGE_START_ROOM);
    if set.is_empty() {
        vec![STAGE_START_ROOM]
    } else {
        set.to_vec()
    }
}

pub struct RoomSet {
    pub collision: Option<crate::sim::collision::Collision>,
    pub mesh: Option<crate::bdl::RoomMesh>,
}

pub fn load_room_set(tables: &crate::dzs::StageTables, rooms: &[u8]) -> Option<RoomSet> {
    let loaded =
        AssetRoot::from_env().and_then(|root| room_set_from_assets(&root, tables, rooms));
    match loaded {
        Ok(set) => Some(set),
        Err(error) => {
            log::warn!("clearsleeper: room set {rooms:?} unavailable ({error})");
            None
        }
    }
}

fn stage_tables_from_assets() -> Result<crate::dzs::StageTables, AssetError> {
    let root = AssetRoot::from_env()?;
    stage_tables(&root)
}

fn stage_tables(root: &AssetRoot) -> Result<crate::dzs::StageTables, AssetError> {
    let map = root.read_archive(STAGE_MAP)?;
    let path = root.root().join(STAGE_MAP);
    let data = map.read("dzs/stage.dzs")?;
    crate::dzs::parse(data).map_err(|error| AssetError::Corrupt {
        path,
        detail: error.as_str(),
    })
}

fn stage_from_assets() -> Result<crate::sim::collision::StageData, AssetError> {
    let root = AssetRoot::from_env()?;
    let tables = stage_tables(&root)?;
    let rooms = start_set(&tables);
    let set = room_set_from_assets(&root, &tables, &rooms)?;
    let collision = set.collision.ok_or_else(|| AssetError::Corrupt {
        path: root.root().join(room_path(STAGE_START_ROOM)),
        detail: "room set has no collision",
    })?;
    let relative = room_path(STAGE_START_ROOM);
    let path = root.root().join(&relative);
    let room = root.read_archive(&relative)?;
    let (spawn, yaw) = stage_spawn(room.read("dzr/room.dzr")?, path)?;
    Ok(crate::sim::collision::StageData {
        spawn,
        yaw,
        collision,
    })
}

fn room_mesh_from_assets() -> Result<crate::bdl::RoomMesh, AssetError> {
    let root = AssetRoot::from_env()?;
    let tables = stage_tables(&root)?;
    let rooms = start_set(&tables);
    let set = room_set_from_assets(&root, &tables, &rooms)?;
    set.mesh.ok_or_else(|| AssetError::Corrupt {
        path: root.root().join(room_path(STAGE_START_ROOM)),
        detail: "room set has no geometry",
    })
}

fn room_set_from_assets(
    root: &AssetRoot,
    tables: &crate::dzs::StageTables,
    rooms: &[u8],
) -> Result<RoomSet, AssetError> {
    let mut collision: Option<crate::sim::collision::Collision> = None;
    let mut mesh: Option<crate::bdl::RoomMesh> = None;
    for &room in rooms {
        let relative = room_path(room);
        let path = root.root().join(&relative);
        let archive = root.read_archive(&relative)?;
        match archive.read("dzb/room.dzb") {
            Ok(data) => {
                let parsed =
                    crate::sim::collision::Collision::parse(data).map_err(|error| {
                        AssetError::Corrupt {
                            path: path.clone(),
                            detail: error.as_str(),
                        }
                    })?;
                match &mut collision {
                    Some(target) => {
                        target.append(parsed).map_err(|error| AssetError::Corrupt {
                            path: path.clone(),
                            detail: error.as_str(),
                        })?;
                    }
                    None => collision = Some(parsed),
                }
            }
            Err(AssetError::MissingEntry { .. }) => {}
            Err(error) => return Err(error),
        }
        let (origin, angle) = table_placement(tables, room);
        let mut room_part = crate::bdl::RoomMesh::default();
        for name in ROOM_MODELS {
            let model = match archive.read(name) {
                Ok(model) => model,
                Err(AssetError::MissingEntry { .. }) => continue,
                Err(error) => return Err(error),
            };
            let btk_name = name.replacen("bdl/", "btk/", 1).replacen(".bdl", ".btk", 1);
            let animation = match archive.read(&btk_name) {
                Ok(data) => Some(crate::btk::parse(data).map_err(|error| {
                    AssetError::Corrupt {
                        path: path.clone(),
                        detail: error.as_str(),
                    }
                })?),
                Err(AssetError::MissingEntry { .. }) => None,
                Err(error) => return Err(error),
            };
            let part = crate::bdl::parse(model, origin, angle, animation.as_ref()).map_err(
                |error| AssetError::Corrupt {
                    path: path.clone(),
                    detail: error.as_str(),
                },
            )?;
            room_part.append(part);
        }
        if !room_part.groups.is_empty() {
            match &mut mesh {
                Some(target) => target.append(room_part),
                None => mesh = Some(room_part),
            }
        }
    }
    Ok(RoomSet { collision, mesh })
}

fn table_placement(tables: &crate::dzs::StageTables, room: u8) -> ([f32; 2], f32) {
    match tables.placement(room) {
        Some(placement) => (placement.trans, f32::from(placement.angle) * RAD_PER_UNIT),
        None => ([0.0, 0.0], 0.0),
    }
}

fn room_path(room: u8) -> String {
    format!("{STAGE_DIR}/Room{room}.arc")
}

pub fn stage_spawn(data: &[u8], path: PathBuf) -> Result<(glam::Vec3, f32), AssetError> {
    let corrupt = |detail: &'static str| AssetError::Corrupt {
        path: path.clone(),
        detail,
    };
    let chunk_count = read_u32(data, 0).ok_or_else(|| corrupt("dzr truncated"))?;
    let nodes_end = (chunk_count as usize)
        .checked_mul(12)
        .and_then(|bytes| bytes.checked_add(4))
        .ok_or_else(|| corrupt("dzr chunk table overflow"))?;
    if nodes_end > data.len() {
        return Err(corrupt("dzr chunk table out of range"));
    }
    let mut plyr = None;
    for index in 0..chunk_count as usize {
        let at = 4 + index * 12;
        if &data[at..at + 4] == b"PLYR" {
            let count = read_u32(data, at + 4).ok_or_else(|| corrupt("dzr truncated"))?;
            let offset = read_u32(data, at + 8).ok_or_else(|| corrupt("dzr truncated"))?;
            plyr = Some((count, offset));
            break;
        }
    }
    let (count, offset) = plyr.ok_or_else(|| corrupt("dzr has no spawn chunk"))?;
    let entries_end = (count as usize)
        .checked_mul(0x20)
        .and_then(|bytes| bytes.checked_add(offset as usize))
        .ok_or_else(|| corrupt("dzr spawn table overflow"))?;
    if entries_end > data.len() {
        return Err(corrupt("dzr spawn table out of range"));
    }
    for index in 0..count as usize {
        let at = offset as usize + index * 0x20;
        let angle_z = read_i16(data, at + 0x1C).ok_or_else(|| corrupt("dzr truncated"))?;
        if i32::from(angle_z as u8) != STAGE_SPAWN_POINT {
            continue;
        }
        let spawn = glam::Vec3::new(
            read_f32(data, at + 0x0C).ok_or_else(|| corrupt("dzr truncated"))?,
            read_f32(data, at + 0x10).ok_or_else(|| corrupt("dzr truncated"))?,
            read_f32(data, at + 0x14).ok_or_else(|| corrupt("dzr truncated"))?,
        );
        let angle_y = read_i16(data, at + 0x1A).ok_or_else(|| corrupt("dzr truncated"))?;
        return Ok((spawn, f32::from(angle_y) * RAD_PER_UNIT));
    }
    Err(corrupt("dzr spawn point missing"))
}

fn read_u32(data: &[u8], at: usize) -> Option<u32> {
    let bytes = data.get(at..at + 4)?;
    Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_f32(data: &[u8], at: usize) -> Option<f32> {
    let bytes = data.get(at..at + 4)?;
    Some(f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_i16(data: &[u8], at: usize) -> Option<i16> {
    let bytes = data.get(at..at + 2)?;
    Some(i16::from_be_bytes([bytes[0], bytes[1]]))
}
