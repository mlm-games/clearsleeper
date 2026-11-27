use crate::text::raster::Canvas;

/// The retail Vrkumo dome (`d_kankyo_rain.cpp`, `d_kankyo_rain_drawVrkumo.cpp`) paints a
/// ring of soft cloud billboards around the horizon. The port's world-space quads are
/// generated but never reach the frame, so paint the same layer into the 2D overlay the
/// game already composites over the 3D view.
pub fn draw_clouds(canvas: &mut Canvas, tick: u32) {
    canvas.clear();
    // Each cluster is a handful of overlapping ellipses with a soft alpha falloff;
    // the retail texture is an intensity map with a feathered edge.
    const CLUSTERS: [(f32, f32, f32, u32); 9] = [
        (0.08, 0.10, 1.0, 0),
        (0.20, 0.06, 1.2, 1),
        (0.33, 0.13, 0.8, 2),
        (0.46, 0.04, 1.1, 3),
        (0.58, 0.11, 0.9, 4),
        (0.70, 0.05, 1.3, 5),
        (0.82, 0.12, 0.9, 6),
        (0.93, 0.07, 1.0, 7),
        (0.50, 0.02, 0.7, 8),
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
