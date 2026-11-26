use std::cell::Cell;
use std::collections::HashMap;

use repose_core::ImageHandle;

use crate::j2d;
use crate::text::raster::Canvas;
use crate::ui_menu_option as ui;
use crate::ui_menu_save::{ArchiveMsgSource, Pane as SavePane};

pub use crate::ui_menu_option::{OptionMenu, SCREEN_ARCHIVE, SCREEN_LABEL};

/// Canvas stage for the retail option screen (`d_menu_option.cpp`), rasterised from the
/// `option.blo` in `optres.arc` instead of a text panel.
pub struct OptionMenuStage {
    pub image: Cell<Option<ImageHandle>>,
    pub uploaded: Cell<u64>,
    pub revision: u64,
}

impl Default for OptionMenuStage {
    fn default() -> Self {
        Self {
            image: Cell::new(None),
            uploaded: Cell::new(0),
            revision: 1,
        }
    }
}

struct Messages<'a> {
    inner: ArchiveMsgSource<'a>,
}

impl ui::MsgSource for Messages<'_> {
    fn message(&self, msg_no: u32) -> String {
        crate::ui_menu_save::MsgSource::message(&self.inner, msg_no)
    }

    fn item_message(&self, msg_no: u32) -> String {
        crate::ui_menu_save::MsgSource::message(&self.inner, msg_no)
    }
}

impl OptionMenuStage {
    /// Bind the BLO, messages and font into the world once.
    pub fn install(world: &mut bevy_ecs::prelude::World) -> bool {
        if world.contains_resource::<ui::OptionMenu>() {
            return true;
        }
        let Ok(root) = crate::assets::AssetRoot::from_env() else {
            return false;
        };
        let Ok(archive) = root.read_archive(ui::SCREEN_ARCHIVE) else {
            return false;
        };
        let raw = match archive.read(ui::SCREEN_LABEL) {
            Ok(bytes) => bytes,
            Err(_) => {
                return false;
            }
        };
        let bytes = if raw.starts_with(&crate::archive::YAZ0_MAGIC) {
            match crate::archive::yaz0_decode(raw, ui::SCREEN_LABEL) {
                Ok(bytes) => bytes,
                Err(_) => return false,
            }
        } else {
            raw.to_vec()
        };
        let mut screens = j2d::Screens::default();
        if screens.load(ui::SCREEN_LABEL, &bytes).is_none() {
            return false;
        }
        let Some(screen) = screens.get(ui::SCREEN_LABEL).cloned() else {
            return false;
        };
        world.insert_resource(screens);

        let font =
            crate::text::font::ResFont::from_bytes(&crate::assets::load_font().unwrap_or_default());
        let messages = crate::assets::MsgArchives::load(&root);
        let source = Messages {
            inner: ArchiveMsgSource {
                archive: &messages,
                clear_count: 0,
            },
        };
        let config = ui::PlayerConfig::default();
        let mut rumbles = ui::Rumbles::default();
        let mut menu = ui::OptionMenu::default();
        menu.create(screen, &source, &font, &config, &mut rumbles);
        world.insert_resource(menu);
        true
    }

    /// Refresh the live BLO panes from the shell's settings values.
    pub fn sync(
        &mut self,
        world: &mut bevy_ecs::prelude::World,
        attention: u8,
        ruby: u8,
        vibration: u8,
        sound: u8,
        cursor: usize,
    ) {
        if !world.contains_resource::<ui::OptionMenu>() {
            return;
        }
        let mut menu = world.resource_mut::<ui::OptionMenu>();
        menu.attention = attention;
        menu.ruby = ruby;
        menu.vibration = vibration;
        menu.sound = sound;
        menu.panes.cursor[1].user_area = cursor.min(3) as i16;
        if menu.panes.backing[0].user_area < ui::OPEN_DONE as i16 {
            menu.panes.backing[0].user_area = ui::OPEN_DONE as i16;
        }
        menu.main_move();
        menu.note_move();
        menu.title_move();
        menu.cursor_move();
        menu.draw();
        self.revision = self.revision.wrapping_add(1);
    }

    /// Rasterise the bound screen with its live pane values.
    pub fn render(
        &self,
        canvas: &mut Canvas,
        assets: &crate::panes::Assets,
        screen: &j2d::Screen,
        menu: &ui::OptionMenu,
    ) {
        // The picture's tone pair must not come from the BLO here: option.blo's note
        // frames carry green pairs, which turn the grayscale note textures into flat
        // green blocks. `J2DPicture`'s own default is black=0/white=0xffffffff
        // (`J2DPicture.cpp:143-144`; HD `J2DPicture.cpp:38,135`), so apply that pair
        // to every pane, bound or not, before the screen is rasterised.
        let mut neutral = screen.clone();
        for pane in &mut neutral.panes {
            pane.color_black = 0;
            pane.color_white = 0xFFFF_FFFF;
        }
        let owned = owned_panes(&menu.panes);
        let live: HashMap<&str, &SavePane> = owned.iter().map(|(k, v)| (*k, v)).collect();
        crate::panes::draw_screen_table(canvas, assets, &neutral, &live);
    }
}

fn save_pane(pane: &ui::Pane) -> SavePane {
    let mut out = SavePane::default();
    out.tag = pane.tag;
    out.found = pane.found;
    out.visible = pane.visible;
    out.pos_top_left_orig = pane.pos_top_left_orig;
    out.pos_top_left = pane.pos_top_left;
    out.pos_center_orig = pane.pos_center_orig;
    out.pos_center = pane.pos_center;
    out.size_orig = pane.size_orig;
    out.size = pane.size;
    out.init_alpha = pane.init_alpha;
    out.now_alpha = pane.now_alpha;
    out.user_area = pane.user_area;
    out.texture = pane.texture;
    // `J2DPicture`'s default tone pair (`J2DPicture.cpp:143-144`; HD
    // `J2DPicture.cpp:38,135`), not the BLO's green note pairs: the option state
    // machine never changes this pair at runtime.
    out.black = crate::ui_menu_save::Color::new(0, 0, 0, 0);
    out.white = crate::ui_menu_save::Color::new(255, 255, 255, 255);
    out.rotation = pane.rotation;
    out.rotate_x = pane.rotate_x;
    out.rotate_y = pane.rotate_y;
    out.text = pane.text.clone();
    out.font_size_x = pane.font_size_x;
    out.font_size_y = pane.font_size_y;
    out.char_space = pane.char_space;
    out.line_space = pane.line_space;
    out.shift_x = pane.shift_x;
    out.shift_y = pane.shift_y;
    out
}

fn owned_panes<'a>(panes: &'a ui::OptionPanes) -> HashMap<&'a str, SavePane> {
    panes
        .live()
        .into_iter()
        .map(|(k, v)| (k, save_pane(v)))
        .collect()
}
