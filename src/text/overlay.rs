use bevy_ecs::world::World;

use super::font::{FontBank, ResFont};
use super::msgbox::PaneSetup;
use super::msgbox::TextPrint;
use super::raster::Canvas;
use crate::text::driver::COLOR_TABLE;

const WHITE: u32 = 0xFFFF_FFFF;

pub struct Overlay {
    pub canvas: Canvas,
    pub live: bool,
}

fn pane_origin(pane: &PaneSetup) -> Option<[f32; 2]> {
    pane.visible.then_some([pane.bounds[0], pane.bounds[1]])
}

fn draw_pane(
    canvas: &mut Canvas,
    fonts: &mut FontBank,
    slot: usize,
    text: &str,
    origin: [f32; 2],
    width: f32,
) {
    if text.is_empty() {
        return;
    }
    let height = 32.0;
    let mut print = TextPrint::default();
    print.private_initiate(fonts, Some(slot), 0.0, 0.0, WHITE, WHITE, 0, WHITE, false);
    print.set_font_size(20.0, 20.0);
    print.locate(0.0, height);
    let layout = print.print_return(
        fonts,
        text.as_bytes(),
        width.max(64.0),
        height,
        0,
        0,
        0.0,
        0.0,
        0xFF,
    );
    if let Some(font) = fonts.get(slot) {
        canvas.draw_layout(&layout, font, origin, 0xFF);
    }
}

pub fn render(world: &mut World, width: usize, height: usize) -> Overlay {
    let mut canvas = Canvas::new(width, height);
    let mut fonts = world.remove_resource::<FontBank>().unwrap_or_default();
    let live = world
        .get_resource::<super::driver::TextDriver>()
        .is_some_and(|driver| driver.live);
    if live {
        draw_msg2(world, &mut canvas, &mut fonts);
        draw_msg1(world, &mut canvas, &mut fonts);
    }
    world.insert_resource(fonts);
    let lit = canvas.pixels.chunks_exact(4).any(|pixel| pixel[3] > 0);
    Overlay {
        canvas,
        live: live && lit,
    }
}

fn draw_msg2(world: &mut World, canvas: &mut Canvas, fonts: &mut FontBank) {
    let Some(screen) = world.get_resource::<super::driver::TextScreen>() else {
        return;
    };
    let Some(two) = world.get_resource::<super::msgbox::Msg2Box>() else {
        return;
    };
    if two.msg_no == 0 || two.output_text.iter().all(String::is_empty) {
        return;
    }
    for index in 0..super::msgbox::MSG2_PANES {
        let Some(origin) = pane_origin(&screen.msg2.text[index]) else {
            continue;
        };
        let text = two.output_text[index].clone();
        let width = screen.msg2.text[index].bounds[2] - screen.msg2.text[index].bounds[0];
        draw_pane(canvas, fonts, 0, &text, origin, width);
    }
}

fn draw_msg1(world: &mut World, canvas: &mut Canvas, fonts: &mut FontBank) {
    let Some(screen) = world.get_resource::<super::driver::TextScreen>() else {
        return;
    };
    let Some(one) = world.get_resource::<super::msgbox1::Msg1Box>() else {
        return;
    };
    if one.output_text.is_empty() && one.select_text.is_empty() {
        return;
    }
    for index in 0..4 {
        let Some(pane) = screen.msg1.text[index].as_ref() else {
            continue;
        };
        let Some(origin) = pane_origin(pane) else {
            continue;
        };
        let text = match index {
            0 => one.output_text.clone(),
            _ => one.select_text.clone(),
        };
        let width = pane.bounds[2] - pane.bounds[0];
        draw_pane(canvas, fonts, 0, &text, origin, width);
    }
}

pub fn fallback_font() -> ResFont {
    ResFont::from_bytes(super::fontdata::ASCENT_FIX12)
}

pub fn table_name() -> &'static str {
    COLOR_TABLE
}

/// Bind the retail message window's own pane geometry into the `TextScreen` resource.
/// Without this every dialogue line renders at (0,0) in the default 64px box
/// (`d_msg.cpp:311` binds `hukidashi_00.blo`; HD drives the same panes through
/// `hd_screen_msg_window_main.cpp`).
pub fn bind_message_screens(world: &mut bevy_ecs::prelude::World) {
    use crate::assets::AssetRoot;
    use crate::text::msgbox::MSG2_PANES;

    let Ok(root) = AssetRoot::from_env() else {
        return;
    };
    let Ok(archive) = root.read_archive("files/res/Msg/msgres.arc") else {
        return;
    };
    let msg2 = archive
        .read("hukidashi_00.blo")
        .ok()
        .and_then(crate::j2d::parse);
    let msg1 = archive
        .read("hukidashi_09.blo")
        .ok()
        .and_then(crate::j2d::parse);
    let Some(mut screens) = world.get_resource_mut::<super::driver::TextScreen>() else {
        return;
    };
    if let Some(screen) = msg2 {
        for (index, target) in screens.msg2.text.iter_mut().enumerate().take(MSG2_PANES) {
            if let Some(pane) = screen.panes.get(index) {
                let b = pane.bounds;
                target.bounds = [b.x, b.y, b.x + b.w, b.y + b.h];
                target.alpha = pane.alpha;
                target.visible = pane.visible;
            }
        }
    }
    if let Some(screen) = msg1 {
        for (index, target) in screens.msg1.text.iter_mut().enumerate() {
            if let Some(target) = target.as_mut() {
                if let Some(pane) = screen.panes.get(index) {
                    let b = pane.bounds;
                    target.bounds = [b.x, b.y, b.x + b.w, b.y + b.h];
                    target.alpha = pane.alpha;
                    target.visible = pane.visible;
                }
            }
        }
    }
}
