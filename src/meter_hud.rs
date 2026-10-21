//! The in-game HUD, composed from the menu archive's own panes.
//!
//! The menu archive is `/res/Msg/menures.arc` (`d_s_logo.cpp:924`, exposed as
//! `dComIfGp_getMenuArchive`), and `d_meter.cpp:7736-7750` binds `main_parts1.blo` and the heart
//! textures from it. The row runs through `dMeter_heartInit`/`dMeter_heartMove`/`dMeter_heartDraw`
//! (`d_meter.cpp:1581-1605`). The pane logic is `ui_meter_vitals`; this module binds it to the
//! archive and composes the row into a canvas. The rest of the meter (item rows, magic, clock,
//! boss gauges) is ported but not drawn by the shell yet.

use std::collections::HashMap;

use crate::assets::AssetRoot;
use crate::bti::{self, Image};
use crate::j2d::Screens;
use crate::sim::health::Health;
use crate::text::raster::Canvas;
use crate::ui_meter_vitals::{self, Vitals, VitalsEnv};

pub const MENU_ARCHIVE: &str = "files/res/Msg/menures.arc";

pub struct MeterAssets {
    pub screens: Screens,
    pub textures: HashMap<String, Image>,
}

impl MeterAssets {
    pub fn load(root: &AssetRoot) -> Option<Self> {
        let archive = root.read_archive(MENU_ARCHIVE).ok()?;
        let bytes = archive.read(ui_meter_vitals::PARTS1).ok()?;
        let mut screens = Screens::default();
        screens.load(ui_meter_vitals::PARTS1, &bytes)?;
        let mut textures = HashMap::new();
        for name in archive.entries() {
            if !name.ends_with(".bti") {
                continue;
            }
            let Ok(bytes) = archive.read(name) else {
                continue;
            };
            if let Some(image) = bti::decode(bytes) {
                textures.insert(name.to_string(), image);
            }
        }
        Some(Self { screens, textures })
    }

    pub fn from_parts(screens: Screens, textures: HashMap<String, Image>) -> Self {
        Self { screens, textures }
    }
}

pub struct Meter {
    vitals: Vitals,
    env: VitalsEnv,
    bound: bool,
}

impl Default for Meter {
    fn default() -> Self {
        Self {
            vitals: Vitals::default(),
            env: VitalsEnv::default(),
            bound: false,
        }
    }
}

impl Meter {
    /// One frame of the retail heart row: `heartInit` once, then `heartMove` and `heartDraw`
    /// (`d_meter.cpp:1581-1605`) over the health the sim carries.
    pub fn update(&mut self, health: Health, assets: &MeterAssets) {
        if !self.bound {
            self.vitals.bind_screens(&assets.screens);
            self.env.max_life = health.max;
            self.env.life = health.units;
            self.vitals.heart_init(&mut self.env);
            self.bound = true;
        }
        // `dMeter_LifeMove` (`d_meter.cpp:1441-1495`) turns a pending life delta into `mAdjustHp`
        // and steps the row one heart per frame, so hand the sim's change to the gauge as the
        // pending item life count instead of snapping the row.
        let life_delta = i32::from(health.units) - i32::from(self.env.life);
        if life_delta != 0 {
            self.env.item_life_count = life_delta as f32;
        }
        let max_delta = i32::from(health.max) - i32::from(self.env.max_life);
        if max_delta != 0 {
            self.env.item_max_life_count = max_delta as i16;
        }
        self.env.show_2d = true;
        self.vitals.heart_move(&mut self.env);
        self.vitals.heart_gauge_alpha();
        self.vitals.heart_draw();
        self.vitals.globals.sounds.clear();
    }

    /// The row's visible panes as a change key, so the shell repaints the HUD canvas only while
    /// the row it draws (textures, rects, alphas) actually moves.
    pub fn row_key(&self) -> Vec<(String, [u32; 4], u32)> {
        self.visible_hearts()
            .map(|(texture, rect, _, alpha)| {
                (
                    texture,
                    [
                        rect[0].to_bits(),
                        rect[1].to_bits(),
                        rect[2].to_bits(),
                        rect[3].to_bits(),
                    ],
                    alpha.to_bits(),
                )
            })
            .collect()
    }

    /// The visible heart slots, each with its retail fill texture, pane rect and white tint
    /// (`heart_change_texture`/`heart_scale_init` in `ui_meter_vitals`).
    pub fn visible_hearts(&self) -> impl Iterator<Item = (String, [f32; 4], [u8; 4], f32)> + '_ {
        self.vitals
            .heart_shadow
            .iter()
            .chain(self.vitals.heart.iter())
            .filter_map(|pane| {
                let quad = pane.quad();
                if !quad.visible || !quad.bound || quad.alpha == 0 || quad.w <= 0.0 || quad.h <= 0.0
                {
                    return None;
                }
                Some((
                    quad.texture.clone(),
                    [quad.x, quad.y, quad.w, quad.h],
                    [quad.white.r, quad.white.g, quad.white.b, quad.white.a],
                    f32::from(quad.alpha) / 255.0,
                ))
            })
    }

    pub fn render(&self, canvas: &mut Canvas, assets: &MeterAssets) {
        for (texture, rect, tint, alpha) in self.visible_hearts() {
            let Some(image) = assets.textures.get(&texture) else {
                continue;
            };
            crate::panes::blit_texture(canvas, image, rect, tint, alpha);
        }
    }
}
