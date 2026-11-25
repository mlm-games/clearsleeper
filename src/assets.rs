use bevy_ecs::prelude::Resource;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::archive::{Archive, AssetError, Rel, YAZ0_MAGIC, yaz0_decode};
use crate::disc::{DiscImage, DiscSummary};

/// The extracted tree [`AssetRoot::from_env`] reads when no disc image is installed. A
/// development fallback on every platform - a dump on disk, a debugger watching loose files -
/// not the path a session ships on; the disc image is.
pub const ASSETS_ENV: &str = "CS_ASSETS";

const STAGE_ROOT: &str = "files/res/Stage";
pub const STAGE_SPAWN_POINT: u8 = 128;
pub const STAGE_START_ROOM: u8 = 44;
pub const SEA_STAGE: &str = "sea";
/// A new save returns to `sea` room 44, Outset Island (`d_save.cpp:63-67`, the default
/// `dSv_player_return_place_c::init`, with `dIsleRoom_OutsetIsland_e = 44` at `d_save.h:348`),
/// at point 206. `sea_T` is the title scene's demo copy of the island - its `LBNK` names
/// `Demo51` (`title.stb`) rather than the prologue's `Demo02` (`awake.stb`), so it is not the
/// stage the sister's opening cutscene runs in.
pub const PROLOGUE_STAGE: &str = "sea";
pub const PROLOGUE_SPAWN_POINT: u8 = 206;
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
    /// A dropped folder: the extracted tree held in memory under its relative paths.
    Memory(Arc<std::collections::HashMap<String, Vec<u8>>>),
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
        if let Some(files) = EXTRACTED_MEMORY
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
        {
            return Ok(Self {
                source: Source::Memory(files),
            });
        }
        if let Some(root) = EXTRACTED
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
        {
            return Ok(Self {
                source: Source::Files(root),
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
            Source::Disc(_) | Source::Memory(_) => None,
        }
    }

    /// The disc's game id, when the source is an image rather than an extracted tree.
    pub fn game_id(&self) -> Option<&str> {
        match &self.source {
            Source::Disc(image) => Some(image.game_id()),
            _ => None,
        }
    }

    /// The label an asset path carries in errors: its path under the extracted root, or
    /// `GAMEID:relative` when it came out of an image.
    pub fn path(&self, rel: &str) -> PathBuf {
        match &self.source {
            Source::Files(root) => root.join(rel),
            Source::Disc(image) => image.path(rel),
            Source::Memory(_) => PathBuf::from(format!("EXTRACT:{rel}")),
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
            Source::Memory(files) => {
                files
                    .get(rel)
                    .cloned()
                    .ok_or_else(|| AssetError::MissingFile {
                        path: self.path(rel),
                    })
            }
        }
    }

    pub fn read_archive(&self, rel: &str) -> Result<Archive, AssetError> {
        let path = self.path(rel);
        let bytes = self.read(rel)?;
        open_archive(bytes, path)
    }

    pub fn stream(&self, rel: &str) -> anyhow::Result<Box<dyn crate::disc::AssetStream>> {
        match &self.source {
            Source::Files(root) => Ok(Box::new(std::fs::File::open(root.join(rel))?)),
            Source::Disc(image) => Ok(image.stream(rel)?),
            Source::Memory(files) => files
                .get(rel)
                .cloned()
                .map(|bytes| {
                    Box::new(std::io::Cursor::new(bytes)) as Box<dyn crate::disc::AssetStream>
                })
                .ok_or_else(|| {
                    AssetError::MissingFile {
                        path: self.path(rel),
                    }
                    .into()
                }),
        }
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

/// An extracted disc tree installed as the source instead of an image: the same loose files a
/// development `CS_ASSETS` tree holds, but picked as a folder at launch rather than named in
/// the environment. Dusklight accepts a folder at its prelaunch picker; this is the same seam.
static EXTRACTED: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/// The extracted tree handed over by the web folder picker, keyed by the relative paths the
/// asset layer already asks for.
static EXTRACTED_MEMORY: std::sync::Mutex<Option<Arc<std::collections::HashMap<String, Vec<u8>>>>> =
    std::sync::Mutex::new(None);

/// Install an extracted tree as the session's asset source.
pub fn install_extracted(path: &Path) {
    let mut disc = INSTALLED.lock().unwrap_or_else(|error| error.into_inner());
    *disc = None;
    drop(disc);
    let mut slot = EXTRACTED.lock().unwrap_or_else(|error| error.into_inner());
    *slot = Some(path.to_path_buf());
}

fn extract_summary(title: Option<String>, files: usize) -> DiscSummary {
    DiscSummary {
        game_id: "EXTRACT".to_string(),
        title,
        files,
    }
}

/// A summary for an extracted tree installed from a directory path.
pub fn extracted_summary(path: &Path) -> DiscSummary {
    extract_summary(
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned()),
        0,
    )
}

/// Install an extracted tree handed over by the web folder picker.
pub fn install_extracted_memory(files: Vec<(String, Vec<u8>)>) -> Result<DiscSummary, AssetError> {
    if files.is_empty() {
        return Err(AssetError::MissingFile {
            path: PathBuf::from("EXTRACT:<empty folder>"),
        });
    }
    let count = files.len();
    let map = Arc::new(
        files
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>(),
    );
    *INSTALLED.lock().unwrap_or_else(|error| error.into_inner()) = None;
    *EXTRACTED.lock().unwrap_or_else(|error| error.into_inner()) = None;
    *EXTRACTED_MEMORY
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(map);
    Ok(extract_summary(Some("extracted folder".to_string()), count))
}

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

/// Mount a disc from a path, for containers `nod` can only read through a file (NFS, the Wii U
/// VC wrapper). The image stays on disk; only the parsed partition is held in the session.
pub fn install_disc_path(path: &std::path::Path) -> Result<DiscSummary, crate::disc::DiscError> {
    let image = Arc::new(DiscImage::open_path(path)?);
    let summary = image.summary();
    *INSTALLED.lock().unwrap_or_else(|error| error.into_inner()) = Some(image);
    Ok(summary)
}

/// The boundary between a source that mounts and a source that can actually reach file select.
/// Every path into a session passes here before the shell stops showing the launcher.
pub fn verify_playable() -> Result<(), &'static str> {
    let root = AssetRoot::from_env().map_err(|_| "no asset source is installed")?;
    if crate::title_screen::TitleAssets::load(&root).is_none() {
        return Err("the title screen assets are missing");
    }
    if crate::panes::load().is_none() {
        return Err("the file-select panes are missing");
    }
    if !crate::assets::load_msg_archives().available() {
        return Err("the message archives are missing");
    }
    if load_stage_tables(SEA_STAGE).is_none() {
        return Err("the starting stage tables are missing");
    }
    if load_stage_at(SEA_STAGE, STAGE_SPAWN_POINT).is_none() {
        return Err("the starting stage is missing");
    }
    Ok(())
}

/// Drop a source that mounted but failed [`verify_playable`], so the launcher stays usable.
pub fn clear_installed() {
    *INSTALLED.lock().unwrap_or_else(|error| error.into_inner()) = None;
    *EXTRACTED.lock().unwrap_or_else(|error| error.into_inner()) = None;
    *EXTRACTED_MEMORY
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = None;
}

fn installed_disc() -> Option<Arc<DiscImage>> {
    let slot = INSTALLED.lock().unwrap_or_else(|error| error.into_inner());
    slot.clone()
}

