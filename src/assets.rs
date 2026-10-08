use bevy_ecs::prelude::Resource;
use std::path::{Path, PathBuf};

use crate::archive::{yaz0_decode, Archive, AssetError, Rel, YAZ0_MAGIC};

pub const ASSETS_ENV: &str = "CS_ASSETS";

const STAGE_ROOT: &str = "files/res/Stage";
pub const STAGE_SPAWN_POINT: u8 = 128;
const STAGE_START_ROOM: u8 = 44;
pub const SEA_STAGE: &str = "sea";
const ROOM_MODELS: [&str; 4] = [
    "bdl/model.bdl",
    "bdl/model1.bdl",
    "bdl/model2.bdl",
    "bdl/model3.bdl",
];
const RAD_PER_UNIT: f32 = std::f32::consts::TAU / 65536.0;
const OBJECT_ALWAYS: &str = "files/res/Object/Always.arc";
const SEA_TEXTURE_ENTRY: &str = "tex/b_sea_tex0and2.bti";
const BTI_IMAGE_OFFSET: usize = 0x1C;

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

    pub fn read_rel(&self, rel: &str) -> Result<Rel, AssetError> {
        let path = self.root.join(rel);
        let bytes = self.read(rel)?;
        open_rel(bytes, path)
    }
}

