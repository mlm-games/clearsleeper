use crate::assets::AssetRoot;
use crate::intro::{self, PaneImages};
use crate::j2d::{self, Bounds, Screen};
use crate::sim::title::{self, PANE_COUNT, PANE_PRES, PANE_TAG, Title};
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
/// those models are not ported, so this composition carries the movie and the six panes. The
/// USA `title_loop.thp` title card already carries the logo and the copyright, so only `pres`
/// is drawn over it; without the movie every pane is the composition.
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
        blit_fullscreen(canvas, rgba, width, height);
    }
    for index in 0..PANE_COUNT {
        if movie.is_some() && index != PANE_PRES {
            continue;
        }
        if title.pane[index].alpha == 0 {
            continue;
        }
        let tag = PANE_TAG[index].to_be_bytes();
        let Ok(name) = std::str::from_utf8(&tag) else {
            continue;
        };
        let Some(pane) = assets.screen.find(name) else {
            continue;
        };
        let bounds = [pane.bounds.x, pane.bounds.y, pane.bounds.w, pane.bounds.h];
        let alpha = f32::from(title.pane[index].alpha) / 255.0;
        intro::draw_pane(canvas, &assets.screen, &assets.images, name, &bounds, alpha);
    }
}

fn blit_fullscreen(canvas: &mut Canvas, rgba: &[u8], width: u32, height: u32) {
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
    if source_width == canvas.width && source_height == canvas.height {
        canvas.pixels.copy_from_slice(&rgba[..source_len]);
        return;
    }
    for y in 0..canvas.height {
        let source_y = y * source_height / canvas.height;
        for x in 0..canvas.width {
            let source_x = x * source_width / canvas.width;
            let from = (source_y * source_width + source_x) * 4;
            let to = (y * canvas.width + x) * 4;
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