/// The installed source's identity for the start UI's disc panel.
pub fn installed_disc_summary() -> Option<DiscSummary> {
    if let Some(image) = installed_disc() {
        return Some(image.summary());
    }
    if let Some(files) = EXTRACTED_MEMORY
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
    {
        return Some(extract_summary(
            Some("extracted folder".to_string()),
            files.len(),
        ));
    }
    EXTRACTED
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
        .map(|path| extracted_summary(path))
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

/// Load a stage's collision and the start point `spawn_point` on its first room. A stage's
/// start point is part of the stage choice: the prologue uses point 206 on `sea`'s room 44,
/// while a normal `sea` arrival uses [`STAGE_SPAWN_POINT`].
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

/// The hair model the player attaches to `cl.bdl`'s head joint (`d_a_player_main.cpp:11598`).
pub const LINK_HAIR_ENTRY: &str = "katsura.bdl";

/// The hands parts model. Retail picks one shape per hand from `hands.bdl` and overwrites its
/// joint matrix with `cl.bdl`'s `cl_LhandA`/`cl_RhandA` (`d_a_player_main.cpp:1471-1483`;
/// `wwhd/wwhd_src/d/actor/d_a_player_main_01.cpp:3371-3406`).
pub const LINK_HANDS_ENTRY: &str = "hands.bdl";

/// Held weapon/shield parts, attached to the hand joints or the back. The blade table is
/// `d_a_player_sword.inc:20-70` (`swa`/`swms`), the grips `d_a_player_main.cpp:1222-1258`,
/// and the shield select `d_a_player_main.cpp:1280-1296`
/// (`wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3112-3250`).
pub const LINK_SWORD_ENTRY: &str = "swa.bdl";
pub const LINK_MASTER_SWORD_ENTRY: &str = "swms.bdl";
pub const LINK_SWORD_GRIP_ENTRY: &str = "swgripa.bdl";
pub const LINK_MASTER_GRIP_ENTRY: &str = "swgripms.bdl";
pub const LINK_SHIELD_ENTRY: &str = "sha.bdl";
pub const LINK_MIRROR_SHIELD_ENTRY: &str = "shms.bdl";

/// Other held item models. `fan`/`fanb` are the leaf's held and open canopies, the hammer and
/// telescope their own bodies (`d_a_player_fan.inc:123,153`, `d_a_player_hammer.inc:23`,
/// `d_a_player_main.cpp:3740`; HD `wwhd/wwhd_src/d/actor/d_a_player_main_01.cpp:362,1112`).
pub const LINK_DEKU_LEAF_ENTRY: &str = "fan.bdl";
pub const LINK_DEKU_LEAF_OPEN_ENTRY: &str = "fanb.bdl";
pub const LINK_HAMMER_ENTRY: &str = "hammer.bdl";
pub const LINK_TELESCOPE_ENTRY: &str = "telescope.bdl";
pub const LINK_WIND_WAKER_ENTRY: &str = "takt.bdl";
pub const LINK_BOTTLE_ENTRY: &str = "bottleemp.bdl";
pub const LINK_CAMERA_ENTRY: &str = "camera.bdl";

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

/// An NPC body and the BCKs its own object archive names, each parsed against that body's joint
/// tree (`tww/src/d/d_npc.cpp:546-563`, `wwhd/wwhd_src/d/d_npc.cpp:58-69`).
pub struct ActorModel {
    pub mesh: crate::j3d::bdl::RoomMesh,
    pub morphs: Vec<(i32, crate::j3d::bck::Morph)>,
    pub cloth: Option<crate::j3d::bdl::RoomMesh>,
}

/// Grandma's archive holds two BDLs; `ba.bdl` is the body and `ba_cloth.bdl` the cape
/// (`tww/assets/GZLE01/res/Object/Ba.h:49-55`; HD `wwhd/wwhd_src/include/d/actor/d_a_npc_ba1.h:380-386`).
pub const BA_ARCHIVE: &str = "Ba";
pub const BA_MODEL_ENTRY: &str = "ba.bdl";
/// Chuchu's body is the older `J3D2bmd3` model `CC_BMD` (id 0x2A) in `Cc.arc`; its animation ids
/// are `0x14..=0x22` (`tww/assets/GZLE01/res/Object/Cc.h:42,86`).
pub const CC_ARCHIVE: &str = "CC";
pub const CC_OBJECT_ARCHIVE: &str = "files/res/Object/Cc.arc";
pub const CC_MODEL_ID: u16 = 0x2A;
/// Armos body `AM_BDL` (id 0x22) and BCK ids 0x14..=0x1F in `Am.arc`
/// (`tww/assets/GZLE01/res/Object/Am.h:25-38`, `d_a_am.cpp:137`).
pub const AM_ARCHIVE: &str = "AM";
pub const AM_OBJECT_ARCHIVE: &str = "files/res/Object/Am.arc";
pub const AM_MODEL_ID: u16 = 0x22;
/// Floormaster body `FM_BDL` (id 0x15) and BCK ids 0x6..=0x12 in `Fm.arc`
/// (`tww/assets/GZLE01/res/Object/fm.h:18-25`, `d_a_fm.cpp:26,388`).
pub const FM_ARCHIVE: &str = "Fm";
pub const FM_OBJECT_ARCHIVE: &str = "files/res/Object/Fm.arc";
pub const FM_MODEL_ID: u16 = 0x15;
/// Amos Knight body `AM2_BDL` (id 0x12) and BCK ids 0x6..=0xF in `Am2.arc`
/// (`tww/assets/GZLE01/res/Object/Am2.h:11-22`, `d_a_am2.cpp:1225`).
pub const AM2_ARCHIVE: &str = "AM2";
pub const AM2_OBJECT_ARCHIVE: &str = "files/res/Object/Am2.arc";
pub const AM2_MODEL_ID: u16 = 0x12;
/// Helmaroc King's `BDK_BDL_DK` body (id 0x4A) and BCK ids 0x1C..=0x42 in `Bdk.arc`
/// (`tww/assets/GZLE01/res/Object/Bdk.h:114-152,159`; HD `wwhd_src/d/actor/d_a_bdk.cpp:217`).
/// `Oq.arc` is the Octorok, not this boss.
pub const BDK_ARCHIVE: &str = "Bdk";
pub const BDK_OBJECT_ARCHIVE: &str = "files/res/Object/Bdk.arc";
pub const BDK_MODEL_ID: u16 = 0x4A;
/// Phantom Ganon's `FGANON_BDL_BPG` body (id 0x1A) and BCK ids 0x6..=0x16 in `Fganon.arc`
/// (`tww/assets/GZLE01/res/Object/Fganon.h:11-30`).
pub const FGANON_ARCHIVE: &str = "Fganon";
pub const FGANON_OBJECT_ARCHIVE: &str = "files/res/Object/Fganon.arc";
pub const FGANON_MODEL_ID: u16 = 0x1A;

pub fn load_chuchu_model() -> Option<ActorModel> {
    let archive = load_object_archive(CC_ARCHIVE)?;
    let mesh = object_model(&archive, CC_MODEL_ID)?;
    let anms = AnmArchive::open(CC_OBJECT_ARCHIVE).ok()?;
    let mut morphs = Vec::new();
    for id in 0x14..=0x22u16 {
        if let Some(morph) = anms.morph(AnmRef::Id(id), &mesh.joint_parents) {
            morphs.push((id as i32, morph));
        }
    }
    Some(ActorModel {
        mesh,
        morphs,
        cloth: None,
    })
}

/// Data-driven actor bodies: (archive, object archive path, model resource id, BCK id range).
/// One row per actor family from the generated `Object/*.h` headers; any model that parses is
/// cached once and posed by the BCK the sim component publishes.
pub const ACTOR_ARCHIVES: &[(&str, &str, u16, i32, i32)] = &[
    ("Bst", "files/res/Object/Bst.arc", 0x2C, 0x6, 0x29),
    ("Btd", "files/res/Object/Btd.arc", 0x40, 0x8, 0x39),
    ("Bmd", "files/res/Object/Bmd.arc", 0x2B, 0x8, 0x25),
    ("Kamome", "files/res/Object/Kamome.arc", 0x17, 0xC, 0x14),
    ("P1", "files/res/Object/P1.arc", 0x31, 0x4D, 0x5F), // BCK dRes_IDs, not dRes_INDEX (tww P1.h:61-76)
    ("Tc", "files/res/Object/Tc.arc", 0x19, 0x6, 0x16),
    ("Bgn", "files/res/Object/Bgn.arc", 0x13, 0x6, 0x10),
    ("Bwds", "files/res/Object/Bwds.arc", 0x8, 0x4, 0x5),
    ("Btsw", "files/res/Object/Btsw.arc", 0x1A, 0x8, 0x11),
    // Third field is a dRes_ID_* (object_model), not a dRes_INDEX_*; wrong ids drop the body.
    ("Niten", "files/res/Object/Niten.arc", 0x1, 0x3, 0x8), // dRes_ID_NITEN_BDL_MT_e (tww Niten.h)
    ("Ls", "files/res/Object/Ls.arc", 0xD, 0x0, 0x1F),      // dRes_ID_LS_BDL_LS_e (tww Ls.h)
    ("Md", "files/res/Object/Md.arc", 0x5C, 0x7, 0x59),
    ("Mk", "files/res/Object/Mk.arc", 0x13, 0x5, 0x10),
    ("Mn", "files/res/Object/Mn.arc", 0x2, 0x4, 0xB), // dRes_ID_MN_BDL_MN_e (tww Mn.h)
    ("Po", "files/res/Object/Po.arc", 0x1, 0x0, 0x7), // dRes_ID_PO_BDL_PO_e (tww Po.h)
    ("Rsh", "files/res/Object/Rsh.arc", 0x20, 0x14, 0x1D),
    ("Sv", "files/res/Object/Sv.arc", 0x1, 0x3, 0xE), // dRes_ID_SV_BDL_SV_e (tww Sv.h)
    ("Hr", "files/res/Object/Hr.arc", 0x14, 0x6, 0xF),
    ("Ah", "files/res/Object/Ah.arc", 0x2, 0x0, 0x1), // dRes_ID_AH_BDL_AH_e (tww Ah.h)
    ("Ho", "files/res/Object/Ho.arc", 0xF, 0x6, 0x9),
    ("Hi", "files/res/Object/Hi.arc", 0x2, 0x0, 0x1), // dRes_ID_HI_BDL_HI_e (tww Hi.h)
    ("Km", "files/res/Object/Km.arc", 0x2, 0x1, 0x1), // dRes_ID_KM_BDL_KM_e (tww Km.h)
    ("Nz", "files/res/Object/Nz.arc", 0x26, 0x15, 0x23),
    ("P2", "files/res/Object/P2.arc", 0x26, 0x7, 0x23),
    ("Tt", "files/res/Object/Tt.arc", 0x1C, 0x11, 0x19),
    ("Toripost", "files/res/Object/Toripost.arc", 0x9, 0x4, 0x6),
    ("Bmsw", "files/res/Object/Bmsw.arc", 0x3, 0x14, 0x20), // dRes_ID_BMSW_BDL_BM_e (tww Bmsw.h)
    ("Jb", "files/res/Object/Jb.arc", 0x2, 0x0, 0x1),       // dRes_ID_JB_BDL_JB_e (tww Jb.h)
    ("Rd", "files/res/Object/Rd.arc", 0x17, 0x6, 0x14),
    ("St", "files/res/Object/St.arc", 0x38, 0x4, 0x35),
    ("Bmdhand", "files/res/Object/Bmdhand.arc", 0x9, 0x5, 0x6),
    // Molgera (`tww/assets/GZLE01/res/Object/Bwd.h:62-84`), Dexivine
    // (`tww/assets/GZLE01/res/Object/Sss.h:21-24`), Beedle (`tww/assets/GZLE01/res/Object/Bs.h:48`),
    // Komali (`tww/assets/GZLE01/res/Object/Ac.h:48`, `tww/assets/GZLE01/res/Object/Co.h:71`),
    // the Roberto brothers (`tww/assets/GZLE01/res/Object/Uk.h:53`), Maggie's parents
    // (`tww/assets/GZLE01/res/Object/Gp.h:34`, `tww/assets/GZLE01/res/Object/Pf.h:36`),
    // Tetra (`tww/assets/GZLE01/res/Object/Zl.h:84`, `tww/assets/GZLE01/res/Object/Zl2.h:24`) and
    // the Great Fairy (`tww/assets/GZLE01/res/Object/Always.h:155`).
    ("Bwd", "files/res/Object/Bwd.arc", 0x1E, 0x7, 0x19),
    ("Sss", "files/res/Object/Sss.arc", 0x9, 0x5, 0x6),
    ("Bs", "files/res/Object/Bs.arc", 0x5, 0x0, 0x10),
    ("Ac", "files/res/Object/Ac.arc", 0x5, 0x0, 0xE),
    ("Co", "files/res/Object/Co.arc", 0x13, 0x0, 0x1D),
    ("Uk", "files/res/Object/Uk.arc", 0x18, 0x7, 0x11),
    ("Gp", "files/res/Object/Gp.arc", 0x6, 0x0, 0x8),
    ("Pf", "files/res/Object/Pf.arc", 0x8, 0x0, 0x7),
    ("Zl", "files/res/Object/Zl.arc", 0xF, 0x0, 0x2A),
    ("Zl2", "files/res/Object/Zl2.arc", 0x1, 0x0, 0x0),
    ("Always", "files/res/Object/Always.arc", 0x1F, 0xC, 0xC),
    // Bokoblin (`tww/assets/GZLE01/res/Object/Bk.h:179`), Moblin
    // (`tww/assets/GZLE01/res/Object/Mo2.h:249`) and Darknut
    // (`tww/assets/GZLE01/res/Object/Tn.h:224`). `read_id` wants the `dRes_ID_*` value
    // (`archive.rs:432-437`).
    ("Bk", "files/res/Object/Bk.arc", 0x56, 0x29, 0x53),
    ("Mo2", "files/res/Object/Mo2.arc", 0x78, 0x3E, 0x75),
    ("Tn", "files/res/Object/Tn.arc", 0x6C, 0x28, 0x67),
    // Travelling merchants use `Ro.arc`, not `Roten.arc` (`tww/src/d/actor/d_a_npc_roten.cpp:1412`).
    ("Ro", "files/res/Object/Ro.arc", 0x2, 0x0, 0x13),
    // Bodies with an animation cursor. Model ids and BCK ranges are from the GC object header
    // for each archive: Bb, Ks, Dr2, Kn, Daiocta, Ki, Dr, Bm, Dk, Bgn and Doguu under
    // tww/assets/GZLE01/res/Object/ (lines 82, 34, 54, 26, 55, 52, 40, 175, 24, 67 and 51).
    ("Bb", "files/res/Object/Bb.arc", 0x25, 0x15, 0x22),
    ("Ks", "files/res/Object/Ks.arc", 0xF, 0xC, 0xC),
    ("Dr2", "files/res/Object/Dr2.arc", 0x17, 0x9, 0xB),
    ("Kn", "files/res/Object/Kn.arc", 0xA, 0x4, 0x7),
    ("Daiocta", "files/res/Object/Daiocta.arc", 0x11, 0x6, 0xD),
    ("Ki", "files/res/Object/Ki.arc", 0x18, 0xF, 0x14),
    ("FK", "files/res/Object/Ki.arc", 0x17, 0xF, 0x14),
    ("Dr", "files/res/Object/Dr.arc", 0x13, 0xB, 0x10),
    ("Bm", "files/res/Object/Bm.arc", 0x33, 0x0, 0x50),
    ("Dk", "files/res/Object/Dk.arc", 0x12, 0x10, 0x11),
    ("Bgn3", "files/res/Object/Bgn.arc", 0x15, 0x6, 0x10),
    ("Doguu", "files/res/Object/Doguu.arc", 0x11, 0x6, 0xA),
    // Bodies with no BCK at all draw the bind pose (`anm.bck < 0`). Model ids are from the GC
    // object headers Boko, Hmos, Shmrgrd, Apzl, Homen and Figure under
    // tww/assets/GZLE01/res/Object/ (lines 16, 26, 18, 278, 20 and 18).
    ("Boko", "files/res/Object/Boko.arc", 0x3, 0x0, 0x0),
    ("Hmos", "files/res/Object/Hmos.arc", 0x5, 0x0, 0x0),
    ("Shmrgrd", "files/res/Object/Shmrgrd.arc", 0x4, 0x0, 0x0),
    ("Apzl", "files/res/Object/Apzl.arc", 0x6, 0x0, 0x0),
    ("Homen", "files/res/Object/Homen.arc", 0x4, 0x0, 0x0),
    ("Homen2", "files/res/Object/Homen.arc", 0x5, 0x0, 0x0),
    ("Figure", "files/res/Object/Figure.arc", 0x1, 0x0, 0x0),
    // Scenery with real textured bodies. Chest shapes share Dalways; the four keys select its
    // four BDLs and open BCKs (`tww/assets/GZLE01/res/Object/Dalways.h`).
    ("Tbox0", "files/res/Object/Dalways.arc", 0xE, 0x8, 0x9),
    ("Tbox1", "files/res/Object/Dalways.arc", 0xF, 0x8, 0x9),
    ("Tbox2", "files/res/Object/Dalways.arc", 0x10, 0x8, 0x9),
    ("Tbox3", "files/res/Object/Dalways.arc", 0x14, 0x8, 0x9),
    ("SalvageBox", "files/res/Object/Salvage.arc", 0x4, 0x0, 0x0),
    ("SalvageSea", "files/res/Object/Salvage.arc", 0x3, 0x0, 0x0),
    ("SalvageItem", "files/res/Object/Yslvg00.arc", 0x5, 0x0, 0x0),
    // Doors and key-door bars; each model id is the body BDL (or BMD for Swtdoor) in its
    // generated object-header enum under tww/assets/GZLE01/res/Object/. All static (no BCK).
    ("DoorBs", "files/res/Object/DoorBs.arc", 0x4, 0x0, 0x0),
    ("Door12", "files/res/Object/door12.arc", 0x4, 0x0, 0x0),
    ("Mdoor", "files/res/Object/Mdoor.arc", 0x4, 0x0, 0x0),
    ("Swtdoor", "files/res/Object/Swtdoor.arc", 0x3, 0x0, 0x0),
    ("MjDoor", "files/res/Object/S_MSPDo.arc", 0x4, 0x0, 0x0),
    ("Mbdoor", "files/res/Object/Mbdoor.arc", 0x4, 0x0, 0x0),
    ("Gbdoor", "files/res/Object/Gbdoor.arc", 0x4, 0x0, 0x0),
    ("Ssk", "files/res/Object/Ssk.arc", 0x4, 0x0, 0x0),
    // Barrels, signs and boxes; ids from the generated enums under
    // tww/assets/GZLE01/res/Object/.
    ("Barrel", "files/res/Object/Ktaru_01.arc", 0x3, 0x0, 0x0),
    ("BarrelSea", "files/res/Object/Ktaru_00.arc", 0x5, 0x0, 0x0),
    ("BarrelMine", "files/res/Object/Ktaru_00.arc", 0x4, 0x0, 0x0),
    ("Kanban", "files/res/Object/Kanban.arc", 0x3, 0x0, 0x0),
    ("Ospbox", "files/res/Object/Ospbox.arc", 0x4, 0x0, 0x0),
    // Shared scenery bodies, bind-pose unless a BCK is listed. Every model id is the body BDL
    // (or BMD) from the generated dRes_ID_* enum in tww/assets/GZLE01/res/Object/<Archive>.h.
    ("Ship", "files/res/Object/Ship.arc", 0x11, 0x7, 0xD),
    ("Oship", "files/res/Object/Oship.arc", 0x3, 0x0, 0x0),
    ("Ayush", "files/res/Object/Ayush.arc", 0x5, 0x0, 0x0),
    (
        "Kaizokusen",
        "files/res/Object/Kaizokusen.arc",
        0xE,
        0x0,
        0x0,
    ),
    ("GaShip", "files/res/Object/GaShip.arc", 0x3, 0x0, 0x0),
    ("YakeRom", "files/res/Object/YakeRom.arc", 0x4, 0x0, 0x0),
    ("Hbw1", "files/res/Object/Hbw1.arc", 0x4, 0x0, 0x0),
    ("Ycage", "files/res/Object/Ycage.arc", 0xA, 0x0, 0x0),
    ("Atdoor", "files/res/Object/Atdoor.arc", 0x4, 0x0, 0x0),
    ("Ashut", "files/res/Object/Ashut.arc", 0x4, 0x0, 0x0),
    ("VgnFD", "files/res/Object/VgnFD.arc", 0x6, 0x0, 0x0),
    ("Ep", "files/res/Object/Ep.arc", 0x5, 0x0, 0x0),
    ("Kantera", "files/res/Object/Kantera.arc", 0xD, 0x0, 0x0),
    ("LampBody", "files/res/Object/Lamp.arc", 0x3, 0x0, 0x0),
    ("Skanran", "files/res/Object/Skanran.arc", 0x7, 0x0, 0x0),
    ("Ebrock", "files/res/Object/Ebrock.arc", 0x4, 0x0, 0x0),
    ("Ekao", "files/res/Object/Ekao.arc", 0x4, 0x0, 0x0),
    ("VigaH", "files/res/Object/VigaH.arc", 0x7, 0x4, 0x4), // tww VigaH.h: VIGAH_BCK=0x4
    ("Lwood", "files/res/Object/Lwood.arc", 0x4, 0x0, 0x0),
    ("Oyashi", "files/res/Object/Oyashi.arc", 0x4, 0x0, 0x0),
    ("Ojtree", "files/res/Object/Ojtree.arc", 0x4, 0x0, 0x0),
    ("Plant", "files/res/Object/Plant.arc", 0x3, 0x0, 0x0),
    ("Vochi", "files/res/Object/Vochi.arc", 0x4, 0x0, 0x0),
    ("Kita", "files/res/Object/kita.arc", 0x4, 0x0, 0x0),
    ("Hbrf1", "files/res/Object/Hbrf1.arc", 0x4, 0x0, 0x0),
    ("Hlift", "files/res/Object/Hlift.arc", 0x4, 0x0, 0x0),
    ("Olift", "files/res/Object/Olift.arc", 0x4, 0x0, 0x0),
    ("Hten1", "files/res/Object/Hten1.arc", 0x4, 0x0, 0x0),
    ("Hmlif", "files/res/Object/Hmlif.arc", 0x5, 0x0, 0x0),
    ("Gkai00", "files/res/Object/Gkai00.arc", 0xB, 0x8, 0x8), // tww Gkai00.h: GKAI00_BCK=0x8
    ("Gbrg00", "files/res/Object/Gbrg00.arc", 0x7, 0x0, 0x0),
    ("Bridge", "files/res/Object/Bridge.arc", 0x4, 0x0, 0x0),
    ("Ylsic", "files/res/Object/Ylsic.arc", 0x4, 0x0, 0x0),
    ("Hfbot", "files/res/Object/Hfbot.arc", 0x5, 0x0, 0x0),
    ("MhmrSW", "files/res/Object/MhmrSW.arc", 0x4, 0x0, 0x0),
    ("Hhbot", "files/res/Object/Hhbot.arc", 0x4, 0x0, 0x0),
    ("Kbota", "files/res/Object/Kbota_00.arc", 0x4, 0x0, 0x0),
    ("Mmirror", "files/res/Object/Mmirror.arc", 0xA, 0x6, 0x6), // tww Mmirror.h: MMIRROR_BCK_MSUSW=0x6
    ("Hpbot1", "files/res/Object/Hpbot1.arc", 0x3, 0x0, 0x0),
    ("Itact", "files/res/Object/Itact.arc", 0x3, 0x0, 0x0),
    ("Hsen1", "files/res/Object/Hsen1.arc", 0x5, 0x0, 0x0),
    ("Hpu1", "files/res/Object/Hpu1.arc", 0x4, 0x0, 0x0),
    ("Kokiie", "files/res/Object/Kokiie.arc", 0x4, 0x0, 0x0),
    ("Hdai1", "files/res/Object/Hdai1.arc", 0x5, 0x0, 0x0),
    ("Vtil", "files/res/Object/Vtil.arc", 0x3, 0x0, 0x0),
    ("Table", "files/res/Object/Table.arc", 0x4, 0x0, 0x0),
    ("Mspot", "files/res/Object/Mspot.arc", 0x4, 0x0, 0x0),
    ("M_door", "files/res/Object/M_door.arc", 0x3, 0x0, 0x0),
    ("Yaflw00", "files/res/Object/Yaflw00.arc", 0x8, 0x5, 0x5), // tww Yaflw00.h: YAFLW00_BCK=0x5
    ("Ferris", "files/res/Object/Skanran.arc", 0x6, 0x0, 0x0),
    ("Msdan", "files/res/Object/Msdan.arc", 0x4, 0x0, 0x0),
    ("SwHit", "files/res/Object/Always.arc", 0x35, 0x0, 0x0),
    // Projectiles and effect bodies with a real J3D model; ids from the generated dRes_ID_*
    // enums in tww/assets/GZLE01/res/Object/ for Link, Spear, Fganon, Ff, Jbo, VbakH, Trnd,
    // Always and Syan.
    ("BombBody", "files/res/Object/Link.arc", 0x3C, 0xB, 0xB),
    ("ArrowBody", "files/res/Object/Link.arc", 0x37, 0x0, 0x0),
    ("BoomerangBody", "files/res/Object/Link.arc", 0x14, 0x0, 0x0),
    ("EsaBody", "files/res/Object/Link.arc", 0x2C, 0x0, 0x0),
    ("SpearBody", "files/res/Object/Spear.arc", 0x3, 0x0, 0x0),
    ("FgMahou", "files/res/Object/Fganon.arc", 0x1C, 0x0, 0x0),
    ("FfBody", "files/res/Object/Ff.arc", 0x5, 0x0, 0x0),
    ("JboBody", "files/res/Object/Jbo.arc", 0x9, 0x4, 0x6),
    ("VbakH", "files/res/Object/VbakH.arc", 0xB, 0x5, 0x8),
    ("Trnd", "files/res/Object/Trnd.arc", 0xA, 0x6, 0x7),
    ("NhBody", "files/res/Object/Always.arc", 0x34, 0x0, 0x0),
    ("SyanBody", "files/res/Object/Syan.arc", 0x8, 0x4, 0x5),
    // Field items: generated from d_item_data.cpp:589- (item_resource[] / item_arcname_tbl[]);
    // each key is <archive>_<dRes_ID> so several models from one archive can coexist.
    ("Always_1E", "files/res/Object/Always.arc", 0x1E, 0x0, 0x0),
    ("Always_2A", "files/res/Object/Always.arc", 0x2A, 0x0, 0x0),
    ("Always_32", "files/res/Object/Always.arc", 0x32, 0x0, 0x0),
    ("Always_33", "files/res/Object/Always.arc", 0x33, 0x0, 0x0),
    ("Bin_4", "files/res/Object/Bin.arc", 0x4, 0x0, 0x0),
    ("Bomb_00_3", "files/res/Object/Bomb_00.arc", 0x3, 0x0, 0x0),
    ("Boom_3", "files/res/Object/Boom.arc", 0x3, 0x0, 0x0),
    ("FwBin_4", "files/res/Object/FwBin.arc", 0x4, 0x0, 0x0),
    ("Med_b_4", "files/res/Object/Med_b.arc", 0x4, 0x0, 0x0),
    ("Med_g_4", "files/res/Object/Med_g.arc", 0x4, 0x0, 0x0),
    ("Med_r_4", "files/res/Object/Med_r.arc", 0x4, 0x0, 0x0),
    ("Shinju_d_4", "files/res/Object/Shinju_d.arc", 0x4, 0x0, 0x0),
    ("Shinju_f_4", "files/res/Object/Shinju_f.arc", 0x4, 0x0, 0x0),
    ("Shinju_n_4", "files/res/Object/Shinju_n.arc", 0x4, 0x0, 0x0),
    ("VBinY_4", "files/res/Object/VBinY.arc", 0x4, 0x0, 0x0),
    ("Vamuk_4", "files/res/Object/Vamuk.arc", 0x4, 0x0, 0x0),
    ("Varfi_4", "files/res/Object/Varfi.arc", 0x4, 0x0, 0x0),
    ("Varrl_4", "files/res/Object/Varrl.arc", 0x4, 0x0, 0x0),
    ("VbagE_3", "files/res/Object/VbagE.arc", 0x3, 0x0, 0x0),
    ("VbagF_3", "files/res/Object/VbagF.arc", 0x3, 0x0, 0x0),
    ("VbagH_3", "files/res/Object/VbagH.arc", 0x3, 0x0, 0x0),
    ("Vbelt_4", "files/res/Object/Vbelt.arc", 0x4, 0x0, 0x0),
    ("Vbeso_3", "files/res/Object/Vbeso.arc", 0x3, 0x0, 0x0),
    ("VbinH_5", "files/res/Object/VbinH.arc", 0x5, 0x0, 0x0),
    ("VbinS_4", "files/res/Object/VbinS.arc", 0x4, 0x0, 0x0),
    ("VbinW_4", "files/res/Object/VbinW.arc", 0x4, 0x0, 0x0),
    ("Vboko_3", "files/res/Object/Vboko.arc", 0x3, 0x0, 0x0),
    ("VbomL_4", "files/res/Object/VbomL.arc", 0x4, 0x0, 0x0),
    ("VbomM_4", "files/res/Object/VbomM.arc", 0x4, 0x0, 0x0),
    ("Vboot_4", "files/res/Object/Vboot.arc", 0x4, 0x0, 0x0),
    ("Vbow_4", "files/res/Object/Vbow.arc", 0x4, 0x0, 0x0),
    ("VboxA_4", "files/res/Object/VboxA.arc", 0x4, 0x0, 0x0),
    ("VboxN_4", "files/res/Object/VboxN.arc", 0x4, 0x0, 0x0),
    ("Vchin_3", "files/res/Object/Vchin.arc", 0x3, 0x0, 0x0),
    ("Vchuj_4", "files/res/Object/Vchuj.arc", 0x4, 0x0, 0x0),
    ("Vdoku_4", "files/res/Object/Vdoku.arc", 0x4, 0x0, 0x0),
    ("VdunC_4", "files/res/Object/VdunC.arc", 0x4, 0x0, 0x0),
    ("VdunM_3", "files/res/Object/VdunM.arc", 0x3, 0x0, 0x0),
    ("Vesa_3", "files/res/Object/Vesa.arc", 0x3, 0x0, 0x0),
    ("Vfuku_3", "files/res/Object/Vfuku.arc", 0x3, 0x0, 0x0),
    ("Vhamm_3", "files/res/Object/Vhamm.arc", 0x3, 0x0, 0x0),
    ("Vhane_5", "files/res/Object/Vhane.arc", 0x5, 0x0, 0x0),
    ("Vhapp_3", "files/res/Object/Vhapp.arc", 0x3, 0x0, 0x0),
    ("Vhkak_4", "files/res/Object/Vhkak.arc", 0x4, 0x0, 0x0),
    ("Vho_3", "files/res/Object/Vho.arc", 0x3, 0x0, 0x0),
    ("Vhook_4", "files/res/Object/Vhook.arc", 0x4, 0x0, 0x0),
    ("Vhutu_4", "files/res/Object/Vhutu.arc", 0x4, 0x0, 0x0),
    ("Vhyoi_3", "files/res/Object/Vhyoi.arc", 0x3, 0x0, 0x0),
    ("VkeyN_3", "files/res/Object/VkeyN.arc", 0x3, 0x0, 0x0),
    ("VleMG_3", "files/res/Object/VleMG.arc", 0x3, 0x0, 0x0),
    ("VleMO_3", "files/res/Object/VleMO.arc", 0x3, 0x0, 0x0),
    ("VleOF_3", "files/res/Object/VleOF.arc", 0x3, 0x0, 0x0),
    ("VleTT_3", "files/res/Object/VleTT.arc", 0x3, 0x0, 0x0),
    ("Vleaf_3", "files/res/Object/Vleaf.arc", 0x3, 0x0, 0x0),
    ("Vlupy_5", "files/res/Object/Vlupy.arc", 0x5, 0x0, 0x0),
    ("VmagS_5", "files/res/Object/VmagS.arc", 0x5, 0x0, 0x0),
    ("Vmant_3", "files/res/Object/Vmant.arc", 0x3, 0x0, 0x0),
    ("VmapC_3", "files/res/Object/VmapC.arc", 0x3, 0x0, 0x0),
    ("VmapG_3", "files/res/Object/VmapG.arc", 0x3, 0x0, 0x0),
    ("VmapH_3", "files/res/Object/VmapH.arc", 0x3, 0x0, 0x0),
    ("VmapQ_3", "files/res/Object/VmapQ.arc", 0x3, 0x0, 0x0),
    ("VmapR_3", "files/res/Object/VmapR.arc", 0x3, 0x0, 0x0),
    ("VmapT_3", "files/res/Object/VmapT.arc", 0x3, 0x0, 0x0),
    ("Vobj00_3", "files/res/Object/Vobj00.arc", 0x3, 0x0, 0x0),
    ("Vobj01_3", "files/res/Object/Vobj01.arc", 0x3, 0x0, 0x0),
    ("Vobj02_3", "files/res/Object/Vobj02.arc", 0x3, 0x0, 0x0),
    ("Vobj03_3", "files/res/Object/Vobj03.arc", 0x3, 0x0, 0x0),
    ("Vobj04_3", "files/res/Object/Vobj04.arc", 0x3, 0x0, 0x0),
    ("Vobj05_3", "files/res/Object/Vobj05.arc", 0x3, 0x0, 0x0),
    ("Vobj06_3", "files/res/Object/Vobj06.arc", 0x3, 0x0, 0x0),
    ("Vobj07_3", "files/res/Object/Vobj07.arc", 0x3, 0x0, 0x0),
    ("Vobj08_3", "files/res/Object/Vobj08.arc", 0x3, 0x0, 0x0),
    ("Vobj09_3", "files/res/Object/Vobj09.arc", 0x3, 0x0, 0x0),
    ("Vobj10_3", "files/res/Object/Vobj10.arc", 0x3, 0x0, 0x0),
    ("Vobj11_3", "files/res/Object/Vobj11.arc", 0x3, 0x0, 0x0),
    ("Vohom_3", "files/res/Object/Vohom.arc", 0x3, 0x0, 0x0),
    ("Vpowr_4", "files/res/Object/Vpowr.arc", 0x4, 0x0, 0x0),
    ("Vrope_4", "files/res/Object/Vrope.arc", 0x4, 0x0, 0x0),
    ("VsaiL_4", "files/res/Object/VsaiL.arc", 0x4, 0x0, 0x0),
    ("VsaiM_4", "files/res/Object/VsaiM.arc", 0x4, 0x0, 0x0),
    ("VshiM_4", "files/res/Object/VshiM.arc", 0x4, 0x0, 0x0),
    ("VshiN_3", "files/res/Object/VshiN.arc", 0x3, 0x0, 0x0),
    ("VswoN_3", "files/res/Object/VswoN.arc", 0x3, 0x0, 0x0),
    ("Vtact_3", "files/res/Object/Vtact.arc", 0x3, 0x0, 0x0),
    ("Vtin1_3", "files/res/Object/Vtin1.arc", 0x3, 0x0, 0x0),
    ("Vtin2_3", "files/res/Object/Vtin2.arc", 0x3, 0x0, 0x0),
    ("Vtin3_3", "files/res/Object/Vtin3.arc", 0x3, 0x0, 0x0),
    ("Vtin4_3", "files/res/Object/Vtin4.arc", 0x3, 0x0, 0x0),
    ("Vtin5_3", "files/res/Object/Vtin5.arc", 0x3, 0x0, 0x0),
    ("Vtri1_4", "files/res/Object/Vtri1.arc", 0x4, 0x0, 0x0),
    ("Vtri2_4", "files/res/Object/Vtri2.arc", 0x4, 0x0, 0x0),
    ("Vtri3_4", "files/res/Object/Vtri3.arc", 0x4, 0x0, 0x0),
    ("Vtri4_4", "files/res/Object/Vtri4.arc", 0x4, 0x0, 0x0),
    ("Vtri5_4", "files/res/Object/Vtri5.arc", 0x4, 0x0, 0x0),
    ("Vtri6_4", "files/res/Object/Vtri6.arc", 0x4, 0x0, 0x0),
    ("Vtri7_4", "files/res/Object/Vtri7.arc", 0x4, 0x0, 0x0),
    ("Vtri8_4", "files/res/Object/Vtri8.arc", 0x4, 0x0, 0x0),
    ("VyatL_4", "files/res/Object/VyatL.arc", 0x4, 0x0, 0x0),
    ("VyatM_4", "files/res/Object/VyatM.arc", 0x4, 0x0, 0x0),
    ("key_01_4", "files/res/Object/key_01.arc", 0x4, 0x0, 0x0),
    ("tele_00_4", "files/res/Object/tele_00.arc", 0x4, 0x0, 0x0),
];

/// All data-driven actor bodies, keyed by archive name.
pub fn load_actor_models() -> std::collections::HashMap<&'static str, ActorModel> {
    let mut out = std::collections::HashMap::new();
    for &(archive, path, model_id, min, max) in ACTOR_ARCHIVES {
        let Some(root) = AssetRoot::from_env().ok() else {
            continue;
        };
        let Ok(data) = root.read_archive(path) else {
            continue;
        };
        let Some(mesh) = object_model(&data, model_id) else {
            continue;
        };
        let mut morphs = Vec::new();
        if let Ok(anms) = AnmArchive::open(path) {
            for id in min..=max {
                if let Some(morph) = anms.morph(AnmRef::Id(id as u16), &mesh.joint_parents) {
                    morphs.push((id, morph));
                }
            }
        }
        out.insert(
            archive,
            ActorModel {
                mesh,
                morphs,
                cloth: None,
            },
        );
    }
    out
}

