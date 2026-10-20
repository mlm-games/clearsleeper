use std::path::{Path, PathBuf};

use glam::Vec3;
use repame_save::{AppId, FsStorage, SaveError, SaveFile, Storage};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::sim::area::Area;
use crate::sim::flags::{Arrivals, EventFlags, StoryBits, Switches};
use crate::sim::frame::CurrentFrame;
use crate::sim::health::Health;
use crate::sim::intent::CameraYaw;
use crate::sim::item::Held;
use crate::sim::pickup::Rupees;
use crate::sim::player::Player;
use crate::sim::talk::Dialogue;
use crate::{App, assets};

/// A slot's schema in the `repame-save` envelope. The payload keeps its own
/// `version` field, stamped with the same number.
pub const SAVE_VERSION: u32 = 1;
pub const DEFAULT_NAME: &str = "Link";
pub const SLOTS: usize = 3;

/// The game's reverse-DNS identity. `AppId::data_dir` resolves the same
/// per-platform directory the stack's other games save into; on Android the
/// runtime dir `set_android_data_dir` records from `android_main` keeps it on
/// the app's internal data path.
const APP: AppId = AppId::new("com.mlm-games.clearsleeper");

/// The directory name every pre-`repame-save` build stamped its raw JSON
/// under, before the data directory moved onto this game's [`APP`].
#[cfg(not(target_arch = "wasm32"))]
const LEGACY_APP: &str = "clearsleeper";

const FLAG_BYTES: usize = 0x100;
const STORY_BYTES: usize = 0x100;
const ARRIVAL_BYTES: usize = 49;

fn padded_flags(saved: &[u8]) -> [u8; FLAG_BYTES] {
    let mut out = [0_u8; FLAG_BYTES];
    for (slot, word) in out.iter_mut().zip(saved) {
        *slot = *word;
    }
    out
}

fn padded_bytes(saved: &[u8]) -> [u8; STORY_BYTES] {
    let mut out = [0_u8; STORY_BYTES];
    for (slot, byte) in out.iter_mut().zip(saved) {
        *slot = *byte;
    }
    out
}

fn padded_arrivals(saved: &[u8]) -> [u8; ARRIVAL_BYTES] {
    let mut out = [0_u8; ARRIVAL_BYTES];
    for (slot, byte) in out.iter_mut().zip(saved) {
        *slot = *byte;
    }
    out
}

