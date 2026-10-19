use repame_save::{SaveError, SaveFile};

use crate::save::Legacy;
use crate::ui_menu_option::{
    ATTENTION_DEFAULT, OptionRow, RUBY_DEFAULT, SOUND_DEFAULT_STEREO, VIBRATION_DEFAULT,
};

/// The game's own options document. `repame-save`'s `SettingsStore` carries the
/// stack's `Settings` (audio gains, display flags, accessibility), which has no
/// field for the four retail options this game binds, and the crate's own rule
/// for a game-specific preference is the game's document, not the shell's. This
/// is that document: the same envelope, store, and `AppId` the `SettingsStore`
/// wraps, under a name that leaves the stack's `settings.ron` free.
const CONFIG_FILE: &str = "options.ron";

/// The raw JSON file the same options were written to before this document.
const LEGACY_CONFIG_FILE: &str = "settings.json";

const CONFIG_SCHEMA: u32 = 1;

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
        let file = config_file();
        match file.load::<Settings>() {
            Ok(Some(settings)) if settings.in_range() => settings,
            Ok(Some(_)) => {
                log::warn!(
                    "clearsleeper: {} holds a value outside its option range; option defaults restored",
                    file.path().display()
                );
                Self::defaults()
            }
            Ok(None) => Self::adopt_legacy(),
            // A corrupt document left no options behind: the legacy copy is
            // the only one left, so it is adopted rather than reset.
            Err(error @ SaveError::Corrupt { .. }) => {
                log::warn!("clearsleeper: {error}");
                Self::adopt_legacy()
            }
            Err(error) => {
                log::warn!("clearsleeper: {error}");
                Self::defaults()
            }
        }
    }

    /// Carry the pre-`repame-save` `settings.json` across: raw JSON under the
    /// live directory first, then the platform directories used before the
    /// data-dir move. The legacy file is left in place.
    fn adopt_legacy() -> Self {
        let adopted = crate::save::adopt_legacy::<Settings>(
            &config_file(),
            LEGACY_CONFIG_FILE,
            &crate::save::slots_dir(),
        );
        let Legacy::Adopted(settings) = adopted else {
            return Self::defaults();
        };
        if settings.in_range() {
            settings
        } else {
            log::warn!(
                "clearsleeper: the adopted options hold a value outside its option range; option defaults restored"
            );
            Self::defaults()
        }
    }

    pub fn save(&self) {
        let file = config_file();
        if let Err(error) = file.save(self) {
            log::warn!(
                "clearsleeper: {} is unwritable ({error})",
                file.path().display()
            );
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

fn config_file() -> SaveFile {
    SaveFile::in_dir(crate::save::slots_dir(), CONFIG_FILE, CONFIG_SCHEMA)
}
