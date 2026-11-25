//! Native start UI, following Dusklight's prelaunch screen (`dusklight/src/dusk/ui/prelaunch.cpp`):
//! a menu column, a disc-status panel and a version line, built from repose views rather than
//! DOM panes. Shown by the pre-boot shell before a disc is installed; the retail title's logo
//! panes stay on the in-game `Screen::Title`.

use repose_core::{Color, Modifier, Sp, UnitExt, View};
use repose_material::material3::{Button, ButtonConfig, TextButton};
use repose_ui::{Center, Column, Row, Text, TextStyle, ViewExt};

use crate::menus::Menu;

const TITLE_HEX: &str = "#F4EAD5";
const DIM_HEX: &str = "#9FB2C8";
const CURSOR_HEX: &str = "#FFD86B";
const ACCENT_HEX: &str = "#86AD81";

fn menu_button(selected: bool, label: &str, on_click: impl Fn() + 'static) -> View {
    let config = ButtonConfig {
        modifier: Modifier::new().width(280.dp()),
        ..Default::default()
    };
    let content = || Text(label.to_owned()).size(Sp(22.0));
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
    #[cfg(target_arch = "wasm32")]
    let labels: &[&str] = &["Load Disc", "Settings", "Quit"];
    #[cfg(not(target_arch = "wasm32"))]
    let labels: &[&str] = &crate::menus::TITLE_ITEMS;

    let mut buttons = Vec::with_capacity(labels.len());
    for (index, label) in labels.iter().enumerate() {
        let click = menu.title_click.clone();
        let on_click = move || {
            #[cfg(target_arch = "wasm32")]
            {
                if index == 0 {
                    crate::web::request_disc_file();
                    return;
                }
            }
            click.set(Some(index));
        };
        buttons.push(menu_button(menu.title_index == index, label, on_click));
    }
    #[cfg(target_arch = "wasm32")]
    buttons.push(menu_button(false, "Load Folder", || {
        crate::web::request_disc_folder()
    }));

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

    let mut children = vec![
        Text("The Legend of Zelda".to_string())
            .size(Sp(20.0))
            .color(Color::from_hex(ACCENT_HEX)),
        Text("The Wind Waker".to_string())
            .size(Sp(48.0))
            .color(Color::from_hex(TITLE_HEX)),
        Column(Modifier::new().gap(10.dp())).child(buttons),
        status_line("DISC", &disc),
        status_line("VERSION", env!("CARGO_PKG_VERSION")),
    ];

    #[cfg(target_arch = "wasm32")]
    {
        let page_status = crate::web::disc_status();
        if !page_status.is_empty() {
            children.push(
                Text(page_status)
                    .size(Sp(16.0))
                    .color(Color::from_hex(CURSOR_HEX)),
            );
        }
    }
    children.push(
        Text("Up/Down or stick: move   A/Space: choose   B/Esc: back".to_string())
            .size(Sp(14.0))
            .color(Color::from_hex(DIM_HEX)),
    );
    children.push(
        Text("Touch: D-pad + A/B/Start   Gamepad: stick + face buttons".to_string())
            .size(Sp(14.0))
            .color(Color::from_hex(DIM_HEX)),
    );

    Center(Modifier::new().fill_max_size().hit_passthrough()).child(
        Column(
            Modifier::new()
                .gap(18.dp())
                .padding(28.dp())
                .hit_passthrough(),
        )
        .child(children),
    )
}
