//! On-screen controls for touch devices: a directional pad and the face buttons, drawn
//! over the game and driven by the live touches. The controls sit in the window's own
//! space, so a finger at the window's corner lands on the control drawn there, and the
//! stylus is the one finger that is not on a control.

use std::cell::{Cell, RefCell};

use glam::Vec2;
use repose_core::input::PhysicalKey;
use repose_core::{ImageFit, Modifier, View};
use repose_ui::{Image, ImageExt, Spacer};

use crate::input::PadSnapshot;
use crate::text::raster::Canvas;

/// The controls: the centre as a fraction of the window, the radius as a fraction of
/// the window's shorter side.
struct Control {
    center: [f32; 2],
    radius: f32,
}

const DPAD: Control = Control {
    center: [0.115, 0.755],
    radius: 0.115,
};
const A_BUTTON: Control = Control {
    center: [0.885, 0.755],
    radius: 0.078,
};
const B_BUTTON: Control = Control {
    center: [0.775, 0.625],
    radius: 0.065,
};
const START_BUTTON: Control = Control {
    center: [0.5, 0.075],
    radius: 0.038,
};
// Dusklight's touch layout exposes the full GameCube button set: X/Y/Z, L/R and the
// menu pages, so the pause menu's assignment and tab keys work on touch too.
const X_BUTTON: Control = Control {
    center: [0.885, 0.555],
    radius: 0.06,
};
const Y_BUTTON: Control = Control {
    center: [0.785, 0.455],
    radius: 0.055,
};
const Z_BUTTON: Control = Control {
    center: [0.885, 0.375],
    radius: 0.05,
};
const L_BUTTON: Control = Control {
    center: [0.125, 0.45],
    radius: 0.05,
};
const R_BUTTON: Control = Control {
    center: [0.125, 0.30],
    radius: 0.05,
};
const MENU_BUTTON: Control = Control {
    center: [0.5, 0.16],
    radius: 0.038,
};

const PAD_COLOR: [u8; 4] = [235, 235, 235, 110];
const A_COLOR: [u8; 4] = [80, 140, 235, 150];
const B_COLOR: [u8; 4] = [235, 90, 90, 150];
const START_COLOR: [u8; 4] = [235, 235, 235, 130];
const RING_COLOR: [u8; 4] = [255, 255, 255, 220];

/// What the live touches say.
pub struct TouchFrame {
    /// The virtual pad's state.
    pub pad: PadSnapshot,
    /// The keys the controls stand in for, so the menus read them like the keyboard's.
    pub keys: Vec<PhysicalKey>,
    /// The finger that is not on a control, in the retail screen's pixels.
    pub stylus: Option<Vec2>,
}

impl Default for TouchFrame {
    fn default() -> Self {
        Self {
            pad: PadSnapshot::default(),
            keys: Vec::new(),
            stylus: None,
        }
    }
}

/// Read the live touches: the controls they press, the keys those controls stand in for,
/// and the finger that is left for the stylus.
pub fn read(window: (u32, u32), touches: &[(u64, f32, f32)]) -> TouchFrame {
    let mut frame = TouchFrame::default();
    for (_, x, y) in touches {
        let point = Vec2::new(*x, *y);
        if let Some(direction) = dpad_direction(point, window) {
            frame.pad.move_axis = direction;
            frame.keys.push(if direction.y > 0.0 {
                PhysicalKey::ArrowUp
            } else if direction.y < 0.0 {
                PhysicalKey::ArrowDown
            } else if direction.x > 0.0 {
                PhysicalKey::ArrowRight
            } else {
                PhysicalKey::ArrowLeft
            });
            continue;
        }
        if inside(point, window, &A_BUTTON) {
            frame.pad.hold.a = true;
            frame.keys.push(PhysicalKey::Space);
            continue;
        }
        if inside(point, window, &B_BUTTON) {
            frame.pad.hold.b = true;
            frame.keys.push(PhysicalKey::Escape);
            continue;
        }
        if inside(point, window, &START_BUTTON) {
            frame.pad.hold.start = true;
            frame.keys.push(PhysicalKey::Escape);
            continue;
        }
        if inside(point, window, &X_BUTTON) {
            frame.pad.hold.x = true;
            frame.keys.push(PhysicalKey::KeyX);
            continue;
        }
        if inside(point, window, &Y_BUTTON) {
            frame.pad.hold.y = true;
            frame.keys.push(PhysicalKey::KeyY);
            continue;
        }
        if inside(point, window, &Z_BUTTON) {
            frame.pad.hold.z = true;
            frame.keys.push(PhysicalKey::KeyZ);
            continue;
        }
        if inside(point, window, &L_BUTTON) {
            frame.pad.hold.l = true;
            frame.keys.push(PhysicalKey::KeyO);
            continue;
        }
        if inside(point, window, &R_BUTTON) {
            frame.pad.hold.r = true;
            frame.keys.push(PhysicalKey::KeyR);
            continue;
        }
        if inside(point, window, &MENU_BUTTON) {
            frame.keys.push(PhysicalKey::KeyI);
            continue;
        }
        if frame.stylus.is_none() {
            frame.stylus = crate::input::canvas_point(window, *x, *y);
        }
    }
    frame
}

