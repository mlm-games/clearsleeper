use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::archive::Archive;
use crate::assets::AssetRoot;
use crate::bti::{self, Image};
use crate::j2d::{Kind, Screen, ScreenPane};
use crate::text::font::{FontBank, ResFont};
use crate::text::msgbox::TextBox;
use crate::text::raster::Canvas;
use crate::ui_file_select::{FileSelectPanes, SCREEN_ARCHIVE, SCREEN_LABEL};
use crate::ui_menu_save::Pane;

const NESTED_ARCHIVE: &str = "dat/file_select.arc";
const FONT_ARCHIVE: &str = "files/res/Msg/fontres.arc";
const TAU: f32 = std::f32::consts::TAU;

pub struct Assets {
    pub screen: Screen,
    pub textures: HashMap<String, Image>,
    names: Vec<String>,
    font_data: Vec<Vec<u8>>,
}

impl Assets {
    fn font_slot(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|entry| entry == name)
    }
}

pub fn load() -> Option<&'static Assets> {
    static CACHE: OnceLock<Option<Assets>> = OnceLock::new();
    CACHE.get_or_init(build).as_ref()
}

fn build() -> Option<Assets> {
    let root = AssetRoot::from_env().ok()?;
    let stage = root.read_archive(SCREEN_ARCHIVE).ok()?;
    let nested = stage.read(NESTED_ARCHIVE).ok()?.to_vec();
    let inner = Archive::parse(nested, PathBuf::from(NESTED_ARCHIVE)).ok()?;
    let screen = crate::j2d::parse(inner.read(SCREEN_LABEL).ok()?)?;
    let mut textures = HashMap::new();
    for name in inner.entries() {
        if !name.ends_with(".bti") {
            continue;
        }
        let Some(bytes) = inner.read(name).ok() else {
            continue;
        };
        if let Some(image) = bti::decode(bytes) {
            textures.insert(name.to_string(), image);
        }
    }
    let mut names = Vec::new();
    let mut font_data = Vec::new();
    for pane in &screen.panes {
        let Some(name) = pane.font.as_deref() else {
            continue;
        };
        if names.iter().any(|entry| entry == name) {
            continue;
        }
        let Some(bytes) = font_bytes(&root, &inner, name) else {
            continue;
        };
        names.push(name.to_string());
        font_data.push(bytes);
    }
    Some(Assets {
        screen,
        textures,
        names,
        font_data,
    })
}

fn font_bytes(root: &AssetRoot, local: &Archive, name: &str) -> Option<Vec<u8>> {
    if let Some(bytes) = local.read(name).ok() {
        return Some(bytes.to_vec());
    }
    let stem = name.strip_suffix(".bfn")?.strip_suffix('_')?;
    let archive = root.read_archive(FONT_ARCHIVE).ok()?;
    let entries = archive.entries();
    let mut locale: Option<&str> = None;
    for entry in &entries {
        let Some(tail) = entry.strip_prefix(stem) else {
            continue;
        };
        if !tail.ends_with(".bfn") || !tail.starts_with('_') {
            continue;
        }
        if locale.is_none_or(|best| tail < best) {
            locale = Some(tail);
        }
    }
    let tail = locale?;
    archive
        .read(&format!("{stem}{tail}"))
        .ok()
        .map(|bytes| bytes.to_vec())
}

thread_local! {
    static FONTS: RefCell<FontBank> = RefCell::new(FontBank::new());
}

fn with_fonts<R>(assets: &Assets, run: impl FnOnce(&mut FontBank) -> R) -> R {
    FONTS.with(|cell| {
        let mut bank = cell.borrow_mut();
        if bank.len() != assets.font_data.len() {
            let fonts = assets
                .font_data
                .iter()
                .map(|bytes| ResFont::from_bytes(bytes))
                .collect();
            *bank = FontBank::with_fonts(fonts);
        }
        run(&mut bank)
    })
}

