use std::cell::Cell;

use repose_core::ImageHandle;

use crate::sim::flags;
use crate::ui_menu_fmap as ui;

/// Chart points come from the chart bits the player owns (tww d_menu_fmap.cpp:894-989).
struct SimChart<'a> {
    charts: &'a flags::Charts,
}

impl ui::ChartTable for SimChart<'_> {
    fn num(&self) -> usize {
        49
    }

    fn at(&self, index: usize) -> ui::ChartPoint {
        ui::ChartPoint {
            grid_no: (index % 7) as i8,
            collect_map_no: index as i8,
            cmap_no: index as i8,
            map_type: i8::from(self.charts.has(index)),
            ..Default::default()
        }
    }
}

/// Island marks come from the save's arrival bits (tww d_menu_fmap.cpp:732-754),
/// laid out on the chart's 7x7 grid, with the arrived flag in field_0x6.
struct SimMarks<'a> {
    arrived: &'a flags::Arrivals,
}

impl ui::IslandMarkTable for SimMarks<'_> {
    fn count(&self) -> usize {
        self.arrived.bytes().len()
    }

    fn at(&self, index: usize) -> ui::IslandMark {
        ui::IslandMark {
            sector_x: (index % 7) as i8,
            sector_y: (index / 7) as i8,
            field_0x2: 0,
            field_0x4: 0,
            field_0x6: u8::from(self.arrived.is_arrived(index)),
        }
    }
}

struct NoTextures;
impl ui::TextureSource for NoTextures {
    fn has(&self, _name: &str) -> bool {
        false
    }
}

struct NoText;
impl ui::TextResolver for NoText {
    fn message(&self, _msg_no: u32) -> String {
        String::new()
    }
    fn select_message(&self, _msg_no: u32) -> (String, String, String, String) {
        Default::default()
    }
}

/// The ported sea chart (`d_menu_fmap.cpp`), driven from the pause menu's SEA tab.
pub struct SeaMapStage {
    pub screen: ui::SeaMapScreen,
    pub out: ui::MapOutput,
    pub created: bool,
    pub draw: Option<ui::MapDraw>,
    pub image: Cell<Option<ImageHandle>>,
    pub uploaded: Cell<u64>,
    pub revision: u64,
}

impl Default for SeaMapStage {
    fn default() -> Self {
        Self {
            screen: ui::SeaMapScreen::new(),
            out: ui::MapOutput::default(),
            created: false,
            draw: None,
            image: Cell::new(None),
            uploaded: Cell::new(0),
            revision: 1,
        }
    }
}

impl SeaMapStage {
    pub fn step(&mut self, world: &mut bevy_ecs::prelude::World, pad: ui::PadInput) {
        let Ok(root) = crate::assets::AssetRoot::from_env() else {
            return;
        };
        let Ok(screen_bytes) = root
            .read_archive(ui::SCREEN_ARCHIVE)
            .and_then(|archive| archive.read(ui::SCREEN_LABEL).map(|b| b.to_vec()))
        else {
            return;
        };
        let Some(blo) = crate::j2d::parse(&screen_bytes) else {
            return;
        };
        let arrivals = world.resource::<flags::Arrivals>();
        let charts = world.resource::<flags::Charts>();
        // Chart points and island marks come from the save's chart/arrival bits
        // (tww d_menu_fmap.cpp:732-754, 894-989).
        let (chart, marks, textures, text) = (
            SimChart { charts },
            SimMarks { arrivals },
            NoTextures,
            NoText,
        );
        let events = world.resource::<flags::EventFlags>();
        let regs = world.resource::<flags::StoryBits>();
        let collect = world.resource::<flags::Collect>();
        let mem = world.resource::<flags::MemBits>();
        let save_view = ui::SaveView::new(arrivals, charts, events, regs, collect, mem);
        let mut out = std::mem::take(&mut self.out);
        let mut ctx = ui::MapContext {
            flags: save_view,
            text: &text,
            marks: &marks,
            textures: &textures,
            env: ui::MapEnv::default(),
            pad,
            out: &mut out,
        };
        if !self.created {
            self.screen.create(&blo, &mut ctx);
            self.created = true;
        }
        self.screen.move_step(&mut ctx);
        self.out = out;
        self.draw = Some(self.screen.draw());
    }

    pub fn render(
        canvas: &mut crate::text::raster::Canvas,
        assets: &crate::panes::Assets,
        draw: &ui::MapDraw,
    ) {
        for pane in &draw.panes {
            if !pane.visible {
                continue;
            }
            let [x, y] = [pane.pos_top_left.x, pane.pos_top_left.y];
            let (w, h) = (pane.size.x, pane.size.y);
            if w <= 0.0 || h <= 0.0 {
                continue;
            }
            canvas.fill_rect(
                [x, y, x + w, y + h],
                [
                    (pane.white >> 16) as u8,
                    (pane.white >> 8) as u8,
                    pane.white as u8,
                    pane.alpha,
                ],
            );
            if !pane.text.is_empty() {
                crate::panes::draw_label(
                    canvas,
                    assets,
                    &pane.text,
                    x,
                    y,
                    pane.font_size_y.max(12.0),
                    pane.alpha as u32 * 0x0101_0101,
                );
            }
        }
    }
}
