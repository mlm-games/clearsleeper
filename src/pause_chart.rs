use std::cell::Cell;

use bevy_ecs::prelude::*;
use repose_core::ImageHandle;

use crate::sim;
use crate::ui_menu_fmap2 as ui;

/// The ported sea-chart / comparison map (`d_menu_fmap2.cpp`), driven from the pause
/// menu's CHART tab through the same ECS state machine the retail menu uses.
pub struct ChartStage {
    pub draw: Option<ui::ChartDraw>,
    pub image: Cell<Option<ImageHandle>>,
    pub uploaded: Cell<u64>,
    pub revision: u64,
}

impl Default for ChartStage {
    fn default() -> Self {
        Self {
            draw: None,
            image: Cell::new(None),
            uploaded: Cell::new(0),
            revision: 1,
        }
    }
}

fn chart_menu_system(mut env: ui::ChartEnv) {
    ui::chart_menu(&mut env);
}

fn chart_draw_system(env: ui::ChartEnv) -> ui::ChartDraw {
    ui::chart_draw(&env)
}

impl ChartStage {
    pub fn install(world: &mut World) {
        if world.contains_resource::<ui::ChartLayer>() {
            return;
        }
        ui::init(world);
        let Ok(root) = crate::assets::AssetRoot::from_env() else {
            return;
        };
        let mut layer = world.resource_mut::<ui::ChartLayer>();
        ui::load_tables(&mut layer, &root);
        layer.create(&root);
    }

    pub fn step(world: &mut World, pad: ui::ChartPadInput) {
        Self::install(world);
        *world.resource_mut::<ui::ChartPadInput>() = pad;
        let _ = world.run_system_cached(chart_menu_system);
    }

    pub fn draw(world: &mut World) -> Option<ui::ChartDraw> {
        world.run_system_cached(chart_draw_system).ok()
    }

    pub fn render(
        canvas: &mut crate::text::raster::Canvas,
        assets: &crate::panes::Assets,
        draw: &ui::ChartDraw,
    ) {
        for pane in &draw.panes {
            if !pane.visible || pane.w <= 0.0 || pane.h <= 0.0 {
                continue;
            }
            canvas.fill_rect(
                [pane.x, pane.y, pane.x + pane.w, pane.y + pane.h],
                [40, 60, 90, pane.alpha],
            );
            if !pane.text.is_empty() {
                crate::panes::draw_label(
                    canvas,
                    assets,
                    &pane.text,
                    pane.x,
                    pane.y,
                    16.0,
                    pane.alpha as u32 * 0x0101_0101,
                );
            }
        }
    }
}
