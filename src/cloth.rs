use crate::text::raster::Canvas;

/// Retail `dMCloth_c::plot` samples an 11x11 wave grid; the edge amplitude comes from
/// `cloth_move_sin` (`d_menu_cloth.cpp:831-878`), where the outermost columns barely
/// move and the middle columns undulate. The same mirrored falloff is used here so the
/// silhouette matches the cloth hanging in the GameCube scene.
const GRID: usize = 11;

const FILE_SELECT_CLOTH: [u8; 4] = [0x8C, 0x6C, 0x44, 0xFF];
const FILE_SELECT_SHADOW: [u8; 4] = [0x47, 0x33, 0x1C, 0xB0];
const FILE_SELECT_SKY: [u8; 4] = [0x8C, 0x8E, 0x98, 0xFF];

const MENU_CLOTH: [u8; 4] = [0x74, 0xA2, 0x3B, 0xFF];
const MENU_SHADOW: [u8; 4] = [0x23, 0x5F, 0x19, 0xCC];
const MENU_SKY: [u8; 4] = [0x2E, 0x3C, 0x56, 0xFF];

/// Mirrored edge factor: 0 at the corners of the cloth, 1 along the straight edges.
/// This is `(fx + fy) * 0.2` from `d_menu_cloth.cpp:841-849`.
fn edge_factor(i: usize) -> f32 {
    let mirrored = if i > GRID / 2 { GRID - i - 1 } else { i } as f32;
    (mirrored * 0.2).min(1.0)
}

/// Sample the vertical wave for one column at the current animation step.
fn wave(t: f32, x: usize, phase: f32) -> f32 {
    let fx = edge_factor(x);
    let a = t + x as f32 * 1.4 + phase;
    fx * (a.sin() * 5.0 + a.cos() * 3.0)
}

/// Draw the wavy-edged cloth that sits behind the BLO on the file-select and menu
/// screens. `dMCloth_c` (`d_menu_cloth.cpp`) simulates this every frame in the retail;
/// the 2D rasteriser has no 3D cloth mesh, so the same silhouette is painted directly
/// into the canvas with the sampled retail colours.
///
/// `file_select`: beige/brown cloth (`d_menu_cloth.cpp:66-69`) over the night sky.
/// `menu`: green cloth (`d_menu_cloth.cpp:30-33`) behind the pause/option screens.
pub fn draw_backdrop(canvas: &mut Canvas, file_select: bool, t: f32) {
    let (cloth, shadow, sky) = if file_select {
        (FILE_SELECT_CLOTH, FILE_SELECT_SHADOW, FILE_SELECT_SKY)
    } else {
        (MENU_CLOTH, MENU_SHADOW, MENU_SKY)
    };
    for y in 0..canvas.height {
        for x in 0..canvas.width {
            canvas.blend(x, y, sky, 1.0);
        }
    }
    let left = 18.0f32;
    let right = 622.0f32;
    let top = 24.0f32;
    let bottom = 456.0f32;
    let cols = GRID;

    // Shadow pass: same silhouette, offset down/right (`plot_shadow` in the retail).
    let shadow_off = 5.0f32;
    for i in 0..cols {
        let j = i + 1;
        let x0 = left + (right - left) * i as f32 / (cols - 1) as f32;
        let x1 = left + (right - left) * j as f32 / (cols - 1) as f32;
        let top0 = top + wave(t, i, 0.0) + shadow_off;
        let top1 = top + wave(t, j, 0.0) + shadow_off;
        let bot0 = bottom + wave(t, i, 2.1) + shadow_off;
        let bot1 = bottom + wave(t, j, 2.1) + shadow_off;
        fill_trapezoid(canvas, x0, top0, x1, top1, bot1, bot0, shadow);
    }

    // Cloth pass with wavy top/bottom edges.
    for i in 0..cols {
        let j = i + 1;
        let x0 = left + (right - left) * i as f32 / (cols - 1) as f32;
        let x1 = left + (right - left) * j as f32 / (cols - 1) as f32;
        let top0 = top + wave(t, i, 0.0);
        let top1 = top + wave(t, j, 0.0);
        let bot0 = bottom + wave(t, i, 2.1);
        let bot1 = bottom + wave(t, j, 2.1);
        fill_trapezoid(canvas, x0, top0, x1, top1, bot1, bot0, cloth);
    }
}

fn fill_trapezoid(
    canvas: &mut Canvas,
    x0: f32,
    top0: f32,
    x1: f32,
    top1: f32,
    bot1: f32,
    bot0: f32,
    color: [u8; 4],
) {
    let xa = x0.max(0.0) as usize;
    let xb = (x1.max(0.0) as usize).min(canvas.width);
    for x in xa..xb {
        let f = (x as f32 + 0.5 - x0) / (x1 - x0).max(1e-3);
        let ty = top0 + (top1 - top0) * f;
        let by = bot0 + (bot1 - bot0) * f;
        let ya = ty.max(0.0) as usize;
        let yb = (by.max(0.0) as usize).min(canvas.height);
        for y in ya..yb {
            canvas.blend(x, y, color, 1.0);
        }
    }
}