pub fn draw_file_select(canvas: &mut Canvas, panes: &FileSelectPanes, assets: &Assets) {
    // The BACK pane binds `sky_cloth.bti`, which the retail game generates at
    // runtime: `dScnName_c::cloth_create` builds an 11x11 vertex grid from trig
    // (`d_menu_cloth.cpp:161`) and the J2D pane binds the mesh it registers. No
    // such file exists on disc, so nothing here can supply it. Until that mesh
    // is ported, paint the canvas opaque so the stage does not show through.
    // The colour is a neutral placeholder, not a decomp-derived constant: the
    // real one arrives via daCLOTH_ChildHIO_c, whose constructor is empty and
    // whose values come from HIO game data.
    canvas.fill_rect([0.0, 0.0, 640.0, 480.0], [16, 18, 32, 255]);
    let live = live_table(panes);
    let nodes = &assets.screen.panes;
    let mut origin = vec![[0.0f32; 2]; nodes.len()];
    let mut shift = vec![[0.0f32; 2]; nodes.len()];
    let mut alpha = vec![0f32; nodes.len()];
    let mut shown = vec![false; nodes.len()];
    with_fonts(assets, |fonts| {
        for (index, node) in nodes.iter().enumerate() {
            let pane = live
                .get(node.name.as_str())
                .copied()
                .filter(|entry| bound_to(entry, node));
            // J2DPane::update adds `parent.globalBounds.origin - parent.bounds.origin`
            // (J2DPane.cpp:268). `changeUseTrans` has already rebased every
            // pane's bounds to (0,0) and moved that origin into the translate,
            // so `parent.bounds.origin` is zero here and the whole of the
            // parent's global origin is what lands on the child: stream bounds
            // are relative to the parent pane, not absolute.
            let parent_offset = node.parent.map_or([0.0f32; 2], |p| origin[p]);
            let parent_alpha = if node.influenced_alpha {
                node.parent.map_or(1.0f32, |p| alpha[p])
            } else {
                1.0
            };
            let parent_shown = node.parent.is_none_or(|p| shown[p]);
            let own_shift = pane.map_or([0.0f32; 2], |entry| {
                [
                    entry.pos_center.x - entry.pos_center_orig.x,
                    entry.pos_center.y - entry.pos_center_orig.y,
                ]
            });
            
            // The parent's global origin already carries the parent's own shift,
            // so the child adds only its own translation on top of the offset.
            // Accumulating the parent's shift here would apply it twice.
            shift[index] = own_shift;
            origin[index] = [
                parent_offset[0] + node.bounds.x + shift[index][0],
                parent_offset[1] + node.bounds.y + shift[index][1],
            ];
            alpha[index] = parent_alpha * own_alpha(node, pane);
            shown[index] = parent_shown && node.visible && own_visible(node, pane);
            let trace = std::env::var_os("FS_TRACE").is_some();
            let trace_slots = std::env::var_os("FS_SLOTS").is_some();
            if trace && (own_shift[0].abs() > 0.5 || own_shift[1].abs() > 0.5) {
                eprintln!(
                    "  RESIDUAL {:>10} shift=({:.1},{:.1}) drawn_at=({:.0},{:.0}) shown={} a={:.2}",
                    node.name, own_shift[0], own_shift[1],
                    origin[index][0], origin[index][1], shown[index], alpha[index]
                );
            }
            if trace_slots {
                eprintln!(
                    "  PANE[{index:3}] {:>8} par={:?} at=({:.0},{:.0}) bounds=({:.0},{:.0} {:.0}x{:.0}) shown={} a={:.2} tex={:?} live={}",
                    node.name, node.parent, origin[index][0], origin[index][1],
                    node.bounds.x, node.bounds.y, node.bounds.w, node.bounds.h,
                    shown[index], alpha[index],
                    node.texture, pane.is_some()
                );
            }
            if shown[index] && alpha[index] > 0.0 {
                let size = pane.map_or([node.bounds.w, node.bounds.h], |entry| {
                    [entry.size.x, entry.size.y]
                });
                let rect = [origin[index][0], origin[index][1], size[0], size[1]];
                match node.kind {
                    Kind::Picture => draw_picture(canvas, assets, node, pane, rect, alpha[index]),
                    Kind::TextBox => {
                        draw_text(canvas, fonts, assets, node, pane, rect, alpha[index])
                    }
                    _ => {}
                }
            }
        }
    });
}

fn bound_to(pane: &Pane, node: &ScreenPane) -> bool {
    pane.pos_center_orig.x == node.bounds.x + node.bounds.w * 0.5
        && pane.pos_center_orig.y == node.bounds.y + node.bounds.h * 0.5
        && pane.size_orig.x == node.bounds.w
        && pane.size_orig.y == node.bounds.h
}

fn own_alpha(node: &ScreenPane, pane: Option<&Pane>) -> f32 {
    let value = match pane {
        Some(entry) if entry.now_alpha != 0 => entry.now_alpha,
        Some(_) if node.kind == Kind::TextBox => return 0.0,
        _ => node.alpha,
    };
    f32::from(value) / 255.0
}

