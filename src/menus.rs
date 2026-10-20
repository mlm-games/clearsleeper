use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;

use repose_core::input::PhysicalKey;
use repose_core::{Color, Modifier, Sp, UnitExt, View};
use repose_material::material3::{Button, ButtonConfig, TextButton};
use repose_ui::{Center, Column, Image, ImageExt, Row, Spacer, Text, TextStyle, ViewExt};

use crate::config::Settings;
use crate::save::{SLOTS, SaveGame};

const TITLE_HEX: &str = "#F4EAD5";
const DIM_HEX: &str = "#9FB2C8";
const CURSOR_HEX: &str = "#FFD86B";
const PANEL_HEX: &str = "#0B1522";
const PANEL_ALPHA: u8 = 170;

pub const TITLE_ITEMS: [&str; 3] = ["Play", "Settings", "Quit"];
pub const QUIT_ITEMS: [&str; 2] = ["Cancel", "Quit"];
pub const SETTINGS_ITEMS: [&str; 4] = ["Attention", "Ruby", "Vibration", "Sound"];

const SETTING_VALUES: [&[&str]; 4] = [
    &["Off", "On"],
    &["Off", "On"],
    &["Off", "On"],
    &["Mono", "Stereo", "Surround"],
];

const SETTING_LABEL_WIDTH: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Title,
    FileSelect,
    Playing,
    GameOver,
    Settings,
    QuitConfirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
    Confirm,
    Back,
    Up,
    Down,
    Left,
    Right,
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
    Quit,
}

