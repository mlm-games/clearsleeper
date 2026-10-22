use crate::assets::AssetRoot;
use crate::intro::{self, PaneImages};
use crate::j2d::{self, Bounds, Screen};
use crate::sim::title::{self, PANE_DRAWN, PANE_TAG, Title};
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
/// those models are not ported, so this composition carries the J2D panes alone. Each
/// pane carries the alpha `calc_2d_alpha` gives it (`d_a_title.cpp:358-419`): `zeld` and `nint`
/// fade in, `zelj` stays at zero outside Japan (`d_a_title.cpp:380`), and `eft1` and `eft2`
/// are the dummies whose alpha is never committed (`d_a_title.cpp:516-518`), which is the
/// alpha the render skips.
///
/// The attract reel is not drawn here. `title_loop.thp` is the movie the title scene's own
/// movie player plays (`d_s_title.cpp:122`, `d_a_movie_player.cpp:4226`), and the title actor
/// hands the game to that scene when the next-stage flag is set (`d_a_title.cpp:661-675`);
/// the reel hands the game back to the title screen on a button press or when it runs out
/// (`d_s_title.cpp:86-89`). The title screen itself is the 3D title scene with this screen
/// over it (`d_a_title.cpp:626-631`).
pub fn render(
    canvas: &mut Canvas,
    assets: &TitleAssets,
    title: &Title,
    _movie: Option<(&[u8], u32, u32)>,
) {
    canvas.pixels.fill(0);
    for pixel in canvas.pixels.chunks_exact_mut(4) {
        pixel[3] = 0xFF;
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
        if !PANE_DRAWN.contains(&index) {
            continue;
        }
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

/// A pane's rect as the blo stores it, for callers that only have a pane name.
pub fn pane_bounds(assets: &TitleAssets, index: usize) -> Option<Bounds> {
    let tag = PANE_TAG.get(index)?.to_be_bytes();
    let name = std::str::from_utf8(&tag).ok()?;
    assets.screen.find(name).map(|pane| pane.bounds)
}
