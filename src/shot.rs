use crate::menus;
use repose_core::RenderContext;
use repose_render_wgpu::apply_render_commands;
use repose_render_wgpu::offscreen::OffscreenRenderer;
use repose_ui::{Interactions, TextFieldState, layout_and_paint};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub const W: u32 = 640;
pub const H: u32 = 480;

/// Renders a settled file select through the same path the game uses -
/// `file_select_view` -> `layout_and_paint` -> offscreen WGPU - and returns
/// tightly packed RGBA8. Only the window is missing.
pub fn file_select_rgba(frames: u32) -> anyhow::Result<Vec<u8>> {
    let mut menu = menus::Menu::default();
    menu.open_file_select();

    let mut settled = 0;
    while !menu.fs.open() && settled < frames {
        settled += 1;
        menu.sync_file_select_cursor();
    }

    let ctx = RenderContext::new();
    let view = menus::file_select_view(&menu, &ctx);
    let states: HashMap<u64, Rc<RefCell<TextFieldState>>> = HashMap::new();
    let (scene, _, _) = layout_and_paint(&view, (W, H), &states, &Interactions::default(), None);

    let mut gpu = OffscreenRenderer::new_blocking(W, H, 1)?;
    apply_render_commands(gpu.renderer_mut(), ctx.drain()).log_failures();
    gpu.render_rgba(&scene, Some([0.0, 0.0, 0.0, 1.0]))
}