fn own_visible(node: &ScreenPane, pane: Option<&Pane>) -> bool {
    pane.map_or(node.visible, |p| p.visible)
}

fn draw_picture(
    canvas: &mut Canvas,
    assets: &Assets,
    node: &ScreenPane,
    pane: Option<&Pane>,
    rect: [f32; 4],
    alpha: f32,
) {
    let Some(name) = pane.and_then(|p| p.texture).or(node.texture.as_deref()) else {
        return;
    };
    let Some(image) = assets.textures.get(name) else {
        return;
    };
    let rotation = f32::from(pane.map_or(node.rotation, |p| p.rotation));
    let uv = tex_coord(node.pic_binding, node.pic_mirror, rect, image);
    blit(
        canvas,
        image,
        rect,
        rotation * TAU / 65536.0,
        node.corner_color.map(unpack_color),
        alpha,
        uv,
    );
}

fn wrap_axis(value: i64, size: usize, mode: u8) -> (usize, f32) {
    let span = size as i64;
    if mode == WRAP_MIRROR {
        let cycle = span * 2;
        let folded = value.rem_euclid(cycle);
        if folded < span {
            return (folded as usize, 0.0);
        }
        return ((cycle - folded - 1) as usize, 1.0);
    }
    if mode == WRAP_REPEAT {
        return (value.rem_euclid(span) as usize, 0.0);
    }
    (value.clamp(0, span - 1) as usize, 0.0)
}

fn sample_nearest(image: &Image, s: f32, t: f32) -> [f32; 4] {
    let sx = wrap_axis(
        (s * image.width as f32).floor() as i64,
        image.width,
        image.wrap_s,
    )
    .0;
    let sy = wrap_axis(
        (t * image.height as f32).floor() as i64,
        image.height,
        image.wrap_t,
    )
    .0;
    let from = (sy * image.width + sx) * 4;
    [
        f32::from(image.pixels[from]),
        f32::from(image.pixels[from + 1]),
        f32::from(image.pixels[from + 2]),
        f32::from(image.pixels[from + 3]),
    ]
}

fn sample_linear(image: &Image, s: f32, t: f32) -> [f32; 4] {
    let fx = s * image.width as f32 - 0.5;
    let fy = t * image.height as f32 - 0.5;
    let ix = fx.floor();
    let iy = fy.floor();
    let frac_x = fx - ix;
    let frac_y = fy - iy;
    let mut taps = [(0usize, 0f32); 4];
    for (index, (dx, dy, base_x, base_y)) in [
        (0i64, 0i64, 1.0 - frac_x, 1.0 - frac_y),
        (1, 0, frac_x, 1.0 - frac_y),
        (0, 1, 1.0 - frac_x, frac_y),
        (1, 1, frac_x, frac_y),
    ]
    .into_iter()
    .enumerate()
    {
        let (sx, mirror_x) = wrap_axis(ix as i64 + dx, image.width, image.wrap_s);
        let (sy, mirror_y) = wrap_axis(iy as i64 + dy, image.height, image.wrap_t);
        let wx = if mirror_x > 0.5 { 1.0 - base_x } else { base_x };
        let wy = if mirror_y > 0.5 { 1.0 - base_y } else { base_y };
        taps[index] = (sy * image.width + sx, wx * wy);
    }
    let mut out = [0f32; 4];
    for channel in 0..4 {
        let mut total = 0f32;
        for (from, weight) in taps {
            total += f32::from(image.pixels[from * 4 + channel]) * weight;
        }
        out[channel] = total;
    }
    out
}

const WRAP_REPEAT: u8 = 1;
const WRAP_MIRROR: u8 = 2;

const BIND_BOTTOM: u8 = 1;
const BIND_TOP: u8 = 2;
const BIND_RIGHT: u8 = 4;
const BIND_LEFT: u8 = 8;