#[derive(Debug)]
pub struct Menu {
    pub screen: Screen,
    pub slot: usize,
    pub slots: Vec<SlotInfo>,
    pub title_index: usize,
    pub quit_index: usize,
    pub settings_index: usize,
    pub settings: Settings,
    pub fs: crate::ui_file_select::FileSelect,
    pub card: crate::ui_menu_save::MemCard,
    pub shot: std::cell::Cell<Option<repose_core::ImageHandle>>,
    pub uploaded: std::cell::Cell<u64>,
    pub revision: std::cell::Cell<u64>,
    pub title_click: Rc<Cell<Option<usize>>>,
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
            title_index: 0,
            quit_index: 0,
            settings_index: 0,
            settings: Settings::defaults(),
            fs: crate::ui_file_select::FileSelect::default(),
            card: crate::ui_menu_save::MemCard::default(),
            shot: std::cell::Cell::new(None),
            uploaded: std::cell::Cell::new(u64::MAX),
            revision: std::cell::Cell::new(0),
            title_click: Rc::new(Cell::new(None)),
        }
    }

    /// Binds the J2D screen, reads the card and pushes it into the panes so the
    /// file select compositor has something to draw.
    pub fn open_file_select(&mut self) {
        self.fs.delete();
        let Some(assets) = crate::panes::load() else {
            return;
        };
        self.fs.create(assets.screen.clone());
        for slot in 0..SLOTS {
            self.card.slots[slot] = crate::save::read_slot_in(&self.card.dir, slot);
        }
        self.fs.set_save_data(&self.card);
    }

    /// Follows the menu cursor so the highlighted slot matches the drawn pane,
    /// and drives the slide-in transition that `create` left mid-flight.
    pub fn sync_file_select_cursor(&mut self) {
        self.revision.set(self.revision.get().wrapping_add(1));
        self.fs.select_num = self.slot as u8;
        self.fs.open();
    }

    pub fn has_pending_click(&self) -> bool {
        self.title_click.get().is_some()
    }

    pub fn step(&mut self, cmds: &[Cmd]) -> Option<Action> {
        if self.screen == Screen::Title
            && let Some(index) = self.title_click.take()
        {
            self.title_index = index.min(TITLE_ITEMS.len().saturating_sub(1));
            return self.confirm_title();
        }
        for cmd in cmds {
            match (self.screen, cmd) {
                (Screen::Title, Cmd::Up) => self.title_index = self.title_index.saturating_sub(1),
                (Screen::Title, Cmd::Down) => {
                    let last = TITLE_ITEMS.len().saturating_sub(1);
                    self.title_index = (self.title_index + 1).min(last);
                }
                (Screen::Title, Cmd::Confirm) => return self.confirm_title(),
                (Screen::Settings, Cmd::Up) => {
                    self.settings_index = self.settings_index.saturating_sub(1);
                }
                (Screen::Settings, Cmd::Down) => {
                    let last = Settings::row_count().saturating_sub(1);
                    self.settings_index = (self.settings_index + 1).min(last);
                }
                (Screen::Settings, Cmd::Left) => self.shift_setting(-1),
                (Screen::Settings, Cmd::Right) => self.shift_setting(1),
                (Screen::Settings, Cmd::Back) => {
                    self.settings.save();
                    self.screen = Screen::Title;
                }
                (Screen::QuitConfirm, Cmd::Up) => {
                    self.quit_index = self.quit_index.saturating_sub(1);
                }
                (Screen::QuitConfirm, Cmd::Down) => {
                    let last = QUIT_ITEMS.len().saturating_sub(1);
                    self.quit_index = (self.quit_index + 1).min(last);
                }
                (Screen::QuitConfirm, Cmd::Confirm) => {
                    if self.quit_index == 1 {
                        return Some(Action::Quit);
                    }
                    self.screen = Screen::Title;
                }
                (Screen::QuitConfirm, Cmd::Back) => self.screen = Screen::Title,
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
                _ => {}
            }
        }
        None
    }

    fn confirm_title(&mut self) -> Option<Action> {
        match self.title_index {
            0 => self.screen = Screen::FileSelect,
            1 => {
                self.settings = Settings::load();
                self.settings_index = 0;
                self.screen = Screen::Settings;
            }
            2 => {
                self.quit_index = 0;
                self.screen = Screen::QuitConfirm;
            }
            _ => {}
        }
        None
    }

    fn shift_setting(&mut self, delta: i16) {
        let row = self.settings_index;
        let Some((value, range, _)) = self.settings.row(row) else {
            return;
        };
        let shifted = if delta < 0 {
            value.saturating_sub(1)
        } else {
            value.saturating_add(1).min(range)
        };
        if shifted == value {
            return;
        }
        self.settings.set_row(row, shifted);
        self.settings.save();
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
    if pressed(PhysicalKey::ArrowLeft) || pressed(PhysicalKey::KeyA) {
        cmds.push(Cmd::Left);
    }
    if pressed(PhysicalKey::ArrowRight) || pressed(PhysicalKey::KeyD) {
        cmds.push(Cmd::Right);
    }
    if pressed(PhysicalKey::Space) || pressed(PhysicalKey::Enter) {
        cmds.push(Cmd::Confirm);
    }
    if pressed(PhysicalKey::Escape) {
        cmds.push(Cmd::Back);
    }
    cmds
}

pub fn title_view(menu: &Menu) -> View {
    let mut items = Vec::with_capacity(TITLE_ITEMS.len());
    for (index, label) in TITLE_ITEMS.iter().enumerate() {
        let target = menu.title_click.clone();
        items.push(title_button(
            menu.title_index == index,
            move || target.set(Some(index)),
            label,
        ));
    }
    let mut children = vec![
        Text("ClearSleeper")
            .size(Sp(56.0))
            .color(Color::from_hex(TITLE_HEX)),
    ];
    children.push(Column(Modifier::new().gap(12.0.dp())).child(items));
    children.push(hint("Up/Down: move    Space: choose"));
    children.push(
        Text("A disc image is required: pass --disc <image> or put disc.iso beside the app")
            .size(Sp(14.0))
            .color(Color::from_hex(DIM_HEX)),
    );
    children.push(
        Text(format!("Version {}", env!("CARGO_PKG_VERSION")))
            .size(Sp(14.0))
            .color(Color::from_hex(DIM_HEX)),
    );
    overlay(children)
}

