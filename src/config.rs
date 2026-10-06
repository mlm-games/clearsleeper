use std::path::{Path, PathBuf};

use crate::ui_menu_option::{
    ATTENTION_DEFAULT, OptionRow, RUBY_DEFAULT, SOUND_DEFAULT_STEREO, VIBRATION_DEFAULT,
};

const CONFIG_FILE: &str = "settings.json";

const ROWS: [OptionRow; 4] = [
    OptionRow::Attention,
    OptionRow::Ruby,
    OptionRow::Vibration,
    OptionRow::Sound,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Settings {
    pub attention: u8,
    pub ruby: u8,
    pub vibration: u8,
    pub sound: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self::defaults()
    }
}

impl Settings {
    pub fn defaults() -> Self {
        Self {
            attention: ATTENTION_DEFAULT,
            ruby: RUBY_DEFAULT,
            vibration: VIBRATION_DEFAULT,
            sound: SOUND_DEFAULT_STEREO,
        }
    }

    pub fn load() -> Self {
        let path = config_path();
        match read_config(&path) {
            Some(settings) if settings.in_range() => settings,
            Some(_) => {
                log::warn!(
                    "clearsleeper: {} holds a value outside its option range; option defaults restored",
                    path.display()
                );
                Self::defaults()
            }
            None => Self::defaults(),
        }
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(parent) = path.parent() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                log::warn!(
                    "clearsleeper: {} is not creatable ({error}); option defaults stay unwritten",
                    parent.display()
                );
                return;
            }
        }
        let text = match serde_json::to_string_pretty(self) {
            Ok(text) => text,
            Err(error) => {
                log::warn!("clearsleeper: option defaults do not serialise ({error})");
                return;
            }
        };
        if let Err(error) = std::fs::write(&path, text) {
            log::warn!("clearsleeper: {} is unwritable ({error})", path.display());
        }
    }

    pub fn row_count() -> usize {
        ROWS.len()
    }

    pub fn row(&self, i: usize) -> Option<(u8, u8, u8)> {
        let row = ROWS.get(i).copied()?;
        Some((self.value_of(row), row.range(), row.default()))
    }

    pub fn set_row(&mut self, i: usize, value: u8) {
        let Some(row) = ROWS.get(i).copied() else {
            return;
        };
        *self.value_of_mut(row) = value;
    }

    pub fn clamp_row(&mut self, i: usize) {
        let Some(row) = ROWS.get(i).copied() else {
            return;
        };
        let limit = row.range();
        let slot = self.value_of_mut(row);
        *slot = (*slot).min(limit);
    }

    fn value_of(&self, row: OptionRow) -> u8 {
        match row {
            OptionRow::Attention => self.attention,
            OptionRow::Ruby => self.ruby,
            OptionRow::Vibration => self.vibration,
            OptionRow::Sound => self.sound,
        }
    }

    fn value_of_mut(&mut self, row: OptionRow) -> &mut u8 {
        match row {
            OptionRow::Attention => &mut self.attention,
            OptionRow::Ruby => &mut self.ruby,
            OptionRow::Vibration => &mut self.vibration,
            OptionRow::Sound => &mut self.sound,
        }
    }

    fn in_range(&self) -> bool {
        ROWS.iter().all(|row| self.value_of(*row) <= row.range())
    }
}

fn config_path() -> PathBuf {
    crate::save::slots_dir().join(CONFIG_FILE)
}

fn read_config(path: &Path) -> Option<Settings> {
    match std::fs::read_to_string(path) {
        Ok(text) => match serde_json::from_str::<Settings>(&text) {
            Ok(settings) => Some(settings),
            Err(error) => {
                log::warn!("clearsleeper: {} is malformed ({error})", path.display());
                None
            }
        },
        Err(error) => {
            if error.kind() != std::io::ErrorKind::NotFound {
                log::warn!("clearsleeper: {} is unreadable ({error})", path.display());
            }
            None
        }
    }
}