/// The direction the pad is pushed, from a touch inside the pad's circle. The middle of
/// the circle is dead, and a touch off the axes picks the stronger one.
fn dpad_direction(point: Vec2, window: (u32, u32)) -> Option<Vec2> {
    let center = center_px(&DPAD, window);
    let radius = radius_px(&DPAD, window);
    let offset = point - center;
    let distance = offset.length();
    if distance > radius || distance < radius * 0.22 {
        return None;
    }
    if offset.x.abs() > offset.y.abs() {
        Some(Vec2::new(offset.x.signum(), 0.0))
    } else {
        Some(Vec2::new(0.0, -offset.y.signum()))
    }
}

fn inside(point: Vec2, window: (u32, u32), control: &Control) -> bool {
    (point - center_px(control, window)).length() <= radius_px(control, window)
}

fn center_px(control: &Control, window: (u32, u32)) -> Vec2 {
    Vec2::new(
        control.center[0] * window.0 as f32,
        control.center[1] * window.1 as f32,
    )
}

fn radius_px(control: &Control, window: (u32, u32)) -> f32 {
    control.radius * window.0.min(window.1) as f32
}

/// The touch controls' live state and the overlay they are drawn on.
pub struct TouchControls {
    /// A finger has been seen, so the controls belong on screen.
    pub seen: bool,
    /// The virtual pad's state, as the last poll left it.
    pub pad: PadSnapshot,
    canvas: RefCell<Canvas>,
    image: Cell<Option<repose_core::ImageHandle>>,
    revision: Cell<u64>,
    uploaded: Cell<u64>,
    size: Cell<(u32, u32)>,
}

impl TouchControls {
    pub fn new() -> Self {
        Self {
            seen: false,
            pad: PadSnapshot::default(),
            canvas: RefCell::new(Canvas::new(1, 1)),
            image: Cell::new(None),
            revision: Cell::new(0),
            uploaded: Cell::new(u64::MAX),
            size: Cell::new((0, 0)),
        }
    }

    /// Fold the frame's touches in. The controls' look changes when one is pressed or
    /// the window changes size, and the overlay is drawn again then.
    pub fn update(&mut self, frame: &TouchFrame, window: (u32, u32)) {
        if frame.pad != PadSnapshot::default() || frame.stylus.is_some() {
            self.seen = true;
        }
        if self.pad != frame.pad {
            self.revision.set(self.revision.get() + 1);
        }
        self.pad = frame.pad;
        let size = (window.0.max(1), window.1.max(1));
        if self.size.get() != size {
            *self.canvas.get_mut() = Canvas::new(size.0 as usize, size.1 as usize);
            self.size.set(size);
            self.revision.set(self.revision.get() + 1);
        }
    }