fn title_button(selected: bool, on_click: impl Fn() + 'static, label: &str) -> View {
    let config = ButtonConfig {
        modifier: Modifier::new().width(300.0.dp()),
        ..Default::default()
    };
    let content = || Text(label.to_owned()).size(Sp(20.0));
    if selected {
        Button(Modifier::new(), on_click, config, content)
    } else {
        TextButton(Modifier::new(), on_click, config, content)
    }
}

pub fn settings_view(menu: &Menu) -> View {
    let mut children = vec![
        Text("Settings")
            .size(Sp(40.0))
            .color(Color::from_hex(TITLE_HEX)),
    ];
    for row in 0..Settings::row_count() {
        children.push(settings_row(menu, row));
    }
    children.push(hint("Up/Down: move    Left/Right: change    Esc: back"));
    panel(children)
}

pub fn quit_confirm_view(menu: &Menu) -> View {
    let mut children = vec![
        Text("Quit?")
            .size(Sp(40.0))
            .color(Color::from_hex(TITLE_HEX)),
    ];
    for (index, label) in QUIT_ITEMS.iter().enumerate() {
        children.push(menu_item(index == menu.quit_index, *label));
    }
    children.push(hint("Up/Down: move    Space: choose    Esc: cancel"));
    panel(children)
}

pub fn file_select_view(menu: &Menu, ctx: &repose_core::RenderContext) -> View {
    let Some(assets) = crate::panes::load() else {
        return Spacer();
    };
    let handle = match menu.shot.get() {
        Some(handle) => handle,
        None => {
            let handle = ctx.alloc_image_handle();
            menu.shot.set(Some(handle));
            handle
        }
    };
    if menu.uploaded.get() != menu.revision.get() {
        let mut canvas = crate::text::raster::Canvas::new(640, 480);
        crate::panes::draw_file_select(&mut canvas, &menu.fs.panes, assets);
        ctx.set_image_rgba8(handle, 640, 480, canvas.pixels, true);
        menu.uploaded.set(menu.revision.get());
    }
    Image(Modifier::new().fill_max_size(), handle).image_fit(repose_core::ImageFit::Contain)
}

fn panel(children: Vec<View>) -> View {
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

fn hint(text: &str) -> View {
    Text(text).size(Sp(15.0)).color(Color::from_hex(DIM_HEX))
}

fn overlay(children: Vec<View>) -> View {
    Center(Modifier::new().fill_max_size().hit_passthrough()).child(
        Column(
            Modifier::new()
                .gap(14.dp())
                .padding(28.dp())
                .hit_passthrough(),
        )
        .child(children),
    )
}

fn menu_item(cursor: bool, label: &str) -> View {
    let marker = if cursor { "> " } else { "  " };
    let color = if cursor {
        Color::from_hex(CURSOR_HEX)
    } else {
        Color::from_hex(TITLE_HEX)
    };
    Text(format!("{marker}{label}")).size(Sp(24.0)).color(color)
}

fn settings_row(menu: &Menu, row: usize) -> View {
    let name = SETTINGS_ITEMS.get(row).copied().unwrap_or_default();
    let (value, _, _) = menu.settings.row(row).unwrap_or((0, 0, 0));
    let active = menu.settings_index == row;
    let marker = if active { "> " } else { "  " };
    let blank = " ".repeat(SETTING_LABEL_WIDTH.saturating_sub(name.len()));
    let shown = setting_value_text(row, value);
    let cell = if active {
        format!("< {shown} >")
    } else {
        shown
    };
    let color = if active {
        Color::from_hex(CURSOR_HEX)
    } else {
        Color::from_hex(TITLE_HEX)
    };
    Text(format!("{marker}{name}{blank}{cell}"))
        .size(Sp(24.0))
        .color(color)
}

fn setting_value_text(row: usize, value: u8) -> String {
    SETTING_VALUES
        .get(row)
        .and_then(|names| names.get(usize::from(value)))
        .map_or_else(|| value.to_string(), |name| (*name).to_string())
}
