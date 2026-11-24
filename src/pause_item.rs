use std::cell::Cell;

use bevy_ecs::prelude::*;
use repose_core::ImageHandle;

use crate::j2d;
use crate::sim;

/// The ported retail item menu (`d_menu_item.cpp`), driven from the pause menu's ITEM tab.
/// The port keeps its state in ECS resources so the same system the retail schedule uses can
/// step it here; its `select_item_writes` are the X/Y/Z assignments the sim consumes.
pub struct ItemMenuStage {
    pub draw: Option<crate::ui_menu_item::WindowDraw>,
    pub image: Cell<Option<ImageHandle>>,
    pub uploaded: Cell<u64>,
    pub revision: u64,
}

impl Default for ItemMenuStage {
    fn default() -> Self {
        Self {
            draw: None,
            image: Cell::new(None),
            uploaded: Cell::new(0),
            revision: 1,
        }
    }
}

impl ItemMenuStage {
    pub fn install(world: &mut World) {
        use crate::ui_menu_item as ui;

        let Ok(root) = crate::assets::AssetRoot::from_env() else {
            return;
        };
        let Ok(archive) = root.read_archive("files/res/Msg/itemres.arc") else {
            return;
        };
        let Ok(bytes) = archive.read(ui::SCREEN_LABEL) else {
            return;
        };
        let mut screens = j2d::Screens::default();
        if screens.load(ui::SCREEN_LABEL, bytes).is_none() {
            return;
        }
        world.insert_resource(screens);
        world.insert_resource(crate::assets::load_msg_archives());
        world.insert_resource(ui::MenuWindow::new());
        world.insert_resource(ui::PadState::default());
        world.insert_resource(ui::ItemRecord::default());
        world.insert_resource(ui::MenuEnv::default());
        world.insert_resource(ui::MenuTuning::default());
        world.insert_resource(ui::ItemMoveBridge::default());
        world.insert_resource(ui::MenuOutput::default());
    }

    /// One frame of the ported state machine. Returns the button→slot writes it produced.
    pub fn step(world: &mut World, pad: crate::ui_menu_item::PadState) -> Vec<(u8, u8)> {
        use crate::ui_menu_item as ui;
        if !world.contains_resource::<ui::MenuWindow>() {
            Self::install(world);
            if !world.contains_resource::<ui::MenuWindow>() {
                return Vec::new();
            }
        }
        *world.resource_mut::<ui::PadState>() = pad;
        if world.run_system_cached(ui::menu_window_step).is_err() {
            return Vec::new();
        }
        std::mem::take(&mut world.resource_mut::<ui::MenuOutput>().select_item_writes)
    }

    pub fn draw_window(world: &mut World) -> Option<crate::ui_menu_item::WindowDraw> {
        world
            .resource_mut::<crate::ui_menu_item::MenuWindow>()
            .map(|window| window.draw())
    }

    /// Rasterise the port's quad/text descriptors onto the shared 640x480 canvas.
    pub fn render(
        canvas: &mut crate::text::raster::Canvas,
        assets: &crate::panes::Assets,
        draw: &crate::ui_menu_item::WindowDraw,
    ) {
        use crate::ui_menu_item as ui;
        if !draw.visible {
            return;
        }
        let Some(item) = draw.item.as_ref() else {
            return;
        };
        for quad in &item.quads {
            if !quad.visible {
                continue;
            }
            let [x, y, w, h] = quad.bounds;
            if w <= 0.0 || h <= 0.0 {
                continue;
            }
            let alpha = quad.alpha;
            canvas.fill_rect(
                [x, y, x + w, y + h],
                [
                    (quad.white >> 16) as u8,
                    (quad.white >> 8) as u8,
                    quad.white as u8,
                    alpha,
                ],
            );
        }
        for text in &item.texts {
            if text.text.is_empty() {
                continue;
            }
            let [x, y, _, h] = text.bounds;
            crate::panes::draw_label(
                canvas,
                assets,
                &text.text,
                x,
                y,
                text.font_size.1.max(12.0).min(h.max(12.0)),
                text.alpha as u32 * 0x0101_0101,
            );
        }
    }
}
