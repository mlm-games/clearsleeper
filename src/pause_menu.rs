use repose_core::{Color, Modifier, Sp, UnitExt, View};
use repose_ui::{Center, Column, Row, Text, TextStyle, ViewExt};

use crate::menus::PAUSE_TABS;
use crate::sim::inventory::{Equipped, ItemSelect, NO_ITEM, SLOT_COUNT};

const TITLE_HEX: &str = "#F4EAD5";
const DIM_HEX: &str = "#9FB2C8";
const CURSOR_HEX: &str = "#FFD86B";
const PANEL_HEX: &str = "#0B1522";
const PANEL_ALPHA: u8 = 190;

fn tab(text: String, cursor: bool) -> View {
    let color = if cursor {
        Color::from_hex(CURSOR_HEX)
    } else {
        Color::from_hex(DIM_HEX)
    };
    let marker = if cursor { ">" } else { " " };
    Text(format!("{marker}{text}")).size(Sp(15.0)).color(color)
}

fn label(text: String, cursor: bool, dim: bool) -> View {
    let color = if cursor {
        Color::from_hex(CURSOR_HEX)
    } else if dim {
        Color::from_hex(DIM_HEX)
    } else {
        Color::from_hex(TITLE_HEX)
    };
    let marker = if cursor { "> " } else { "  " };
    Text(format!("{marker}{text}")).size(Sp(22.0)).color(color)
}

fn heading(text: &str) -> View {
    Text(text.to_string())
        .size(Sp(30.0))
        .color(Color::from_hex(CURSOR_HEX))
}

fn item_name(item: u8) -> String {
    match item {
        0x0B => "Bombs".to_string(),
        0x0C => "Bombs x10".to_string(),
        0x0D => "Bombs x20".to_string(),
        0x0E => "Bombs x30".to_string(),
        0x10 => "Arrows".to_string(),
        0x11 => "Arrows x20".to_string(),
        0x12 => "Arrows x30".to_string(),
        0x22 => "Wind Waker".to_string(),
        0x23 => "Picto Box".to_string(),
        0x26 => "Deluxe Picto Box".to_string(),
        0x27 => "Bow".to_string(),
        0x29 => "Iron Boots".to_string(),
        0x2A => "Magic Armor".to_string(),
        0x2D => "Boomerang".to_string(),
        0x2F => "Hookshot".to_string(),
        0x31 => "Bomb Bag".to_string(),
        0x33 => "Skull Hammer".to_string(),
        0x34 => "Deku Leaf".to_string(),
        NO_ITEM => "---".to_string(),
        other => format!("Item {other:#04x}"),
    }
}