pub fn load_helmaroc_model() -> Option<ActorModel> {
    let archive = load_object_archive(BDK_ARCHIVE)?;
    let mesh = object_model(&archive, BDK_MODEL_ID)?;
    let anms = AnmArchive::open(BDK_OBJECT_ARCHIVE).ok()?;
    let mut morphs = Vec::new();
    for id in 0x1C..=0x42u16 {
        if let Some(morph) = anms.morph(AnmRef::Id(id), &mesh.joint_parents) {
            morphs.push((id as i32, morph));
        }
    }
    Some(ActorModel {
        mesh,
        morphs,
        cloth: None,
    })
}

pub fn load_phantom_ganon_model() -> Option<ActorModel> {
    let archive = load_object_archive(FGANON_ARCHIVE)?;
    let mesh = object_model(&archive, FGANON_MODEL_ID)?;
    let anms = AnmArchive::open(FGANON_OBJECT_ARCHIVE).ok()?;
    let mut morphs = Vec::new();
    for id in 0x6..=0x16u16 {
        if let Some(morph) = anms.morph(AnmRef::Id(id), &mesh.joint_parents) {
            morphs.push((id as i32, morph));
        }
    }
    Some(ActorModel {
        mesh,
        morphs,
        cloth: None,
    })
}

pub fn load_amos_knight_model() -> Option<ActorModel> {
    let archive = load_object_archive(AM2_ARCHIVE)?;
    let mesh = object_model(&archive, AM2_MODEL_ID)?;
    let anms = AnmArchive::open(AM2_OBJECT_ARCHIVE).ok()?;
    let mut morphs = Vec::new();
    for id in 0x6..=0xFu16 {
        if let Some(morph) = anms.morph(AnmRef::Id(id), &mesh.joint_parents) {
            morphs.push((id as i32, morph));
        }
    }
    Some(ActorModel {
        mesh,
        morphs,
        cloth: None,
    })
}

