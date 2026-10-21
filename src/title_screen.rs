use crate::assets::AssetRoot;
use crate::intro::{self, PaneImages};
use crate::j2d::{self, Bounds, Screen};
use crate::sim::title::{self, PANE_TAG, Title};
use crate::text::raster::Canvas;

/// `TlogoE` holds `title_logo_e.blo` and the textures its panes name
/// (`d_a_title.cpp:242`, `:244-249`). Every pane rect is the blo's own.
pub struct TitleAssets {
    pub screen: Screen,
    pub images: PaneImages,
}

impl TitleAssets {
    pub fn load(root: &AssetRoot) -> Option<Self> {
        let path = format!("files/res/Object/{}.arc", title::ARCHIVE);
        let archive = root.read_archive(&path).ok()?;
        let screen = j2d::parse(archive.read(title::SCREEN_FILE).ok()?)?;
        let images = PaneImages::load(&archive);
        Some(Self { screen, images })
    }
}

/// The J2D screen is drawn over the title scene at the origin (`d_a_title.cpp:566-572`).
/// Retail also draws the title ship, subtitle and kirari models first (`d_a_title.cpp:533-563`);
/// those models are not ported, so this composition carries the movie and the six panes. Each
/// pane carries the alpha `calc_2d_alpha` gives it (`d_a_title.cpp:358-419`): `zeld` and `nint`
/// fade in, `pres` blinks once the title is entered, and `zelj`, `eft1` and `eft2` stay at zero
/// on USA (`d_a_title.cpp:380`, `:435`), which is the alpha the render skips.
pub fn render(
    canvas: &mut Canvas,
    assets: &TitleAssets,
    title: &Title,
    movie: Option<(&[u8], u32, u32)>,
) {
    canvas.pixels.fill(0);
    for pixel in canvas.pixels.chunks_exact_mut(4) {
        pixel[3] = 0xFF;
    }
    if let Some((rgba, width, height)) = movie {
        blit_movie(canvas, rgba, width, height);
    }
    // `m_Screen->draw(0.0f, 0.0f, graf)` walks the screen's panes in the order the blo stores
    // them (`J2DPane.cpp:200-202`), so the draw order is the screen's own.
    for pane in &assets.screen.panes {
        let Some(index) = PANE_TAG
            .iter()
            .position(|tag| pane.name.as_bytes() == tag.to_be_bytes().as_slice())
        else {
            continue;
        };
        if title.pane[index].alpha == 0 {
            continue;
        }
        let bounds = [pane.bounds.x, pane.bounds.y, pane.bounds.w, pane.bounds.h];
        let alpha = f32::from(title.pane[index].alpha) / 255.0;
        intro::draw_pane(
            canvas,
            &assets.screen,
            &assets.images,
            &pane.name,
            &bounds,
            alpha,
        );
    }
}

/// The movie is drawn at its own size, centred in the frame: `daMP_DrawPosX =
/// (fbWidth - xSize) >> 1` and `daMP_DrawPosY = (efbHeight - ySize) >> 1`
/// (`d_a_movie_player.cpp:4065-4066`).
fn blit_movie(canvas: &mut Canvas, rgba: &[u8], width: u32, height: u32) {
    let source_width = width as usize;
    let source_height = height as usize;
    let Some(source_len) = source_width
        .checked_mul(source_height)
        .and_then(|pixels| pixels.checked_mul(4))
    else {
        return;
    };
    if source_width == 0 || source_height == 0 || rgba.len() < source_len {
        return;
    }
    let origin_x = (canvas.width as i32 - width as i32) / 2;
    let origin_y = (canvas.height as i32 - height as i32) / 2;
    for y in 0..source_height {
        let to_y = origin_y + y as i32;
        if to_y < 0 || to_y >= canvas.height as i32 {
            continue;
        }
        for x in 0..source_width {
            let to_x = origin_x + x as i32;
            if to_x < 0 || to_x >= canvas.width as i32 {
                continue;
            }
            let from = (y * source_width + x) * 4;
            let to = (to_y as usize * canvas.width + to_x as usize) * 4;
            canvas.pixels[to..to + 4].copy_from_slice(&rgba[from..from + 4]);
        }
    }
}

/// A pane's rect as the blo stores it, for callers that only have a pane name.
pub fn pane_bounds(assets: &TitleAssets, index: usize) -> Option<Bounds> {
    let tag = PANE_TAG.get(index)?.to_be_bytes();
    let name = std::str::from_utf8(&tag).ok()?;
    assets.screen.find(name).map(|pane| pane.bounds)
}
