use super::font::ResFont;
use super::msgbox::TextLayout;

pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height * 4],
        }
    }

    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }

    pub fn fill_rect(&mut self, bounds: [f32; 4], color: [u8; 4]) {
        let x0 = bounds[0].max(0.0) as usize;
        let y0 = bounds[1].max(0.0) as usize;
        let x1 = (bounds[2].max(0.0) as usize).min(self.width);
        let y1 = (bounds[3].max(0.0) as usize).min(self.height);
        for y in y0..y1 {
            for x in x0..x1 {
                self.blend(x, y, color, color[3] as f32 / 255.0);
            }
        }
    }

    pub fn blend(&mut self, x: usize, y: usize, color: [u8; 4], coverage: f32) {
        if x >= self.width || y >= self.height || coverage <= 0.0 {
            return;
        }
        let alpha = coverage * color[3] as f32 / 255.0;
        if alpha <= 0.0 {
            return;
        }
        let at = (y * self.width + x) * 4;
        for (index, value) in color.iter().enumerate().take(3) {
            let dst = self.pixels[at + index] as f32;
            self.pixels[at + index] = (dst + (f32::from(*value) - dst) * alpha).round() as u8;
        }
        let dst = self.pixels[at + 3] as f32;
        self.pixels[at + 3] = (dst + (255.0 - dst) * alpha).round() as u8;
    }

    pub fn draw_layout(&mut self, layout: &TextLayout, font: &ResFont, origin: [f32; 2], alpha: u8) {
        let glyph = &font.glyph;
        let cells = u32::from(glyph.texture_width) * u32::from(glyph.texture_height);
        if cells == 0 || font.tex_page().len() * 2 != cells as usize {
            return;
        }
        let texels = unswizzle(
            font.tex_page(),
            usize::from(glyph.texture_width),
            usize::from(glyph.texture_height),
        );
        let (tex_w, tex_h) = (
            f32::from(u16::from(glyph.texture_width).max(1)),
            f32::from(u16::from(glyph.texture_height).max(1)),
        );
        for run in &layout.quads {
            let quad = &run.quad;
            let min_x = quad.pos[0][0].min(quad.pos[2][0]) + origin[0];
            let max_x = quad.pos[0][0].max(quad.pos[2][0]) + origin[0];
            let min_y = quad.pos[0][1].min(quad.pos[2][1]) + origin[1];
            let max_y = quad.pos[0][1].max(quad.pos[2][1]) + origin[1];
            if max_x < min_x || max_y < min_y {
                continue;
            }
            let span_x = (max_x - min_x).max(1.0);
            let span_y = (max_y - min_y).max(1.0);
            let x0 = min_x.floor().max(0.0) as usize;
            let y0 = min_y.floor().max(0.0) as usize;
            let x1 = (max_x.ceil().max(0.0) as usize).min(self.width);
            let y1 = (max_y.ceil().max(0.0) as usize).min(self.height);
            for y in y0..y1 {
                for x in x0..x1 {
                    let fx = (x as f32 + 0.5 - min_x) / span_x;
                    let fy = (y as f32 + 0.5 - min_y) / span_y;
                    let (u, v) = sample_uv(quad, fx, fy, tex_w, tex_h);
                    let Some(&index) = texels.get(
                        usize::try_from(v).unwrap_or(usize::MAX) * usize::from(glyph.texture_width)
                            + usize::try_from(u).unwrap_or(usize::MAX),
                    ) else {
                        continue;
                    };
                    if index == 0 {
                        continue;
                    }
                    let coverage = f32::from(index) / 15.0;
                    let color = mix(quad.color, fx, fy);
                    self.blend(x, y, [color[0], color[1], color[2], alpha], coverage);
                }
            }
        }
    }
}

fn unswizzle(page: &[u8], width: usize, height: usize) -> Vec<u8> {
    const TILE: usize = 8;
    let mut out = vec![0_u8; width * height];
    let mut tile = 0_usize;
    for block_y in 0..height / TILE {
        for block_x in 0..width / TILE {
            for row in 0..TILE {
                let at = tile * TILE * TILE / 2 + row * (TILE / 2);
                for col in 0..TILE {
                    let byte = page.get(at + col / 2).copied().unwrap_or(0);
                    let value = if col % 2 == 0 { byte >> 4 } else { byte & 0x0F };
                    out[(block_y * TILE + row) * width + block_x * TILE + col] = value;
                }
            }
            tile += 1;
        }
    }
    out
}

fn sample_uv(quad: &super::font::CharQuad, fx: f32, fy: f32, tex_w: f32, tex_h: f32) -> (i32, i32) {
    let unit = |value: i32| value as f32 / 32768.0;
    let u0 = unit(quad.uv[0][0]) * tex_w;
    let u1 = unit(quad.uv[1][0]) * tex_w;
    let v0 = unit(quad.uv[0][1]) * tex_h;
    let v1 = unit(quad.uv[2][1]) * tex_h;
    (
        (u0 + (u1 - u0) * fx).floor() as i32,
        (v0 + (v1 - v0) * fy).floor() as i32,
    )
}

fn mix(colors: [u32; 4], fx: f32, fy: f32) -> [u8; 3] {
    let split = |color: u32| [(color >> 24) as u8, (color >> 16) as u8, (color >> 8) as u8];
    let (top_left, top_right, bottom_right, bottom_left) = (
        split(colors[0]),
        split(colors[1]),
        split(colors[2]),
        split(colors[3]),
    );
    let mut out = [0_u8; 3];
    for index in 0..3 {
        let top = f32::from(top_left[index]) * (1.0 - fx) + f32::from(top_right[index]) * fx;
        let bottom = f32::from(bottom_left[index]) * (1.0 - fx) + f32::from(bottom_right[index]) * fx;
        out[index] = (top * (1.0 - fy) + bottom * fy).round() as u8;
    }
    out
}