fn open_rel(bytes: Vec<u8>, path: PathBuf) -> Result<Rel, AssetError> {
    if bytes.len() >= 4 && bytes[..4] == YAZ0_MAGIC {
        let raw = yaz0_decode(&bytes, &path)?;
        Rel::parse(raw, path)
    } else {
        Rel::parse(bytes, path)
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


pub fn asset_root() -> PathBuf {
    AssetRoot::from_env().map_or_else(|_| PathBuf::new(), |root| root.root.clone())
}

pub fn load_font() -> Option<Vec<u8>> {
    let root = asset_root();
    std::fs::read(root.join(FONT_FILE)).ok()
}

pub fn load_stage(stage: &str) -> Option<crate::sim::collision::StageData> {
    match stage_from_assets(stage) {
        Ok(stage) => Some(stage),
        Err(error) => {
            log::warn!("clearsleeper: stage assets unavailable ({error})");
            None
        }
    }
}

pub fn load_room_mesh(stage: &str) -> Option<crate::j3d::bdl::RoomMesh> {
    match room_mesh_from_assets(stage) {
        Ok(mesh) => Some(mesh),
        Err(error) => {
            log::warn!("clearsleeper: room mesh unavailable ({error})");
            None
        }
    }
}

const OBJECT_ROOT: &str = "files/res/Object";

/// The character model `dComIfG_getObjectRes` hands the player, `res/Object/Link.arc`'s `cl.bdl`.
///
/// `Link.arc` is a parts archive, not a body: its 54 `bdl` entries are the weapons, the shield, the
/// hat and the `hands`, and `cl` is the only one whose joint tree is the player. Every `bdl` entry
/// carries the `J3D2bdl4` magic and a `JNT1` block, so the name is the only thing that picks the
/// body out, which is why taking the first `bdl` that parses returns `binarify`-scale sub-models
/// rather than Link.
pub const LINK_MODEL_ENTRY: &str = "cl.bdl";

/// The body `res/Object/<name>.arc` carries, read by entry name where one is named.
///
/// Object archives are named for what they hold, and a character archive holds several `bdl`
/// entries, so the first one that parses is not the character. An archive with no named match
/// falls back to the first `bdl` with a joint tree, which is what an archive of one body wants.
pub fn load_actor_model(archive_name: &str) -> Option<crate::j3d::bdl::RoomMesh> {
    actor_model_entry(archive_name, None)
}

/// [`load_actor_model`] for an archive whose body entry is known.
pub fn load_actor_model_entry(archive_name: &str, entry_name: &str) -> Option<crate::j3d::bdl::RoomMesh> {
    actor_model_entry(archive_name, Some(entry_name))
}

fn actor_model_entry(archive_name: &str, entry_name: Option<&str>) -> Option<crate::j3d::bdl::RoomMesh> {
    let path = asset_root().join(format!("{OBJECT_ROOT}/{archive_name}.arc"));
    let bytes = std::fs::read(&path).ok()?;
    let archive = match open_archive(bytes, path) {
        Ok(archive) => archive,
        Err(error) => {
            log::warn!("clearsleeper: {archive_name} unreadable ({error})");
            return None;
        }
    };
    let entries: Vec<String> = archive
        .entries()
        .into_iter()
        .filter(|entry| entry.ends_with(".bdl"))
        .map(str::to_owned)
        .collect();
    if entries.is_empty() {
        log::warn!("clearsleeper: {archive_name} holds no bdl model");
        return None;
    }
    let ordered = match entry_name {
        Some(wanted) => match entries.iter().find(|entry| entry.as_str() == wanted) {
            Some(found) => vec![found.clone()],
            None => {
                log::warn!("clearsleeper: {archive_name} has no {wanted}");
                return None;
            }
        },
        None => entries.clone(),
    };
    for entry in ordered {
        let model = match archive.read(entry.as_str()) {
            Ok(model) => model,
            Err(AssetError::MissingEntry { .. }) => continue,
            Err(_) => return None,
        };
        match crate::j3d::bdl::parse(model, [0.0, 0.0], 0.0, None) {
            Ok(mesh) if !mesh.joints.is_empty() => return Some(mesh),
            Ok(_) => continue,
            Err(error) => {
                log::warn!(
                    "clearsleeper: {archive_name}/{entry} unusable ({})",
                    error.as_str()
                );
                return None;
            }
        }
    }
    None
}

/// The bind-pose joint frames of an actor model, which `JntHit` colliders are defined against.
/// `res/Object/<name>.arc` holds the `bdl` entry.
pub fn load_actor_joints(archive_name: &str) -> Option<Vec<[f32; 3]>> {
    Some(load_actor_model(archive_name)?.joints)
}

/// [`load_actor_joints`] for an archive whose body entry is known.
pub fn load_actor_joints_entry(archive_name: &str, entry_name: &str) -> Option<Vec<[f32; 3]>> {
    Some(load_actor_model_entry(archive_name, entry_name)?.joints)
}

pub fn load_event_list(stage: &str) -> Option<crate::sim::event::EventList> {
    match event_list_from_assets(stage) {
        Ok(list) => Some(list),
        Err(error) => {
            log::warn!("clearsleeper: event list unavailable ({error})");
            None
        }
    }
}

const LINK_ANM_ARCHIVE: &str = "files/res/Object/LkAnm.arc";

pub fn load_link_anms() -> Option<crate::sim::event::LinkAnms> {
    match link_anms_from_assets() {
        Ok(anms) => Some(anms),
        Err(error) => {
            log::warn!("clearsleeper: link animations unavailable ({error})");
            None
        }
    }
}

fn link_anms_from_assets() -> Result<crate::sim::event::LinkAnms, AssetError> {
    let root = AssetRoot::from_env()?;
    let archive = root.read_archive(LINK_ANM_ARCHIVE)?;
    let path = root.root().join(LINK_ANM_ARCHIVE);
    let mut anms = crate::sim::event::LinkAnms::default();
    for stem in crate::sim::event::LINK_ANIM_STEMS {
        let Ok(data) = archive.read(&format!("bcks/{stem}.bck")) else {
            log::warn!("clearsleeper: missing link animation {stem}");
            continue;
        };
        let raw = if data.len() >= 4 && data[..4] == YAZ0_MAGIC {
            let Ok(raw) = yaz0_decode(data, &path) else {
                log::warn!("clearsleeper: failed to expand link animation {stem}");
                continue;
            };
            raw
        } else {
            data.to_vec()
        };
        match bck_anm(&raw) {
            Some((attribute, frames)) => anms.insert(stem, attribute, frames),
            None => log::warn!("clearsleeper: unreadable link animation {stem}"),
        }
    }
    Ok(anms)
}

fn bck_anm(data: &[u8]) -> Option<(u8, u16)> {
    if data.len() < 0x20 || data[..4] != *b"J3D1" || data[4..8] != *b"bck1" {
        return None;
    }
    let blocks = read_u32(data, 0x0C)? as usize;
    let mut at = 0x20;
    for _ in 0..blocks {
        if data.len() < usize::try_from(at).unwrap_or(usize::MAX).saturating_add(0x0C) || read_u32(data, at + 4)? < 8 {
            return None;
        }
        if data[at..at + 4] == *b"ANK1" {
            let frames = read_i16(data, at + 0x0A)?;
            return Some((data[at + 8], frames.max(0) as u16));
        }
        at = at.saturating_add(read_u32(data, at + 4)? as usize);
    }
    None
}

#[derive(Resource, Debug)]
pub struct MsgArchives {
    pub normal: Option<Archive>,
    pub hyrule: Option<Archive>,
}

impl MsgArchives {
    pub fn load(root: &AssetRoot) -> Self {
        Self {
            normal: root.read_archive(MSG_ARCHIVE).ok(),
            hyrule: root.read_archive(MSG_ARCHIVE_HYRULE).ok(),
        }
    }

    pub fn available(&self) -> bool {
        self.normal.is_some()
    }
}

impl crate::text::msgget::MsgArchive for MsgArchives {
    fn resource(&self, name: &str) -> Option<&[u8]> {
        self.normal
            .as_ref()
            .and_then(|archive| archive.read(name).ok())
            .or_else(|| {
                self.hyrule
                    .as_ref()
                    .and_then(|archive| archive.read(name).ok())
            })
    }
}

const FONT_FILE: &str = "files/res/Menu/kanfont_fix16.bfn";
const MSG_ARCHIVE: &str = "files/res/Msg/bmgres.arc";
const MSG_ARCHIVE_HYRULE: &str = "files/res/Msg/bmgresh.arc";
const MSG_LINE_MAX: u16 = 4;

pub fn load_msg_table() -> Option<crate::sim::msg::MsgTable> {
    match msg_table_from_assets() {
        Ok(table) => Some(table),
        Err(error) => {
            log::warn!("clearsleeper: message text unavailable ({error})");
            None
        }
    }
}

fn msg_table_from_assets() -> Result<crate::sim::msg::MsgTable, AssetError> {
    let root = AssetRoot::from_env()?;
    let mut table = crate::sim::msg::MsgTable::default();
    let archive = root.read_archive(MSG_ARCHIVE)?;
    let data = archive.read("zel_00.bmg")?;
    if msg_table_from_bmg(data, &mut table).is_none() {
        return Err(AssetError::Corrupt {
            path: root.root().join(MSG_ARCHIVE),
            detail: "message archive header is malformed",
        });
    }
    match root.read_archive(MSG_ARCHIVE_HYRULE) {
        Ok(hyrule) => match hyrule.read("zel_01.bmg") {
            Ok(data) => {
                if msg_table_from_bmg(data, &mut table).is_none() {
                    log::warn!("clearsleeper: hyrule message archive header is malformed");
                }
            }
            Err(error) => log::warn!("clearsleeper: hyrule message entry unavailable ({error})"),
        },
        Err(error) => log::warn!("clearsleeper: hyrule message archive unavailable ({error})"),
    }
    Ok(table)
}

pub fn load_msg_archives() -> MsgArchives {
    match AssetRoot::from_env() {
        Ok(root) => MsgArchives::load(&root),
        Err(_) => MsgArchives {
            normal: None,
            hyrule: None,
        },
    }
}

pub fn msg_table_from_bmg(data: &[u8], table: &mut crate::sim::msg::MsgTable) -> Option<()> {
    if data.len() < 0x20 || data[0..4] != *b"MESG" || data[4..8] != *b"bmg1" {
        return None;
    }
    let blocks = read_u32(data, 0x0C)? as usize;
    let mut at = 0x20;
    let mut info = None;
    let mut text_block = None;
    for _ in 0..blocks {
        let size = read_u32(data, at + 4)? as usize;
        if size < 8 || data.len() < at.saturating_add(size) {
            return None;
        }
        let tag = [data[at], data[at + 1], data[at + 2], data[at + 3]];
        if tag == *b"INF1" {
            info = Some((at, size));
        } else if tag == *b"DAT1" {
            text_block = Some((at, size));
        }
        at += size;
    }
    let (info, info_size) = info?;
    let (text_at, text_size) = text_block?;
    let count = usize::from(read_u16(data, info + 8)?);
    if read_u16(data, info + 0x0A)? != 0x18 || 0x10 + count * 0x18 > info_size {
        return None;
    }
    let text = data.get(text_at + 8..text_at + text_size)?;
    let mut found = Vec::with_capacity(count);
    for index in 0..count {
        let entry = info + 0x10 + index * 0x18;
        let Some(offset) = read_u32(data, entry) else {
            return None;
        };
        let Some(msg_no) = read_u16(data, entry + 4) else {
            return None;
        };
        let (Some(textbox_type), Some(draw_type)) = (
            data.get(entry + 0x0C).copied(),
            data.get(entry + 0x0D).copied(),
        ) else {
            return None;
        };
        let offset = offset as usize;
        if offset == 0 || offset >= text.len() {
            continue;
        }
        let Some(bytes) = bmg_text(text, offset) else {
            continue;
        };
        found.push((
            u32::from(msg_no),
            crate::sim::msg::MsgEntry {
                text: bytes.to_vec(),
                draw_type,
                textbox_type,
                lines: MSG_LINE_MAX,
            },
        ));
    }
    for (msg_no, entry) in found {
        table.insert(msg_no, entry);
    }
    Some(())
}

fn bmg_text(data: &[u8], start: usize) -> Option<&[u8]> {
    let mut at = start;
    while at < data.len() {
        let byte = data[at];
        if byte == 0 {
            return Some(&data[start..=at]);
        }
        if byte == 0x1A {
            let size = usize::from(*data.get(at + 1)?);
            if size == 0 {
                return None;
            }
            at = at.saturating_add(size);
            continue;
        }
        if byte >> 4 == 8 || byte >> 4 == 9 {
            at += 2;
            continue;
        }
        at += 1;
    }
    None
}

fn event_list_from_assets(stage: &str) -> Result<crate::sim::event::EventList, AssetError> {
    let root = AssetRoot::from_env()?;
    let map = root.read_archive(&stage_map(stage))?;
    let path = root.root().join(stage_map(stage));
    let data = map.read(crate::sim::event::EVENT_LIST_ENTRY)?;
    crate::sim::event::EventList::parse(data).map_err(|_| AssetError::Corrupt {
        path,
        detail: "event list header is malformed",
    })
}

pub fn load_stage_tables(stage: &str) -> Option<crate::dzs::StageTables> {
    match stage_tables_from_assets(stage) {
        Ok(tables) => Some(tables),
        Err(error) => {
            log::warn!("clearsleeper: stage tables unavailable ({error})");
            None
        }
    }
}

pub fn load_stage_spawn(stage: &str, room: u8, point: u8) -> Option<(glam::Vec3, f32)> {
    match stage_spawn_from_assets(stage, room, point) {
        Ok(spawn) => Some(spawn),
        Err(error) => {
            log::warn!("clearsleeper: stage spawn unavailable ({error})");
            None
        }
    }
}

pub fn load_sea_shade() -> Option<f32> {
    match sea_shade_from_assets() {
        Ok(shade) => Some(shade),
        Err(error) => {
            log::warn!("clearsleeper: sea texture unavailable ({error})");
            None
        }
    }
}

fn sea_shade_from_assets() -> Result<f32, AssetError> {
    let root = AssetRoot::from_env()?;
    let archive = root.read_archive(OBJECT_ALWAYS)?;
    let path = root.root().join(OBJECT_ALWAYS);
    let bti = archive.read(SEA_TEXTURE_ENTRY)?;
    sea_shade(bti).ok_or(AssetError::Corrupt {
        path,
        detail: "sea texture unsupported",
    })
}

fn sea_shade(bti: &[u8]) -> Option<f32> {
    if bti.first() != Some(&0) {
        return None;
    }
    let width = u16::from_be_bytes([*bti.get(2)?, *bti.get(3)?]) as usize;
    let height = u16::from_be_bytes([*bti.get(4)?, *bti.get(5)?]) as usize;
    let offset = u32::from_be_bytes([
        *bti.get(BTI_IMAGE_OFFSET)?,
        *bti.get(BTI_IMAGE_OFFSET + 1)?,
        *bti.get(BTI_IMAGE_OFFSET + 2)?,
        *bti.get(BTI_IMAGE_OFFSET + 3)?,
    ]) as usize;
    let pixels = width.checked_mul(height)?;
    let data = bti.get(offset..offset + pixels.div_ceil(2))?;
    let total: u64 = data
        .iter()
        .map(|byte| u64::from(byte >> 4) + u64::from(byte & 0x0F))
        .sum();
    Some(total as f32 / (15.0 * pixels as f32))
}

pub fn room_set_for(tables: &crate::dzs::StageTables, room: u8) -> Vec<u8> {
    let set = tables.load_set(room);
    if set.is_empty() {
        vec![room]
    } else {
        set.to_vec()
    }
}

pub fn start_set(tables: &crate::dzs::StageTables) -> Vec<u8> {
    room_set_for(tables, STAGE_START_ROOM)
}

pub struct RoomSet {
    pub collision: Option<crate::sim::collision::Collision>,
    pub mesh: Option<crate::j3d::bdl::RoomMesh>,
    pub actors: Vec<crate::dzs::StageActor>,
    /// The per-room camera and arrow chunks, keyed by room number, which is how the reference
    /// reaches them: `dComIfGp_getRoomCamera(i_roomNo)` (`d_camera.cpp:1484`).
    pub cameras: Vec<(u8, crate::dzs::StageCameras)>,
}

pub fn load_room_set(
    tables: &crate::dzs::StageTables,
    rooms: &[u8],
    layer: impl Fn(u8) -> u8,
) -> Option<RoomSet> {
    let loaded =
        AssetRoot::from_env().and_then(|root| room_set_from_assets(&root, tables, rooms, &layer));
    match loaded {
        Ok(set) => Some(set),
        Err(error) => {
            log::warn!("clearsleeper: room set {rooms:?} unavailable ({error})");
            None
        }
    }
}

fn stage_map(stage: &str) -> String {
    format!("{STAGE_ROOT}/{stage}/Stage.arc")
}

fn stage_tables_from_assets(stage: &str) -> Result<crate::dzs::StageTables, AssetError> {
    let root = AssetRoot::from_env()?;
    stage_tables(&root, stage)
}

fn stage_tables(root: &AssetRoot, stage: &str) -> Result<crate::dzs::StageTables, AssetError> {
    let map = root.read_archive(&stage_map(stage))?;
    let path = root.root().join(stage_map(stage));
    let data = map.read("dzs/stage.dzs")?;
    crate::dzs::parse(data)
        .map(|tables| tables.with_name(stage))
        .map_err(|error| AssetError::Corrupt {
            path,
            detail: error.as_str(),
        })
}

fn fresh_layer(tables: &crate::dzs::StageTables) -> impl Fn(u8) -> u8 + '_ {
    let stories = crate::sim::flags::StoryBits::default();
    move |room| crate::sim::layer::no(tables.name(), room, &stories)
}

fn stage_from_assets(stage: &str) -> Result<crate::sim::collision::StageData, AssetError> {
    let root = AssetRoot::from_env()?;
    let tables = stage_tables(&root, stage)?;
    let rooms = start_set(&tables);
    let set = room_set_from_assets(&root, &tables, &rooms, &fresh_layer(&tables))?;
    let collision = set.collision.ok_or_else(|| AssetError::Corrupt {
        path: root.root().join(room_path(stage, STAGE_START_ROOM)),
        detail: "room set has no collision",
    })?;
    let relative = room_path(stage, STAGE_START_ROOM);
    let path = root.root().join(&relative);
    let room = root.read_archive(&relative)?;
    let (spawn, yaw) = stage_spawn(room.read("dzr/room.dzr")?, path, STAGE_SPAWN_POINT)?;
    Ok(crate::sim::collision::StageData {
        spawn,
        yaw,
        collision,
    })
}

fn stage_spawn_from_assets(
    stage: &str,
    room: u8,
    point: u8,
) -> Result<(glam::Vec3, f32), AssetError> {
    let root = AssetRoot::from_env()?;
    let relative = room_path(stage, room);
    let path = root.root().join(&relative);
    let archive = root.read_archive(&relative)?;
    stage_spawn(archive.read("dzr/room.dzr")?, path, point)
}

fn room_mesh_from_assets(stage: &str) -> Result<crate::j3d::bdl::RoomMesh, AssetError> {
    let root = AssetRoot::from_env()?;
    let tables = stage_tables(&root, stage)?;
    let rooms = start_set(&tables);
    let set = room_set_from_assets(&root, &tables, &rooms, &fresh_layer(&tables))?;
    set.mesh.ok_or_else(|| AssetError::Corrupt {
        path: root.root().join(room_path(stage, STAGE_START_ROOM)),
        detail: "room set has no geometry",
    })
}

fn room_set_from_assets(
    root: &AssetRoot,
    tables: &crate::dzs::StageTables,
    rooms: &[u8],
    layer: &impl Fn(u8) -> u8,
) -> Result<RoomSet, AssetError> {
    let mut collision: Option<crate::sim::collision::Collision> = None;
    let mut mesh: Option<crate::j3d::bdl::RoomMesh> = None;
    let mut actors: Vec<crate::dzs::StageActor> = Vec::new();
    let mut cameras: Vec<(u8, crate::dzs::StageCameras)> = Vec::new();
    for &room in rooms {
        let relative = room_path(tables.name(), room);
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
        let mut room_part = crate::j3d::bdl::RoomMesh::default();
        for name in ROOM_MODELS {
            let model = match archive.read(name) {
                Ok(model) => model,
                Err(AssetError::MissingEntry { .. }) => continue,
                Err(error) => return Err(error),
            };
            let btk_name = name.replacen("bdl/", "btk/", 1).replacen(".bdl", ".btk", 1);
            let animation = match archive.read(&btk_name) {
                Ok(data) => Some(crate::j3d::btk::parse(data).map_err(|error| {
                    AssetError::Corrupt {
                        path: path.clone(),
                        detail: error.as_str(),
                    }
                })?),
                Err(AssetError::MissingEntry { .. }) => None,
                Err(error) => return Err(error),
            };
            let part = crate::j3d::bdl::parse(model, origin, angle, animation.as_ref()).map_err(
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
        match archive.read("dzr/room.dzr") {
            Ok(data) => {
                match crate::dzs::room_actors(data, room as i8, layer(room)) {
                    Ok(placed) => actors.extend(placed),
                    Err(error) => {
                        log::warn!(
                            "clearsleeper: room {room} actor table unusable ({})",
                            error.as_str()
                        );
                    }
                }
                if let Ok(parsed) = crate::dzs::room_cameras(data) {
                    cameras.push((room, parsed));
                }
            }
            Err(AssetError::MissingEntry { .. }) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(RoomSet {
        collision,
        mesh,
        actors,
        cameras,
    })
}

fn table_placement(tables: &crate::dzs::StageTables, room: u8) -> ([f32; 2], f32) {
    match tables.placement(room) {
        Some(placement) => (placement.trans, f32::from(placement.angle) * RAD_PER_UNIT),
        None => ([0.0, 0.0], 0.0),
    }
}

fn room_path(stage: &str, room: u8) -> String {
    format!("{STAGE_ROOT}/{stage}/Room{room}.arc")
}

pub fn stage_spawn(data: &[u8], path: PathBuf, point: u8) -> Result<(glam::Vec3, f32), AssetError> {
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
        if angle_z as u8 != point {
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

fn read_u16(data: &[u8], at: usize) -> Option<u16> {
    let bytes = data.get(at..at + 2)?;
    Some(u16::from_be_bytes([bytes[0], bytes[1]]))
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
