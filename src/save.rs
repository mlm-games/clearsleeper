use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::sim::area::Area;
use crate::sim::frame::CurrentFrame;
use crate::sim::health::Health;
use crate::sim::intent::CameraYaw;
use crate::sim::player::Player;
use crate::App;

pub const SAVE_VERSION: u32 = 1;
pub const DEFAULT_NAME: &str = "Link";
pub const SLOTS: usize = 3;

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
            speed: saved.speed,
            speed_y: saved.speed_y,
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
        let player = *query
            .iter(world)
            .next()
            .expect("player exists at boot");
        let health = *world.resource::<Health>();
        SaveGame {
            version: SAVE_VERSION,
            name: self.name.clone(),
            tick,
            camera_yaw,
            player: (&player).into(),
            area: (&area).into(),
            health: (&health).into(),
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
        let world = &mut self.sim.world;
        *world.resource_mut::<CurrentFrame>() = CurrentFrame(save.tick);
        world.resource_mut::<CameraYaw>().0 = save.camera_yaw;
        *world.resource_mut::<Area>() = save.area.clone().into();
        *world.resource_mut::<Health>() = save.health.clone().into();
        let mut query = world.query::<&mut Player>();
        let mut player = query
            .iter_mut(world)
            .next()
            .expect("player exists at boot");
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

pub fn write_save(path: &Path, save: &SaveGame) -> anyhow::Result<()> {
    let text = serde_json::to_string(save)?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, text).with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(&tmp, path).with_context(|| format!("committing {}", path.display()))?;
    Ok(())
}

pub fn read_save(path: &Path) -> anyhow::Result<SaveGame> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(serde_json::from_str(&text)?)
}

pub fn slots_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CS_SAVE_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    let mut dir = windows_appdata()
        .or_else(|| non_empty_env("XDG_DATA_HOME"))
        .or_else(|| non_empty_env("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .unwrap_or_else(std::env::temp_dir);
    dir.push("clearsleeper");
    dir
}

fn non_empty_env(key: &str) -> Option<PathBuf> {
    std::env::var(key)
        .ok()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(target_os = "windows")]
fn windows_appdata() -> Option<PathBuf> {
    non_empty_env("APPDATA")
}

#[cfg(not(target_os = "windows"))]
fn windows_appdata() -> Option<PathBuf> {
    None
}

pub fn slot_path_in(dir: &Path, slot: usize) -> PathBuf {
    dir.join(format!("slot{slot}.json"))
}

pub fn read_slot_in(dir: &Path, slot: usize) -> Option<SaveGame> {
    read_save(&slot_path_in(dir, slot)).ok()
}

pub fn write_slot_in(dir: &Path, slot: usize, save: &SaveGame) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    write_save(&slot_path_in(dir, slot), save)
}
