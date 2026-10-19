use std::path::{Path, PathBuf};

use game_utils::save::SaveManager;
use game_utils::save_store::{LoadStatus, SaveStore};
use game_utils::storage::FsStorage;
use glam::Vec3;
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

pub const SAVE_VERSION: u32 = 1;
pub const DEFAULT_NAME: &str = "Link";
pub const SLOTS: usize = 3;
const SAVE_QUALIFIER: &str = "com";
const SAVE_ORG: &str = "mlm-games";
const SAVE_APP: &str = "clearsleeper";
const SLOT_FILE: &str = "slot0.json";
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

fn is_save(bytes: &[u8]) -> bool {
    serde_json::from_slice::<SaveGame>(bytes).is_ok()
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
        Ok(())
    }

    pub fn set_save_dir(&mut self, dir: PathBuf) {
        self.save_dir = dir;
    }

    pub fn read_slot(&self, slot: usize) -> Option<SaveGame> {
        read_slot_in(&self.save_dir, slot)
    }

    pub fn write_slot(&mut self, slot: usize) -> anyhow::Result<()> {
        let save = self.save();
        write_slot_in(&self.save_dir, slot, &save)
    }
}

fn save_manager() -> SaveManager<FsStorage> {
    SaveManager::new(SAVE_QUALIFIER, SAVE_ORG, SAVE_APP, SLOT_FILE, SAVE_VERSION)
}

/// The save directory: `CS_SAVE_DIR` when set, otherwise the platform data dir
/// `game-utils` resolves. Linux stays on `$XDG_DATA_HOME/clearsleeper`; Windows, macOS and
/// Android move to the stack's standard `ProjectDirs`/internal-data directory, where the
/// other repame games keep their saves. On wasm it is the virtual
/// `com-mlm-games-clearsleeper` path `ropfs` backs with localStorage, so no
/// `std::env::temp_dir` is reachable.
pub fn slots_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CS_SAVE_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    save_manager().data_dir()
}

fn slot_store(dir: &Path, slot: usize) -> SaveStore<FsStorage> {
    SaveStore::new(dir, format!("slot{slot}.json")).with_validator(is_save)
}

fn store_at(path: &Path) -> anyhow::Result<SaveStore<FsStorage>> {
    let dir = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("save path {} has no file name", path.display()))?;
    Ok(SaveStore::new(dir, name.to_string_lossy().into_owned()).with_validator(is_save))
}

pub fn write_save(path: &Path, save: &SaveGame) -> anyhow::Result<()> {
    let text = serde_json::to_string(save)?;
    store_at(path)?
        .write(text.as_bytes())
        .map_err(|error| anyhow::anyhow!("writing {}: {error}", path.display()))
}

pub fn read_save(path: &Path) -> anyhow::Result<SaveGame> {
    let loaded = store_at(path)?.load(&is_save, &[]);
    let Some(bytes) = loaded.data else {
        anyhow::bail!("reading {}: {:?}", path.display(), loaded.status);
    };
    Ok(serde_json::from_slice(&bytes)?)
}

pub fn slot_path_in(dir: &Path, slot: usize) -> PathBuf {
    dir.join(format!("slot{slot}.json"))
}

pub fn read_slot_state_in(dir: &Path, slot: usize) -> SlotRead {
    let loaded = slot_store(dir, slot).load(&is_save, &[]);
    match loaded.data {
        Some(bytes) => match serde_json::from_slice::<SaveGame>(&bytes) {
            Ok(save) => SlotRead::Loaded(save),
            Err(_) => SlotRead::Corrupt,
        },
        None => match loaded.status {
            LoadStatus::Missing => SlotRead::Missing,
            LoadStatus::Unreadable => SlotRead::Unreadable,
            _ => SlotRead::Corrupt,
        },
    }
}

pub fn read_slot_in(dir: &Path, slot: usize) -> Option<SaveGame> {
    match read_slot_state_in(dir, slot) {
        SlotRead::Loaded(save) => Some(save),
        _ => None,
    }
}

pub fn write_slot_in(dir: &Path, slot: usize, save: &SaveGame) -> anyhow::Result<()> {
    let text = serde_json::to_string(save)?;
    slot_store(dir, slot)
        .write(text.as_bytes())
        .map_err(|error| anyhow::anyhow!("writing {}: {error}", slot_path_in(dir, slot).display()))
}
