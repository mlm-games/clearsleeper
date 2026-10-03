use std::collections::HashSet;

use repose_core::input::PhysicalKey;
use repose_core::{Color, Modifier, Sp, UnitExt, View};
use repose_ui::{Center, Column, Row, Text, TextStyle, ViewExt};

use crate::hud;
use crate::save::{SLOTS, SaveGame};
use crate::sim::health::Health;
use crate::sim::intent::GameOverChoice;

const TITLE_HEX: &str = "#F4EAD5";
const DIM_HEX: &str = "#9FB2C8";
const CURSOR_HEX: &str = "#FFD86B";
const PANEL_HEX: &str = "#0B1522";
const PANEL_ALPHA: u8 = 170;
const SLOT_HEART_SIZE: f32 = 18.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Title,
    FileSelect,
    Playing,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
    Confirm,
    Back,
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SlotInfo {
    Empty,
    File {
        name: String,
        units: u8,
        max: u8,
        tick: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Load(usize),
    New(usize),
    GameOver(GameOverChoice),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Menu {
    pub screen: Screen,
    pub slot: usize,
    pub slots: Vec<SlotInfo>,
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    pub fn new() -> Self {
        Self {
            screen: Screen::Title,
            slot: 0,
            slots: vec![SlotInfo::Empty; SLOTS],
        }
    }

    pub fn step(&mut self, cmds: &[Cmd]) -> Option<Action> {
        for cmd in cmds {
            match (self.screen, cmd) {
                (Screen::Title, Cmd::Confirm) => self.screen = Screen::FileSelect,
                (Screen::FileSelect, Cmd::Up) => self.slot = self.slot.saturating_sub(1),
                (Screen::FileSelect, Cmd::Down) => {
                    self.slot = (self.slot + 1).min(SLOTS - 1);
                }
                (Screen::FileSelect, Cmd::Confirm) => {
                    let slot = self.slot;
                    return Some(match self.slots.get(slot) {
                        Some(SlotInfo::File { .. }) => Action::Load(slot),
                        _ => Action::New(slot),
                    });
                }
                (Screen::FileSelect, Cmd::Back) => self.screen = Screen::Title,
                (Screen::GameOver, Cmd::Up) => self.slot = self.slot.saturating_sub(1),
                (Screen::GameOver, Cmd::Down) => self.slot = self.slot.min(1),
                (Screen::GameOver, Cmd::Confirm) => {
                    let choice = if self.slot == 0 {
                        GameOverChoice::Continue
                    } else {
                        GameOverChoice::Quit
                    };
                    return Some(Action::GameOver(choice));
                }
                _ => {}
            }
        }
        None
    }
}

impl SlotInfo {
    pub fn from_save(save: &SaveGame) -> Self {
        SlotInfo::File {
            name: save.name.clone(),
            units: save.health.units,
            max: save.health.max,
            tick: save.tick,
        }
    }
}

pub fn cmds_from(held: &HashSet<PhysicalKey>, prev: &HashSet<PhysicalKey>) -> Vec<Cmd> {
    let mut cmds = Vec::new();
    let pressed = |key: PhysicalKey| held.contains(&key) && !prev.contains(&key);
    if pressed(PhysicalKey::ArrowUp) || pressed(PhysicalKey::KeyW) {
        cmds.push(Cmd::Up);
    }
    if pressed(PhysicalKey::ArrowDown) || pressed(PhysicalKey::KeyS) {
        cmds.push(Cmd::Down);
    }
    if pressed(PhysicalKey::Space) || pressed(PhysicalKey::Enter) {
        cmds.push(Cmd::Confirm);
    }
    if pressed(PhysicalKey::Escape) {
        cmds.push(Cmd::Back);
    }
    cmds
}

pub fn play_time(tick: u64) -> String {
    let seconds = tick / crate::SIM_HZ as u64;
    format!("{}:{:02}", seconds / 3600, (seconds / 60) % 60)
}

pub fn title_view() -> View {
    Center(Modifier::new().fill_max_size().hit_passthrough()).child(
        Column(Modifier::new().gap(14.dp()).hit_passthrough()).child([
            Text("ClearSleeper")
                .size(Sp(76.0))
                .color(Color::from_hex(TITLE_HEX)),
            Text("Press Space")
                .size(Sp(24.0))
                .color(Color::from_hex(DIM_HEX)),
        ]),
    )
}

pub fn file_select_view(menu: &Menu) -> View {
    let mut children = vec![
        Text("Select a file")
            .size(Sp(28.0))
            .color(Color::from_hex(TITLE_HEX)),
    ];
    for (slot, info) in menu.slots.iter().take(SLOTS).enumerate() {
        children.push(slot_row(slot == menu.slot, info));
    }
    children.push(
        Text("Space: choose    Esc: back")
            .size(Sp(15.0))
            .color(Color::from_hex(DIM_HEX)),
    );
    Center(Modifier::new().fill_max_size().hit_passthrough()).child(
        Column(
            Modifier::new()
                .gap(10.dp())
                .padding(24.dp())
                .background(Color::from_hex(PANEL_HEX).with_alpha(PANEL_ALPHA))
                .hit_passthrough(),
        )
        .child(children),
    )
}

pub fn game_over_view(menu: &Menu) -> View {
    let mut children = vec![
        Text("Game Over")
            .size(Sp(48.0))
            .color(Color::from_hex(TITLE_HEX)),
    ];
    for (slot, label) in [(0, "Continue"), (1, "Return to title")] {
        let cursor = menu.slot == slot;
        let marker = if cursor { "> " } else { "  " };
        let color = if cursor {
            Color::from_hex(CURSOR_HEX)
        } else {
            Color::from_hex(TITLE_HEX)
        };
        children.push(Text(format!("{marker}{label}")).size(Sp(24.0)).color(color));
    }
    children.push(
        Text("Space: choose")
            .size(Sp(15.0))
            .color(Color::from_hex(DIM_HEX)),
    );
    Center(Modifier::new().fill_max_size().hit_passthrough()).child(
        Column(
            Modifier::new()
                .gap(14.dp())
                .padding(28.dp())
                .background(Color::from_hex(PANEL_HEX).with_alpha(PANEL_ALPHA))
                .hit_passthrough(),
        )
        .child(children),
    )
}

fn slot_row(cursor: bool, info: &SlotInfo) -> View {
    let marker = if cursor { "> " } else { "  " };
    let color = if cursor {
        Color::from_hex(CURSOR_HEX)
    } else {
        Color::from_hex(TITLE_HEX)
    };
    let mut parts = Vec::new();
    match info {
        SlotInfo::Empty => {
            parts.push(
                Text(format!("{marker}New file"))
                    .size(Sp(22.0))
                    .color(color),
            );
        }
        SlotInfo::File {
            name,
            units,
            max,
            tick,
        } => {
            parts.push(Text(format!("{marker}{name}")).size(Sp(22.0)).color(color));
            parts.push(hud::hearts_strip(
                Health {
                    units: *units,
                    max: *max,
                },
                SLOT_HEART_SIZE,
            ));
            parts.push(
                Text(play_time(*tick))
                    .size(Sp(16.0))
                    .color(Color::from_hex(DIM_HEX)),
            );
        }
    }
    Row(Modifier::new().gap(14.dp()).hit_passthrough()).child(parts)
}