fn tex_coord(binding: u8, mirror: u8, rect: [f32; 4], image: &Image) -> [f32; 4] {
    let tumble = mirror & 4 != 0;
    let mirror_x = mirror & 2 != 0;
    let mirror_y = mirror & 1 != 0;
    let (bind_left, bind_right, bind_top, bind_bottom) = if !tumble {
        (
            if mirror_x { binding & BIND_RIGHT != 0 } else { binding & BIND_LEFT != 0 },
            if mirror_x { binding & BIND_LEFT != 0 } else { binding & BIND_RIGHT != 0 },
            if mirror_y { binding & BIND_BOTTOM != 0 } else { binding & BIND_TOP != 0 },
            if mirror_y { binding & BIND_TOP != 0 } else { binding & BIND_BOTTOM != 0 },
        )
    } else {
        (
            if mirror_x { binding & BIND_BOTTOM != 0 } else { binding & BIND_TOP != 0 },
            if mirror_x { binding & BIND_TOP != 0 } else { binding & BIND_BOTTOM != 0 },
            if mirror_y { binding & BIND_LEFT != 0 } else { binding & BIND_RIGHT != 0 },
            if mirror_y { binding & BIND_RIGHT != 0 } else { binding & BIND_LEFT != 0 },
        )
    };
    let (rect_w, rect_h) = if tumble {
        (rect[3], rect[2])
    } else {
        (rect[2], rect[3])
    };
    let (tex_w, tex_h) = if tumble {
        (image.height as f32, image.width as f32)
    } else {
        (image.width as f32, image.height as f32)
    };
    let (mut s0, mut s1) = if bind_left {
        (0.0f32, if bind_right { 1.0 } else { rect_w / tex_w })
    } else if bind_right {
        (1.0 - rect_w / tex_w, 1.0)
    } else {
        let half = rect_w / tex_w / 2.0;
        (0.5 - half, 0.5 + half)
    };
    let (mut t0, mut t1) = if bind_top {
        (0.0f32, if bind_bottom { 1.0 } else { rect_h / tex_h })
    } else if bind_bottom {
        (1.0 - rect_h / tex_h, 1.0)
    } else {
        let half = rect_h / tex_h / 2.0;
        (0.5 - half, 0.5 + half)
    };
    if mirror_x {
        core::mem::swap(&mut s0, &mut s1);
    }
    if mirror_y {
        core::mem::swap(&mut t0, &mut t1);
    }
    [s0, t0, s1, t1]
}

fn draw_text(
    canvas: &mut Canvas,
    fonts: &mut FontBank,
    assets: &Assets,
    node: &ScreenPane,
    pane: Option<&Pane>,
    rect: [f32; 4],
    alpha: f32,
) {
    let text = pane.map_or(node.text.as_str(), |p| p.text.as_str());
    if text.is_empty() || rect[2] <= 0.0 || rect[3] <= 0.0 {
        return;
    }
    let Some(slot) = node.font.as_deref().and_then(|name| assets.font_slot(name)) else {
        return;
    };
    let mut box_ = TextBox::default();
    box_.initiate(
        fonts,
        Some(slot),
        text.as_bytes(),
        node.binding_h,
        node.binding_v,
    );
    box_.set_char_color(node.char_color);
    box_.set_grad_color(node.grad_color);
    if let Some(live) = pane {
        box_.shift_set(live.shift_x, live.shift_y);
    }
    let size_x = pane.map_or(f32::from(node.font_size_x), |p| p.font_size_x);
    let size_y = pane.map_or(f32::from(node.font_size_y), |p| p.font_size_y);
    if size_x > 0.0 {
        box_.set_font_size(size_x, size_y);
    }
    let space = pane.map_or(node.char_space as f32, |p| p.char_space);
    if space != 0.0 {
        box_.set_char_space(space);
    }
    let line = pane.map_or(node.line_space as f32, |p| p.line_space);
    if line > 0.0 {
        box_.set_line_space(line);
    }
    let bounds = [rect[0], rect[1], rect[0] + rect[2], rect[1] + rect[3]];
    let level = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    let layout = box_.draw_self(fonts, bounds, rect[0], rect[1], level);
    if let Some(font) = fonts.get(slot) {
        canvas.draw_layout(&layout, font, [rect[0], rect[1]], level);
    }
}

fn unpack_color(value: u32) -> [u8; 4] {
    [
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    ]
}

fn corner(tint: [[u8; 4]; 4], u: f32, v: f32) -> [f32; 4] {
    let mut out = [0.0f32; 4];
    let weights = [(1.0 - u) * (1.0 - v), u * (1.0 - v), (1.0 - u) * v, u * v];
    for (channel, slot) in out.iter_mut().enumerate() {
        let mut sum = 0.0f32;
        for (index, weight) in weights.iter().enumerate() {
            sum += f32::from(tint[index][channel]) * weight;
        }
        *slot = sum;
    }
    out
}

