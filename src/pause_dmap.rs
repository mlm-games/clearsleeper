use std::cell::Cell;

use repose_core::ImageHandle;

use crate::text::font::FontBank;
use crate::text::msgdata::MsgLenState;
use crate::text::msglayout::MsgLayoutState;
use crate::ui_menu_dmap as ui;

/// A stage with no floor diagram. The port has no `StageFloors` implementation of its own;
/// the dungeon map still draws its rooms and markers with a single flat floor.
struct NoFloors;

impl ui::StageFloors for NoFloors {
    fn floor_no(&self, _y: f32) -> u8 {
        0
    }

    fn floor_no_for_room(&self, _room: i32, _y: f32) -> u8 {
        0
    }

    fn top_bottom_floor(&self, highest: &mut u8, lowest: &mut u8) -> bool {
        *highest = 0;
        *lowest = 0;
        false
    }
}

/// Live facts the ported map needs, gathered from the sim by the caller.
#[derive(Clone, Copy, Debug, Default)]
pub struct DMapFacts {
    pub dungeon_item: u8,
    pub player_pos: [f32; 3],
    pub visited_room: [u8; ui::VISITED_ROOM_WORDS],
    pub stay_room: i32,
}

/// The ported dungeon map (`d_menu_dmap.cpp`), driven from the pause menu's MAP tab.
pub struct DMapStage {
    pub map: ui::DungeonMap,
    pub created: bool,
    pub draw: Option<ui::MapDraw>,
    pub image: Cell<Option<ImageHandle>>,
    pub uploaded: Cell<u64>,
    pub revision: u64,
}

impl Default for DMapStage {
    fn default() -> Self {
        Self {
            map: ui::DungeonMap::default(),
            created: false,
            draw: None,
            image: Cell::new(None),
            uploaded: Cell::new(0),
            revision: 1,
        }
    }
}

impl DMapStage {
    pub fn create(map: &mut ui::DungeonMap, facts: &DMapFacts) -> bool {
        let Ok(root) = crate::assets::AssetRoot::from_env() else {
            return false;
        };
        let Ok(archive) = root.read_archive("files/res/Msg/dmapres.arc") else {
            return false;
        };
        let (Ok(main_bytes), Ok(explain_bytes)) = (
            archive.read(ui::SCREEN_LABEL),
            archive.read(ui::EXPLAIN_LABEL),
        ) else {
            return false;
        };
        let (Some(main), Some(explain)) = (
            crate::j2d::parse(main_bytes),
            crate::j2d::parse(explain_bytes),
        ) else {
            return false;
        };
        let floors = NoFloors;
        let archives = crate::assets::load_msg_archives();
        let fonts = FontBank::default();
        let (len, layout) = (MsgLenState::default(), MsgLayoutState::default());
        let env = ui::MapEnv {
            floors: &floors,
            player_pos: facts.player_pos,
            priest_flag: 0,
            priest_pos: [0.0; 3],
            cb1_player_pos: None,
            stage_type: 0,
            save_index: 0,
            stage_map: None,
            stay_room: facts.stay_room,
            doors: &[],
            treasures: &[],
            dungeon_item: facts.dungeon_item,
            visited_room: facts.visited_room,
            tbox_bits: 0,
            collect_bits: [0; 8],
            opt_ruby: false,
        };
        let text = ui::MapText {
            archive: &archives,
            clear_count: 0,
            font: &fonts,
            main_font_slot: 0,
            ruby_font_slot: 0,
            note_font_size: 24,
            note_line_space: 28,
            len_state: len,
            layout_state: layout,
        };
        map.create(main, explain, &env, &text);
        true
    }

    pub fn step(
        map: &mut ui::DungeonMap,
        facts: &DMapFacts,
        pad: &mut ui::PadInput,
    ) -> Option<ui::MapDraw> {
        let floors = NoFloors;
        let archives = crate::assets::load_msg_archives();
        let fonts = FontBank::default();
        let (len, layout) = (MsgLenState::default(), MsgLayoutState::default());
        let env = ui::MapEnv {
            floors: &floors,
            player_pos: facts.player_pos,
            priest_flag: 0,
            priest_pos: [0.0; 3],
            cb1_player_pos: None,
            stage_type: 0,
            save_index: 0,
            stage_map: None,
            stay_room: facts.stay_room,
            doors: &[],
            treasures: &[],
            dungeon_item: facts.dungeon_item,
            visited_room: facts.visited_room,
            tbox_bits: 0,
            collect_bits: [0; 8],
            opt_ruby: false,
        };
        let text = ui::MapText {
            archive: &archives,
            clear_count: 0,
            font: &fonts,
            main_font_slot: 0,
            ruby_font_slot: 0,
            note_font_size: 24,
            note_line_space: 28,
            len_state: len,
            layout_state: layout,
        };
        map.move_frame(&env, &text, pad);
        Some(map.draw())
    }

    pub fn render(
        canvas: &mut crate::text::raster::Canvas,
        assets: &crate::panes::Assets,
        draw: &ui::MapDraw,
    ) {
        for item in draw
            .main
            .iter()
            .chain(draw.markers.iter())
            .chain(draw.explain.iter())
        {
            if !item.visible || item.width <= 0.0 || item.height <= 0.0 {
                continue;
            }
            canvas.fill_rect(
                [item.x, item.y, item.x + item.width, item.y + item.height],
                item.color,
            );
        }
        let _ = assets;
    }
}
