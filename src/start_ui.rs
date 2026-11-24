//! Native start UI, following Dusklight's prelaunch screen (`dusklight/src/dusk/ui/prelaunch.cpp`):
//! a menu column, a disc-status panel and a version line, built from repose views rather than
//! composited BLO panes. Shown by the pre-boot shell before a disc is installed; the retail
//! title's logo panes stay on the in-game `Screen::Title`.

use repose_core::{Color, Modifier, Sp, UnitExt, View};
use repose_material::material3::{Button, ButtonConfig, TextButton};
use repose_ui::{Center, Column, Row, Text, ViewExt};

use crate::menus::{Menu, TITLE_ITEMS};

const TITLE_HEX: &str = "#F4EAD5";
const DIM_HEX: &str = "#9FB2C8";
const CURSOR_HEX: &str = "#FFD86B";
const ACCENT_HEX: &str = "#86AD81";

fn menu_button(
    selected: bool,
    label: &str,
    index: usize,
    click: std::rc::Rc<std::cell::Cell<Option<usize>>>,
) -> View {
    let config = ButtonConfig {
        modifier: Modifier::new().width(280.dp()),
        ..Default::default()
    };
    let on_click = move || click.set(Some(index));
    let content =
        || Text(format!("{}  {}", if selected { "▶" } else { " " }, label)).size(Sp(22.0));
    if selected {
        Button(Modifier::new(), on_click, config, content)
    } else {
        TextButton(Modifier::new(), on_click, config, content)
    }
}

fn status_line(label: &str, value: &str) -> View {
    Row(Modifier::new().gap(8.dp())).child([
        Text(label.to_string())
            .size(Sp(14.0))
            .color(Color::from_hex(DIM_HEX)),
        Text(value.to_string())
            .size(Sp(14.0))
            .color(Color::from_hex(TITLE_HEX)),
    ])
}

/// The native prelaunch screen the shell shows before a disc is installed.
pub fn view(menu: &Menu) -> View {
    let mut buttons = Vec::with_capacity(TITLE_ITEMS.len());
    for (index, label) in TITLE_ITEMS.iter().enumerate() {
        buttons.push(menu_button(
            menu.title_index == index,
            label,
            index,
            menu.title_click.clone(),
        ));
    }

    let disc = match crate::assets::installed_disc_summary() {
        Some(summary) => match summary.title {
            Some(title) => format!("{} · {title}", summary.game_id),
            None => summary.game_id,
        },
        None => match crate::assets::AssetRoot::from_env() {
            Ok(_) => "development tree".to_string(),
            Err(_) => "no disc".to_string(),
        },
    };

    Center(Modifier::new().fill_max_size().hit_passthrough()).child(
        Column(
            Modifier::new()
                .gap(18.dp())
                .padding(28.dp())
                .hit_passthrough(),
        )
        .child([
            Text("The Legend of Zelda".to_string())
                .size(Sp(20.0))
                .color(Color::from_hex(ACCENT_HEX)),
            Text("The Wind Waker".to_string())
                .size(Sp(48.0))
                .color(Color::from_hex(TITLE_HEX)),
            Column(Modifier::new().gap(10.dp())).child(buttons),
            status_line("DISC", &disc),
            status_line("VERSION", env!("CARGO_PKG_VERSION")),
            Text(
                "Pass --disc <image>, put disc.iso beside the app, or drop the image here"
                    .to_string(),
            )
            .size(Sp(14.0))
            .color(Color::from_hex(DIM_HEX)),
            Text("Up/Down or stick: move   A/Space: choose   B/Esc: back".to_string())
                .size(Sp(14.0))
                .color(Color::from_hex(DIM_HEX)),
            Text("Touch: D-pad + A/B/Start   Gamepad: stick + face buttons".to_string())
                .size(Sp(14.0))
                .color(Color::from_hex(DIM_HEX)),
        ]),
    )
}
