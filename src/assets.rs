use bevy_ecs::prelude::Resource;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::archive::{Archive, AssetError, Rel, YAZ0_MAGIC, yaz0_decode};
use crate::disc::DiscImage;

/// The extracted tree [`AssetRoot::from_env`] reads when no disc image is installed. A
/// development fallback on every platform - a dump on disk, a debugger watching loose files -
/// not the path a session ships on; the disc image is.
pub const ASSETS_ENV: &str = "CS_ASSETS";

const STAGE_ROOT: &str = "files/res/Stage";
pub const STAGE_SPAWN_POINT: u8 = 128;
pub const STAGE_START_ROOM: u8 = 44;
pub const SEA_STAGE: &str = "sea";
/// The opening demo's next stage: `dComIfGp_setNextStage("sea_T", 0, 44, 0)`
/// (`d_com_inf_game.cpp:496`). Its start point 0 is the one on Outset; the `sea` stage's
/// point 128 puts the player on the water and is not the prologue's spawn.
pub const PROLOGUE_STAGE: &str = "sea_T";
pub const PROLOGUE_SPAWN_POINT: u8 = 0;
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

/// The asset tree the `load_*` functions read: a disc image installed before boot (the web drop
/// zone, a desktop `--disc`/default image, the Android picker; [`AssetRoot::from_disc`]), or the
/// extracted disc under [`ASSETS_ENV`] when no image is installed. Both answer the same relative
/// paths, from `files/res/Stage/sea/Room44.arc` down, so nothing above this type knows which of
/// the two it is reading.
pub struct AssetRoot {
    source: Source,
}

enum Source {
    Files(PathBuf),
    Disc(Arc<DiscImage>),
}

impl AssetRoot {
    /// The session's asset source: an installed disc image first (the web drop, a desktop
    /// `--disc`/default image, the Android picker), then the extracted [`ASSETS_ENV`] tree. The
    /// image outranks the tree on every platform, so a machine holding both reads the disc; the
    /// tree is the development fallback, not a second choice of content.
    pub fn from_env() -> Result<Self, AssetError> {
        if let Some(image) = installed_disc() {
            return Ok(Self {
                source: Source::Disc(image),
            });
        }
        if let Some(value) = std::env::var_os(ASSETS_ENV).filter(|value| !value.is_empty()) {
            return Ok(Self {
                source: Source::Files(PathBuf::from(value)),
            });
        }
        Err(missing_root())
    }

    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self {
            source: Source::Files(root.into()),
        }
    }

    /// An asset root over a disc image already in memory: what an installed source resolves to,
    /// and the seam the tests exercise the image reader through.
    pub fn from_disc(image: Arc<DiscImage>) -> Self {
        Self {
            source: Source::Disc(image),
        }
    }

    /// The filesystem root behind an extracted tree. Absent for a disc image, which has no path:
    /// the callers that hand a path to another library (the title movie's `THP`) find nothing
    /// there, as they do today on the web.
    pub fn fs_root(&self) -> Option<&Path> {
        match &self.source {
            Source::Files(root) => Some(root),
            Source::Disc(_) => None,
        }
    }

    /// The label an asset path carries in errors: its path under the extracted root, or
    /// `GAMEID:relative` when it came out of an image.
    pub fn path(&self, rel: &str) -> PathBuf {
        match &self.source {
            Source::Files(root) => root.join(rel),
            Source::Disc(image) => image.path(rel),
        }
    }

    pub fn read(&self, rel: &str) -> Result<Vec<u8>, AssetError> {
        match &self.source {
            Source::Files(root) => {
                if !root.is_dir() {
                    return Err(AssetError::MissingRoot { root: root.clone() });
                }
                let path = root.join(rel);
                std::fs::read(&path).map_err(|source| match source.kind() {
                    std::io::ErrorKind::NotFound => AssetError::MissingFile { path },
                    _ => AssetError::Io { path, source },
                })
            }
            Source::Disc(image) => image.read(rel).map_err(|source| match source {
                crate::disc::DiscError::MissingFile { .. } => AssetError::MissingFile {
                    path: self.path(rel),
                },
                source => AssetError::Disc {
                    path: self.path(rel),
                    source,
                },
            }),
        }
    }

    pub fn read_archive(&self, rel: &str) -> Result<Archive, AssetError> {
        let path = self.path(rel);
        let bytes = self.read(rel)?;
        open_archive(bytes, path)
    }

    pub fn read_rel(&self, rel: &str) -> Result<Rel, AssetError> {
        let path = self.path(rel);
        let bytes = self.read(rel)?;
        open_rel(bytes, path)
    }
}

fn missing_root() -> AssetError {
    #[cfg(target_arch = "wasm32")]
    let root = format!("<unset {ASSETS_ENV} and no disc image installed>");
    #[cfg(not(target_arch = "wasm32"))]
    let root = format!("<no disc image installed and {ASSETS_ENV} unset>");
    AssetError::MissingRoot {
        root: PathBuf::from(root),
    }
}

