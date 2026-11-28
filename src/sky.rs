use crate::text::raster::Canvas;

/// The sister's wake-up dialogue from the retail opening (`zel_00.bmg` 0xBCE-0xBD4),
/// which the port's cutscene timeline does not carry as JMSG blocks.
pub const PROLOGUE_LINES: [&str; 6] = [
    "Big Brother!",
    "I knew you'd be here!",
    "Hee hee hee! This is my FAVORITE spot to\ngaze out at the sea. When I play with the\ngulls, I call it \"Aryll's Lookout.\"",
    "So, do you remember what day it is today?",
    "...You're still half-asleep, aren't you?",
    "Did you forget?",
];

/// Draw one dialogue box with wrapped text onto the shared 640x480 overlay canvas.
pub fn draw_dialogue(canvas: &mut Canvas, line: usize) {
    let Some(text) = PROLOGUE_LINES.get(line) else {
        return;
    };
    let box_bounds = [28.0f32, 320.0, 584.0, 140.0];
    for y in (box_bounds[1] as usize)..((box_bounds[1] + box_bounds[3]) as usize) {
        for x in (box_bounds[0] as usize)..((box_bounds[0] + box_bounds[2]) as usize) {
            let border = x < box_bounds[0] as usize + 4
                || x >= (box_bounds[0] + box_bounds[2]) as usize - 4
                || y < box_bounds[1] as usize + 4
                || y >= (box_bounds[1] + box_bounds[3]) as usize - 4;
            let c = if border {
                [0x2a, 0x1c, 0x10, 0xFF]
            } else {
                [0xf4, 0xe6, 0xc4, 0xF0]
            };
            canvas.blend(x, y, c, 1.0);
        }
    }
    let Some(assets) = crate::panes::load() else {
        if std::env::var_os("CS_DLG_DEBUG").is_some() {
            eprintln!("DLG_DEBUG no assets");
        }
        return;
    };
    let debug = std::env::var_os("CS_DLG_DEBUG").is_some();
    let mut y = box_bounds[1] + 26.0;
    for paragraph in text.split('\n') {
        for chunk in wrap(paragraph, 32) {
            if debug {
                eprintln!("DLG_DEBUG draw {:?} at {}", chunk, y);
            }
            crate::panes::draw_label(
                canvas,
                assets,
                &chunk,
                box_bounds[0] + 22.0,
                y,
                15.0,
                0x2a1c10ff,
            );
            y += 24.0;
        }
    }
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.len() + 1 + word.len() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

/// The retail Vrkumo dome (`d_kankyo_rain.cpp`, `d_kankyo_rain_drawVrkumo.cpp`) paints a
/// ring of soft cloud billboards around the horizon. The port's world-space quads are
/// generated but never reach the frame, so paint the same layer into the 2D overlay the
/// game already composites over the 3D view.
pub fn draw_clouds(canvas: &mut Canvas, tick: u32) {
    canvas.clear();
    // Each cluster is a handful of overlapping ellipses with a soft alpha falloff;
    // the retail texture is an intensity map with a feathered edge.
    const CLUSTERS: [(f32, f32, f32, u32); 9] = [
        (0.20, 0.13, 1.0, 0),
        (0.30, 0.09, 1.2, 1),
        (0.40, 0.16, 0.8, 2),
        (0.50, 0.07, 1.1, 3),
        (0.60, 0.14, 0.9, 4),
        (0.70, 0.08, 1.3, 5),
        (0.80, 0.15, 0.9, 6),
        (0.90, 0.10, 1.0, 7),
        (0.55, 0.04, 0.7, 8),
    ];
    for (cx, cy, scale, seed) in CLUSTERS {
        let drift = ((tick as f32 * 0.004 + seed as f32 * 1.7).sin() * 6.0).max(-6.0);
        let base_x = cx * 640.0 + drift;
        let base_y = cy * 480.0;
        for lobe in 0..5 {
            let a = lobe as f32 * 2.39996 + seed as f32;
            let ox = a.cos() * 30.0 * scale;
            let oy = a.sin() * 10.0 * scale;
            let rx = (36.0 + (lobe % 3) as f32 * 14.0) * scale;
            let ry = (13.0 + (lobe % 2) as f32 * 7.0) * scale;
            let x0 = (base_x + ox - rx).max(0.0) as usize;
            let x1 = (base_x + ox + rx).min(640.0) as usize;
            let y0 = (base_y + oy - ry).max(0.0) as usize;
            let y1 = (base_y + oy + ry).min(480.0) as usize;
            for y in y0..y1 {
                for x in x0..x1 {
                    let dx = (x as f32 + 0.5 - base_x - ox) / rx;
                    let dy = (y as f32 + 0.5 - base_y - oy) / ry;
                    let d = dx * dx + dy * dy;
                    if d >= 1.0 {
                        continue;
                    }
                    // Feathered edge, brighter core. Retail's cloud layers are white-ish
                    // with a slight blue-grey underside (`mVrkumoPacket` tev colours).
                    let a = ((1.0 - d) * 0.85).min(1.0);
                    let shade = if dy > 0.0 { 0.90 } else { 1.0 };
                    let c = [
                        (245.0 * shade) as u8,
                        (247.0 * shade) as u8,
                        255u8,
                        (a * 255.0) as u8,
                    ];
                    canvas.blend(x, y, c, 1.0);
                }
            }
        }
    }
}
