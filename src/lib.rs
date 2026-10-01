//! Wind Waker recreation on repame: fixed-step sim + repose views, no bevy.

use std::time::Duration;

use repame_sim::Sim;
use repame_view3d::{BatchDesc, GeomHandle, OrbitCamera, View3dEvent, Viewport3d};
use repose_core::{Signal, signal};

pub mod assets;
pub mod input;
pub mod save;
pub mod sim;
pub mod view;

/// Fixed gameplay tick rate: 30 Hz (NTSC pacing).
pub const SIM_HZ: f64 = 30.0;

pub struct App {
    pub sim: Sim,
    pub cam: Signal<OrbitCamera>,
}

impl App {
    pub fn new() -> Self {
        let mut sim = Sim::new(Duration::from_secs_f64(1.0 / SIM_HZ));
        sim::build(&mut sim);
        let spawn = sim.world.resource::<sim::area::Area>().spawn;
        let cam = signal(OrbitCamera {
            target: spawn,
            ..OrbitCamera::default()
        });
        Self { sim, cam }
    }

    /// Frame boundary: poll the shell into the sim's intent resource.
    pub fn stage_polled(&mut self, sched: &repose_core::runtime::Scheduler) {
        self.stage_axis(input::axis_from_keys(&sched.held_keys, sched.window_focused));
    }

    /// Stage the orbit view yaw into the sim's camera-relative walk frame.
    pub fn stage_camera(&mut self) {
        let cam_yaw = self.cam.get().yaw;
        self.stage_camera_yaw(view::sim_yaw_for_camera(cam_yaw));
    }

    pub fn stage_axis(&mut self, axis: glam::Vec2) {
        self.sim.world.resource_mut::<sim::intent::MoveIntent>().axis = axis;
    }

    pub fn stage_camera_yaw(&mut self, yaw: f32) {
        self.sim.world.resource_mut::<sim::intent::CameraYaw>().0 = yaw;
    }

    pub fn snapshot(&self) -> view::SceneSnapshot {
        let player = self
            .sim
            .world
            .iter_entities()
            .find_map(|entity| entity.get::<sim::player::Player>().copied());
        let area = self.sim.world.resource::<sim::area::Area>();
        let (player_pos, player_yaw) = match player {
            Some(player) => (player.pos, player.yaw),
            None => (area.spawn, 0.0),
        };
        view::SceneSnapshot {
            player_pos,
            player_yaw,
            bounds_min: area.bounds_min,
            bounds_max: area.bounds_max,
        }
    }

    pub fn advance(&mut self, dt: Duration) -> u32 {
        self.sim.step(dt)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

pub fn root_view(
    app: &App,
    _sched: &mut repose_core::runtime::Scheduler,
    _ctx: &repose_core::RenderContext,
) -> repose_core::View {
    let frame = view::build_frame(&app.snapshot(), app.cam.get());
    let cam = app.cam.clone();
    Viewport3d(
        frame,
        GeomHandle::new(),
        "scene.main",
        BatchDesc::default(),
        move |ev| match ev {
            View3dEvent::Orbit { dx, dy } => cam.update(|c| c.orbit(dx, dy)),
            View3dEvent::Zoom { factor } => cam.update(|c| c.zoom(factor)),
            _ => {}
        },
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
    let mut app = App::new();
    let mut last = std::time::Instant::now();
    if let Err(e) = repame_shell::run_android(android_app, move |sched, ctx| {
        let now = std::time::Instant::now();
        let dt = now
            .duration_since(last)
            .min(std::time::Duration::from_secs_f32(0.25));
        last = now;
        app.stage_polled(sched);
        app.stage_camera();
        app.advance(dt);
        root_view(&app, sched, ctx)
    }) {
        log::error!("clearsleeper: android run failed: {e:?}");
    }
}