/// The disc image an installed source was parsed from - the web drop zone's, the desktop
/// entry's, the Android picker's - which [`AssetRoot::from_env`] answers with.
static INSTALLED: std::sync::Mutex<Option<Arc<DiscImage>>> = std::sync::Mutex::new(None);

/// Parse a disc image and install it as the source [`AssetRoot::from_env`] answers with. The
/// bytes are taken by value, so a platform that already holds them in a shareable buffer does
/// not copy the disc to hand it over.
pub fn install_disc_image(
    bytes: impl Into<Vec<u8>>,
) -> Result<crate::disc::DiscSummary, crate::disc::DiscError> {
    let image = Arc::new(DiscImage::open(bytes.into())?);
    let summary = image.summary();
    let mut slot = INSTALLED.lock().unwrap_or_else(|error| error.into_inner());
    *slot = Some(image);
    Ok(summary)
}

fn installed_disc() -> Option<Arc<DiscImage>> {
    let slot = INSTALLED.lock().unwrap_or_else(|error| error.into_inner());
    slot.clone()
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

pub fn load_font() -> Option<Vec<u8>> {
    let root = AssetRoot::from_env().ok()?;
    let archive = root.read_archive(FONT_FILE).ok()?;
    archive.read(MESSAGE_FONT).ok().map(<[u8]>::to_vec)
}

pub fn load_stage(stage: &str) -> Option<crate::sim::collision::StageData> {
    load_stage_at(stage, STAGE_SPAWN_POINT)
}

/// Load a stage's collision and the start point `spawn_point` on its first room. `sea_T` uses
/// point 0 while `sea` uses [`STAGE_SPAWN_POINT`], so the spawn is part of the stage choice.
pub fn load_stage_at(stage: &str, spawn_point: u8) -> Option<crate::sim::collision::StageData> {
    match stage_from_assets(stage, spawn_point) {
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
pub fn load_actor_model_entry(
    archive_name: &str,
    entry_name: &str,
) -> Option<crate::j3d::bdl::RoomMesh> {
    actor_model_entry(archive_name, Some(entry_name))
}

fn actor_model_entry(
    archive_name: &str,
    entry_name: Option<&str>,
) -> Option<crate::j3d::bdl::RoomMesh> {
    let relative = format!("{OBJECT_ROOT}/{archive_name}.arc");
    let archive = match AssetRoot::from_env().and_then(|root| root.read_archive(&relative)) {
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

/// `dComIfG_setObjectRes(name, ...)` (`d_com_inf_game.h:4066-4068`): mount one `res/Object`
/// archive. The room's demo archive is the caller - `d_s_room.cpp:230` mounts the `Demo%02d`
/// `:229` names - and the objects come out of the mounted archive through [`object_model`].
pub fn load_object_archive(name: &str) -> Option<Archive> {
    let relative = format!("{OBJECT_ROOT}/{name}.arc");
    match AssetRoot::from_env().and_then(|root| root.read_archive(&relative)) {
        Ok(archive) => Some(archive),
        Err(error) => {
            log::warn!("clearsleeper: object archive {name} unavailable ({error})");
            None
        }
    }
}

/// `dComIfG_getObjectIDRes(arcName, id)` (`d_com_inf_game.h:4118-4120`) over an archive this port
/// has already read: the `bdl` under the entry's `mFileId`, which is the lookup
/// `daDemo00_c::createHeap` makes for a cut's model (`d_a_demo00.cpp:169`). The shape is parsed
/// the way [`load_actor_model_entry`] parses one; an id the archive does not carry, or a body
/// this port cannot parse, is `None`, where the reference reads a NULL pointer.
pub fn object_model(archive: &Archive, id: u16) -> Option<crate::j3d::bdl::RoomMesh> {
    let model = archive.read_id(id).ok()?;
    match crate::j3d::bdl::parse(model, [0.0, 0.0], 0.0, None) {
        Ok(mesh) if !mesh.joints.is_empty() => Some(mesh),
        _ => None,
    }
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
    let path = root.path(LINK_ANM_ARCHIVE);
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
    let (attribute, frames) = anm_block(data, b"J3D1bck1", b"ANK1")?;
    Some((attribute, frames.max(0) as u16))
}

/// The `ANK1` block of a `J3D1bck1` or the `TPT1` block of a `J3D1btp1`
/// (`J3DAnmLoaderDataBase::load`, `J3DAnmLoader.cpp:106-160`): `JUTDataBlockHeader`'s type
/// and size, then the payload `J3DAnmTransformKeyData` and `J3DAnmTexPatternFullData` both
/// put `field_0x8` at `0x08` and `mFrameMax` at `0x0A` (`J3DAnimation.h:195-220`), which
/// `J3DAnmKeyLoader_v15::readAnmTransform` and `J3DAnmFullLoader_v15::readAnmTexPattern`
/// (`J3DAnmLoader.cpp:294-306`) copy into the object's own `mFrameMax`.
fn anm_block(data: &[u8], magic: &[u8; 8], tag: &[u8; 4]) -> Option<(u8, i16)> {
    if data.len() < 0x20 || data[..8] != *magic {
        return None;
    }
    let blocks = read_u32(data, 0x0C)? as usize;
    let mut at = 0x20;
    for _ in 0..blocks {
        let size = read_u32(data, at + 4)? as usize;
        if size < 8 || at.checked_add(size)? > data.len() {
            return None;
        }
        if data[at..at + 4] == *tag {
            return Some((data[at + 8], read_i16(data, at + 0x0A)?));
        }
        at = at.saturating_add(size);
    }
    None
}

const BA_OBJECT_ARCHIVE: &str = "files/res/Object/Ba.arc";

/// The `Ba` object archive's animation frame counts, `getFrameMax()`'s only source on this
/// side of the disc.
pub fn load_ba_anms() -> Option<crate::sim::ba1::Ba1Anms> {
    match ba_anms_from_assets() {
        Ok(anms) => Some(anms),
        Err(error) => {
            log::warn!("clearsleeper: Ba animations unavailable ({error})");
            None
        }
    }
}

/// Every `dRes_ID_BA_BCK_*` of `anmNum_toResID` and every `dRes_ID_BA_BTP_*` of
/// `btpNum_toResID` (`d_a_npc_ba1.cpp:301-336`), read through the archive id that
/// `dComIfG_getObjectIDRes("Ba", ...)` looks up.
fn ba_anms_from_assets() -> Result<crate::sim::ba1::Ba1Anms, AssetError> {
    let root = AssetRoot::from_env()?;
    let archive = root.read_archive(BA_OBJECT_ARCHIVE)?;
    let path = root.path(BA_OBJECT_ARCHIVE);
    let mut anms = crate::sim::ba1::Ba1Anms::default();
    let ids = crate::sim::ba1::BCK
        .iter()
        .map(|id| (*id, *b"J3D1bck1", *b"ANK1"))
        .chain(
            crate::sim::ba1::BTP
                .iter()
                .map(|id| (*id, *b"J3D1btp1", *b"TPT1")),
        );
    for (id, magic, tag) in ids {
        let Ok(id) = u16::try_from(id) else {
            continue;
        };
        let Ok(data) = archive.read_id(id) else {
            log::warn!("clearsleeper: Ba.arc has no animation id {id:#06x}");
            continue;
        };
        let raw = if data.len() >= 4 && data[..4] == YAZ0_MAGIC {
            let Ok(raw) = yaz0_decode(data, &path) else {
                log::warn!("clearsleeper: failed to expand Ba animation {id:#06x}");
                continue;
            };
            raw
        } else {
            data.to_vec()
        };
        match anm_block(&raw, &magic, &tag) {
            Some((_, frames)) => anms.insert(id, frames),
            None => log::warn!("clearsleeper: unreadable Ba animation {id:#06x}"),
        }
    }
    Ok(anms)
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

/// `mDoExt_getMesgFont`'s mount, `/res/Msg/fontres.arc` (`d_s_logo.cpp:926`).
const FONT_FILE: &str = "files/res/Msg/fontres.arc";
/// The USA message font (`m_Do_ext.cpp:3148-3151`), the one `d_s_open_sub.cpp:177` hands the
/// opening's text boxes. The menu's `kanfont` is a kanji sheet and leaves Latin text blank.
const MESSAGE_FONT: &str = "rock_24_20_4i_usa.bfn";
/// `dComIfGp_getMsgDtArchive()`'s mount - `/res/Msg/bmgres.arc` (`d_s_logo.cpp:911-917`).
const MSG_ARCHIVE: &str = "files/res/Msg/bmgres.arc";
/// `dComIfGp_getMsgDt2Archive()`'s mount - `/res/Msg/bmgresh.arc` (`:919-921`).
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

/// `dMesg_parse` (`d_mesg.cpp:2181-2202`): both of the game's message resources, each parsed into
/// the same container under the group its own `INF1` block carries. A `bmg` the dump does not ship
/// leaves its group absent from the table, which is the state a demo code for that group is
/// reported in.
fn msg_table_from_assets() -> Result<crate::sim::msg::MsgTable, AssetError> {
    let root = AssetRoot::from_env()?;
    let mut table = crate::sim::msg::MsgTable::default();
    let archive = root.read_archive(MSG_ARCHIVE)?;
    let data = archive.read("zel_00.bmg")?;
    if msg_table_from_bmg(data, &mut table).is_none() {
        return Err(AssetError::Corrupt {
            path: root.path(MSG_ARCHIVE),
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

/// One `bmg` into the table, under the group the file itself carries: `dMesg_parse` hands each
/// archive to `JMessage::TParse`, which reads the INF1 block's own `groupID`
/// (`d_mesg.cpp:2186-2202`, `resource.cpp`, `data.h:64-73`), so the group is a property of the
/// `bmg`, not of the archive it arrived in.
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
    let group = read_u16(data, info + 0x0C)?;
    if read_u16(data, info + 0x0A)? != 0x18 || 0x10 + count * 0x18 > info_size {
        return None;
    }
    // `TResourceContainer::Get_groupID` answers with the first resource carrying the group, so a
    // second `bmg` claiming one already in the container is unreachable: its entry list must not
    // be appended to the first's, which would renumber every entry the position read indexes.
    if table.has_group(group) {
        log::warn!("clearsleeper: message group {group} is already in the message container");
        return Some(());
    }
    let blob = data.get(text_at + 8..text_at + text_size)?;
    // Every row keeps its place: `TResource::getMessageEntry(messageIndex)` is a stride read
    // (`resource.h:34-44`), so a row the port cannot read text for is still the index's entry.
    for index in 0..count {
        let entry = info + 0x10 + index * 0x18;
        let offset = read_u32(data, entry)?;
        let msg_no = read_u16(data, entry + 4)?;
        let (Some(textbox_type), Some(draw_type)) = (
            data.get(entry + 0x0C).copied(),
            data.get(entry + 0x0D).copied(),
        ) else {
            return None;
        };
        let text = match usize::try_from(offset)
            .ok()
            .filter(|&offset| offset < blob.len())
            .and_then(|offset| bmg_text(blob, offset))
        {
            Some(bytes) => bytes.to_vec(),
            None => Vec::new(),
        };
        table.insert(
            group,
            crate::sim::msg::MsgEntry {
                msg_no: u32::from(msg_no),
                data_offs: offset,
                text,
                draw_type,
                textbox_type,
                lines: MSG_LINE_MAX,
            },
        );
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
    let path = root.path(&stage_map(stage));
    let data = map.read(crate::sim::event::EVENT_LIST_ENTRY)?;
    crate::sim::event::EventList::parse(data).map_err(|_| AssetError::Corrupt {
        path,
        detail: "event list header is malformed",
    })
}

/// The demo-file leg `specialProcPackage` falls back to: `dComIfG_getStageRes("Stage", filename)`
/// (`d_event_data.cpp:800-801`), an entry of the stage's own `Stage.arc` under the name the cut's
/// `FileName` key carries. The first leg - `dComIfG_getObjectRes(demo_arc, filename)` out of the
/// room's demo archive (`:796-799`) - is
/// [`crate::sim::demo::RoomDemoArchive::data`], which the `PACKAGE` staff reads before this one;
/// this function is the reference's own second try, not a substitute for the first.
pub fn load_demo_data(stage: &str, name: &str) -> Option<Vec<u8>> {
    let root = AssetRoot::from_env().ok()?;
    let relative = stage_map(stage);
    let archive = root.read_archive(&relative).ok()?;
    match archive.read(name) {
        Ok(data) => Some(data.to_vec()),
        Err(error) => {
            log::warn!("clearsleeper: {relative} has no demo entry {name} ({error})");
            None
        }
    }
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
    let path = root.path(OBJECT_ALWAYS);
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
    /// `dStage_roomControl_c::mDemoArcName` (`d_stage.h:933`) as `d_s_room.cpp:220-231` names it:
    /// the first room in the set whose `LBNK` carries a bank for the layer in play names
    /// `Demo%02d` (`:229`). `None` is a set where no room named one.
    pub demo_archive: Option<String>,
    /// `dStage_FileList_dt_GetParticleNo` (`d_stage.h:1118-1120`) off the first room in the set
    /// whose `FILI` record names a particle scene number - `d_s_room.cpp:237-241` reads each
    /// room's own record and hands it to `readScene`. `None` is a set whose rooms named none,
    /// and a room whose record reads `0xFF` names none either: the reader leaves its own
    /// first-read counter alone there (`d_particle.cpp:758-765`), which is what lets a later room
    /// answer instead.
    pub particle_no: Option<u8>,
    /// Each room's own `SCLS` table, keyed by room number: `dComIfG_roomControl_getStatusRoomDt
    /// (room_no)->getSclsInfo()` (`d_lib.cpp:245-246`), the table `dLib_setNextStageBySclsNum`
    /// resolves a scene number through when the caller passes a room. A room file without the
    /// chunk contributes nothing, which is the null table the reference returns on (`:248-250`).
    pub exits: Vec<(u8, Vec<crate::dzs::StageExit>)>,
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
    let path = root.path(&stage_map(stage));
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

fn stage_from_assets(
    stage: &str,
    spawn_point: u8,
) -> Result<crate::sim::collision::StageData, AssetError> {
    let root = AssetRoot::from_env()?;
    let tables = stage_tables(&root, stage)?;
    let rooms = start_set(&tables);
    let set = room_set_from_assets(&root, &tables, &rooms, &fresh_layer(&tables))?;
    let collision = set.collision.ok_or_else(|| AssetError::Corrupt {
        path: root.path(&room_path(stage, STAGE_START_ROOM)),
        detail: "room set has no collision",
    })?;
    let relative = room_path(stage, STAGE_START_ROOM);
    let path = root.path(&relative);
    let room = root.read_archive(&relative)?;
    let (spawn, yaw) = stage_spawn(room.read("dzr/room.dzr")?, path, spawn_point)?;
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
    let path = root.path(&relative);
    let archive = root.read_archive(&relative)?;
    stage_spawn(archive.read("dzr/room.dzr")?, path, point)
}

fn room_mesh_from_assets(stage: &str) -> Result<crate::j3d::bdl::RoomMesh, AssetError> {
    let root = AssetRoot::from_env()?;
    let tables = stage_tables(&root, stage)?;
    let rooms = start_set(&tables);
    let set = room_set_from_assets(&root, &tables, &rooms, &fresh_layer(&tables))?;
    set.mesh.ok_or_else(|| AssetError::Corrupt {
        path: root.path(&room_path(stage, STAGE_START_ROOM)),
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
    let mut demo_archive: Option<String> = None;
    let mut particle_no: Option<u8> = None;
    let mut exits: Vec<(u8, Vec<crate::dzs::StageExit>)> = Vec::new();
    for &room in rooms {
        let relative = room_path(tables.name(), room);
        let path = root.path(&relative);
        let archive = root.read_archive(&relative)?;
        match archive.read("dzb/room.dzb") {
            Ok(data) => {
                let parsed = crate::sim::collision::Collision::parse(data).map_err(|error| {
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
                Ok(data) => {
                    Some(
                        crate::j3d::btk::parse(data).map_err(|error| AssetError::Corrupt {
                            path: path.clone(),
                            detail: error.as_str(),
                        })?,
                    )
                }
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
                if demo_archive.is_none() {
                    demo_archive = room_demo_archive(data, layer(room));
                }
                if particle_no.is_none() {
                    particle_no = room_particle_no(data);
                }
                match crate::dzs::room_exits(data) {
                    Ok(parsed) => {
                        if !parsed.is_empty() {
                            exits.push((room, parsed));
                        }
                    }
                    Err(error) => {
                        log::warn!(
                            "clearsleeper: room {room} scene list unusable ({})",
                            error.as_str()
                        );
                    }
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
        demo_archive,
        particle_no,
        exits,
    })
}

/// `d_s_room.cpp:220-231`: the room file's `LBNK` banks are indexed by
/// `dComIfG_play_c::getLayerNo(roomNo)` (`d_com_inf_game.cpp:184-216`) and a bank that is not
/// `0xFF` names `sprintf("Demo%02d", bank)` (`:226-229`). A room whose table has no bank for the
/// layer names nothing, which is the `bank == 0xFF` arm the reference skips.
fn room_demo_archive(data: &[u8], layer: u8) -> Option<String> {
    match crate::dzs::room_lbnk(data) {
        Ok(lbnk) => lbnk.bank(layer).map(|bank| format!("Demo{bank:02}")),
        Err(error) => {
            log::warn!(
                "clearsleeper: room LBNK table unusable ({})",
                error.as_str()
            );
            None
        }
    }
}

/// `dStage_FileList_dt_GetParticleNo(fileList)` - `d_s_room.cpp:237-241`: the room file's `FILI`
/// record's own `Pscene` number. `0xFF` is the reader's no-archive token
/// (`d_particle.cpp:758-760`), which never consumes the read, so it answers `None` here and a
/// later room of the set may name one instead. A room file without the record answers `None` too,
/// which is the null `getFileListInfo()` the reference asserts against (`:238-239`).
fn room_particle_no(data: &[u8]) -> Option<u8> {
    match crate::dzs::room_file_list(data) {
        Ok(Some(file_list)) => match file_list.particle_no() {
            crate::sim::jpa::NO_SCENE => None,
            number => Some(number),
        },
        Ok(None) => None,
        Err(error) => {
            log::warn!(
                "clearsleeper: room FILI record unusable ({})",
                error.as_str()
            );
            None
        }
    }
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
    // `dStage_dt_c_offsetToPtr` (`d_stage.cpp:2052-2055`) keeps a zero node offset as the null
    // pointer, leaving the chunk absent instead of pointing at the start of the blob.
    if offset == 0 {
        return Err(corrupt("dzr has no spawn chunk"));
    }
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

pub const GX_TF_I4: u8 = 0x0;
pub const GX_TF_I8: u8 = 0x1;
pub const GX_TF_IA4: u8 = 0x2;
pub const GX_TF_IA8: u8 = 0x3;
pub const GX_TF_RGB565: u8 = 0x4;
pub const GX_TF_RGB5A3: u8 = 0x5;
pub const GX_TF_RGBA8: u8 = 0x6;
pub const GX_TF_CI4: u8 = 0x8;
pub const GX_TF_CI8: u8 = 0x9;
pub const GX_TF_C4X2: u8 = 0xA;
pub const GX_TF_CMPR: u8 = 0xE;

pub const BDL_MAGIC: [u8; 8] = *b"J3D2bdl4";
pub const BDL_HEADER_LEN: usize = 0x20;
pub const BDL_CHUNK_COUNT_OFFSET: usize = 0x0C;
pub const BDL_CHUNK_STRIDE: usize = 8;

pub fn bdl_chunk_count(data: &[u8]) -> Option<usize> {
    if data.len() < BDL_HEADER_LEN || data[..8] != BDL_MAGIC {
        return None;
    }
    Some(read_u32(data, BDL_CHUNK_COUNT_OFFSET)? as usize)
}

pub fn bdl_sections(data: &[u8]) -> Option<Vec<([u8; 4], std::ops::Range<usize>)>> {
    let count = bdl_chunk_count(data)?;
    if count > (data.len() - BDL_HEADER_LEN) / BDL_CHUNK_STRIDE {
        return None;
    }
    let mut sections = Vec::with_capacity(count);
    let mut cursor = BDL_HEADER_LEN;
    for _ in 0..count {
        let size = read_u32(data, cursor + 4)? as usize;
        if size < BDL_CHUNK_STRIDE {
            return None;
        }
        let end = cursor.checked_add(size)?;
        if end > data.len() {
            return None;
        }
        sections.push((
            [
                data[cursor],
                data[cursor + 1],
                data[cursor + 2],
                data[cursor + 3],
            ],
            cursor..end,
        ));
        cursor = end;
    }
    if cursor != data.len() {
        return None;
    }
    Some(sections)
}

pub fn bdl_section<'a>(data: &'a [u8], tag: &[u8; 4]) -> Option<&'a [u8]> {
    let sections = bdl_sections(data)?;
    let (_, range) = sections.iter().find(|(candidate, _)| candidate == tag)?;
    Some(&data[range.start..range.end])
}

pub fn rgb565_to_rgba(texel: u16) -> [u8; 4] {
    [
        expand_to_8(5, u32::from(texel >> 11)),
        expand_to_8(6, u32::from(texel >> 5)),
        expand_to_8(5, u32::from(texel)),
        0xFF,
    ]
}

pub fn rgb5a3_to_rgba(texel: u16) -> [u8; 4] {
    if texel & 0x8000 != 0 {
        [
            expand_to_8(5, u32::from(texel >> 10)),
            expand_to_8(5, u32::from(texel >> 5)),
            expand_to_8(5, u32::from(texel)),
            0xFF,
        ]
    } else {
        [
            expand_to_8(4, u32::from(texel >> 8)),
            expand_to_8(4, u32::from(texel >> 4)),
            expand_to_8(4, u32::from(texel)),
            expand_to_8(3, u32::from(texel >> 12)),
        ]
    }
}

pub fn expand_to_8(bits: u32, value: u32) -> u8 {
    let value = value & ((1 << bits) - 1);
    if bits == 3 {
        ((value << 5) | (value << 2) | (value >> 1)) as u8
    } else {
        (((value << (8 - bits)) | (value >> (bits * 2 - 8))) & 0xFF) as u8
    }
}

fn s3tc_blend(first: u8, second: u8) -> u8 {
    ((u16::from(first) * 3 + u16::from(second) * 5) >> 3) as u8
}

fn half_blend(first: u8, second: u8) -> u8 {
    ((u16::from(first) + u16::from(second)) >> 1) as u8
}

fn cmpr_block(source: &[u8], at: usize, table: &mut [[u8; 4]; 4]) {
    let color1 = u16::from_be_bytes([source[at], source[at + 1]]);
    let color2 = u16::from_be_bytes([source[at + 2], source[at + 3]]);
    table[0] = [
        expand_to_8(5, u32::from(color1 >> 11)),
        expand_to_8(6, u32::from(color1 >> 5)),
        expand_to_8(5, u32::from(color1)),
        0xFF,
    ];
    table[1] = [
        expand_to_8(5, u32::from(color2 >> 11)),
        expand_to_8(6, u32::from(color2 >> 5)),
        expand_to_8(5, u32::from(color2)),
        0xFF,
    ];
    if color1 > color2 {
        table[2] = [
            s3tc_blend(table[1][0], table[0][0]),
            s3tc_blend(table[1][1], table[0][1]),
            s3tc_blend(table[1][2], table[0][2]),
            0xFF,
        ];
        table[3] = [
            s3tc_blend(table[0][0], table[1][0]),
            s3tc_blend(table[0][1], table[1][1]),
            s3tc_blend(table[0][2], table[1][2]),
            0xFF,
        ];
    } else {
        let midway = [
            half_blend(table[0][0], table[1][0]),
            half_blend(table[0][1], table[1][1]),
            half_blend(table[0][2], table[1][2]),
            0xFF,
        ];
        table[2] = midway;
        table[3] = [midway[0], midway[1], midway[2], 0];
    }
}

pub fn cmpr_to_rgba(width: usize, height: usize, source: &[u8]) -> Option<Vec<u8>> {
    if width == 0 || height == 0 {
        return None;
    }
    let blocks_wide = width.div_ceil(8);
    let blocks_high = height.div_ceil(8);
    let needed = blocks_wide.checked_mul(blocks_high)?.checked_mul(32)?;
    if source.len() < needed {
        return None;
    }
    let mut out = vec![0_u8; width.checked_mul(height)?.checked_mul(4)?];
    let mut cursor = 0_usize;
    let mut table = [[0_u8; 4]; 4];
    for block_y in 0..blocks_high {
        for block_x in 0..blocks_wide {
            for sub in 0..4 {
                let base_x = block_x * 8 + (sub & 1) * 4;
                let base_y = block_y * 8 + (sub >> 1) * 4;
                cmpr_block(source, cursor, &mut table);
                cursor += 4;
                for row in 0..4 {
                    let bits = source[cursor + row];
                    for column in 0..4 {
                        let x = base_x + column;
                        let y = base_y + row;
                        if x >= width || y >= height {
                            continue;
                        }
                        let entry = table[usize::from(bits >> (6 - column * 2)) & 3];
                        let at = (y * width + x) * 4;
                        out[at..at + 4].copy_from_slice(&entry);
                    }
                }
                cursor += 4;
            }
        }
    }
    Some(out)
}

fn gc_tiled<F>(
    width: usize,
    height: usize,
    block_w: usize,
    block_h: usize,
    row_bytes: usize,
    source: &[u8],
    mut read_texel: F,
) -> Option<Vec<u8>>
where
    F: FnMut(&[u8], usize) -> [u8; 4],
{
    if width == 0 || height == 0 || block_w == 0 || block_h == 0 {
        return None;
    }
    let blocks_wide = width.div_ceil(block_w);
    let blocks_high = height.div_ceil(block_h);
    let needed = blocks_wide
        .checked_mul(blocks_high)?
        .checked_mul(row_bytes)?
        .checked_mul(block_h)?;
    if source.len() < needed {
        return None;
    }
    let mut out = vec![0_u8; width.checked_mul(height)?.checked_mul(4)?];
    let mut cursor = 0_usize;
    for block_y in 0..blocks_high {
        let base_y = block_y * block_h;
        let rows = (height - base_y).min(block_h);
        for block_x in 0..blocks_wide {
            let base_x = block_x * block_w;
            let columns = (width - base_x).min(block_w);
            for row in 0..rows {
                let line = source.get(cursor..cursor + row_bytes)?;
                for column in 0..columns {
                    let entry = read_texel(line, column);
                    let at = ((base_y + row) * width + base_x + column) * 4;
                    out[at..at + 4].copy_from_slice(&entry);
                }
                cursor += row_bytes;
            }
            cursor += row_bytes * (block_h - rows);
        }
    }
    Some(out)
}

fn gc_rgba8(width: usize, height: usize, source: &[u8]) -> Option<Vec<u8>> {
    if width == 0 || height == 0 {
        return None;
    }
    let blocks_wide = width.div_ceil(4);
    let blocks_high = height.div_ceil(4);
    let needed = blocks_wide.checked_mul(blocks_high)?.checked_mul(64)?;
    if source.len() < needed {
        return None;
    }
    let mut out = vec![0_u8; width.checked_mul(height)?.checked_mul(4)?];
    let mut cursor = 0_usize;
    for block_y in 0..blocks_high {
        let base_y = block_y * 4;
        let rows = (height - base_y).min(4);
        for block_x in 0..blocks_wide {
            let base_x = block_x * 4;
            let columns = (width - base_x).min(4);
            let block = &source[cursor..cursor + 64];
            for row in 0..rows {
                for column in 0..columns {
                    let ar = row * 8 + column * 2;
                    let gb = 32 + row * 8 + column * 2;
                    let at = ((base_y + row) * width + base_x + column) * 4;
                    out[at..at + 4].copy_from_slice(&[
                        block[ar + 1],
                        block[gb],
                        block[gb + 1],
                        block[ar],
                    ]);
                }
            }
            cursor += 64;
        }
    }
    Some(out)
}

fn gc_geometry(format: u8) -> Option<(usize, usize, usize)> {
    Some(match format {
        GX_TF_I4 => (8, 8, 4),
        GX_TF_I8 => (8, 4, 8),
        GX_TF_IA4 => (8, 4, 8),
        GX_TF_IA8 => (4, 4, 8),
        GX_TF_RGB565 | GX_TF_RGB5A3 => (4, 4, 8),
        GX_TF_RGBA8 => (4, 4, 16),
        GX_TF_CI4 => (8, 8, 4),
        GX_TF_CI8 => (8, 4, 8),
        GX_TF_C4X2 => (4, 4, 8),
        GX_TF_CMPR => (8, 8, 4),
        _ => return None,
    })
}

pub fn gc_texture_bytes(format: u8, width: usize, height: usize) -> Option<usize> {
    let (block_w, block_h, row_bytes) = gc_geometry(format)?;
    width
        .checked_add(block_w - 1)?
        .checked_div(block_w)?
        .checked_mul(height.checked_add(block_h - 1)?.checked_div(block_h)?)?
        .checked_mul(row_bytes)?
        .checked_mul(block_h)
}

pub fn gc_texture_to_rgba(
    format: u8,
    width: usize,
    height: usize,
    source: &[u8],
    palette: &[[u8; 4]],
) -> Option<Vec<u8>> {
    if format == GX_TF_CMPR {
        return cmpr_to_rgba(width, height, source);
    }
    let (block_w, block_h, row_bytes) = gc_geometry(format)?;
    let indexed = matches!(format, GX_TF_CI4 | GX_TF_CI8 | GX_TF_C4X2);
    if indexed && palette.is_empty() {
        return None;
    }
    let entry = |index: u32| palette.get(index as usize).copied().unwrap_or([0, 0, 0, 0]);
    match format {
        GX_TF_I4 => gc_tiled(
            width,
            height,
            block_w,
            block_h,
            row_bytes,
            source,
            |line, x| {
                let level = expand_to_8(
                    4,
                    u32::from(line[x / 2] >> (if x & 1 == 0 { 4 } else { 0 })),
                );
                [level, level, level, level]
            },
        ),
        GX_TF_I8 => gc_tiled(
            width,
            height,
            block_w,
            block_h,
            row_bytes,
            source,
            |line, x| {
                let level = line[x];
                [level, level, level, level]
            },
        ),
        GX_TF_IA4 => gc_tiled(
            width,
            height,
            block_w,
            block_h,
            row_bytes,
            source,
            |line, x| {
                let value = line[x];
                let level = expand_to_8(4, u32::from(value & 0x0F));
                [level, level, level, expand_to_8(4, u32::from(value >> 4))]
            },
        ),
        GX_TF_IA8 => gc_tiled(
            width,
            height,
            block_w,
            block_h,
            row_bytes,
            source,
            |line, x| {
                let at = x * 2;
                let value = u16::from_be_bytes([line[at], line[at + 1]]);
                let level = (value >> 8) as u8;
                [level, level, level, value as u8]
            },
        ),
        GX_TF_RGB5A3 => gc_tiled(
            width,
            height,
            block_w,
            block_h,
            row_bytes,
            source,
            |line, x| {
                let at = x * 2;
                let value = u16::from_be_bytes([line[at], line[at + 1]]);
                if value & 0x8000 != 0 {
                    [
                        expand_to_8(5, u32::from(value >> 10)),
                        expand_to_8(5, u32::from(value >> 5)),
                        expand_to_8(5, u32::from(value)),
                        0xFF,
                    ]
                } else {
                    [
                        expand_to_8(4, u32::from(value >> 8)),
                        expand_to_8(4, u32::from(value >> 4)),
                        expand_to_8(4, u32::from(value)),
                        expand_to_8(3, u32::from(value >> 12)),
                    ]
                }
            },
        ),
        GX_TF_RGB565 => gc_tiled(
            width,
            height,
            block_w,
            block_h,
            row_bytes,
            source,
            |line, x| {
                let at = x * 2;
                let value = u16::from_be_bytes([line[at], line[at + 1]]);
                [
                    expand_to_8(5, u32::from(value >> 11)),
                    expand_to_8(6, u32::from(value >> 5)),
                    expand_to_8(5, u32::from(value)),
                    0xFF,
                ]
            },
        ),
        GX_TF_RGBA8 => gc_rgba8(width, height, source),
        GX_TF_CI4 => gc_tiled(
            width,
            height,
            block_w,
            block_h,
            row_bytes,
            source,
            |line, x| {
                entry(u32::from(
                    line[x / 2] >> (if x & 1 == 0 { 4 } else { 0 }) & 0x0F,
                ))
            },
        ),
        GX_TF_CI8 => gc_tiled(
            width,
            height,
            block_w,
            block_h,
            row_bytes,
            source,
            |line, x| entry(u32::from(line[x])),
        ),
        GX_TF_C4X2 => gc_tiled(
            width,
            height,
            block_w,
            block_h,
            row_bytes,
            source,
            |line, x| {
                let at = x * 2;
                entry(u32::from(
                    u16::from_be_bytes([line[at], line[at + 1]]) & 0x3FFF,
                ))
            },
        ),
        _ => None,
    }
}

/// The JPA archive bytes `dPa_control_c` mounts: `d_s_logo.cpp:931` reads
/// `/res/Particle/common.jpc` into `dPa_control_c::createCommon` (`d_particle.cpp:713-735`) and
/// `dPa_control_c::readScene` (`d_particle.cpp:757-770`) names the per-stage
/// `/res/Particle/Pscene%03d.jpc` for `createRoomScene` (`:736-756`). The bytes are a raw `JPAC`
/// image - [`crate::sim::jpa_bank::JpaBank`] parses them where `JPAResourceManager` does - so
/// nothing is decoded here.
pub fn load_particle_archive(name: &str) -> Option<Vec<u8>> {
    let root = AssetRoot::from_env().ok()?;
    match root.read(&format!("files/res/Particle/{name}")) {
        Ok(bytes) => Some(bytes),
        Err(error) => {
            log::warn!("clearsleeper: particle archive {name} unavailable ({error})");
            None
        }
    }
}