fn default_name() -> String {
    DEFAULT_NAME.to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveGame {
    pub version: u32,
    #[serde(default = "default_name")]
    pub name: String,
    pub tick: u64,
    pub camera_yaw: f32,
    pub player: SavedPlayer,
    pub area: SavedArea,
    #[serde(default)]
    pub health: SavedHealth,
    #[serde(default)]
    pub stage: String,
    #[serde(default)]
    pub rooms: Vec<u8>,
    #[serde(default)]
    pub rupees: u32,
    #[serde(default)]
    pub events: Vec<u8>,
    #[serde(default)]
    pub switches: [u32; 4],
    #[serde(default)]
    pub arrivals: Vec<u8>,
    #[serde(default)]
    pub story: Vec<u8>,
    #[serde(default)]
    pub held_item: u8,
    #[serde(default)]
    pub held_count: u8,
    #[serde(default)]
    pub time: f32,
    #[serde(default)]
    pub date: u8,
    #[serde(default)]
    pub get_bag: u16,
}

/// What one slot's file held. `Missing` and `Corrupt` are different answers to the file select:
/// the first offers a new file, the second offers an erase.
#[derive(Debug, Clone, PartialEq)]
pub enum SlotRead {
    Missing,
    Unreadable,
    Corrupt,
    Loaded(SaveGame),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedPlayer {
    pub pos: [f32; 3],
    pub yaw: f32,
    pub speed: f32,
    pub speed_y: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedArea {
    pub bounds_min: [f32; 2],
    pub bounds_max: [f32; 2],
    pub spawn: [f32; 3],
    #[serde(default)]
    pub yaw: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedHealth {
    pub units: u8,
    pub max: u8,
}

impl Default for SavedHealth {
    fn default() -> Self {
        (&Health::default()).into()
    }
}

impl From<&Player> for SavedPlayer {
    fn from(player: &Player) -> Self {
        Self {
            pos: player.pos.into(),
            yaw: player.yaw,
            speed: player.speed,
            speed_y: player.speed_y,
        }
    }
}

impl From<SavedPlayer> for Player {
    fn from(saved: SavedPlayer) -> Self {
        Player {
            pos: saved.pos.into(),
            yaw: saved.yaw,
            face: saved.yaw,
            speed: saved.speed,
            speed_y: saved.speed_y,
            drift: Vec3::ZERO,
            no_draw: false,
        }
    }
}

impl From<&Area> for SavedArea {
    fn from(area: &Area) -> Self {
        Self {
            bounds_min: area.bounds_min.into(),
            bounds_max: area.bounds_max.into(),
            spawn: area.spawn.into(),
            yaw: area.yaw,
        }
    }
}

impl From<SavedArea> for Area {
    fn from(saved: SavedArea) -> Self {
        Area {
            bounds_min: saved.bounds_min.into(),
            bounds_max: saved.bounds_max.into(),
            spawn: saved.spawn.into(),
            yaw: saved.yaw,
        }
    }
}

impl From<&Health> for SavedHealth {
    fn from(health: &Health) -> Self {
        Self {
            units: health.units,
            max: health.max,
        }
    }
}

impl From<SavedHealth> for Health {
    fn from(saved: SavedHealth) -> Self {
        Health {
            units: saved.units,
            max: saved.max,
        }
    }
}

impl App {
    pub fn save(&mut self) -> SaveGame {
        let world = &mut self.sim.world;
        let tick = world.resource::<CurrentFrame>().0;
        let camera_yaw = world.resource::<CameraYaw>().0;
        let area = *world.resource::<Area>();
        let mut query = world.query::<&Player>();
        let player = *query.iter(world).next().expect("player exists at boot");
        let health = *world.resource::<Health>();
        let rupees = world.resource::<Rupees>().count;
        let events = world.resource::<EventFlags>().words().to_vec();
        let switches = world.resource::<Switches>().words();
        let story = world.resource::<StoryBits>().bytes.to_vec();
        let arrivals = world.resource::<Arrivals>().bytes();
        let held = *world.resource::<Held>();
        let get_bag = world
            .query_filtered::<&crate::sim::zunari::Zunari, bevy_ecs::query::With<crate::sim::zunari::Zunari>>()
            .iter(world)
            .next()
            .map_or(0, |stall| stall.roten_reserve);
        let clock = world
            .get_resource::<crate::sim::daynight::DayNight>()
            .copied()
            .unwrap_or_default();
        let stage = self
            .stage
            .as_ref()
            .map(|tables| tables.name().to_owned())
            .unwrap_or_default();
        SaveGame {
            version: SAVE_VERSION,
            name: self.name.clone(),
            tick,
            camera_yaw,
            player: (&player).into(),
            area: (&area).into(),
            health: (&health).into(),
            stage,
            rooms: self.rooms.clone(),
            rupees,
            events,
            story,
            switches,
            arrivals: arrivals.to_vec(),
            held_item: held.item,
            held_count: held.count,
            time: clock.time,
            date: clock.date,
            get_bag,
        }
    }

    pub fn load(&mut self, save: &SaveGame) -> anyhow::Result<()> {
        if save.version != SAVE_VERSION {
            anyhow::bail!(
                "unsupported save version {} (expected {})",
                save.version,
                SAVE_VERSION
            );
        }
        self.name = save.name.clone();
        {
            let world = &mut self.sim.world;
            *world.resource_mut::<CurrentFrame>() = CurrentFrame(save.tick);
            world.resource_mut::<CameraYaw>().0 = save.camera_yaw;
            *world.resource_mut::<Health>() = save.health.clone().into();
            *world.resource_mut::<Rupees>() = Rupees { count: save.rupees };
            *world.resource_mut::<EventFlags>() =
                EventFlags::from_words(padded_flags(save.events.as_slice()));
            *world.resource_mut::<Switches>() = Switches::from_words(save.switches);
            *world.resource_mut::<Arrivals>() =
                Arrivals::from_bytes(padded_arrivals(save.arrivals.as_slice()));
            let mut stories = world.resource_mut::<StoryBits>();
            stories.bytes = padded_bytes(save.story.as_slice());
            let mut held = world.resource_mut::<Held>();
            held.item = save.held_item;
            held.count = save.held_count;
            held.pressed = false;
            let mut clock = world.resource_mut::<crate::sim::daynight::DayNight>();
            clock.time = save.time;
            clock.date = save.date;
            *world.resource_mut::<Dialogue>() = Dialogue::default();
        }
        crate::sim::clear_shots(&mut self.sim.world);
        let current = self
            .stage
            .as_ref()
            .map(|tables| tables.name().to_owned())
            .unwrap_or_default();
        if !save.stage.is_empty() && save.stage != current {
            if let Some(tables) = assets::load_stage_tables(&save.stage) {
                self.swap_stage(
                    tables,
                    save.rooms.clone(),
                    save.area.spawn.into(),
                    save.area.yaw,
                    crate::sim::transition::LAYER_NONE,
                );
            }
        } else {
            let rooms = save.rooms.clone();
            if !rooms.is_empty() {
                let stories = *self.sim.world.resource::<StoryBits>();
                let set = self.stage.as_ref().and_then(|tables| {
                    crate::load_rooms(tables, &rooms, stories, self.start_layer)
                });
                match set {
                    Some(set) => {
                        let frame = self.sim.world.resource::<CurrentFrame>().0;
                        self.install_room_set(set, frame);
                        self.rooms = rooms;
                    }
                    None => self.install_room_actors(Vec::new()),
                }
            }
        }
        let world = &mut self.sim.world;
        *world.resource_mut::<Area>() = save.area.clone().into();
        let mut stalls =
            world.query_filtered::<&mut crate::sim::zunari::Zunari, bevy_ecs::query::With<crate::sim::zunari::Zunari>>();
        for mut stall in stalls.iter_mut(world) {
            stall.roten_reserve = save.get_bag;
        }
        let mut query = world.query::<&mut Player>();
        let mut player = query.iter_mut(world).next().expect("player exists at boot");
        *player = save.player.clone().into();
        if save.stage == assets::PROLOGUE_STAGE {
            self.place_camera(save.area.spawn.into(), save.area.yaw, true);
        }
        Ok(())
    }

    pub fn set_save_dir(&mut self, dir: PathBuf) {
        self.save_dir = dir.clone();
        self.menu.card.dir = dir.clone();
        self.menu.slots = (0..SLOTS)
            .map(|slot| match read_slot_in(&dir, slot) {
                Some(saved) => crate::menus::SlotInfo::from_save(&saved),
                None => crate::menus::SlotInfo::Empty,
            })
            .collect();
    }

    pub fn read_slot(&self, slot: usize) -> Option<SaveGame> {
        read_slot_in(&self.save_dir, slot)
    }

    pub fn write_slot(&mut self, slot: usize) -> anyhow::Result<()> {
        let save = self.save();
        write_slot_in(&self.save_dir, slot, &save)
    }
}

/// [`CS_SAVE_DIR`] when set, otherwise the app's data directory. On wasm that
/// is the virtual `com-mlm-games-clearsleeper` path `ropfs` backs with
/// localStorage: no platform environment resolves there and no `temp_dir` is
/// reachable.
pub fn slots_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CS_SAVE_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    APP.data_dir()
}

/// One slot's versioned document: `slotN.ron` under `dir`.
fn slot_file(dir: &Path, slot: usize) -> SaveFile {
    SaveFile::in_dir(dir, format!("slot{slot}.ron"), SAVE_VERSION)
}

fn store_at(path: &Path) -> anyhow::Result<SaveFile> {
    let dir = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("save path {} has no file name", path.display()))?;
    Ok(SaveFile::in_dir(
        dir,
        name.to_string_lossy().into_owned(),
        SAVE_VERSION,
    ))
}

pub fn write_save(path: &Path, save: &SaveGame) -> anyhow::Result<()> {
    store_at(path)?
        .save(save)
        .map_err(|error| anyhow::anyhow!("writing {}: {error}", path.display()))
}

pub fn read_save(path: &Path) -> anyhow::Result<SaveGame> {
    match store_at(path)?.load::<SaveGame>() {
        Ok(Some(save)) => Ok(save),
        Ok(None) => anyhow::bail!("reading {}: no save file", path.display()),
        Err(error) => anyhow::bail!("reading {}: {error}", path.display()),
    }
}

pub fn slot_path_in(dir: &Path, slot: usize) -> PathBuf {
    dir.join(format!("slot{slot}.ron"))
}

pub fn read_slot_state_in(dir: &Path, slot: usize) -> SlotRead {
    let file = slot_file(dir, slot);
    match file.load::<SaveGame>() {
        Ok(Some(save)) => SlotRead::Loaded(save),
        // No document yet is a first run, or a build that predates this one,
        // whose raw JSON sits beside it.
        Ok(None) => match adopt_slot(dir, slot, &file) {
            Legacy::Adopted(save) => SlotRead::Loaded(save),
            // A legacy file is there and is not a save: the erase answer, not
            // a first run.
            Legacy::Present => SlotRead::Corrupt,
            Legacy::Absent => SlotRead::Missing,
        },
        // A corrupt document left no data behind: the legacy copy is the only
        // progress left, so it is adopted rather than reset to a new file.
        Err(error @ SaveError::Corrupt { .. }) => {
            log::warn!("clearsleeper: {error}");
            match adopt_slot(dir, slot, &file) {
                Legacy::Adopted(save) => SlotRead::Loaded(save),
                _ => SlotRead::Corrupt,
            }
        }
        // Unreadable is a refusal: a file that may still be intact is never
        // replaced by an older copy.
        Err(error @ SaveError::Unreadable { .. }) => {
            log::warn!("clearsleeper: {error}");
            SlotRead::Unreadable
        }
        // A refusal the file layer reports outside those two (a document from a
        // newer build, a payload that does not fit) is not a new file either.
        Err(error) => {
            log::warn!("clearsleeper: {error}");
            SlotRead::Corrupt
        }
    }
}

pub fn read_slot_in(dir: &Path, slot: usize) -> Option<SaveGame> {
    match read_slot_state_in(dir, slot) {
        SlotRead::Loaded(save) => Some(save),
        _ => None,
    }
}

pub fn write_slot_in(dir: &Path, slot: usize, save: &SaveGame) -> anyhow::Result<()> {
    slot_file(dir, slot)
        .save(save)
        .map_err(|error| anyhow::anyhow!("writing {}: {error}", slot_path_in(dir, slot).display()))
}

fn adopt_slot(dir: &Path, slot: usize, file: &SaveFile) -> Legacy<SaveGame> {
    adopt_legacy(file, &format!("slot{slot}.json"), dir)
}

/// What a pre-`repame-save` file left behind.
pub(crate) enum Legacy<T> {
    /// A legacy file parsed and was written through the envelope.
    Adopted(T),
    /// A legacy file is there and none of the candidates parses.
    Present,
    /// No legacy file at any candidate path.
    Absent,
}

/// Carry a document written before this build onto `repame-save`: the raw JSON
/// of `file`'s old name, read from the live directory first and from the
/// platform directories the game used before the data dir moved. The legacy
/// file is read through the same storage the store writes with (`ropfs` on
/// wasm), left in place, and the adopted copy goes through the envelope.
pub(crate) fn adopt_legacy<T: DeserializeOwned + Serialize>(
    store: &SaveFile,
    file: &str,
    dir: &Path,
) -> Legacy<T> {
    let mut present = false;
    for path in legacy_file_paths(dir, file) {
        let Some(bytes) = read_legacy(&path) else {
            continue;
        };
        present = true;
        match serde_json::from_slice::<T>(&bytes) {
            Ok(value) => {
                if let Err(error) = store.save(&value) {
                    log::warn!("clearsleeper: adopting {}: {error}", path.display());
                }
                return Legacy::Adopted(value);
            }
            Err(error) => log::warn!(
                "clearsleeper: {} is not the document it names ({error})",
                path.display()
            ),
        }
    }
    if present {
        Legacy::Present
    } else {
        Legacy::Absent
    }
}

/// The places the same document may still sit under its old name: the live
/// directory first (Linux never moved, and a build after the data-dir move
/// wrote raw JSON there too), then, for the game's own data directory, the
/// platform directories used before the move. A directory the game did not
/// choose itself, such as a test's scratch dir or a `CS_SAVE_DIR` escape
/// hatch, reads only itself.
pub(crate) fn legacy_file_paths(dir: &Path, file: &str) -> Vec<PathBuf> {
    let mut paths = vec![dir.join(file)];
    if is_default_app_dir(dir) {
        paths.extend(legacy_dirs().into_iter().map(|legacy| legacy.join(file)));
    }
    paths
}

/// Whether `dir` is the app's own data directory, the one `CS_SAVE_DIR` did
/// not redirect. Only then does adoption reach into the pre-move platform
/// directories, so an explicit save dir stays isolated.
fn is_default_app_dir(dir: &Path) -> bool {
    if std::env::var("CS_SAVE_DIR").is_ok_and(|value| !value.is_empty()) {
        return false;
    }
    dir == APP.data_dir().as_path()
}

/// The directories a pre-`repame-save` build wrote into. Windows used
/// `%APPDATA%\clearsleeper`, macOS and the rest of the unix-like platforms
/// `~/.local/share/clearsleeper` (or `$XDG_DATA_HOME/clearsleeper`), and where
/// no home was visible, Android included, `temp_dir()/clearsleeper`, the
/// app's cache directory.
#[cfg(not(target_arch = "wasm32"))]
fn legacy_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    #[cfg(target_os = "windows")]
    if let Some(appdata) = non_empty_env("APPDATA") {
        dirs.push(appdata.join(LEGACY_APP));
    }
    if let Some(data) = non_empty_env("XDG_DATA_HOME") {
        dirs.push(data.join(LEGACY_APP));
    }
    if let Some(home) = non_empty_env("HOME") {
        dirs.push(home.join(".local/share").join(LEGACY_APP));
    }
    dirs.push(std::env::temp_dir().join(LEGACY_APP));
    dirs
}

/// wasm kept no persistent file before the current `ropfs` path, so there is
/// no legacy directory to read and `temp_dir` stays off the boot path.
#[cfg(target_arch = "wasm32")]
fn legacy_dirs() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
fn non_empty_env(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// Read a raw JSON document a pre-`repame-save` build left behind, without
/// touching it: a missing or unreadable file is skipped, never quarantined,
/// renamed, or deleted.
fn read_legacy(path: &Path) -> Option<Vec<u8>> {
    match FsStorage.read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            log::warn!("clearsleeper: reading {} ({error})", path.display());
            None
        }
    }
}