fn blit(
    canvas: &mut Canvas,
    image: &Image,
    rect: [f32; 4],
    angle: f32,
    tint: [[u8; 4]; 4],
    opacity: f32,
    uv: [f32; 4],
) {
    let opacity = opacity.clamp(0.0, 1.0);
    if opacity <= 0.0 || rect[2] <= 0.0 || rect[3] <= 0.0 {
        return;
    }
    let linear = image.mag_linear;
    if image.width == 0 || image.height == 0 {
        return;
    }
    let cx = rect[0] + rect[2] * 0.5;
    let cy = rect[1] + rect[3] * 0.5;
    let half_w = rect[2] * 0.5;
    let half_h = rect[3] * 0.5;
    let (sin, cos) = angle.sin_cos();
    let radius = (half_w * half_w + half_h * half_h).sqrt();
    let x0 = (cx - radius).floor().max(0.0) as usize;
    let y0 = (cy - radius).floor().max(0.0) as usize;
    let x1 = ((cx + radius).ceil().max(0.0) as usize).min(canvas.width);
    let y1 = ((cy + radius).ceil().max(0.0) as usize).min(canvas.height);
    let span_u = uv[2] - uv[0];
    let span_v = uv[3] - uv[1];
    for y in y0..y1 {
        for x in x0..x1 {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let local_x = cos * dx + sin * dy;
            let local_y = -sin * dx + cos * dy;
            let u = (local_x + half_w) / rect[2];
            let v = (local_y + half_h) / rect[3];
            if !(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v) {
                continue;
            }
            let s = uv[0] + span_u * u;
            let t = uv[1] + span_v * v;
            let texel = if linear {
                sample_linear(image, s, t)
            } else {
                sample_nearest(image, s, t)
            };
            let shade = corner(tint, u, v);
            let alpha = if image.alpha_enabled {
                f32::from(texel[3])
            } else {
                255.0
            } * shade[3]
                / 255.0
                * opacity;
            if alpha <= 0.0 {
                continue;
            }
            let mut color = [0u8; 4];
            for channel in 0..3 {
                color[channel] = (texel[channel] * shade[channel] / 255.0).round() as u8;
            }
            color[3] = 255;
            canvas.blend(x, y, color, alpha);
        }
    }
}

fn live_table<'a>(panes: &'a FileSelectPanes) -> HashMap<&'static str, &'a Pane> {
    let mut table = HashMap::new();
    visit_all(panes, &mut |pane| {
        table.entry(pane.tag).or_insert(pane);
    });
    table
}

fn visit_all<'a>(panes: &'a FileSelectPanes, visit: &mut impl FnMut(&'a Pane)) {
    let slots = &panes.slots;
    for row in &slots.key {
        for pane in row {
            visit(pane);
        }
    }
    for row in &slots.info {
        for pane in row {
            visit(pane);
        }
    }
    for pane in &slots.record {
        visit(pane);
    }
    for pane in &slots.collect {
        visit(pane);
    }
    for pane in &slots.pearl {
        visit(pane);
    }
    for pane in &slots.pearl_shadow {
        visit(pane);
    }
    visit(&slots.triforce_bar);
    for pane in &slots.shard {
        visit(pane);
    }
    for pane in &slots.heart {
        visit(pane);
    }
    for pane in &slots.heart_shadow {
        visit(pane);
    }
    for pane in &slots.slot_mark {
        visit(pane);
    }
    for pane in &slots.slot_mark_key {
        visit(pane);
    }
    let groups = &panes.groups;
    visit(&groups.title_key);
    visit(&groups.title_label);
    for pane in &groups.title_text {
        visit(pane);
    }
    for pane in &groups.message_key {
        visit(pane);
    }
    visit(&groups.message_text);
    for pane in &groups.record_cursor {
        visit(pane);
    }
    visit(&groups.menu_cursor);
    for pane in &groups.yes_key {
        visit(pane);
    }
    visit(&groups.yes_label);
    visit(&groups.yes_text);
    for pane in &groups.no_key {
        visit(pane);
    }
    visit(&groups.no_label);
    visit(&groups.no_text);
    for pane in &groups.start_key {
        visit(pane);
    }
    visit(&groups.start_label);
    for pane in &groups.copy_key {
        visit(pane);
    }
    for pane in &groups.copy_pic {
        visit(pane);
    }
    visit(&groups.copy_label);
    for pane in &groups.erase_key {
        visit(pane);
    }
    for pane in &groups.erase_pic {
        visit(pane);
    }
    visit(&groups.erase_label);
    for pane in &groups.back_key {
        visit(pane);
    }
    visit(&groups.back_pic);
    visit(&groups.back_label);
    visit(&groups.back_text);
}
