//! The button prompt screen the retail file select draws over itself.
//!
//! `dScnName_c::buttonIconCreate` (`tww/src/d/d_s_name.cpp:359-386`) loads
//! `main_parts_fileselect.blo` out of the file select archive and binds five of
//! its panes; `dScnName_c::draw` (`:590-599`) queues that screen after the
//! file select's own, so the "Choose" and "Return" prompts land over the file
//! select's top right corner.
//!
//! The retail slides the prompts down from above when the file select opens
//! (`paneTransButtonIcon`, `d_s_name.cpp:470-494`) and leaves them where that
//! transition ends (`d_s_name.cpp:1124-1125`); the port has no clock for the
//! name scene, so the prompts are drawn at that resting place.

use std::collections::HashMap;

use crate::j2d::{Screen, unpack_color};
use crate::panes::Assets;
use crate::text::raster::Canvas;
use crate::ui_menu_save::{Color, Pane};

/// The screen `buttonIconCreate` loads (`d_s_name.cpp:362`).
pub const PARTS_LABEL: &str = "main_parts_fileselect.blo";

/// The panes `fopMsgM_setPaneData` binds (`d_s_name.cpp:363-365`, `:380-381`).
/// `bawp` and `bawd` are the two text labels (`ba_kettei.bti`,
/// `ba_modoru.bti`), `cent`, `baat` and `bab` the button plates and the glyph.
const BOUND: [&str; 5] = ["cent", "bab", "baat", "bawp", "bawd"];

/// The prompt screen and the live panes the retail bound to it.
pub struct Parts {
    pub screen: Screen,
    panes: Vec<Pane>,
}

impl Parts {
    /// `fopMsgM_setPaneData` reads the pane's bounds, size and alpha out of the
    /// screen it searches (`f_op_msg_mng.cpp:9799-9837`); `parts_set` does the
    /// same and leaves the file select's own colour ramp behind, so the
    /// picture's own black and white follow: a colour word is `0xRRGGBBAA`,
    /// because the GameCube is big endian and `JUtility::TColor::set` copies
    /// the word into the `{r, g, b, a}` bytes (`TColor.h:22-29`).
    pub fn bind(screen: Screen) -> Parts {
        let mut panes = Vec::new();
        for tag in BOUND {
            let mut pane = Pane::new(tag);
            if let Some(layout) = screen.find(tag) {
                pane.parts_set(layout);
                pane.set_black_white(
                    Color::from(unpack_color(layout.color_black)),
                    Color::from(unpack_color(layout.color_white)),
                );
            }
            panes.push(pane);
        }
        Parts { screen, panes }
    }

    /// The bound panes, keyed by the tag the screen carries.
    pub fn live_table(&self) -> HashMap<&'static str, &Pane> {
        self.panes.iter().map(|pane| (pane.tag, pane)).collect()
    }

    /// Draw the prompt screen over the file select. The retail queues it after
    /// the file select's own screen (`d_s_name.cpp:597-598`), so it lands on
    /// top of it.
    pub fn draw(&self, canvas: &mut Canvas, assets: &Assets) {
        crate::panes::draw_screen_table(canvas, assets, &self.screen, &self.live_table());
    }
}

/// Draw the prompt screen the file select archive carries, if it has one.
pub fn draw(canvas: &mut Canvas, assets: &Assets) {
    let Some(parts) = assets.parts.as_ref() else {
        return;
    };
    parts.draw(canvas, assets);
}