    /// The overlay: the controls drawn over the game.
    pub fn view(&self, ctx: &repose_core::RenderContext) -> View {
        if !self.seen {
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
        if self.uploaded.get() != self.revision.get() {
            let mut canvas = self.canvas.borrow_mut();
            draw(&mut canvas, &self.pad);
            ctx.set_image_rgba8(
                handle,
                canvas.width as u32,
                canvas.height as u32,
                canvas.pixels.clone(),
                true,
            );
            self.uploaded.set(self.revision.get());
        }
        Image(Modifier::new().fill_max_size(), handle).image_fit(ImageFit::FillBounds)
    }
}

impl Default for TouchControls {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw the controls: the pad with its four directions, and the three buttons.
fn draw(canvas: &mut Canvas, pad: &PadSnapshot) {
    let window = (canvas.width as u32, canvas.height as u32);
    let pressed = 220;
    let idle = 110;
    circle(
        canvas,
        center_px(&DPAD, window),
        radius_px(&DPAD, window),
        [
            PAD_COLOR[0],
            PAD_COLOR[1],
            PAD_COLOR[2],
            if pad.move_axis == Vec2::ZERO {
                idle
            } else {
                pressed
            },
        ],
    );
    for direction in [
        Vec2::new(0.0, -1.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(-1.0, 0.0),
        Vec2::new(1.0, 0.0),
    ] {
        let center = center_px(&DPAD, window) + direction * radius_px(&DPAD, window) * 0.55;
        let active = pad.move_axis == direction;
        circle(
            canvas,
            center,
            radius_px(&DPAD, window) * 0.16,
            [255, 255, 255, if active { pressed } else { idle + 40 }],
        );
    }
    circle(
        canvas,
        center_px(&A_BUTTON, window),
        radius_px(&A_BUTTON, window),
        [
            A_COLOR[0],
            A_COLOR[1],
            A_COLOR[2],
            if pad.hold.a { pressed } else { A_COLOR[3] },
        ],
    );
    glyph(
        canvas,
        center_px(&A_BUTTON, window),
        radius_px(&A_BUTTON, window),
        &GLYPH_A,
    );
    circle(
        canvas,
        center_px(&B_BUTTON, window),
        radius_px(&B_BUTTON, window),
        [
            B_COLOR[0],
            B_COLOR[1],
            B_COLOR[2],
            if pad.hold.b { pressed } else { B_COLOR[3] },
        ],
    );
    glyph(
        canvas,
        center_px(&B_BUTTON, window),
        radius_px(&B_BUTTON, window),
        &GLYPH_B,
    );
    circle(
        canvas,
        center_px(&START_BUTTON, window),
        radius_px(&START_BUTTON, window),
        [
            START_COLOR[0],
            START_COLOR[1],
            START_COLOR[2],
            if pad.hold.start {
                pressed
            } else {
                START_COLOR[3]
            },
        ],
    );
    face_button(canvas, &X_BUTTON, A_COLOR, pad.hold.x, &GLYPH_X, window);
    face_button(canvas, &Y_BUTTON, A_COLOR, pad.hold.y, &GLYPH_Y, window);
    face_button(canvas, &Z_BUTTON, A_COLOR, pad.hold.z, &GLYPH_Z, window);
    face_button(canvas, &L_BUTTON, B_COLOR, pad.hold.l, &GLYPH_L, window);
    face_button(canvas, &R_BUTTON, B_COLOR, pad.hold.r, &GLYPH_R, window);
    circle(
        canvas,
        center_px(&MENU_BUTTON, window),
        radius_px(&MENU_BUTTON, window),
        START_COLOR,
    );
}

fn face_button(
    canvas: &mut Canvas,
    control: &Control,
    color: [u8; 4],
    held: bool,
    letter: &[[u8; 3]; 5],
    window: (u32, u32),
) {
    let center = center_px(control, window);
    let radius = radius_px(control, window);
    circle(
        canvas,
        center,
        radius,
        [
            color[0],
            color[1],
            color[2],
            if held { 220 } else { color[3] },
        ],
    );
    glyph(canvas, center, radius, letter);
}

/// A circle with a rim, in the canvas's own pixels.
fn circle(canvas: &mut Canvas, center: Vec2, radius: f32, fill: [u8; 4]) {
    let x0 = (center.x - radius - 1.0).max(0.0) as usize;
    let x1 = ((center.x + radius + 1.0) as usize).min(canvas.width);
    let y0 = (center.y - radius - 1.0).max(0.0) as usize;
    let y1 = ((center.y + radius + 1.0) as usize).min(canvas.height);
    for y in y0..y1 {
        for x in x0..x1 {
            let dx = x as f32 + 0.5 - center.x;
            let dy = y as f32 + 0.5 - center.y;
            let distance = (dx * dx + dy * dy).sqrt();
            let coverage = (radius - distance).clamp(0.0, 1.0);
            if coverage > 0.0 {
                canvas.blend(x, y, fill, coverage);
            }
            let rim = ((radius - distance).abs()).min(1.0);
            if rim > 0.0 {
                canvas.blend(x, y, RING_COLOR, rim * 0.8);
            }
        }
    }
}

/// The letters, three cells wide and five tall, one bit per cell.
const GLYPH_A: [[u8; 3]; 5] = [[0, 1, 0], [1, 0, 1], [1, 1, 1], [1, 0, 1], [1, 0, 1]];
const GLYPH_B: [[u8; 3]; 5] = [[1, 1, 0], [1, 0, 1], [1, 1, 0], [1, 0, 1], [1, 1, 0]];
const GLYPH_X: [[u8; 3]; 5] = [[1, 0, 1], [1, 0, 1], [0, 1, 0], [1, 0, 1], [1, 0, 1]];
const GLYPH_Y: [[u8; 3]; 5] = [[1, 0, 1], [1, 0, 1], [0, 1, 0], [0, 1, 0], [0, 1, 0]];
const GLYPH_Z: [[u8; 3]; 5] = [[1, 1, 1], [0, 0, 1], [0, 1, 0], [1, 0, 0], [1, 1, 1]];
const GLYPH_L: [[u8; 3]; 5] = [[1, 0, 0], [1, 0, 0], [1, 0, 0], [1, 0, 0], [1, 1, 1]];
const GLYPH_R: [[u8; 3]; 5] = [[1, 1, 0], [1, 0, 1], [1, 1, 0], [1, 0, 1], [1, 0, 1]];

/// Draw a letter in the middle of a button.
fn glyph(canvas: &mut Canvas, center: Vec2, radius: f32, glyph: &[[u8; 3]; 5]) {
    let cell = radius * 0.28;
    let width = cell * 3.0;
    let height = cell * 5.0;
    let left = center.x - width * 0.5;
    let top = center.y - height * 0.5;
    for (row, bits) in glyph.iter().enumerate() {
        for (column, bit) in bits.iter().enumerate() {
            if *bit == 0 {
                continue;
            }
            let x = left + column as f32 * cell;
            let y = top + row as f32 * cell;
            canvas.fill_rect([x, y, x + cell * 0.8, y + cell * 0.8], [255, 255, 255, 240]);
        }
    }
}