/// The in-game pause menu. `d_menu_window.cpp:1437-1580` opens the item page on Start,
/// uses L/R to reach the collection page and B/Start to close; the port presents the
/// item, status, dungeon-map, sea-chart and collection pages as five tabs so the whole
/// menu is reachable from the keyboard as well as the gamepad.
pub fn view(app: &crate::App) -> View {
    let pause = &app.menu.pause;
    let world = &app.sim.world;
    let equipped = world.resource::<Equipped>();
    let select = world.resource::<ItemSelect>();
    let mut children = Vec::new();
    children.push(heading("PAUSE"));
    let mut tabs = Vec::new();
    for (index, name) in PAUSE_TABS.iter().enumerate() {
        tabs.push(tab(name.to_string(), index == pause.tab));
    }
    children.push(Row(Modifier::new().gap(8.dp()).hit_passthrough()).child(tabs));

    match pause.tab {
        0 => {
            children.push(heading("ITEMS"));
            let mut slots: Vec<(usize, u8)> = (0..SLOT_COUNT)
                .filter_map(|slot| {
                    let item = equipped.item(slot);
                    (item != NO_ITEM).then_some((slot, item))
                })
                .collect();
            slots.sort_by_key(|(_, item)| *item);
            if slots.is_empty() {
                children.push(label("No items yet".to_string(), false, true));
            }
            for (row, &(slot, item)) in slots.iter().enumerate() {
                children.push(label(
                    format!("{:<18}", item_name(item)),
                    row == pause.cursor,
                    false,
                ));
            }
            let bindings = select.slots.map(|slot| {
                if slot == NO_ITEM {
                    "---".to_string()
                } else {
                    item_name(equipped.item(usize::from(slot)))
                }
            });
            children.push(
                Text(format!(
                    "X: {:<14} Y: {:<14} Z: {}",
                    bindings[0], bindings[1], bindings[2]
                ))
                .size(Sp(18.0))
                .color(Color::from_hex(DIM_HEX)),
            );
            children.push(label(
                if pause.assigning {
                    format!(
                        "Assigning to {} - press X / Y / Z, B to cancel",
                        ["X", "Y", "Z"][pause.button]
                    )
                } else {
                    "A: select   X/Y/Z: assign   L/R: tabs".to_string()
                },
                pause.assigning,
                false,
            ));
        }
        1 => {
            children.push(heading("QUEST STATUS"));
            let health = world.resource::<crate::sim::health::Health>();
            let magic = world.resource::<crate::sim::health::Magic>();
            let rupees = world.resource::<crate::sim::pickup::Rupees>();
            let clock = world.resource::<crate::sim::clock::Clock>();
            let collect = world.resource::<crate::sim::flags::Collect>();
            let story = world.resource::<crate::sim::flags::StoryBits>();
            children.push(label(
                format!("Hearts   {}/{}", health.units, health.max),
                false,
                false,
            ));
            children.push(label(
                format!("Magic    {}/{}", magic.units, magic.max),
                false,
                false,
            ));
            children.push(label(format!("Rupees   {}", rupees.count), false, false));
            children.push(label(format!("Day      {}", clock.date), false, false));
            let treasures = (0u8..16).filter(|idx| collect.check(*idx) != 0).count();
            children.push(label(format!("Treasures {treasures}"), false, false));
            let story_flags = story.bytes.iter().filter(|byte| **byte != 0).count();
            children.push(label(format!("Story    {story_flags}"), false, false));
        }
        2 => {
            children.push(heading("DUNGEON MAP"));
            let stage = app.stage_name();
            children.push(label(format!("Stage    {stage}"), false, false));
            let progress = app
                .stage_progress
                .values()
                .filter(|entry| entry.dungeon != 0)
                .count();
            children.push(label(format!("Dungeons visited  {progress}"), false, false));
        }
        3 => {
            children.push(heading("SEA CHART"));
            let charts = world.resource::<crate::sim::flags::Charts>();
            children.push(label("Charted islands".to_string(), false, false));
            for index in 0..8 {
                let mark = if charts.has(index) {
                    "found"
                } else if charts.is_open(index) {
                    "open"
                } else {
                    "---"
                };
                children.push(label(
                    format!("{index:>2}: {mark}"),
                    index == pause.cursor,
                    false,
                ));
            }
        }
        _ => {
            children.push(heading("COLLECTION"));
            let collect = world.resource::<crate::sim::flags::Collect>();
            let treasures = (0u8..16).filter(|idx| collect.check(*idx) != 0).count();
            children.push(label(format!("Treasures  {treasures}"), false, false));
            children.push(label(
                "Pictures   A: open Photo Album (P)".to_string(),
                false,
                true,
            ));
        }
    }
    children.push(label(
        "Keys: Esc/Tab pause   I item   N status   M dungeon map   G chart   E sea   C collection"
            .to_string(),
        false,
        true,
    ));
    children.push(label(
        "L/R/O: tab   Arrows/WASD: cursor   Q/Space: choose   X/Y/Z: assign   V/B: close"
            .to_string(),
        false,
        true,
    ));
    Center(Modifier::new().fill_max_size().hit_passthrough()).child(
        Column(
            Modifier::new()
                .gap(10.dp())
                .padding(28.dp())
                .background(Color::from_hex(PANEL_HEX).with_alpha(PANEL_ALPHA))
                .hit_passthrough(),
        )
        .child(children),
    )
}
