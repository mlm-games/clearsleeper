//! Wind Waker recreation on repame: fixed-step sim + repose views, no bevy.

use repame_view3d::{BatchDesc, Frame3d, GeomHandle, Viewport3d};

pub fn root_view(
    _sched: &mut repose_core::runtime::Scheduler,
    _ctx: &repose_core::RenderContext,
) -> repose_core::View {
    Viewport3d(
        Frame3d::default(),
        GeomHandle::new(),
        "scene.main",
        BatchDesc::default(),
        |_ev| {},
    )
}

/// Android entry (`cargo rapk`): the lib target owns `android_main` — a bin
/// target's symbol never lands in the `.so` the NativeActivity loader opens.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    rlobkit_app_events::android_log::init(env!("CARGO_PKG_NAME"), "warn");
    rlobkit_app_events::system_bars::set_immersive_sticky(true);
    repose_core::locals::set_theme_default(repose_core::locals::Theme::default());
    if let Err(e) = repame_shell::run_android(android_app, root_view) {
        log::error!("clearsleeper: android run failed: {e:?}");
    }
}
