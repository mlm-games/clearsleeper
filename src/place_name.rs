use std::cell::Cell;

use repose_core::ImageHandle;
use repose_ui::{Image, ImageExt, Spacer};

use crate::text::raster::Canvas;
use crate::ui_overlay::{self, PlaceNameEnv, PlaceNameProcess};

/// The area-name card. The `tlpNo` index comes from the event cut
/// (`d_event_data.cpp:321-322`); the card opens for ten frames and closes after a hold
/// (`d_place_name.cpp:82-107`).
pub struct PlaceNameCard {
    pub process: PlaceNameProcess,
    timer: i16,
    active: bool,
    last_sequence: u32,
    draw: Vec<ui_overlay::PaneQuad>,
    image: Cell<Option<ImageHandle>>,
    uploaded: Cell<u64>,
    revision: u64,
}

impl Default for PlaceNameCard {
    fn default() -> Self {
        Self {
            process: PlaceNameProcess::default(),
            timer: 0,
            active: false,
            last_sequence: 0,
            draw: Vec::new(),
            image: Cell::new(None),
            uploaded: Cell::new(0),
            revision: 1,
        }
    }
}

const HOLD_FRAMES: i16 = 70;

impl PlaceNameCard {
    pub fn step(&mut self, index: u8, sequence: u32) {
        if sequence != self.last_sequence {
            self.last_sequence = sequence;
            self.timer = 0;
            self.process = PlaceNameProcess::default();
            let Ok(root) = crate::assets::AssetRoot::from_env() else {
                return;
            };
            let Ok(archive) = root
                .read_archive("files/res/Object/PName.arc")
                .or_else(|_| root.read_archive("files/res/Msg/PName.arc"))
            else {
                return;
            };
            let Some(bytes) = archive.read(ui_overlay::PLACE_NAME_LAYOUT).ok() else {
                return;
            };
            let Some(layout) = crate::j2d::parse(bytes) else {
                return;
            };
            let env = PlaceNameEnv {
                now_stage: u32::from(index.min(ui_overlay::PLACE_NAME_STAGE_MAX as u8 - 1)),
                res_load: ui_overlay::Phase::Complete,
                ..PlaceNameEnv::default()
            };
            self.process.create(&env, &layout);
            if let Some(texture) = ui_overlay::name_texture(u32::from(index)) {
                if let Ok(dvd) = root.read(texture.trim_start_matches('/')) {
                    self.process.card.change_texture(&dvd);
                }
            }
            self.active = true;
        }
        if !self.active {
            return;
        }
        self.timer += 1;
        if self.timer <= ui_overlay::CARD_FADE_FRAMES as i16 {
            self.process.execute(2);
        } else if self.timer > HOLD_FRAMES {
            if self.process.execute(1) && self.process.card.get_counter() <= 0 {
                self.active = false;
                self.process.delete();
            }
        }
        self.draw = self.process.card.draw();
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn view(&self, ctx: &repose_core::RenderContext) -> repose_core::View {
        if !self.active {
            return Spacer();
        }
        let handle = match self.image.get() {
            Some(handle) => handle,
            None => {
                let handle = ctx.alloc_image_handle();
                self.image.set(Some(handle));
                handle
            }
        };
        if self.uploaded.get() != self.revision {
            let mut canvas = Canvas::new(640, 480);
            self.render_into(&mut canvas);
            ctx.set_image_rgba8(handle, 640, 480, canvas.pixels, true);
            self.uploaded.set(self.revision);
        }
        Image(repose_core::Modifier::new().fill_max_size(), handle)
            .image_fit(repose_core::ImageFit::Contain)
    }

    fn render_into(&self, canvas: &mut Canvas) {
        for quad in &self.draw {
            if !quad.visible || quad.bounds.w <= 0.0 || quad.bounds.h <= 0.0 {
                continue;
            }
            let b = quad.bounds;
            canvas.fill_rect(
                [b.x, b.y, b.x + b.w, b.y + b.h],
                [
                    (quad.color_white >> 16) as u8,
                    (quad.color_white >> 8) as u8,
                    quad.color_white as u8,
                    quad.alpha,
                ],
            );
            if !quad.text.is_empty() {
                if let Some(assets) = crate::panes::load() {
                    crate::panes::draw_label(
                        canvas,
                        assets,
                        &quad.text,
                        b.x,
                        b.y,
                        18.0,
                        quad.alpha as u32 * 0x0101_0101,
                    );
                }
            }
        }
    }
}