pub fn load_floormaster_model() -> Option<ActorModel> {
    let archive = load_object_archive(FM_ARCHIVE)?;
    let mesh = object_model(&archive, FM_MODEL_ID)?;
    let anms = AnmArchive::open(FM_OBJECT_ARCHIVE).ok()?;
    let mut morphs = Vec::new();
    for id in 0x6..=0x12u16 {
        if let Some(morph) = anms.morph(AnmRef::Id(id), &mesh.joint_parents) {
            morphs.push((id as i32, morph));
        }
    }
    Some(ActorModel {
        mesh,
        morphs,
        cloth: None,
    })
}

pub fn load_armos_model() -> Option<ActorModel> {
    let archive = load_object_archive(AM_ARCHIVE)?;
    let mesh = object_model(&archive, AM_MODEL_ID)?;
    let anms = AnmArchive::open(AM_OBJECT_ARCHIVE).ok()?;
    let mut morphs = Vec::new();
    for id in 0x14..=0x1Fu16 {
        if let Some(morph) = anms.morph(AnmRef::Id(id), &mesh.joint_parents) {
            morphs.push((id as i32, morph));
        }
    }
    Some(ActorModel {
        mesh,
        morphs,
        cloth: None,
    })
}

pub fn load_ba_model() -> Option<ActorModel> {
    let mesh = load_actor_model_entry(BA_ARCHIVE, BA_MODEL_ENTRY)?;
    let anms = AnmArchive::open(BA_OBJECT_ARCHIVE).ok()?;
    let mut morphs = Vec::new();
    for id in 0..=0x08u16 {
        if let Some(morph) = anms.morph(AnmRef::Id(id), &mesh.joint_parents) {
            morphs.push((id as i32, morph));
        }
    }
    if let Some(morph) = anms.morph(AnmRef::Id(0x17), &mesh.joint_parents) {
        morphs.push((0x17, morph));
    }
    let cloth = load_actor_model_entry(BA_ARCHIVE, "ba_cloth.bdl");
    Some(ActorModel {
        mesh,
        morphs,
        cloth,
    })
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

pub const LINK_ANM_ARCHIVE: &str = "files/res/Object/LkAnm.arc";
/// `mAnmDataTable[ANM_WAITS]` (`d_a_player_main_data.inc:1031-1037`): the standing pose is the
/// `WAITS` lower body under the `WAIT` upper body. [`PlayerIdle`] applies the second one only to
/// the `BODY_CHN` subtree, which is the calc split at `d_a_player_main.cpp:11574-11576`.
const LINK_IDLE_UNDER: &str = "bcks/waits.bck";
const LINK_IDLE_UPPER: &str = "bcks/wait.bck";
const LINK_UPPER_ROOT: &str = "body_chn";
/// `mAnmDataTable[ANM_CUTA..ANM_CUTEB]` (`tww/src/d/actor/d_a_player_main_data.inc:1242-1282`,
/// `wwhd/wwhd_src/d/actor/d_a_player_main.cpp:43-48`): the six body animations the sword processes
/// install; walk and dash are already in `link_demo::STEMS`, but they are loaded here too so
/// gameplay sampling never depends on a demo being mounted. Jump/fall, roll and swim use the
/// same single-morph rows (`d_a_player_main_data.inc:1123,1382,1452,1935-1942`;
/// `wwhd/wwhd_src/d/actor/d_a_player_main.cpp:26,63,73,142-143`). Crouch/crawl, hang and ladder
/// rows are in `tww/src/d/actor/d_a_player_main_data.inc:1473-1492,1956-2011` and
/// `wwhd/wwhd_src/d/actor/d_a_player_main.cpp:76-78,145-152`. The Deku Leaf swing is
/// `tww/src/d/actor/d_a_player_main_data.inc:2159-2164` and
/// `wwhd/wwhd_src/d/actor/d_a_player_main.cpp:161`; the Wind Waker play animation is
/// `tww/src/d/actor/d_a_player_main_data.inc:2446-2448` and
/// `wwhd/wwhd_src/d/actor/d_a_player_main.cpp:215`. Bottle and grab rows are in
/// `tww/src/d/actor/d_a_player_main_data.inc:1746-1762,2306-2342` and
/// `wwhd/wwhd_src/d/actor/d_a_player_main.cpp:115,196,199`. The remaining body states use the rows
/// in `tww/src/d/actor/d_a_player_main_data.inc:1368-1429,1620-1710,1718-1738,1900-1906,1921-1934,2194-2250` and
/// `wwhd/wwhd_src/d/actor/d_a_player_main.cpp:61-69,97-113,137,140-141,179,183-186`.
pub const PLAYER_GAMEPLAY_STEMS: &[&str] = &[
    "walk",
    "dash",
    "cuta",
    "cutf",
    "cutr",
    "cutl",
    "cutea",
    "cuteb",
    "jmped",
    "jmpst",
    "rollf",
    "rollfmis",
    "swimp",
    "swimwait",
    "crouch",
    "difence",
    "lieforward",
    "hanging",
    "hangup",
    "ladderupst",
    "ladderdwst",
    "usefana",
    "usefanb",
    "taktdge",
    "bindrinking",
    "binopena",
    "grabup",
    "grabwait",
    "landdama",
    "landdamast",
    "hamswinga",
    "hamswingbpre",
    "hamswingbhit",
    "hamswingbend",
    "hookshotjmp",
    "walkpush",
    "walkpull",
    "slidef",
    "slidefland",
    "slideb",
    "slidebland",
    "dielong",
    "swimdie",
    "boomcatch",
    "damf",
    "atng",
    "difencea",
    "cutturnp",
    "cutturnpwfb",
    "cutturnpwlr",
    "cutturnb",
];

/// The standing animation pair, sampled with the player model's joint order, plus the demo bcks
/// a cutscene can switch the player onto. A missing pair leaves the player on the bind pose
/// rather than substituting a made-up motion, and a demo stem that failed to load leaves it on
/// the standing pair.
pub struct PlayerIdle {
    under: crate::j3d::bck::Morph,
    upper: crate::j3d::bck::Morph,
    upper_mask: Vec<bool>,
    parents: Vec<u16>,
    demos: Vec<(&'static str, crate::j3d::bck::Morph)>,
    gameplay: Vec<(&'static str, crate::j3d::bck::Morph)>,
}

impl PlayerIdle {
    /// The blended world matrix per joint: the `WAITS` pose everywhere, with the `WAIT` locals
    /// composed on the `BODY_CHN` branch under the already posed parent, which is what the
    /// upper `J3DMtxCalcAnm` runs over (`mDoExt_MtxCalcAnmBlendTbl::calc`, `m_Do_ext.h:1129`).
    pub fn pose(&self, frame: f32) -> Vec<glam::Mat4> {
        let under_frame = frame % self.under.frame_max().max(1.0);
        let upper_frame = frame % self.upper.frame_max().max(1.0);
        let mut world = self.under.joint_matrices(under_frame);
        let upper = self.upper.joint_matrices(upper_frame);
        for joint in 0..world.len() {
            if !self.upper_mask.get(joint).copied().unwrap_or(false) {
                continue;
            }
            let Some(parent) = self
                .parents
                .get(joint)
                .copied()
                .and_then(|parent| usize::try_from(parent).ok())
            else {
                continue;
            };
            let (Some(upper_parent), Some(posed_parent)) = (upper.get(parent), world.get(parent))
            else {
                continue;
            };
            let Some(inverse) = upper_parent.try_inverse() else {
                continue;
            };
            world[joint] = *posed_parent * inverse * upper[joint];
        }
        world
    }

    /// The pose for the bck a demo step selected - [`crate::sim::link_demo::PlayerAnm`] - with
    /// `idle_frame` falling back to the standing pair. `None` is the reference's morf still on
    /// `ANM_WAITS`, either because no demo drives the player or because the mode named no stem.
    ///
    /// A demo bck plays once and holds its last frame: `setAnm`'s end frame is `-1.0f`
    /// (`d_demo.cpp:229`), the value that stops the controller at the animation's last frame
    /// rather than wrapping, which is why this clamps instead of the idle pose's modulo.
    pub fn pose_stem(&self, demo: Option<(&str, f32)>, idle_frame: f32) -> Vec<glam::Mat4> {
        self.pose_named(demo.map(|(stem, frame)| (stem, frame, false)), idle_frame)
    }

    /// The controller's end frame for a loaded stem, the `mFrameCtrl::getEnd` the mirror shield's
    /// sway divides its phase by (`tww/src/d/actor/d_a_player_main.cpp:1328-1329`).
    pub fn stem_frame_max(&self, stem: &str) -> f32 {
        self.gameplay
            .iter()
            .chain(self.demos.iter())
            .find(|(name, _)| *name == stem)
            .map_or(1.0, |(_, morph)| morph.frame_max())
    }

    /// The bck gameplay put the body on. `looping` wraps walk and dash the way the retail move
    /// controller does, while a sword cut holds its last frame like a demo (`d_demo.cpp:229`).
    pub fn pose_named(&self, named: Option<(&str, f32, bool)>, idle_frame: f32) -> Vec<glam::Mat4> {
        let posed = named.and_then(|(stem, frame, looping)| {
            self.gameplay
                .iter()
                .chain(self.demos.iter())
                .find(|(name, _)| *name == stem)
                .map(|(_, morph)| {
                    let frame = if looping {
                        frame % morph.frame_max().max(1.0)
                    } else {
                        frame.min(morph.frame_max().max(1.0))
                    };
                    morph.joint_matrices(frame)
                })
        });
        posed.unwrap_or_else(|| self.pose(idle_frame))
    }
}

pub fn load_link_idle(mesh: &crate::j3d::bdl::RoomMesh) -> Option<PlayerIdle> {
    let anms = AnmArchive::open(LINK_ANM_ARCHIVE).ok()?;
    let parents = &mesh.joint_parents;
    let under = anms.morph(AnmRef::Entry(LINK_IDLE_UNDER), parents)?;
    let upper = anms.morph(AnmRef::Entry(LINK_IDLE_UPPER), parents)?;
    let upper_root = mesh
        .joint_names
        .iter()
        .position(|name| name.eq_ignore_ascii_case(LINK_UPPER_ROOT))?;
    let mut upper_mask = vec![false; mesh.joint_parents.len()];
    upper_mask[upper_root] = true;
    for joint in upper_root + 1..upper_mask.len() {
        let parent = mesh
            .joint_parents
            .get(joint)
            .copied()
            .and_then(|parent| usize::try_from(parent).ok());
        if parent.is_some_and(|parent| parent < joint && upper_mask[parent]) {
            upper_mask[joint] = true;
        }
    }
    let mut demos = Vec::new();
    for stem in crate::sim::link_demo::STEMS {
        let entry = format!("bcks/{stem}.bck");
        match anms.morph(AnmRef::Entry(&entry), parents) {
            Some(morph) => demos.push((*stem, morph)),
            None => log::warn!("clearsleeper: missing link demo animation {stem}"),
        }
    }
    let mut gameplay = Vec::new();
    for stem in PLAYER_GAMEPLAY_STEMS {
        let entry = format!("bcks/{stem}.bck");
        match anms.morph(AnmRef::Entry(&entry), parents) {
            Some(morph) => gameplay.push((*stem, morph)),
            None => log::warn!("clearsleeper: missing link gameplay animation {stem}"),
        }
    }
    Some(PlayerIdle {
        under,
        upper,
        upper_mask,
        parents: mesh.joint_parents.clone(),
        demos,
        gameplay,
    })
}

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
    let anms_archive = AnmArchive::open(LINK_ANM_ARCHIVE)?;
    let mut anms = crate::sim::event::LinkAnms::default();
    for stem in crate::sim::event::LINK_ANIM_STEMS {
        let entry = format!("bcks/{stem}.bck");
        match anms_archive.anm_fields(AnmRef::Entry(&entry), AnmKind::Bck) {
            Some((attribute, frames)) => anms.insert(stem, attribute, frames),
            None => log::warn!("clearsleeper: missing or unreadable link animation {stem}"),
        }
    }
    Ok(anms)
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

/// How an animation is addressed inside an object archive. Link's animations are named
/// (`bcks/walk.bck`), an NPC's are indexed by the `dRes_ID_*` its anm table names
/// (`dNpc_setAnmIDRes`, `d_npc.cpp:546-563`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnmRef<'a> {
    Entry(&'a str),
    Id(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnmKind {
    Bck,
    Btp,
}

impl AnmKind {
    fn header(self) -> (&'static [u8; 8], &'static [u8; 4]) {
        match self {
            Self::Bck => (b"J3D1bck1", b"ANK1"),
            Self::Btp => (b"J3D1btp1", b"TPT1"),
        }
    }
}

/// One object archive opened for animation reads: the bytes are decoded the same way whether
/// the archive came from the extracted tree or a disc image, and both the morph and the
/// frame count come from the same decoded block.
pub struct AnmArchive {
    archive: Archive,
    path: PathBuf,
}

impl AnmArchive {
    pub fn open(name: &str) -> Result<Self, AssetError> {
        let root = AssetRoot::from_env()?;
        Ok(Self {
            archive: root.read_archive(name)?,
            path: root.path(name),
        })
    }

    pub fn bytes(&self, reference: AnmRef<'_>) -> Result<Vec<u8>, AssetError> {
        let data = match reference {
            AnmRef::Entry(entry) => self.archive.read(entry)?,
            AnmRef::Id(id) => self.archive.read_id(id)?,
        };
        if data.len() >= 4 && data[..4] == YAZ0_MAGIC {
            yaz0_decode(data, &self.path)
        } else {
            Ok(data.to_vec())
        }
    }

    /// The `ANK1`/`TPT1` attribute and frame count both loaders copy
    /// (`J3DAnmLoader.cpp:294-306`); Link keeps the attribute, NPCs keep the count.
    pub fn anm_fields(&self, reference: AnmRef<'_>, kind: AnmKind) -> Option<(u8, u16)> {
        let raw = self.bytes(reference).ok()?;
        let (magic, tag) = kind.header();
        let (attribute, frames) = anm_block(&raw, magic, tag)?;
        Some((attribute, frames.max(0) as u16))
    }

    pub fn frame_count(&self, reference: AnmRef<'_>, kind: AnmKind) -> Option<u16> {
        self.anm_fields(reference, kind).map(|(_, frames)| frames)
    }

    /// A BCK posed against a model's joint order, `mDoExt_McaMorf::setAnm`'s
    /// `J3DAnmTransform` (`m_Do_ext.cpp:1412-1424`).
    pub fn morph(&self, reference: AnmRef<'_>, parents: &[u16]) -> Option<crate::j3d::bck::Morph> {
        let raw = self.bytes(reference).ok()?;
        crate::j3d::bck::Morph::parse(&raw, parents).ok()
    }
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
    let archive = AnmArchive::open(BA_OBJECT_ARCHIVE)?;
    let mut anms = crate::sim::ba1::Ba1Anms::default();
    let ids = crate::sim::ba1::BCK
        .iter()
        .map(|id| (*id, AnmKind::Bck))
        .chain(crate::sim::ba1::BTP.iter().map(|id| (*id, AnmKind::Btp)));
    for (id, kind) in ids {
        let Ok(id) = u16::try_from(id) else {
            continue;
        };
        match archive.frame_count(AnmRef::Id(id), kind) {
            Some(frames) => anms.insert(id, frames as i16),
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

/// One `PLYR` row of a room: where the player wakes up on entering through that point, and the
/// row's own parameter word (its start event lives in bits 24-31, `d_a_player_main.h:1881`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageSpawn {
    pub pos: glam::Vec3,
    pub yaw: f32,
    pub player: u32,
}

pub fn load_stage_spawn(stage: &str, room: u8, point: u8) -> Option<StageSpawn> {
    match stage_spawn_from_assets(stage, room, point) {
        Ok(spawn) => Some(spawn),
        Err(error) => {
            log::warn!("clearsleeper: stage spawn unavailable ({error})");
            None
        }
    }
}

/// Index into the weather texture array: the six `Always.arc` entries followed by three
/// stage-archive cloud layers. Every producer and consumer uses this so the order cannot drift
/// (`tww/src/d/d_kankyo_rain.cpp:3947`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum WeatherPage {
    /// `Always.arc` 0x81 (`tx_snow01`): rain, snow, sun, stars and wave caps.
    Rain = 0,
    /// `Always.arc` 0x82: the second lens-flare layer.
    LenzA = 1,
    /// `Always.arc` 0x85: the first lens-flare layer and the sibuki splash.
    LenzB = 2,
    /// `Always.arc` 0x6D: housi smoke.
    Smoke = 3,
    /// `Always.arc` 0x6E: poison gas.
    Poison = 4,
    /// `Always.arc` 0x84: cloud shadows.
    CloudShadow = 5,
    Vrkumo0 = 6,
    Vrkumo1 = 7,
    Vrkumo2 = 8,
}

impl WeatherPage {
    pub const COUNT: usize = Self::Vrkumo2 as usize + 1;

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn vrkumo(layer: usize) -> Self {
        match layer {
            0 => Self::Vrkumo0,
            1 => Self::Vrkumo1,
            _ => Self::Vrkumo2,
        }
    }
}

pub const WEATHER_IDS: [u16; 6] = [0x81, 0x82, 0x85, 0x6D, 0x6E, 0x84];

pub fn load_weather_textures(stage: &str) -> Vec<Option<(Vec<u8>, u32, u32)>> {
    let Some(root) = AssetRoot::from_env().ok() else {
        return vec![None; WeatherPage::COUNT];
    };
    let Ok(archive) = root.read_archive(OBJECT_ALWAYS) else {
        return vec![None; WeatherPage::COUNT];
    };
    let mut pages = WEATHER_IDS
        .iter()
        .map(|&id| {
            let raw = archive.read_id(id).ok()?;
            let image = crate::bti::decode(raw).or_else(|| {
                // Always 0x81 (`tx_snow01`) is a headerless 64x64 raw I8 image, not a BTI.
                if id != 0x81 || raw.len() < 64 * 64 {
                    return None;
                }
                Some(crate::bti::Image {
                    width: 64,
                    height: 64,
                    pixels: raw[..64 * 64].iter().flat_map(|&v| [v, v, v, v]).collect(),
                    mag_linear: true,
                    wrap_s: 0,
                    wrap_t: 0,
                    alpha_enabled: true,
                })
            })?;
            Some((image.pixels, image.width as u32, image.height as u32))
        })
        .collect::<Vec<_>>();
    // Vrkumo cloud layers use three BTI entries from the stage archive
    // (`tww/src/d/d_kankyo_wether.cpp:818-834`;
    // `wwhd/wwhd_src/d/d_kankyo_wether.cpp:2830-2846`).
    if let Ok(map) = root.read_archive(&stage_map(stage)) {
        for name in ["cloud_01.bti", "cloud_02.bti", "cloud_03.bti"] {
            let image = map
                .read(name)
                .ok()
                .and_then(crate::bti::decode)
                .map(|image| (image.pixels, image.width as u32, image.height as u32));
            pages.push(image);
        }
    } else {
        pages.resize(WeatherPage::COUNT, None);
    }
    pages
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

/// The sea surface texture `b_sea_tex0and2.bti`, decoded to RGBA so the surface can sample it
/// with scrolling UVs (`tww/src/d/actor/d_a_sea.cpp:750-810`; HD `wwhd_src/d/actor/d_a_sea.cpp:796`).
pub fn load_sea_texture() -> Option<(Vec<u8>, u32, u32)> {
    let root = AssetRoot::from_env().ok()?;
    let archive = root.read_archive(OBJECT_ALWAYS).ok()?;
    let image = crate::bti::decode(archive.read(SEA_TEXTURE_ENTRY).ok()?)?;
    Some((image.pixels, image.width as u32, image.height as u32))
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
    let spawn = stage_spawn(room.read("dzr/room.dzr")?, path, spawn_point)?;
    Ok(crate::sim::collision::StageData {
        spawn: spawn.pos,
        yaw: spawn.yaw,
        player: spawn.player,
        collision,
    })
}

fn stage_spawn_from_assets(stage: &str, room: u8, point: u8) -> Result<StageSpawn, AssetError> {
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

pub fn stage_spawn(data: &[u8], path: PathBuf, point: u8) -> Result<StageSpawn, AssetError> {
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
        return Ok(StageSpawn {
            pos: spawn,
            yaw: f32::from(angle_y) * RAD_PER_UNIT,
            player: read_u32(data, at + 0x08).ok_or_else(|| corrupt("dzr truncated"))?,
        });
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
                // An IA8 texel is a 16-bit word with the alpha in the high half and the
                // intensity in the low half, the same way IA4 keeps its alpha in the high
                // nibble (`gecko/crates/gecko/src/flipper/gx/texture.rs:200-238`, `:170-194`).
                let alpha = line[at];
                let level = line[at + 1];
                [level, level, level, alpha]
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
