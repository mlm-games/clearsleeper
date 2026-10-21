//! Wind Waker recreation on repame: fixed-step sim + repose views (no direct bevy).

use std::cell::Cell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Without, World};
use repame_sim::Sim;
use repame_view3d::{GeomHandle, OrbitCamera, View3dEvent, Viewport3d};
use repose_core::input::PhysicalKey;
use repose_core::{ImageFit, Modifier, Signal, signal};
use repose_ui::{Image, ImageExt, Spacer, ViewExt, ZStack};

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_os = "android")]
mod android;

pub mod archive;
pub mod assets;
pub mod audio;
pub mod bti;
pub mod config;
pub mod dff;
pub mod disc;
pub mod dzs;
pub mod game_over_screen;
pub mod gx_decode;
pub mod input;
pub mod input_tape;
pub mod intro;
pub mod j2d;
pub mod j3d;
pub mod menus;
pub mod meter_hud;
pub mod movie;
pub mod panes;
pub mod quit;
pub mod report;
pub mod save;
pub mod shot;
pub mod sim;
pub mod source;
pub mod text;
pub mod title_screen;
pub mod trace;
pub mod ui_file_select;
pub mod ui_menu_collect;
pub mod ui_menu_dmap;
pub mod ui_menu_fmap;
pub mod ui_menu_fmap2;
pub mod ui_menu_item;
pub mod ui_menu_option;
pub mod ui_menu_save;
pub mod ui_meter_gauges;
pub mod ui_meter_vitals;
pub mod ui_meter_widgets;
pub mod ui_number;
pub mod ui_overlay;
pub mod view;

/// Fixed gameplay tick rate: 30 Hz (NTSC pacing).
pub const SIM_HZ: f64 = 30.0;

pub struct App {
    pub sim: Sim,
    pub cam: Signal<OrbitCamera>,
    pub audio: repame_audio::Audio,
    pub room: Option<j3d::bdl::RoomMesh>,
    pub actor_model: Option<j3d::bdl::RoomMesh>,
    /// `katsura.bdl`; retail attaches it to `CL_JNT_HEAD_JNT` rather than drawing the hair shape
    /// in `cl.bdl` (`d_a_player_main.cpp:11598`, `:11934`).
    pub hair_model: Option<j3d::bdl::RoomMesh>,
    /// The player model's standing pose from `LkAnm.arc`; absent on a bind-pose-only source.
    pub player_anim: Option<assets::PlayerIdle>,
    /// A `.dff` recording shown instead of the world, from `CS_DFF`. Its pages
    /// are its own, so it is presented alone rather than mixed into a room.
    dff: Option<j3d::present::Presented>,
    pub menu: menus::Menu,
    stage: Option<dzs::StageTables>,
    rooms: Vec<u8>,
    /// The mounted room set's own `SCLS` tables, keyed by room: the table
    /// `dLib_setNextStageBySclsNum` reads through `dComIfG_roomControl_getStatusRoomDt(room)`
    /// (`d_lib.cpp:245-246`). [`sim::transition::NextStage::scls`] is the request these resolve.
    room_exits: Vec<(u8, Vec<dzs::StageExit>)>,
    /// `dComIfGp_getStartStageLayer` (`d_com_inf_game.h:285`): the layer the stage was
    /// entered on, which every room read of the session keeps.
    start_layer: i8,
    room_tick: u64,
    room_actors: Vec<Entity>,
    sea_shade: f32,
    save_dir: PathBuf,
    name: String,
    menu_prev: HashSet<PhysicalKey>,
    pad_prev: input::ButtonBits,
    /// The input log the session writes, when `CS_INPUT_LOG` names a file. Each line is a
    /// tape line, so the session replays through `CS_INPUT`.
    input_log: Option<std::fs::File>,
    /// The keys the last poll held, as the log's bit mask.
    input_keys: u64,
    /// The frames the session has drawn, the retrace the input log numbers.
    frame: u64,
    /// The last input the web build logged, so the console gets a line per change.
    input_state: u64,
    movie: Option<movie::Movie>,
    movie_frame: usize,
    movie_clock: f32,
    intro_clock: f32,
    movie_pixels: Vec<u8>,
    movie_size: (u32, u32),
    movie_sound: bool,
    movie_done: bool,
    pub intro: Option<IntroStage>,
    /// True once an asset source (disc image or the development tree) is installed. The shell's
    /// own screens are pre-boot only: from here on every frame is the game's own rendering.
    assets_installed: bool,
    title_stage: Option<TitleStage>,
    meter: Option<MeterStage>,
    game_over: Option<GameOverStage>,
    game_over_failed: bool,
    title_actor: Option<Entity>,
    pending_slot: Option<usize>,
    dialogue_canvas: text::raster::Canvas,
    dialogue_live: bool,
    dialogue_image: Cell<Option<repose_core::ImageHandle>>,
    dialogue_uploaded: Cell<u64>,
    dialogue_revision: u64,
    quit: quit::Quit,
}

pub struct IntroStage {
    pub intro: intro::Intro,
    screen: j2d::Screen,
    images: intro::PaneImages,
    fonts: text::font::FontBank,
    archives: assets::MsgArchives,
    canvas: text::raster::Canvas,
    image: Cell<Option<repose_core::ImageHandle>>,
    uploaded: Cell<u64>,
    revision: u64,
}

struct TitleStage {
    assets: title_screen::TitleAssets,
    canvas: text::raster::Canvas,
    image: Cell<Option<repose_core::ImageHandle>>,
    uploaded: Cell<u64>,
    revision: u64,
}

struct MeterStage {
    assets: meter_hud::MeterAssets,
    meter: meter_hud::Meter,
    canvas: text::raster::Canvas,
    image: Cell<Option<repose_core::ImageHandle>>,
    uploaded: Cell<u64>,
    revision: u64,
    painted: Option<Vec<(String, [u32; 4], u32)>>,
}

struct GameOverStage {
    game: game_over_screen::GameOver,
    assets: game_over_screen::GameOverAssets,
    save_assets: panes::SaveAssets,
    flags: ui_menu_save::MenuFlags,
    system: ui_menu_save::SystemRequest,
    particles: ui_menu_save::Particles,
    fade: f32,
    image: Cell<Option<repose_core::ImageHandle>>,
    uploaded: Cell<u64>,
    revision: u64,
}

/// The input log `CS_INPUT_LOG` names, emptied when the session opens it. A line is a tape
/// line, so the session replays through `CS_INPUT` exactly as it was played.
fn open_input_log() -> Option<std::fs::File> {
    let path = std::env::var_os("CS_INPUT_LOG").filter(|value| !value.is_empty())?;
    let file = std::fs::File::create(path).ok()?;
    Some(file)
}

impl App {
    /// Headless construction: silent audio, no device. Tests and tools.
    /// The headless session: what the tests and headless runs drive. A session exists only with
    /// an asset source, so the source counts as installed; [`new_live`] reads the real one.
    pub fn new() -> Self {
        let mut app = Self::with_audio(repame_audio::Audio::noop());
        app.assets_installed = true;
        app.menu.screen = menus::Screen::Playing;
        app.sync_title_actor();
        app
    }

    /// The shell without a source: the screens a launch shows when no disc is found, held up
    /// until one is installed. `new` is the session because it needs a source.
    pub fn new_pre_boot() -> Self {
        let mut app = Self::with_audio(repame_audio::Audio::noop());
        app.assets_installed = false;
        app.menu.screen = menus::Screen::Title;
        app
    }

    /// Shell construction: opens the output device, silent fallback.
    pub fn new_live() -> Self {
        Self::with_audio(audio::open())
    }

    fn with_audio(mut audio: repame_audio::Audio) -> Self {
        if let Err(e) = audio::load_cues(&mut audio) {
            log::warn!("clearsleeper: cue bank failed to load ({e})");
        }
        let sea_shade = assets::load_sea_shade().unwrap_or(0.0);
        let mut app = Self {
            sim: Sim::new(Duration::from_secs_f64(1.0 / SIM_HZ)),
            cam: signal(OrbitCamera::default()),
            audio,
            room: None,
            actor_model: None,
            hair_model: None,
            player_anim: None,
            dff: None,
            menu: menus::Menu::new(),
            stage: None,
            rooms: Vec::new(),
            room_exits: Vec::new(),
            start_layer: sim::transition::LAYER_NONE,
            room_tick: 0,
            room_actors: Vec::new(),
            sea_shade,
            save_dir: save::slots_dir(),
            name: save::DEFAULT_NAME.to_string(),
            menu_prev: HashSet::new(),
            pad_prev: input::ButtonBits::default(),
            input_log: open_input_log(),
            input_keys: 0,
            frame: 0,
            input_state: 0,
            movie: None,
            movie_frame: 0,
            movie_clock: 0.0,
            intro_clock: 0.0,
            movie_pixels: Vec::new(),
            movie_size: (0, 0),
            movie_sound: false,
            movie_done: false,
            intro: None,
            assets_installed: assets::AssetRoot::from_env().is_ok(),
            title_stage: None,
            meter: None,
            game_over: None,
            game_over_failed: false,
            title_actor: None,
            pending_slot: None,
            dialogue_canvas: text::raster::Canvas::new(640, 480),
            dialogue_live: false,
            dialogue_image: Cell::new(None),
            dialogue_uploaded: Cell::new(0),
            dialogue_revision: 0,
            quit: quit::Quit::default(),
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            app.dff = std::env::var_os("CS_DFF").map(|path| j3d::present::load(path.as_ref()));
        }
        app.movie = movie::title_path().and_then(|path| movie::Movie::open(&path).ok());
        if app.movie.is_none() {
            // A disc image has no path for the file handle, so the same entry is read as bytes.
            app.movie = assets::AssetRoot::from_env()
                .ok()
                .and_then(|root| root.read(movie::TITLE_PATH).ok())
                .and_then(|bytes| movie::Movie::from_bytes(bytes).ok());
        }
        app.title_stage = assets::AssetRoot::from_env()
            .ok()
            .and_then(|root| title_screen::TitleAssets::load(&root))
            .map(|assets| TitleStage {
                assets,
                canvas: text::raster::Canvas::new(640, 480),
                image: Cell::new(None),
                uploaded: Cell::new(0),
                revision: 0,
            });
        app.meter = assets::AssetRoot::from_env()
            .ok()
            .and_then(|root| meter_hud::MeterAssets::load(&root))
            .map(|assets| MeterStage {
                assets,
                meter: meter_hud::Meter::default(),
                canvas: text::raster::Canvas::new(640, 480),
                image: Cell::new(None),
                uploaded: Cell::new(0),
                revision: 0,
                painted: None,
            });
        app.actor_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_MODEL_ENTRY);
        if app.actor_model.is_none() {
            log::warn!("clearsleeper: actor model unavailable");
        }
        app.hair_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_HAIR_ENTRY);
        app.player_anim = app.actor_model.as_ref().and_then(assets::load_link_idle);
        if app.player_anim.is_none() {
            log::warn!("clearsleeper: player idle animation unavailable");
        }
        app.reset_world();
        app.sync_title_actor();
        app
    }

    fn reset_world(&mut self) {
        self.reset_world_at(assets::SEA_STAGE, assets::STAGE_SPAWN_POINT);
    }

    /// The orbit is the frame before `camera::track` has published a view. Point it at the
    /// player's torso rather than the shell default's feet; the retail camera takes over on the
    /// next tick.
    pub(crate) fn place_camera(&mut self, spawn: glam::Vec3, yaw: f32, prologue: bool) {
        let default = OrbitCamera::default();
        self.cam.set(OrbitCamera {
            target: spawn + glam::Vec3::Y * 70.0,
            yaw: if prologue {
                yaw + std::f32::consts::PI
            } else {
                default.yaw
            },
            pitch: if prologue { 0.3 } else { default.pitch },
            dist: if prologue { 420.0 } else { default.dist },
            fov_y_deg: if prologue { 55.0 } else { default.fov_y_deg },
            ..default
        });
    }

    fn reset_world_at(&mut self, stage: &str, spawn_point: u8) {
        self.title_actor = None;
        self.sim = Sim::new(Duration::from_secs_f64(1.0 / SIM_HZ));
        self.stage = assets::load_stage_tables(stage);
        sim::build(
            &mut self.sim,
            assets::load_stage_at(stage, spawn_point),
            self.stage.as_ref(),
        );
        self.rooms = self
            .stage
            .as_ref()
            .map(assets::start_set)
            .unwrap_or_default();
        self.start_layer = sim::transition::LAYER_NONE;
        let stories = *self.sim.world.resource::<sim::flags::StoryBits>();
        let set = self.stage.as_ref().and_then(|tables| {
            load_rooms(tables, &self.rooms, stories, sim::transition::LAYER_NONE)
        });
        let particle_no = self
            .stage
            .as_ref()
            .and_then(|tables| tables.stag())
            .map_or(sim::jpa::NO_SCENE, |stag| stag.particle_no());
        let room_particle_no = set.as_ref().and_then(|set| set.particle_no);
        self.mount_scene_particles(particle_no, room_particle_no);
        self.sim.world.insert_resource(sim::door::StayRoom(-1));
        self.room_tick = 0;
        self.room_actors.clear();
        let area = self.sim.world.resource::<sim::area::Area>();
        let spawn = area.spawn;
        let spawn_yaw = area.yaw;
        self.place_camera(spawn, spawn_yaw, stage == assets::PROLOGUE_STAGE);
        if let Some(set) = set {
            self.install_room_set(set, 0);
        }
        trace::install(&mut self.sim);
    }

    /// Frame boundary: poll the shell into the sim's intent resources.
    pub fn stage_polled(&mut self, sched: &repose_core::runtime::Scheduler) {
        let held = &sched.held_keys;
        self.stage_keys(held, sched.window_focused);
    }

    /// The shell's keys as the game reads them: the move axis, the debug and action keys, the
    /// pad's face buttons and the menus' commands. The input log replays a session through
    /// this path, so a logged frame and a live frame stage the same thing.
    pub fn stage_keys(&mut self, held: &HashSet<PhysicalKey>, window_focused: bool) {
        self.input_keys = input::key_mask(held);
        self.stage_axis(input::axis_from_keys(held, window_focused));
        self.stage_damage(input::damage_from_keys(held, window_focused));
        self.stage_do(input::do_from_keys(held, window_focused));
        self.stage_target(input::target_from_keys(held, window_focused));
        self.stage_item(input::item_from_keys(held, window_focused));
        self.stage_use(input::use_from_keys(held, window_focused));
        self.stage_pad(
            input::pad_held_from_keys(held, window_focused),
            input::c_stick_from_keys(held, window_focused),
        );
        self.stage_menu(held, window_focused);
    }

    /// One tape line for the frame's input, written to the log `CS_INPUT_LOG` names. The
    /// line is the pad word, the stick and the key mask, which is what a replay needs to
    /// stage the same frame again.
    fn log_input(&mut self) {
        use std::io::Write;
        let tick = self.frame;
        let hold = *self
            .sim
            .world
            .get_resource_or_insert_with(sim::intent::PadButtons::default);
        let stick = *self
            .sim
            .world
            .get_resource_or_insert_with(sim::intent::CStick::default);
        let mut word = 0_u16;
        if hold.hold.a {
            word |= input_tape::BUTTON_A;
        }
        if hold.hold.b {
            word |= input_tape::BUTTON_B;
        }
        if hold.hold.x {
            word |= input_tape::BUTTON_X;
        }
        if hold.hold.y {
            word |= input_tape::BUTTON_Y;
        }
        if hold.hold.z {
            word |= input_tape::BUTTON_Z;
        }
        if hold.hold.l {
            word |= input_tape::BUTTON_L;
        }
        if hold.hold.r {
            word |= input_tape::BUTTON_R;
        }
        if hold.hold.start {
            word |= input_tape::BUTTON_START;
        }
        let axis = self
            .sim
            .world
            .get_resource_or_insert_with(sim::intent::MoveIntent::default)
            .axis;
        let stick_x = (axis.x * 127.0).round() as i8;
        let stick_y = (axis.y * 127.0).round() as i8;
        let c_x = (stick.axis.x * 127.0).round() as i8;
        let c_y = (stick.axis.y * 127.0).round() as i8;
        let line = format!(
            "retrace={tick} button={word:#06x} stick={stick_x},{stick_y} c={c_x},{stick_y} l=0 r=0 keys={:#x}\n",
            self.input_keys
        );
        #[cfg(target_arch = "wasm32")]
        {
            // The console gets the line when the input changes, so a session's log stays
            // readable; the desktop's file gets every frame, so a replay is exact.
            let state = (word as u64) << 32 | self.input_keys;
            if state != self.input_state {
                self.input_state = state;
                crate::web::log_line(&line);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(file) = self.input_log.as_mut() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }

    /// Stage the pad state one tape line carries: the main stick as the move axis, the face
    /// buttons as the pad's hold word and the C-stick as the sub-stick. The trigger word is
    /// the edge against the previous poll, the way `mDoCPd_Convert` splits the pad's hold and
    /// trigger words (`m_Do/m_Do_controller_pad.cpp:20-45`).
    pub fn stage_pad_state(&mut self, state: &input_tape::PadState) {
        if state.keys_given {
            let held = input::keys_from_mask(state.keys);
            self.stage_keys(&held, true);
            return;
        }
        self.stage_axis(state.move_axis());
        self.stage_pad(state.hold(), state.c_axis());
    }

    /// Stage the pad's face buttons and C-stick. The trigger word is the edge against the
    /// previous poll, so it lasts one frame whatever the fixed step then does with it and a
    /// held button never reads as a fresh press.
    fn stage_pad(&mut self, held: input::ButtonBits, c_stick: glam::Vec2) {
        let prev = std::mem::replace(&mut self.pad_prev, held);
        *self
            .sim
            .world
            .get_resource_or_insert_with(sim::intent::PadButtons::default) =
            sim::intent::PadButtons {
                trig: input::pad_trig(held, prev),
                hold: held,
            };
        self.sim
            .world
            .get_resource_or_insert_with(sim::intent::CStick::default)
            .poll(c_stick);
    }

    /// Stage the pad for whichever screen owns the frame. The shell's screens take the pad only
    /// while no asset source is installed; from then on the retail screens read it themselves
    /// (the title actor, the game-over sequence) or through the ported menu state machine
    /// behind the file select panes.
    fn stage_menu(&mut self, held: &HashSet<PhysicalKey>, window_focused: bool) {
        if self.intro.is_some() || self.menu.screen != menus::Screen::Playing {
            self.stage_axis(glam::Vec2::ZERO);
            self.stage_damage(0);
            self.stage_do(false);
            self.stage_target(false);
            self.stage_item(false);
            self.stage_use(false);
            if self.menu.screen != menus::Screen::GameOver {
                self.stage_pad_menu();
            }
            if self.menu.screen == menus::Screen::Title
                && self.assets_installed
                && self.intro.is_none()
            {
                self.stage_pad_title(held, window_focused);
            }
        }
        if self.intro.is_some() {
            self.menu_prev.clone_from(held);
            return;
        }
        let cmds = if window_focused {
            menus::cmds_from(held, &self.menu_prev)
        } else {
            Vec::new()
        };
        self.menu_prev.clone_from(held);
        if !self.assets_installed {
            self.stage_host_screens(&cmds);
            return;
        }
        match self.menu.screen {
            menus::Screen::FileSelect => self.stage_file_select(&cmds),
            menus::Screen::Settings | menus::Screen::QuitConfirm => {
                // Only the pre-boot shell enters these; return to the retail title instead of
                // drawing the shell over a session that has the disc.
                self.menu.screen = menus::Screen::Title;
            }
            _ => {}
        }
    }

    /// The shell's own screens, pre-boot only: the title menu, options and quit. No action here
    /// starts a session - the retail game needs an asset source (`start_new`).
    fn stage_host_screens(&mut self, cmds: &[menus::Cmd]) {
        if cmds.is_empty() && !self.menu.has_pending_click() {
            return;
        }
        let before = self.menu.screen;
        let action = self.menu.step(cmds);
        if before == menus::Screen::Title && self.menu.screen == menus::Screen::FileSelect {
            // The retail file select needs the disc's panes; without a source it is not a
            // screen the shell can show.
            self.menu.screen = menus::Screen::Title;
        }
        match action {
            Some(menus::Action::Quit) => self.quit.request(),
            Some(menus::Action::Load(_) | menus::Action::New(_)) => {
                self.menu.screen = menus::Screen::Title;
            }
            None => {}
        }
    }

    /// The ported menu state machine that drives the retail file select panes: choosing a slot
    /// loads its save or starts a new game.
    fn stage_file_select(&mut self, cmds: &[menus::Cmd]) {
        self.menu.tick_file_select();
        if cmds.is_empty() && !self.menu.has_pending_click() {
            return;
        }
        let action = self.menu.step(cmds);
        if self.menu.screen == menus::Screen::FileSelect {
            self.menu.sync_file_select_cursor();
        }
        self.apply_menu_action(action);
    }

    fn apply_menu_action(&mut self, action: Option<menus::Action>) {
        match action {
            Some(menus::Action::Load(slot)) => match self.read_slot(slot) {
                Some(saved) => match self.load(&saved) {
                    Ok(()) => {
                        self.name = saved.name;
                        self.menu.screen = menus::Screen::Playing;
                    }
                    Err(e) => log::warn!("clearsleeper: slot {slot} load failed ({e})"),
                },
                None => self.start_new(slot),
            },
            Some(menus::Action::New(slot)) => self.start_new(slot),
            Some(menus::Action::Quit) => self.quit.request(),
            None => {}
        }
    }

    fn start_new(&mut self, slot: usize) {
        if !self.assets_installed {
            self.menu.screen = menus::Screen::Title;
            return;
        }
        self.reset_world_at(assets::PROLOGUE_STAGE, assets::PROLOGUE_SPAWN_POINT);
        self.pending_slot = Some(slot);
        self.begin_intro();
        // A new game resets into the opening scene, `d_s_play.cpp:305`'s `dComIfG_resetToOpening`
        // (`:1237`, `fpcNm_OPENING_SCENE_e`), and the shell plays that over the title; when the
        // opening cannot start there is nothing left to show and `enter_play` has already put the
        // game in play and written the slot.
        if self.intro.is_some() {
            self.menu.screen = menus::Screen::Title;
        }
    }

    fn begin_intro(&mut self) {
        let Some(root) = assets::AssetRoot::from_env().ok() else {
            self.enter_play();
            return;
        };
        let Ok(arc) = root.read_archive("files/res/Object/Opening.arc") else {
            self.enter_play();
            return;
        };
        let Some(blo) = arc.read("opening.blo").ok() else {
            self.enter_play();
            return;
        };
        let Some(mut screen) = j2d::parse(blo) else {
            self.enter_play();
            return;
        };
        intro::usa_text_box(&mut screen);
        let images = intro::PaneImages::load(&arc);
        let archives = assets::MsgArchives::load(&root);
        let mut fonts = text::font::FontBank::new();
        text::driver::load_fonts(&mut fonts);
        let mut machine = intro::Intro::default();
        machine.start(&mut fonts, &archives);
        self.intro_clock = 0.0;
        self.intro = Some(IntroStage {
            intro: machine,
            screen,
            images,
            fonts,
            archives,
            canvas: text::raster::Canvas::new(640, 480),
            image: Cell::new(None),
            uploaded: Cell::new(0),
            revision: 0,
        });
    }

    fn enter_play(&mut self) {
        self.intro = None;
        if !self.assets_installed {
            // No disc, no session: the shell stays up until a source is installed.
            self.menu.screen = menus::Screen::Title;
            return;
        }
        self.menu.screen = menus::Screen::Playing;
        if let Some(slot) = self.pending_slot.take() {
            if let Err(e) = self.write_slot(slot) {
                log::warn!("clearsleeper: slot {slot} write failed ({e})");
            }
        }
    }

    /// The opening scene's `execute` runs once a 30 Hz frame (`d_s_open.cpp:67-72`), so its
    /// slides step with the sim clock, not with the display's refresh rate.
    fn step_intro(&mut self, dt: Duration) {
        self.intro_clock += dt.as_secs_f32();
        let step = 1.0 / SIM_HZ as f32;
        let mut budget = 4;
        let mut stepped = false;
        while self.intro_clock >= step && budget > 0 {
            self.intro_clock -= step;
            budget -= 1;
            stepped = true;
            let done = {
                let Some(stage) = self.intro.as_mut() else {
                    return;
                };
                stage.intro.step(&mut stage.fonts, &stage.archives);
                stage.intro.done
            };
            if done {
                self.enter_play();
                return;
            }
        }
        if budget == 0 {
            self.intro_clock = 0.0;
        }
        if !stepped {
            return;
        }
        let Some(stage) = self.intro.as_mut() else {
            return;
        };
        stage.canvas = text::raster::Canvas::new(640, 480);
        let IntroStage {
            intro,
            screen,
            images,
            fonts,
            canvas,
            ..
        } = stage;
        let layout: &j2d::Screen = screen;
        intro::render(canvas, layout, intro, fonts, &|c, n, b, a| {
            intro::draw_pane(c, layout, images, n, b, a);
        });
        stage.revision = stage.revision.wrapping_add(1);
    }

    fn step_movie(&mut self, dt: Duration) {
        if self.movie.is_none() {
            return;
        }
        self.start_movie_sound();
        if self.movie_pixels.is_empty() {
            self.movie_frame = 0;
        } else {
            let interval = self.movie_interval();
            self.movie_clock += dt.as_secs_f32();
            if self.movie_clock >= interval {
                self.movie_clock -= interval;
                let frames = self.movie.as_ref().map_or(1, movie::Movie::len).max(1);
                let next = self.movie_frame + 1;
                if next >= frames {
                    self.audio.play(audio::MOVIE_CUE);
                    self.movie_done = true;
                }
                self.movie_frame = next % frames;
            } else {
                return;
            }
        }
        self.decode_movie_frame();
    }

    fn movie_interval(&self) -> f32 {
        self.movie
            .as_ref()
            .map_or(1.0 / 30.0, movie::Movie::interval)
    }

    /// `daTitle_c::execute` owns the title actor while the scene is up; a request for
    /// `fpcNm_NAME_SCENE_e` (`d_a_title.cpp:657`) is the transition to the file select.
    fn handle_title_request(&mut self) {
        if self.menu.screen != menus::Screen::Title || !self.assets_installed {
            return;
        }
        let request = self.sim.world.resource::<sim::title::TitleScene>().request;
        let Some(request) = request else {
            return;
        };
        if request.scene_proc != sim::title::FPC_NM_NAME_SCENE {
            return;
        }
        self.sim
            .world
            .resource_mut::<sim::title::TitleScene>()
            .request = None;
        self.menu.screen = menus::Screen::FileSelect;
        let slots = (0..save::SLOTS)
            .map(|slot| match self.read_slot(slot) {
                Some(saved) => menus::SlotInfo::from_save(&saved),
                None => menus::SlotInfo::Empty,
            })
            .collect();
        self.menu.slots = slots;
        self.menu.open_file_select();
        self.menu.sync_file_select_cursor();
    }

    /// The actor and its scene request are rebuilt on every entry, as the title scene is
    /// recreated after `dComIfG_resetToOpening` (`d_s_title.cpp:48`, `d_s_play.cpp:305`).
    fn sync_title_actor(&mut self) {
        let wanted = self.menu.screen == menus::Screen::Title
            && self.intro.is_none()
            && self.title_stage.is_some();
        if !wanted {
            if let Some(entity) = self.title_actor.take() {
                self.sim.world.despawn(entity);
            }
            return;
        }
        if self.title_actor.is_some() {
            return;
        }
        let actor = sim::actor::Actor {
            name: sim::title::TITLE_ACTOR.to_string(),
            ..Default::default()
        };
        self.title_actor = Some(
            self.sim
                .world
                .spawn((actor, sim::title::Title::spawn(0)))
                .id(),
        );
        *self
            .sim
            .world
            .get_resource_or_insert_with(sim::title::TitleScene::default) =
            sim::title::TitleScene::default();
        *self
            .sim
            .world
            .get_resource_or_insert_with(sim::title::TitleKirakira::default) =
            sim::title::TitleKirakira::default();
        self.movie_frame = 0;
        self.movie_clock = 0.0;
        self.movie_done = false;
        self.movie_pixels.clear();
        self.decode_movie_frame();
    }

    fn decode_movie_frame(&mut self) {
        if let Some((w, h, pixels)) = self
            .movie
            .as_mut()
            .and_then(|m| m.frame(self.movie_frame).ok())
        {
            self.movie_size = (w, h);
            self.movie_pixels = movie::to_rgba(&pixels);
        }
    }

    fn step_title(&mut self) {
        if self.menu.screen != menus::Screen::Title {
            return;
        }
        let title = {
            let mut query = self.sim.world.query::<&sim::title::Title>();
            query.iter(&self.sim.world).next().copied()
        };
        let Some(title) = title else {
            return;
        };
        let Some(stage) = self.title_stage.as_mut() else {
            return;
        };
        let movie = if self.movie_pixels.is_empty() {
            None
        } else {
            Some((
                self.movie_pixels.as_slice(),
                self.movie_size.0,
                self.movie_size.1,
            ))
        };
        title_screen::render(&mut stage.canvas, &stage.assets, &title, movie);
        stage.revision = stage.revision.wrapping_add(1);
    }

    /// `dMeter_heartMove`/`dMeter_heartDraw` (`d_meter.cpp:1581-1605`) once per tick while the
    /// game is on screen; the canvas is repainted only when the row it draws changed.
    fn step_meter(&mut self) {
        if self.menu.screen != menus::Screen::Playing || self.intro.is_some() {
            return;
        }
        let health = *self.sim.world.resource::<sim::health::Health>();
        let Some(stage) = self.meter.as_mut() else {
            return;
        };
        stage.meter.update(health, &stage.assets);
        let key = stage.meter.row_key();
        if stage.painted.as_ref() == Some(&key) {
            return;
        }
        stage.painted = Some(key);
        stage.canvas.clear();
        stage.meter.render(&mut stage.canvas, &stage.assets);
        stage.revision = stage.revision.wrapping_add(1);
    }

    fn start_movie_sound(&mut self) {
        if self.movie_sound {
            return;
        }
        self.movie_sound = true;
        let Some(wave) = self.movie.as_mut().and_then(|m| m.soundtrack().ok()) else {
            return;
        };
        if self
            .audio
            .load_cue(audio::MOVIE_CUE, audio::movie_cue(), &[&wave])
            .is_ok()
        {
            self.audio.play(audio::MOVIE_CUE);
        }
    }

    /// Stage the orbit view yaw into the sim's camera-relative walk frame.
    pub fn stage_camera(&mut self) {
        let cam_yaw = self.cam.get().yaw;
        self.stage_camera_yaw(view::sim_yaw_for_camera(cam_yaw));
    }

    pub fn stage_axis(&mut self, axis: glam::Vec2) {
        self.sim
            .world
            .resource_mut::<sim::intent::MoveIntent>()
            .axis = axis;
    }

    pub fn stage_camera_yaw(&mut self, yaw: f32) {
        self.sim.world.resource_mut::<sim::intent::CameraYaw>().0 = yaw;
    }

    pub fn stage_damage(&mut self, amount: u8) {
        self.sim.world.resource_mut::<sim::intent::DamageIntent>().0 = amount;
    }

    pub fn stage_do(&mut self, held: bool) {
        self.sim.world.resource_mut::<sim::intent::DoIntent>().0 = held;
    }

    pub fn stage_item(&mut self, held: bool) {
        self.sim.world.resource_mut::<sim::intent::ItemIntent>().0 = held;
    }

    pub fn stage_use(&mut self, held: bool) {
        self.sim.world.resource_mut::<sim::intent::UseIntent>().0 = held;
    }

    pub fn stage_target(&mut self, held: bool) {
        self.sim.world.resource_mut::<sim::intent::TargetIntent>().0 = held;
    }

    /// Release the pad and centre the C-stick while a menu owns the screen, leaving START
    /// readable: retail's START is both the edge that opens a pause menu and the edge that
    /// closes it. The previous poll is left alone, so a button still down when the menu
    /// closes does not report a fresh press.
    fn stage_pad_menu(&mut self) {
        {
            let mut pad = self
                .sim
                .world
                .get_resource_or_insert_with(sim::intent::PadButtons::default);
            let hold = pad.hold.except_start();
            let trig = pad.trig.except_start();
            pad.hold = hold;
            pad.trig = trig;
        }
        self.sim
            .world
            .get_resource_or_insert_with(sim::intent::CStick::default)
            .poll(glam::Vec2::ZERO);
    }

    /// The retail title checks pad A, B and START (`d_a_title.cpp:638-659`): the menu's
    /// Space/Enter confirm keys read as A for one frame, and Escape is already `PAD_START`.
    fn stage_pad_title(&mut self, held: &HashSet<PhysicalKey>, window_focused: bool) {
        if self.title_actor.is_none() {
            return;
        }
        let pressed = window_focused
            && (held.contains(&PhysicalKey::Space) || held.contains(&PhysicalKey::Enter));
        let previous = window_focused
            && (self.menu_prev.contains(&PhysicalKey::Space)
                || self.menu_prev.contains(&PhysicalKey::Enter));
        let mut pad = self
            .sim
            .world
            .get_resource_or_insert_with(sim::intent::PadButtons::default);
        pad.hold.a |= pressed;
        pad.trig.a |= pressed && !previous;
    }

    /// Load the game over sequence's assets: `Gover.arc` for the letters and
    /// `saveres.arc` for the save screen, the two resources `dGameover_c::_create`
    /// pulls (`d_gameover.cpp:68`, `:82`, `:84-92`). Without them the frame is the only
    /// thing on screen: the shell has no screen once a disc exists.
    fn open_game_over(&mut self) -> bool {
        if self.game_over.is_some() {
            return true;
        }
        let Ok(root) = assets::AssetRoot::from_env() else {
            return false;
        };
        let Some(assets) = game_over_screen::GameOverAssets::load(&root) else {
            return false;
        };
        let Some(save_assets) = panes::SaveAssets::load(&root) else {
            return false;
        };
        let mut game = game_over_screen::GameOver::default();
        game.create(&assets.assets.screen);
        game.create_save(save_assets.assets.screen.clone());
        self.menu.card.dir = self.save_dir.clone();
        self.menu.card.restore();
        self.game_over = Some(GameOverStage {
            game,
            assets,
            save_assets,
            flags: ui_menu_save::MenuFlags::default(),
            system: ui_menu_save::SystemRequest::default(),
            particles: ui_menu_save::Particles::default(),
            fade: 0.0,
            image: Cell::new(None),
            uploaded: Cell::new(0),
            revision: 1,
        });
        true
    }

    /// One frame of `dGameover_c::_execute` (`d_gameover.cpp:109-166`), fed by
    /// the pad the shell staged. The death fade is the player procedure's:
    /// `m35A0` chases 0 by 0.01 a frame, the backdrop takes `1 - m35A0`, and
    /// `animeStart` lands below 0.38 (`d_a_player_dproc.inc:1069-1085`).
    fn step_game_over(&mut self) {
        if self.menu.screen != menus::Screen::GameOver {
            self.game_over = None;
            return;
        }
        if self.game_over.is_none() && !self.game_over_failed && !self.open_game_over() {
            self.game_over_failed = true;
        }
        let pad = {
            let buttons = *self.sim.world.resource::<sim::intent::PadButtons>();
            let stick = *self.sim.world.resource::<sim::intent::CStick>();
            ui_menu_save::PadInput {
                trig_a: buttons.trig.a,
                trig_b: buttons.trig.b,
                trig_x: buttons.trig.x,
                trig_y: buttons.trig.y,
                trig_z: buttons.trig.z,
                trig_l: buttons.trig.l,
                trig_r: buttons.trig.r,
                trig_start: buttons.trig.start,
                stick_value: stick.value(),
                stick_angle: stick.angle,
            }
        };
        let mut vibration = *self.sim.world.resource::<sim::vibration::Vibration>();
        let mut health = *self.sim.world.resource::<sim::health::Health>();
        let mut outcome = None;
        if let Some(stage) = self.game_over.as_mut() {
            stage.fade = (stage.fade + 0.01).min(1.0);
            stage.game.letters.set_back_alpha(stage.fade);
            if 1.0 - stage.fade < 0.38 {
                stage.game.anime_start();
            }
            let life = u16::from(health.units);
            let mut env = ui_menu_save::SaveEnv {
                pad,
                card: &mut self.menu.card,
                menu: &mut stage.flags,
                system: &mut stage.system,
                particles: &mut stage.particles,
                pending_save: None,
                vibration: &mut vibration,
                health: &mut health,
                life_on_open: life,
            };
            outcome = stage.game.step(&mut env);
            stage.revision = stage.revision.wrapping_add(1);
        }
        *self.sim.world.resource_mut::<sim::vibration::Vibration>() = vibration;
        *self.sim.world.resource_mut::<sim::health::Health>() = health;
        let Some(outcome) = outcome else {
            return;
        };
        self.game_over = None;
        let choice = match outcome {
            game_over_screen::Outcome::Continue => sim::intent::GameOverChoice::Continue,
            game_over_screen::Outcome::Title => sim::intent::GameOverChoice::Quit,
        };
        *self.sim.world.resource_mut::<sim::intent::GameOverChoice>() = choice;
        self.menu.screen = match outcome {
            game_over_screen::Outcome::Continue => menus::Screen::Playing,
            game_over_screen::Outcome::Title => menus::Screen::Title,
        };
    }

    pub fn health(&self) -> sim::health::Health {
        *self.sim.world.resource::<sim::health::Health>()
    }

    pub fn rupees(&self) -> u32 {
        self.sim.world.resource::<sim::pickup::Rupees>().count
    }

    pub fn held(&self) -> sim::item::Held {
        *self.sim.world.resource::<sim::item::Held>()
    }

    pub fn dialogue(&self) -> sim::talk::Dialogue {
        *self.sim.world.resource::<sim::talk::Dialogue>()
    }

    fn pump_dialogue(&mut self) {
        let live = self
            .sim
            .world
            .get_resource::<text::driver::TextDriver>()
            .is_some_and(|driver| driver.live);
        if !live {
            self.dialogue_live = false;
            return;
        }
        let overlay = text::overlay::render(&mut self.sim.world, 640, 480);
        self.dialogue_live = overlay.live;
        if overlay.live {
            self.dialogue_canvas = overlay.canvas;
            self.dialogue_revision = self.dialogue_revision.wrapping_add(1);
        }
    }

    /// Take every sim event waiting for presentation.
    pub fn drain_events(&mut self) -> Vec<sim::events::SimEvent> {
        std::mem::take(&mut self.sim.world.resource_mut::<sim::events::SimEvents>().list)
    }

    /// The particles `cc_at_check` queued this frame. `pump_damage_fx` turns each
    /// one into emitters before the view reads them.
    pub fn particles(&self) -> &[sim::damage_fx::DamageParticle] {
        &self
            .sim
            .world
            .resource::<sim::damage_fx::DamageParticles>()
            .items
    }

    /// Frame boundary for audio: advance voices, realize queued sim events.
    pub fn pump_audio(&mut self, dt: Duration) {
        self.audio.update(dt.as_secs_f32());
        self.audio.take_finished();
        self.pump_hit_audio();
        self.pump_damage_fx(dt);
        for event in self.drain_events() {
            let cue = match event {
                sim::events::SimEvent::Footstep => Some(audio::FOOTSTEP_CUE),
                sim::events::SimEvent::Slash => Some(audio::SLASH_CUE),
                sim::events::SimEvent::Hit => Some(audio::HIT_CUE),
                sim::events::SimEvent::Explode => Some(audio::BOOM_CUE),
                sim::events::SimEvent::Collect => Some(audio::PICKUP_CUE),
                sim::events::SimEvent::Hurt => Some(audio::HURT_CUE),
                sim::events::SimEvent::Se(sim::hammer::SE_SWING) => Some(audio::HAMMER_SWING_CUE),
                sim::events::SimEvent::Se(sim::hammer::SE_ON) => Some(audio::HAMMER_CUE),
                sim::events::SimEvent::Se(sim::hammer::SE_OFF) => Some(audio::HAMMER_CUE),
                sim::events::SimEvent::GameOver
                | sim::events::SimEvent::Restart
                | sim::events::SimEvent::Credits
                | sim::events::SimEvent::CreditsDone
                | sim::events::SimEvent::Se(_)
                | sim::events::SimEvent::SeAt(_, _)
                | sim::events::SimEvent::SeStop(_)
                | sim::events::SimEvent::Bgm(_)
                | sim::events::SimEvent::StealItem(_) => None,
            };
            if let Some(cue) = cue {
                self.audio.play(cue);
            }
        }
    }

    /// Play what `cc_at_check` queued: every sound start it made, the positional ones at their
    /// own point.
    ///
    /// `mDoAud_bgmHitSound` is `JAIZelBasic::bgmHitSound` at `802A564C-802A579C` — 0x150 bytes
    /// whose body the reference project carries as a stub that does not match those addresses,
    /// so its selector cannot be resolved. Every neighbour in that file drives the running
    /// battle music (`bgmNowKaitengiri` fades a `JAIZelParam` track, `onEnemyDamage` writes a
    /// sequence port), which makes the selector a music accent rather than a sound id, and
    /// nothing in the tree names a table for it. One tone therefore stands in for the whole
    /// range; splitting it into five cues would be invented, not recovered.
    ///
    /// `mDoAud_onEnemyDamage` at `802A57A4-802A5818` does match, and its body only writes `1`
    /// into port 9 of the sea-enemy sub-music or port app `0x000b0000` of the normal-battle
    /// music. It touches no wave, no flag and no track id, so there is nothing to hear without
    /// a sequencer. `SimEvent::Bgm` is the shell's music channel and it has no consumer either,
    /// so the flag is cleared rather than turned into a sound of its own.
    fn pump_hit_audio(&mut self) {
        let (se, bgm_hit_sound) = {
            let mut hit = self
                .sim
                .world
                .resource_mut::<sim::damage_resolve::HitAudio>();
            hit.enemy_damage = false;
            (std::mem::take(&mut hit.se), hit.bgm_hit_sound.take())
        };
        if bgm_hit_sound.is_some() {
            self.audio.play(audio::BGM_HIT_CUE);
        }
        if se.iter().any(|sound| sound.pos.is_some()) {
            let listener = self
                .sim
                .world
                .iter_entities()
                .find_map(|entity| entity.get::<sim::player::Player>().copied())
                .map(|player| (player.pos.x, player.pos.z))
                .unwrap_or_default();
            self.audio.bank().set_listener(listener.0, listener.1);
            self.audio
                .bank()
                .set_spatial_scale(1.0 / sim::events::STRIDE_UNITS);
        }
        for sound in se {
            let Some(cue) = audio::hit_cue(sound.id) else {
                continue;
            };
            match sound.pos {
                Some(pos) => {
                    self.audio.play_at(cue, pos.x, pos.z);
                }
                None => {
                    self.audio.play(cue);
                }
            }
        }
    }

    /// Split the frame's damage effects: the particles replace the view layer's one-frame list,
    /// and the steal-item creates become events for the item layer. The list then becomes the
    /// pool's spawns and the pool advances by the frame, which is where an emitter ages, moves
    /// and finally retires. Retail's particle manager also advances once per frame rather than
    /// on the fixed step, so this rides the frame `dt` and not a tick.
    fn pump_damage_fx(&mut self, dt: Duration) {
        let mut particles = std::mem::take(
            self.sim
                .world
                .resource_mut::<sim::damage_fx::DamageParticles>()
                .as_mut(),
        );
        let steals = self
            .sim
            .world
            .resource_mut::<sim::damage_fx::DamageFxQueue>()
            .drain(&mut particles);
        {
            let mut emitters = self
                .sim
                .world
                .get_resource_or_insert_with(sim::particle::Particles::default);
            for spawn in &particles.items {
                emitters.spawn(spawn);
            }
            emitters.step(dt.as_secs_f32());
        }
        *self
            .sim
            .world
            .resource_mut::<sim::damage_fx::DamageParticles>() = particles;
        if steals.is_empty() {
            return;
        }
        let mut events = self.sim.world.resource_mut::<sim::events::SimEvents>();
        for steal in steals {
            events.list.push(sim::events::SimEvent::StealItem(steal));
        }
    }

    /// The retail field camera `camera::track` publishes each tick. The orbit is only a debug
    /// fallback: it has no room collision and cannot frame an interior, which is why the prologue
    /// starts with the eye inside the lookout's wall.
    pub(crate) fn game_camera(&self) -> Option<OrbitCamera> {
        let camera = self.sim.world.get_resource::<sim::camera::Camera>()?;
        let view = camera.view()?;
        let eye = view.lookat.eye;
        let target = view.lookat.center;
        let offset = eye - target;
        let dist = offset.length();
        if !(dist > 0.001) {
            return None;
        }
        Some(OrbitCamera {
            target,
            yaw: offset.z.atan2(offset.x),
            pitch: (offset.y / dist).clamp(-1.0, 1.0).asin(),
            dist,
            fov_y_deg: view.fovy.to_degrees().clamp(20.0, 100.0),
        })
    }

    fn camera_underground(&self, camera: &OrbitCamera, snapshot: &view::SceneSnapshot) -> bool {
        let Some(collision) = self.sim.world.get_resource::<sim::collision::Collision>() else {
            return false;
        };
        let eye = camera.eye();
        // The floor the player stands on, which is the ground under the player's own column
        // with the ceiling the ground check uses (`main.cpp:4629`). The eye is under the
        // ground when it drops below that floor: the surfaces above the player are roofs and
        // ceilings the retail camera looks under, not ground.
        let floor = sim::ground::surface(Some(collision), snapshot.player_pos);
        floor.is_some_and(|floor| eye.y < floor + 8.0)
    }

    /// The field camera's eye below the floor the player stands on. The retail follow camera
    /// pulls in on its own when a wall is behind the player, so the fixed side view only
    /// stands in for an eye that would sit under the ground.
    fn fallback_camera(&self, snapshot: &view::SceneSnapshot) -> OrbitCamera {
        OrbitCamera {
            target: snapshot.player_pos + glam::Vec3::Y * 70.0,
            yaw: 1.5708,
            pitch: 0.28,
            dist: 440.0,
            fov_y_deg: 55.0,
        }
    }

    pub fn snapshot(&self) -> view::SceneSnapshot {
        let player = self
            .sim
            .world
            .iter_entities()
            .find_map(|entity| entity.get::<sim::player::Player>().copied());
        let area = self.sim.world.resource::<sim::area::Area>();
        let (player_pos, player_yaw, player_face) = match player {
            Some(player) => (player.pos, player.yaw, player.face),
            None => (area.spawn, area.yaw, area.yaw),
        };
        let collision = self.sim.world.get_resource::<sim::collision::Collision>();
        let water = self.sim.world.get_resource::<sim::water::Water>();
        let (ground_y, walls) = match collision {
            Some(collision) => {
                let mut y =
                    sim::ground::surface(Some(collision), player_pos).unwrap_or(player_pos.y);
                if let Some(water) = water.filter(|water| water.in_area) {
                    y = y.max(water.surface.min(player_pos.y));
                }
                (y, false)
            }
            None => (0.0, true),
        };
        view::SceneSnapshot {
            player_pos,
            player_yaw,
            player_face,
            bounds_min: area.bounds_min,
            bounds_max: area.bounds_max,
            ground_y,
            walls,
            tick: self
                .sim
                .world
                .resource::<sim::frame::CurrentFrame>()
                .0
                .saturating_sub(self.room_tick),
            sky: self
                .stage
                .as_ref()
                .and_then(dzs::StageTables::boot_sky)
                .unwrap_or(view::SKY),
            sea: self.sea_view(),
            actors: self.actor_views(),
            particles: self.particle_views(),
        }
    }

    /// Every emitter still inside its lifetime, as the draw layer reads them.
    fn particle_views(&self) -> Vec<view::ParticleView> {
        self.sim
            .world
            .get_resource::<sim::particle::Particles>()
            .map(|pool| {
                pool.live()
                    .map(|emitter| view::ParticleView {
                        pos: emitter.pos,
                        half: emitter.half(),
                        color: emitter.color,
                        alpha: emitter.alpha(),
                        roll: emitter.roll,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn actor_views(&self) -> Vec<view::ActorView> {
        let mut out = Vec::new();
        for entity in self.sim.world.iter_entities() {
            if let Some(bomb) = entity.get::<sim::item::Bomb>() {
                out.push(view::ActorView {
                    pos: bomb.pos,
                    kind: view::ActorKind::Bomb,
                    item: 0,
                    height: 12.0,
                    half: 6.0,
                });
                continue;
            }
            if let Some(arrow) = entity.get::<sim::arrow::Arrow>() {
                if arrow.visible {
                    out.push(view::ActorView {
                        pos: arrow.pos,
                        kind: view::ActorKind::Arrow,
                        item: 0,
                        height: 3.0,
                        half: 3.0,
                    });
                }
                continue;
            }
            if let Some(boom) = entity.get::<sim::boomerang::Boomerang>() {
                out.push(view::ActorView {
                    pos: boom.pos,
                    kind: view::ActorKind::Boomerang,
                    item: 0,
                    height: 10.0,
                    half: 10.0,
                });
                continue;
            }
            if let Some(spear) = entity.get::<sim::enemy::Spear>() {
                out.push(view::ActorView {
                    pos: spear.pos,
                    kind: view::ActorKind::Spear,
                    item: 0,
                    height: 4.0,
                    half: 4.0,
                });
                continue;
            }
            if let Some(shot) = entity.get::<sim::warship::Cannon>() {
                out.push(view::ActorView {
                    pos: shot.pos,
                    kind: view::ActorKind::Cannon,
                    item: 0,
                    height: 24.0,
                    half: 24.0,
                });
                continue;
            }
            if let Some(fire) = entity.get::<sim::fire::Fire>() {
                let Some(actor) = entity.get::<sim::actor::Actor>() else {
                    continue;
                };
                if !fire.lit {
                    continue;
                }
                out.push(view::ActorView {
                    pos: actor.pos,
                    kind: view::ActorKind::Fire,
                    item: 0,
                    height: 30.0,
                    half: 12.0,
                });
                continue;
            }
            let Some(actor) = entity.get::<sim::actor::Actor>() else {
                continue;
            };
            let mut lift = 0.0;
            let (kind, item, height, half) = if entity.contains::<sim::ship::Ship>() {
                (view::ActorKind::Ship, 0, 26.0, 44.0)
            } else if let Some(pickup) = entity.get::<sim::pickup::Pickup>() {
                let size = pickup.size();
                (view::ActorKind::Pickup, pickup.item, size, size)
            } else if let Some(present) = entity.get::<sim::event::PresentItem>() {
                if present.hidden {
                    continue;
                }
                let size = sim::pickup::Pickup::spawn(present.item as u32).size();
                (view::ActorKind::Pickup, present.item, size, size)
            } else if entity.contains::<sim::stalfos::Stalfos>() {
                (view::ActorKind::Enemy, 0, 162.5, 125.0)
            } else if let Some(enemy) = entity.get::<sim::enemy::Enemy>() {
                (
                    view::ActorKind::Enemy,
                    0,
                    enemy.profile.body_height,
                    enemy.profile.body_half,
                )
            } else if entity.contains::<sim::balloon::Balloon>() {
                (view::ActorKind::Balloon, 0, 36.0, 18.0)
            } else if entity.contains::<sim::gull::Gull>() || entity.contains::<sim::bird::Bird>() {
                (view::ActorKind::Gull, 0, 12.0, 8.0)
            } else if let Some(x) = entity.get::<sim::marlon::Marlon>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::shop::Shop>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(grandma) = entity.get::<sim::ba1::Ba1>() {
                // `fopAc_Draw`'s own gate (`f_op_actor.cpp:129-148`), which
                // `ba1::control_draw` evaluates and leaves as `fopAcCnd_NODRAW_e`: a culled or
                // no-draw actor's `_draw` never runs, so nothing of hers is emitted.
                let culled = self
                    .sim
                    .world
                    .resource::<sim::registry::ActorRegistry>()
                    .entry_of(entity.id())
                    .is_some_and(|entry| entry.condition & sim::registry::CONDITION_NO_DRAW != 0);
                if culled {
                    continue;
                }
                // `_draw`'s `if(mbInitGrandma0 || m7F7) return TRUE;` (`d_a_npc_ba1.cpp:1853-1855`):
                // the event's own model hide, which the box stands in for.
                if !grandma.drawn() {
                    continue;
                }
                (view::ActorKind::Talker, 0, 30.0, 12.0)
            } else if entity.contains::<sim::lenzo::Lenzo>()
                || entity.contains::<sim::zunari::Zunari>()
                || entity.contains::<sim::aryll::Aryll>()
                || entity.contains::<sim::medli::Medli>()
            {
                (view::ActorKind::Talker, 0, 30.0, 12.0)
            } else if entity.contains::<sim::zelda::Zelda>() {
                if entity.get::<sim::zelda::Zelda>().is_some_and(|z| !z.drawn) {
                    continue;
                }
                (view::ActorKind::Talker, 0, 30.0, 12.0)
            } else if entity.contains::<sim::tingle::Tingle>() {
                (view::ActorKind::Scenery, 0, 24.0, 10.0)
            } else if entity.contains::<sim::chest::Chest>() || entity.contains::<sim::tbox::Tbox>()
            {
                if entity.get::<sim::chest::Chest>().is_some_and(|c| c.opened) {
                    continue;
                }
                if entity.get::<sim::tbox::Tbox>().is_some_and(|t| !t.drawn()) {
                    continue;
                }
                (view::ActorKind::Scenery, 0, 24.0, 12.0)
            } else if entity.contains::<sim::stone::Stone>() {
                if entity.get::<sim::stone::Stone>().is_some_and(|s| s.broken) {
                    continue;
                }
                (view::ActorKind::Scenery, 0, 20.0, 10.0)
            } else if let Some(w) = entity.get::<sim::wall::Wall>() {
                if w.state == sim::wall::State::Break {
                    continue;
                }
                let (min, max) = w.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if entity.contains::<sim::bomb::Bomb>() {
                (view::ActorKind::Scenery, 0, 16.0, 8.0)
            } else if let Some(ladder) = entity.get::<sim::ladder::Ladder>() {
                (
                    view::ActorKind::Scenery,
                    0,
                    ladder.height() + sim::ladder::CULL_HEIGHT,
                    sim::ladder::CULL_WIDTH,
                )
            } else if entity.contains::<sim::warship::Warship>() {
                let (height, half) = sim::warship::BODY;
                (view::ActorKind::Warship, 0, height, half)
            } else if let Some(talk) = entity.get::<sim::talk::Talk>() {
                if talk.gone {
                    continue;
                }
                (view::ActorKind::Talker, 0, 30.0, 12.0)
            } else if entity.contains::<sim::scenery::Scenery>()
                || entity.contains::<sim::tsubo::Tsubo>()
                || entity.contains::<sim::carry::Carryable>()
            {
                (view::ActorKind::Scenery, 0, 24.0, 10.0)
            } else if entity.contains::<sim::raft::Raft>() {
                (
                    view::ActorKind::Ship,
                    0,
                    sim::raft::HULL_RADIUS * 0.5,
                    sim::raft::HULL_RADIUS * 0.5,
                )
            } else if let Some(field) = entity.get::<sim::barrier::Barrier>() {
                if field.phase == sim::barrier::Phase::Cut {
                    continue;
                }
                (view::ActorKind::Fire, 0, field.height(), field.radius(true))
            } else if let Some(hook) = entity.get::<sim::hookshot::Hookshot>() {
                if hook.phase == sim::hookshot::Phase::Wait {
                    continue;
                }
                (view::ActorKind::Spear, 0, 6.0, 3.0)
            } else if let Some(door) = entity.get::<sim::door::Door>() {
                let height = door.key_height() + sim::door::BAR_HEIGHT - door.bars_y;
                (view::ActorKind::Scenery, 0, height, 40.0)
            } else if let Some(wreck) = entity.get::<sim::ghostship::Ghostship>() {
                if wreck.alpha <= 0.0 {
                    continue;
                }
                (view::ActorKind::Ship, 0, 300.0, 150.0)
            } else if let Some(vessel) = entity.get::<sim::pirateship::Pirate>() {
                if !vessel.visible {
                    continue;
                }
                let (min, max) = vessel.cull();
                lift = min.y;
                (
                    view::ActorKind::Ship,
                    0,
                    max.y - min.y,
                    (max.x - min.x) * 0.5,
                )
            } else if let Some(rock) = entity.get::<sim::fallrock::FallRock>() {
                if rock.gone {
                    continue;
                }
                (
                    view::ActorKind::Scenery,
                    0,
                    rock.hit_height(),
                    rock.hit_radius(),
                )
            } else if let Some(mill) = entity.get::<sim::windmill::Windmill>() {
                let (_, max) = mill.cull();
                (view::ActorKind::Scenery, 0, max.y, max.x)
            } else if let Some(gale) = entity.get::<sim::wind::WindTag>() {
                if !gale.visible() {
                    continue;
                }
                lift = gale.cur;
                (view::ActorKind::Fire, 0, gale.span, gale.radius())
            } else if let Some(float) = entity.get::<sim::buoybarrel::BuoyBarrel>() {
                if float.gone {
                    continue;
                }
                let (height, radius) = float.hull();
                (view::ActorKind::Scenery, 0, height, radius)
            } else if let Some(octo) = entity.get::<sim::daiocta::Daiocta>() {
                let (min, max) = octo.cull();
                lift = min.y;
                (
                    view::ActorKind::Enemy,
                    0,
                    max.y - min.y,
                    (max.x - min.x) * 0.5,
                )
            } else if let Some(stick) = entity.get::<sim::boko::Boko>() {
                let (min, max) = stick.cull();
                lift = min.y;
                (view::ActorKind::Scenery, 0, max.y - min.y, max.x)
            } else if let Some(cask) = entity.get::<sim::barrel::Barrel>() {
                if cask.broken {
                    continue;
                }
                let (height, radius) = sim::barrel::hull();
                (view::ActorKind::Scenery, 0, height, radius)
            } else if let Some(block) = entity.get::<sim::ice::Ice>() {
                if block.state == sim::ice::State::Gone {
                    continue;
                }
                (
                    view::ActorKind::Scenery,
                    0,
                    block.hull_height(),
                    block.hull(),
                )
            } else if let Some(idol) = entity.get::<sim::statue::Statue>() {
                if idol.alpha == sim::statue::TRANSPARENT {
                    continue;
                }
                let (half, top) = idol.cull();
                (view::ActorKind::Scenery, 0, top, half)
            } else if let Some(wall) = entity.get::<sim::ajav::Ajav>() {
                if !wall.standing() {
                    continue;
                }
                let (drop, height, radius) = wall.hull();
                lift = drop;
                (view::ActorKind::Scenery, 0, height, radius)
            } else if let Some(statuette) = entity.get::<sim::armos::Armos>() {
                if !statuette.drawn() {
                    continue;
                }
                let (min, max) = statuette.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(sage) = entity.get::<sim::makar::Makar>() {
                if !sage.drawn() {
                    continue;
                }
                let (min, max) = sage.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(bird) = entity.get::<sim::helmaroc::Helmaroc>() {
                if !bird.drawn() {
                    continue;
                }
                let (min, max) = bird.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(boss) = entity.get::<sim::gohdan::Gohdan>() {
                if !boss.drawn() {
                    continue;
                }
                let (min, max) = boss.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(boss) = entity.get::<sim::gohma::Gohma>() {
                if !boss.drawn() {
                    continue;
                }
                let (min, max) = boss.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(boss) = entity.get::<sim::phantom_ganon::PhantomGanon>() {
                if !boss.drawn() {
                    continue;
                }
                let (min, max) = boss.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(boss) = entity.get::<sim::puppet_ganon::PuppetGanon>() {
                if !boss.drawn() {
                    continue;
                }
                let (min, max) = boss.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::ivan::Ivan>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(ghost) = entity.get::<sim::ks::Ks>() {
                if !ghost.drawn() {
                    continue;
                }
                let (min, max) = ghost.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(a) = entity.get::<sim::dr2::Dr2>() {
                if !a.drawn() {
                    continue;
                }
                let (min, max) = a.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(t) = entity.get::<sim::bmdhand::Bmdhand>() {
                if !t.drawn() {
                    continue;
                }
                let (min, max) = t.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::obj_doguu::ObjDoguu>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::sss::Sss>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::tag_hint::TagHint>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::tag_md_cb::TagMdCb>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(a) = entity.get::<sim::obj_hha::ObjHha>() {
                if !a.drawn() {
                    continue;
                }
                let (min, max) = a.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::title::Title>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(a) = entity.get::<sim::shand::Shand>() {
                if !a.drawn() {
                    continue;
                }
                let (min, max) = a.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::pirate_flag::PirateFlag>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(orb) = entity.get::<sim::fgmahou::FgMahou>() {
                if !orb.drawn() {
                    continue;
                }
                let (min, max) = orb.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(m) = entity.get::<sim::obj_mmrr::ObjMmrr>() {
                if !m.drawn() {
                    continue;
                }
                let (min, max) = m.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::bwdg::Bwdg>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::morth::Morth>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::kalle_demos::KalleDemos>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::agbsw0::Agbsw0>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::npc_tc::NpcTc>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::movebox::Movebox>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::npc_bmsw::NpcBmsw>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::auction::Auction>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::kamome::Kamome>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::npc_mt::NpcMt>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::npc_p1::NpcP1>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::amos_knight::AmosKnight>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::ganon_form3::GanonForm3>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::npc_btsw::NpcBtsw>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::apzl::Apzl>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::bwds::Bwds>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::npc_nz::NpcNz>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::npc_ho::NpcHo>() {
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::npc_hi1::NpcHi1>() {
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::npc_fa1::NpcFa1>() {
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::toripost::Toripost>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::majuu_flag::MajuuFlag>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::goal_flag::GoalFlag>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::bridge::Bridge>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(v) = entity.get::<sim::bgn3::Bgn3>() {
                if !v.drawn() {
                    continue;
                }
                let (min, max) = v.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(robot) = entity.get::<sim::floormaster::Floormaster>() {
                if !robot.drawn() {
                    continue;
                }
                let (min, max) = robot.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(idol) = entity.get::<sim::figure::Figure>() {
                if !idol.drawn() {
                    continue;
                }
                let (min, max) = idol.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(mask) = entity.get::<sim::homen::Homen>() {
                if !mask.drawn() {
                    continue;
                }
                let (min, max) = mask.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(pile) = entity.get::<sim::leaves::Leaves>() {
                if !pile.drawn() {
                    continue;
                }
                let (half, top) = pile.cull();
                (view::ActorKind::Scenery, 0, top, half)
            } else if let Some(crater) = entity.get::<sim::volcano::Volcano>() {
                if !crater.visible() {
                    continue;
                }
                lift = crater.lava.y - actor.pos.y;
                let (height, half) = crater.hull();
                (view::ActorKind::Fire, 0, height, half)
            } else if let Some(plunger) = entity.get::<sim::swhammer::Swhammer>() {
                let (radius, top) = plunger.cull();
                lift = plunger.top;
                (view::ActorKind::Scenery, 0, top, radius)
            } else if let Some(guard) = entity.get::<sim::shmrgrd::Shmrgrd>() {
                let (_, max) = guard.cull();
                lift = guard.top;
                (view::ActorKind::Scenery, 0, max[1], max[0])
            } else if let Some(brand) = entity.get::<sim::torch::Torch>() {
                if brand.state == sim::torch::State::Gone {
                    continue;
                }
                (
                    view::ActorKind::Fire,
                    0,
                    sim::torch::CULL_TOP,
                    sim::torch::POST_RADIUS,
                )
            } else if let Some(spot) = entity.get::<sim::taglight::TagLight>() {
                if !spot.drawn() {
                    continue;
                }
                let (half, top) = spot.cull();
                (view::ActorKind::Fire, 0, top, half)
            } else if let Some(button) = entity.get::<sim::swpush::Swpush>() {
                lift = button.cull_base + button.top;
                (
                    view::ActorKind::Scenery,
                    0,
                    button.cull_top - lift,
                    button.half,
                )
            } else if let Some(hatch) = entity.get::<sim::trapdoor::Trapdoor>() {
                if hatch.hinge == sim::trapdoor::Hinge::Swinging {
                    continue;
                }
                (
                    view::ActorKind::Scenery,
                    0,
                    sim::trapdoor::TRIGGER_HEIGHT,
                    sim::trapdoor::TRIGGER_RADIUS * 0.5,
                )
            } else if let Some(fence) = entity.get::<sim::barricade::Barricade>() {
                if !fence.blocking(0) {
                    continue;
                }
                (
                    view::ActorKind::Scenery,
                    0,
                    fence.standing(),
                    sim::barricade::POST_SIDE,
                )
            } else if let Some(flower) = entity.get::<sim::bombflower::Bombflower>() {
                (
                    view::ActorKind::Scenery,
                    0,
                    sim::bombflower::FLOWER_TOP,
                    flower.radius(),
                )
            } else if let Some(crawler) = entity.get::<sim::bombflower::Crawler>() {
                if crawler.spent {
                    continue;
                }
                (
                    view::ActorKind::Bomb,
                    0,
                    sim::bombflower::CRAWLER_TOP,
                    sim::bombflower::CRAWLER_HALF,
                )
            } else if let Some(hoist) = entity.get::<sim::mflft::Mflft>() {
                let (half, top) = hoist.cull();
                (view::ActorKind::Scenery, 0, top, half)
            } else if let Some(slime) = entity.get::<sim::chuchu::Chuchu>() {
                if !slime.drawn() {
                    continue;
                }
                let (min, max) = slime.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(trader) = entity.get::<sim::merchant::Merchant>() {
                let (min, max) = trader.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(crab) = entity.get::<sim::kn::Kn>() {
                if !crab.drawn() {
                    continue;
                }
                let (half, top) = crab.cull();
                lift = crab.bury;
                (view::ActorKind::Scenery, 0, top, half)
            } else if let Some(sage) = entity.get::<sim::sage::Sage>() {
                if !sage.drawn() {
                    continue;
                }
                let (half, top) = sage.cull();
                (view::ActorKind::Talker, 0, top, half)
            } else if entity.contains::<sim::iball::Iball>() {
                (
                    view::ActorKind::Fire,
                    0,
                    sim::iball::HEIGHT,
                    sim::iball::RADIUS,
                )
            } else if entity.contains::<sim::redead::ReDead>() {
                (view::ActorKind::Enemy, 0, 260.0, 100.0)
            } else if let Some(flower) = entity.get::<sim::kita::Kita>() {
                let (min, max) = flower.cull();
                lift = min.y;
                (
                    view::ActorKind::Scenery,
                    0,
                    max.y - min.y,
                    (max.x - min.x) * 0.5,
                )
            } else if let Some(door) = entity.get::<sim::vgnfd::Vgnfd>() {
                if !door.drawn() {
                    continue;
                }
                let (min, max) = door.cull();
                lift = min.y;
                (
                    view::ActorKind::Scenery,
                    0,
                    max.y - min.y,
                    (max.x - min.x) * 0.5,
                )
            } else if let Some(platform) = entity.get::<sim::hmlif::Hmlif>() {
                let (min, max) = platform.cull();
                lift = min.y;
                (
                    view::ActorKind::Scenery,
                    0,
                    max.y - min.y,
                    (max.x - min.x) * 0.5,
                )
            } else if entity.contains::<sim::cafelmp::Cafelmp>() {
                let (min, max) = sim::cafelmp::Cafelmp::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::xfuta::Xfuta>() {
                let (min, max) = sim::xfuta::Xfuta::spawn(glam::Vec3::ZERO).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::mjdoor::MjDoor>() {
                let (min, max) =
                    sim::mjdoor::MjDoor::spawn(0, &sim::flags::Switches::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::turu::Turu>() {
                let (min, max) = sim::turu::Turu::spawn(0, &sim::flags::Switches::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::spcitem::SpcItem>() {
                let (min, max) =
                    sim::spcitem::SpcItem::spawn(0, &sim::flags::EventFlags::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::doguud::DoguuD>() {
                let (min, max) = sim::doguud::DoguuD::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::eayogn::Eayogn>() {
                let (min, max) = sim::eayogn::Eayogn::spawn(&sim::flags::Collect::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::ktaruo::Ktaruo>() {
                let (min, max) = sim::ktaruo::Ktaruo::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if let Some(doc) = entity.get::<sim::npc_ds1::NpcDs1>() {
                let (min, max) = doc.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if entity.contains::<sim::plant::Plant>() {
                let (min, max) = sim::plant::Plant::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::amiprop::Amiprop>() {
                let (min, max) = sim::amiprop::Amiprop::spawn(0, [0i16; 3]).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::msdansb::Msdansb>() {
                let (min, max) =
                    sim::msdansb::Msdansb::spawn(0, 0.0, &sim::flags::Switches::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::yboil::Yboil>() {
                let (min, max) =
                    sim::yboil::Yboil::spawn(0, &sim::flags::EventFlags::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::lamp::Lamp>() {
                let (min, max) = sim::lamp::Lamp::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::rflw::Rflw>() {
                let (min, max) = sim::rflw::Rflw::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::gtaki::Gtaki>() {
                let (min, max) = sim::gtaki::Gtaki::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if let Some(plate) = entity.get::<sim::swtact::Swtact>() {
                if !plate.mark {
                    continue;
                }
                let Some((min, max)) = plate.cull() else {
                    continue;
                };
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::nzg::Nzg>() {
                let (min, max) = sim::nzg::Nzg::spawn(0).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::hys::Hys>() {
                let (min, max) = sim::hys::Hys::spawn(0, &sim::flags::Switches::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::syan::Syan>() {
                let (min, max) = sim::syan::Syan::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::ebomzo::Ebomzo>() {
                let (min, max) =
                    sim::ebomzo::Ebomzo::spawn(0, &sim::flags::Switches::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::sk2::Sk2>() {
                let (min, max) = sim::sk2::Sk2::spawn(0, &sim::flags::Switches::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::rohat::RoHat>() {
                let (min, max) = sim::rohat::RoHat::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::vfan::Vfan>() {
                let (min, max) = sim::vfan::Vfan::spawn(0, &sim::flags::Switches::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::htobi3::Htobi3>() {
                let (min, max) = sim::htobi3::Htobi3::spawn(0).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::branch::Branch>() {
                let (min, max) = sim::branch::Branch::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::hami3::Hami3>() {
                let (min, max) =
                    sim::hami3::Hami3::spawn(0, &sim::flags::Switches::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::atdoor::Atdoor>() {
                let (min, max) =
                    sim::atdoor::Atdoor::spawn(0, &sim::flags::Switches::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if let Some(x) = entity.get::<sim::npc_ah::NpcAh>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::npc_btsw2::NpcBtsw2>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::npc_jb1::NpcJb1>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::npc_tt::NpcTt>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::sitem::SItem>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::dekuitem::DekuItem>() {
                if !x.drawn {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (
                    view::ActorKind::Pickup,
                    sim::dekuitem::ITEM_NO,
                    max[1] - min[1],
                    max[0],
                )
            } else if let Some(x) = entity.get::<sim::ospbox::Ospbox>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::ojtree::Ojtree>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::lwood::Lwood>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::lpalm::Lpalm>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::bita::Bita>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::esa::Esa>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::hbrf1::Hbrf1>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::fan::Fan>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::shelf::Shelf>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::knob00::Knob00>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::ki::Ki>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::shutter2::Shutter2>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::auzu::Auzu>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::hole::Hole>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::toge::Toge>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::lbridge::Lbridge>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::stair::Stair>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::msdansub2::MsdanSub2>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::swpropeller::SwPropeller>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::buoyrace::BuoyRace>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::ohatch::Ohatch>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::ff::Ff>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::leaflift::Olift>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::hlift::Hlift>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::lstair::Lstair>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::dai::Dai>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::mtest::Mtest>() {
                let Some((min, max)) = x.cull() else { continue };
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::agb::Agb>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::warpf::Warpf>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::himo3::Himo3>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::demo00::Demo00>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::hsh::Hsh>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if entity.contains::<sim::grid::Grid>() {
                let (min, max) = sim::grid::Grid::spawn(0).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if let Some(x) = entity.get::<sim::obj_correct::ObjCorrect>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::obj_swheavy::SwHeavy>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::obj_mkie::ObjMkie>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::swhit0::SwHit0>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::pedestal::Pedestal>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::warphr::WarpRule>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::vmc::Vmc>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::machine::Machine>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::drift::Drift>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::mdoor::Mdoor>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::kokiie::Kokiie>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::warpls::Warpls>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::mshokki::Mshokki>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::dk::Dk>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::warpdm20::WarpDm20>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::ferris::Ferris>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::sail::Sail>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::mkiek::Mkiek>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if entity.contains::<sim::gaship::Gaship>() {
                let (min, max) = sim::gaship::Gaship::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::hhyu::Hhyu>() {
                let (min, max) = sim::hhyu::Hhyu::spawn(0, &sim::flags::Switches::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::xtower::Xtower>() {
                let (min, max) =
                    sim::xtower::Xtower::spawn(&sim::flags::EventFlags::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::dmgroom::Dmgroom>() {
                let (min, max) = sim::dmgroom::Dmgroom::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::komore::Komore>() {
                let (min, max) = sim::komore::Komore::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::pbka::Pbka>() {
                let (min, max) = sim::pbka::Pbka::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::dummy::Dummy>() {
                let (min, max) = sim::dummy::Dummy::spawn().cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if let Some(door) = entity.get::<sim::swtdoor::Swtdoor>() {
                let (min, max) = door.cull();
                lift = min[1] + door.sink;
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::spotbox::SpotBox>() {
                lift = -(sim::spotbox::HALF);
                (
                    view::ActorKind::Scenery,
                    0,
                    sim::spotbox::HALF * 2.0,
                    sim::spotbox::HALF,
                )
            } else if let Some(gun) = entity.get::<sim::canon::Canon>() {
                if !gun.drawn() {
                    continue;
                }
                let (base, height, half) = gun.hull();
                lift = base;
                (view::ActorKind::Scenery, 0, height, half)
            } else if let Some(cell) = entity.get::<sim::door::JailDoor>() {
                if cell.open() {
                    continue;
                }
                (view::ActorKind::Scenery, 0, sim::door::JAIL_HEIGHT, 40.0)
            } else {
                continue;
            };
            out.push(view::ActorView {
                pos: actor.pos + glam::Vec3::new(0.0, lift, 0.0),
                kind,
                item,
                height,
                half,
            });
        }
        out
    }

    fn sea_view(&self) -> Option<view::SeaView> {
        let sea = self.sim.world.get_resource::<sim::sea::Sea>()?;
        if sea.culled() {
            return None;
        }
        let (amb, dif) = self.stage.as_ref()?.boot_sea_colors()?;
        let tone = sea.flat() * sea.flat();
        let mut color = [0.0; 3];
        for axis in 0..3 {
            let full = dif[axis] + tone * (amb[axis] - dif[axis]);
            color[axis] = dif[axis] + self.sea_shade * (full - dif[axis]);
        }
        Some(view::SeaView {
            origin: sea.origin(),
            side: sim::sea::GRID_CELLS,
            heights: sea.heights().to_vec(),
            color,
        })
    }

    pub fn advance(&mut self, dt: Duration) -> u32 {
        self.frame += 1;
        self.log_input();
        let intro_running = self.intro.is_some();
        if intro_running {
            self.step_intro(dt);
        } else if self.menu.screen == menus::Screen::Title && self.assets_installed {
            self.step_movie(dt);
        } else if self.movie_sound {
            // The movie player belongs to the title scene (`d_s_title.cpp:122`), so its
            // soundtrack stops with it and a title that comes back plays it again
            // (`d_a_movie_player.cpp:4080-4091`).
            self.audio.stop_cue(audio::MOVIE_CUE);
            self.movie_sound = false;
        }
        let held = sim::pause::advance(&mut self.sim.world, dt, self.sim.step)
            && self.menu.screen == menus::Screen::Playing;
        // The opening demo is its own scene, `d_s_open`; the prologue stage is loaded for the
        // handoff but does not run a tick until `enter_play` hands it to the player.
        let steps = if held || intro_running {
            0
        } else {
            self.sim.step(dt)
        };
        self.drain_demo_creates();
        self.drain_demo_gaps();
        let game_over = self.sim.world.resource::<sim::death::GameOver>().active;
        if game_over && self.menu.screen == menus::Screen::Playing {
            self.menu.screen = menus::Screen::GameOver;
            self.menu.slot = 0;
            self.game_over = None;
            self.game_over_failed = false;
        }
        self.step_game_over();
        self.apply_exit();
        self.sync_rooms();
        self.pump_dialogue();
        self.handle_title_request();
        self.sync_title_actor();
        self.step_title();
        self.step_meter();
        steps
    }

    pub fn stage(&self) -> Option<&dzs::StageTables> {
        self.stage.as_ref()
    }

    /// Spawn what a `CREATER` staff asked for. `fopAcM_create` (`d_event_data.cpp:560`) runs the
    /// spawn branch the name is keyed on, and in this port that branch is
    /// `sim::spawn_actor`'s single if/else chain, which needs `&mut World` - so the request waits
    /// in `sim::demo::DemoCreates` and is drained here, the same shape `kalle_demos` uses for the
    /// actors its own demo spawns.
    fn drain_demo_creates(&mut self) {
        let creates: Vec<sim::demo::DemoCreate> =
            std::mem::take(&mut self.sim.world.resource_mut::<sim::demo::DemoCreates>().list);
        for create in creates {
            let source = dzs::StageActor {
                name: create.name.clone(),
                parameters: create.parameters,
                pos: create.pos.to_array(),
                angle: create.angle,
                set_id: create.set_id,
                room: create.room,
                scale: create.scale.to_array(),
            };
            let entity = sim::spawn_actor(&mut self.sim.world, &source);
            let Some(id) = create.demo_actor_id else {
                continue;
            };
            if let Some(mut actor) = self.sim.world.get_mut::<sim::actor::Actor>(entity) {
                actor.demo_actor_id = id;
            }
            self.sim
                .world
                .resource_mut::<sim::demo::DemoManager>()
                .object_mut()
                .bind_actor(id, entity);
        }
    }

    /// Every command the demo reader could not represent, logged once per step. The reader
    /// records them per command (`sim::demo::DemoCommandGap`); retail has no equivalent surface,
    /// and a command with no representable value left its model untouched rather than writing a
    /// stand-in.
    fn drain_demo_gaps(&mut self) {
        let gaps = self
            .sim
            .world
            .resource_mut::<sim::demo::DemoManager>()
            .take_gaps();
        for gap in gaps {
            log::warn!("clearsleeper: demo command gap ({gap})");
        }
    }

    fn apply_exit(&mut self) {
        let next =
            std::mem::take(&mut *self.sim.world.resource_mut::<sim::transition::NextStage>());
        let layer = next.layer;
        let dest = match (next.target, next.exit, next.scls) {
            (Some(target), _, _) => target,
            (None, Some(exit_id), _) => {
                let Some(exit) = self.stage.as_ref().and_then(|tables| tables.exit(exit_id)) else {
                    log::warn!("clearsleeper: exit {exit_id} has no destination");
                    return;
                };
                sim::transition::Target {
                    stage: exit.stage.clone(),
                    room: exit.room,
                    start: exit.start,
                }
            }
            // `dLib_setNextStageBySclsNum(i_sclsnum, room_no)` (`d_lib.cpp:237-256`):
            // `dComIfG_roomControl_getStatusRoomDt(room)->getSclsInfo()` for a room (`:245-246`),
            // the stage's own `getSclsInfo()` for `room_no == -1` (`:242-243`), and the entry's
            // stage/start/room at `:250-255`. The table is what this shell loaded.
            (None, None, Some((scls, room))) => {
                let entry = if room >= 0 {
                    self.room_exits
                        .iter()
                        .find(|(no, _)| *no == room as u8)
                        .and_then(|(_, exits)| exits.get(usize::from(scls)))
                } else {
                    self.stage.as_ref().and_then(|tables| tables.exit(scls))
                };
                let Some(entry) = entry else {
                    log::warn!("clearsleeper: scene number {scls} of room {room} resolves nothing");
                    return;
                };
                sim::transition::Target {
                    stage: entry.stage.clone(),
                    room: entry.room,
                    start: entry.start,
                }
            }
            (None, None, None) => return,
        };
        let Some(tables) = assets::load_stage_tables(&dest.stage) else {
            return;
        };
        let rooms = assets::room_set_for(&tables, dest.room);
        let Some((spawn, yaw)) = assets::load_stage_spawn(&dest.stage, dest.room, dest.start)
        else {
            return;
        };
        self.swap_stage(tables, rooms, spawn, yaw, layer);
    }

    pub fn swap_stage(
        &mut self,
        tables: dzs::StageTables,
        rooms: Vec<u8>,
        spawn: glam::Vec3,
        yaw: f32,
        layer: i8,
    ) {
        // `dScnKy_env_light_c::init`'s time arms (`d_kankyo.cpp:467-481`): the pending time a cut
        // asked for with `dKy_set_nexttime` (`d_event_data.cpp:45-46`) becomes the clock as the
        // next stage mounts and the pending slot goes back to `-1.0f`, then the new stage's own
        // `dStage_stagInfo_GetTimeH` write (`:472-475`), then the event-night-stop zero
        // (`:479-481`).
        let flags = self
            .sim
            .world
            .get_resource::<sim::flags::EventFlags>()
            .copied()
            .unwrap_or_default();
        let collect = self
            .sim
            .world
            .get_resource::<sim::flags::Collect>()
            .copied()
            .unwrap_or_default();
        let night_stop = sim::daynight::check_event_night_stop(&flags, &collect);
        self.sim
            .world
            .resource_mut::<sim::daynight::DayNight>()
            .apply_stage_init(tables.stag().map(|stag| stag.time_h), night_stop);
        let has_sea = tables.has_sea_actor();
        {
            let tool_id = tables
                .stag()
                .map_or(-1i32, |stag| i32::from(stag.camera_map_tool_id));
            let mut room = self.sim.world.resource_mut::<sim::cam_room::CamRoom>();
            room.set_stage(&tables);
            let resolved = room.type_from_map_tool_id(tool_id, -1);
            if resolved != sim::cam_room::NO_CAM_TYPE {
                let index = usize::from(resolved);
                let mode = self.sim.world.resource::<sim::camera::Camera>().mode;
                self.sim
                    .world
                    .resource_mut::<sim::camera::Camera>()
                    .set_style(index, mode);
            }
        }
        let frame = self.sim.world.resource::<sim::frame::CurrentFrame>().0;
        let stories = *self.sim.world.resource::<sim::flags::StoryBits>();
        let set = if rooms.is_empty() {
            None
        } else {
            load_rooms(&tables, &rooms, stories, layer)
        };
        // `dScnPly_ply_c` phase 2 mounts this stage's particle archive (`d_s_play.cpp:1378-1379`)
        // ahead of the new stage's own create pass, with the room `FILI` number
        // (`d_s_room.cpp:237-241`) as the fallback the reader's first-read rule leaves it.
        let particle_no = tables
            .stag()
            .map_or(sim::jpa::NO_SCENE, |stag| stag.particle_no());
        self.mount_scene_particles(particle_no, set.as_ref().and_then(|set| set.particle_no));
        *self.sim.world.resource_mut::<sim::transition::NextStage>() =
            sim::transition::NextStage::default();
        {
            let mut area = self.sim.world.resource_mut::<sim::area::Area>();
            area.spawn = spawn;
            area.yaw = yaw;
        }
        if has_sea {
            let sea = sim::sea::Sea::new(sim::sea::SeaTables::new(|room| tables.wave_max(room)));
            self.sim.world.insert_resource(sea);
        } else {
            self.sim.world.remove_resource::<sim::sea::Sea>();
        }
        self.sim
            .world
            .insert_resource(sim::area::StageName(tables.name().to_owned()));
        self.sim
            .world
            .insert_resource(assets::load_event_list(tables.name()).unwrap_or_default());
        self.sim
            .world
            .insert_resource(sim::event::StageEvents(tables.event_names().to_vec()));
        self.sim
            .world
            .insert_resource(sim::event::ActiveEvent::default());
        let mut partner = sim::event::ItemPartner::default();
        partner.max_life = self.sim.world.resource::<sim::health::Health>().max;
        self.sim.world.insert_resource(partner);
        sim::clear_actors(&mut self.sim.world);
        self.sim.world.insert_resource(sim::door::StayRoom(
            rooms.first().map_or(-1, |room| *room as i8),
        ));
        self.room_actors.clear();
        self.sim
            .world
            .resource_mut::<sim::flags::ZoneActors>()
            .clear();
        *self.sim.world.resource_mut::<sim::talk::Dialogue>() = sim::talk::Dialogue::default();
        sim::weight::init(&mut self.sim.world);
        sim::spawn_actors(&mut self.sim.world, &tables);
        Self::install_tag_events(&mut self.sim.world, tables.actors());
        if let Some(set) = set {
            self.install_room_set(set, frame);
        }
        *self.sim.world.resource_mut::<sim::swim::Swim>() = sim::swim::Swim::default();
        *self.sim.world.resource_mut::<sim::roll::Roll>() = sim::roll::Roll::default();
        *self.sim.world.resource_mut::<sim::hop::Hop>() = sim::hop::Hop::default();
        let mut query = self.sim.world.query::<&mut sim::player::Player>();
        if let Some(mut player) = query.iter_mut(&mut self.sim.world).next() {
            player.pos = spawn;
            player.yaw = yaw;
            player.speed = 0.0;
            player.speed_y = 0.0;
        }
        *self.sim.world.resource_mut::<sim::collision::MoveFrom>() =
            sim::collision::MoveFrom(spawn);
        self.stage = Some(tables);
        self.rooms = rooms;
        self.start_layer = layer;
    }

    fn install_tag_events(world: &mut World, sources: &[dzs::StageActor]) {
        let indices: Vec<Option<usize>> = match (
            world.get_resource::<sim::event::StageEvents>(),
            world.get_resource::<sim::event::EventList>(),
        ) {
            (Some(names), Some(list)) => (0..=0xFE)
                .map(|id| names.name(id as u8).and_then(|name| list.event_index(name)))
                .collect(),
            _ => return,
        };
        let mut query =
            world.query_filtered::<(Entity, &sim::actor::Actor), Without<sim::flags::TagEvent>>();
        let pending: Vec<(Entity, sim::flags::TagEvent)> = query
            .iter(world)
            .filter(|(_, actor)| sim::flags::is_tag_event(&actor.name))
            .filter_map(|(entity, actor)| {
                let index = indices
                    .get((actor.parameters >> 24) as usize)
                    .copied()
                    .flatten()?;
                let source = sources.iter().find(|source| {
                    source.name == actor.name
                        && source.pos[0].to_bits() == actor.pos.x.to_bits()
                        && source.pos[1].to_bits() == actor.pos.y.to_bits()
                        && source.pos[2].to_bits() == actor.pos.z.to_bits()
                });
                let scale = source.map_or([1.0, 1.0, 1.0], |source| source.scale);
                Some((
                    entity,
                    sim::flags::tag_event(
                        actor.parameters,
                        actor.angle[2] as u16,
                        scale,
                        actor.name == sim::flags::TAG_ISLAND_ACTOR,
                        index,
                    ),
                ))
            })
            .collect();
        for (entity, tag) in pending {
            world.entity_mut(entity).insert(tag);
        }
        let mut hints =
            world.query_filtered::<(Entity, &sim::actor::Actor), Without<sim::talk::Talk>>();
        let pending: Vec<(Entity, u16, i8)> = hints
            .iter(world)
            .filter(|(_, actor)| actor.name == "TagHt" || actor.name == "TagHt2")
            .map(|(entity, actor)| (entity, actor.angle[0] as u16, actor.room))
            .collect();
        for (entity, msg, room) in pending {
            world
                .entity_mut(entity)
                .insert(sim::talk::Talk::new(msg, room));
        }
    }

    /// `dScnPly_ply_c` phase 2's stage-side half: `dStage_stagInfo_GetParticleNo` read off the
    /// stage's stag info (`d_s_play.cpp:1378-1379`) hands `dPa_control_c::readScene` the
    /// `Pscene%03d.jpc` that `createScene` mounts (`d_particle.cpp:757-780`), and the room-side
    /// half is `dStage_FileList_dt_GetParticleNo` off the first room that names one
    /// (`d_s_room.cpp:237-241`); `readScene`'s first-read rule is what makes the room's the
    /// fallback. Every stage load runs it - the boot stage through [`App::reset_world`], every
    /// transition through [`App::swap_stage`].
    fn mount_scene_particles(&mut self, particle_no: u8, room_particle_no: Option<u8>) {
        sim::jpa::mount_scene_archive(
            &mut self.sim.world.resource_mut::<sim::jpa::Jpa>(),
            particle_no,
            room_particle_no,
        );
    }

    fn install_room_set(&mut self, set: assets::RoomSet, frame: u64) {
        // `d_s_room.cpp:220-231`: the room's `LBNK` names the demo archive the cut models come out
        // of, and the mount is what makes them readable.
        self.sim
            .world
            .resource_mut::<sim::demo::RoomDemoArchive>()
            .name_from_room(set.demo_archive.as_deref());
        self.sim
            .world
            .resource_mut::<sim::cam_room::CamRoom>()
            .set_rooms(set.cameras);
        match set.collision {
            Some(collision) => {
                let (bounds_min, bounds_max) = collision.bounds();
                let mut area = self.sim.world.resource_mut::<sim::area::Area>();
                area.bounds_min = bounds_min;
                area.bounds_max = bounds_max;
                self.sim.world.insert_resource(collision);
            }
            None => {
                self.sim
                    .world
                    .remove_resource::<sim::collision::Collision>();
                let default = sim::area::Area::default();
                let mut area = self.sim.world.resource_mut::<sim::area::Area>();
                area.bounds_min = default.bounds_min;
                area.bounds_max = default.bounds_max;
            }
        }
        self.room = set.mesh;
        self.room_exits = set.exits;
        self.room_tick = frame;
        self.install_room_actors(set.actors);
    }

    fn install_room_actors(&mut self, actors: Vec<dzs::StageActor>) {
        for entity in std::mem::take(&mut self.room_actors) {
            let world = &mut self.sim.world;
            if world
                .resource_entities()
                .iter()
                .any(|(_, held)| held == entity)
            {
                continue;
            }
            if world.entities().contains(entity) {
                let _ = world.despawn(entity);
            }
        }
        let zones = *self.sim.world.resource::<sim::flags::ZoneActors>();
        self.room_actors = actors
            .iter()
            .filter(|source| !zones.is_done(source.room, source.set_id))
            .map(|source| sim::spawn_actor(&mut self.sim.world, source))
            .collect();
        Self::install_tag_events(&mut self.sim.world, &actors);
    }

    pub fn sync_rooms(&mut self) {
        let stories = *self.sim.world.resource::<sim::flags::StoryBits>();
        let Some(tables) = self.stage.as_ref() else {
            return;
        };
        if !tables.has_sea_actor() {
            return;
        }
        let player = self
            .sim
            .world
            .iter_entities()
            .find_map(|entity| entity.get::<sim::player::Player>().copied());
        let Some(player) = player else {
            return;
        };
        let room = tables.room_at(player.pos.x, player.pos.z);
        let fallback = std::slice::from_ref(&room);
        let requested = tables.load_set(room);
        let requested = if requested.is_empty() {
            fallback
        } else {
            requested
        };
        if requested == self.rooms {
            return;
        }
        let rooms = requested.to_vec();
        match load_rooms(tables, &rooms, stories, self.start_layer) {
            Some(set) => {
                let frame = self.sim.world.resource::<sim::frame::CurrentFrame>().0;
                self.install_room_set(set, frame);
            }
            None => self.install_room_actors(Vec::new()),
        }
        self.rooms = rooms;
    }
}

/// `dComIfG_play_c::getLayerNo` (`d_com_inf_game.cpp:184-216`): a start-stage layer
/// from `dComIfGp_setNextStage` wins for the whole stage session, and only
/// [`sim::transition::LAYER_NONE`] falls through to the story-bit table
/// [`sim::layer::no`] holds.
pub(crate) fn load_rooms(
    tables: &dzs::StageTables,
    rooms: &[u8],
    stories: sim::flags::StoryBits,
    layer: i8,
) -> Option<assets::RoomSet> {
    if layer >= 0 {
        let layer = layer as u8;
        return assets::load_room_set(tables, rooms, move |_room| layer);
    }
    assets::load_room_set(tables, rooms, |room| {
        sim::layer::no(tables.name(), room, &stories)
    })
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Platform entry: the desktop window, or on the web the shell loop [`web::boot`] starts once a
/// disc image is installed; the loop stages input, steps the sim, pumps audio, composes.
///
/// Desktop resolves its image here, before anything reads assets: `--disc <path>` or a bare
/// image path, else `disc.iso`/`disc.rvz` beside the executable or in the data dir, else the
/// extracted [`assets::ASSETS_ENV`] tree. An image that is named or found but unreadable exits
/// with the typed reason on stderr rather than opening a window over a session without assets.
#[cfg(not(target_os = "android"))]
pub fn run() {
    use std::time::Duration;
    use web_time::Instant;

    #[cfg(not(target_arch = "wasm32"))]
    {
        let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
        if args.iter().any(|arg| arg == "--help" || arg == "-h") {
            println!("{}", source::USAGE);
            return;
        }
        match source::install_launch(args) {
            Ok(Some(summary)) => eprintln!("clearsleeper: disc image {summary}"),
            Ok(None) => {}
            Err(error) => {
                eprintln!("clearsleeper: {error}");
                eprintln!("{}", source::USAGE);
                std::process::exit(1);
            }
        }
    }

    let mut app = App::new_live();
    let mut last = Instant::now();
    let root = move |sched: &mut repose_core::runtime::Scheduler,
                     ctx: &repose_core::RenderContext| {
        repose_core::request_frame();
        let now = Instant::now();
        let dt = now.duration_since(last).min(Duration::from_secs_f32(0.25));
        last = now;
        app.stage_polled(sched);
        app.stage_camera();
        app.advance(dt);
        app.pump_audio(dt);
        root_view(&app, sched, ctx)
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        let size = std::env::var("CS_WINDOW")
            .ok()
            .and_then(|v| {
                let (a, b) = v.split_once('x')?;
                Some((a.parse().ok()?, b.parse().ok()?))
            })
            .unwrap_or((1280, 720));
        repame_shell::run_desktop("clearsleeper", size, root)
            .expect("clearsleeper failed to start");
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = repame_shell::run_web(root);
    }
}

pub fn root_view(
    app: &App,
    _sched: &mut repose_core::runtime::Scheduler,
    ctx: &repose_core::RenderContext,
) -> repose_core::View {
    if app.quit.latched() {
        return quit::Quit::parked();
    }
    if !app.assets_installed {
        return pre_boot_view(app, ctx);
    }
    if app.intro.is_some() {
        return intro_view(app, ctx);
    }
    if app.menu.screen == menus::Screen::Title {
        return title_screen_view(app, ctx);
    }
    let snapshot = app.snapshot();
    let mut camera = app.game_camera().unwrap_or_else(|| app.cam.get());
    if app.camera_underground(&camera, &snapshot) {
        camera = app.fallback_camera(&snapshot);
    }
    let frame = match &app.dff {
        Some(presented) => j3d::present::frame(presented, camera),
        None => view::build_frame_with_actor(
            &snapshot,
            camera,
            app.room.as_ref(),
            app.actor_model.as_ref(),
            app.player_anim.as_ref(),
            app.hair_model.as_ref(),
        ),
    };
    let desc = match &app.dff {
        Some(_) => view::batch_desc(app.room.as_ref()),
        None => view::batch_desc_with_actor(
            app.room.as_ref(),
            app.actor_model.as_ref(),
            app.hair_model.as_ref(),
        ),
    };
    let cam = app.cam.clone();
    let viewport = Viewport3d(
        frame,
        GeomHandle::new(),
        "scene.main",
        desc,
        move |ev| match ev {
            View3dEvent::Orbit { dx, dy } => cam.update(|c| c.orbit(dx, dy)),
            View3dEvent::Zoom { factor } => cam.update(|c| c.zoom(factor)),
            _ => {}
        },
    );
    let overlay = match app.menu.screen {
        menus::Screen::Playing => meter_hud_view(app, ctx),
        menus::Screen::FileSelect => menus::file_select_view(&app.menu, ctx),
        menus::Screen::GameOver => game_over_overlay(app, ctx),
        // The shell's own screens belong to the pre-boot session; a session that somehow
        // lands on one of them draws the retail frame alone rather than the shell over it.
        menus::Screen::Title | menus::Screen::Settings | menus::Screen::QuitConfirm => Spacer(),
    };
    ZStack(Modifier::new().fill_max_size()).child([viewport, overlay, dialogue_view(app, ctx)])
}

/// The host shell, and the only place it exists: with no asset source installed there is no
/// retail screen to draw, so the shell's title, options and quit stand in - the boundary
/// Dusklight's own launcher keeps. Once a disc is installed [`root_view`] never reaches here.
fn pre_boot_view(app: &App, _ctx: &repose_core::RenderContext) -> repose_core::View {
    match app.menu.screen {
        menus::Screen::Title => menus::title_view(&app.menu),
        menus::Screen::Settings => menus::settings_view(&app.menu),
        menus::Screen::QuitConfirm => menus::quit_confirm_view(&app.menu),
        // A session only starts with an asset source (`start_new`), and the retail file
        // select needs the disc's panes, so neither belongs to the pre-boot shell.
        menus::Screen::FileSelect | menus::Screen::Playing | menus::Screen::GameOver => Spacer(),
    }
}

/// `dGameover_c::_draw` (`d_gameover.cpp:168-175`): the letters screen, then the
/// save menu's `_draw2`, composited over the live view.
fn game_over_overlay(app: &App, ctx: &repose_core::RenderContext) -> repose_core::View {
    let Some(stage) = app.game_over.as_ref() else {
        return Spacer();
    };
    let handle = match stage.image.get() {
        Some(handle) => handle,
        None => {
            let handle = ctx.alloc_image_handle();
            stage.image.set(Some(handle));
            handle
        }
    };
    if stage.uploaded.get() != stage.revision {
        let mut canvas = text::raster::Canvas::new(640, 480);
        game_over_screen::render(&mut canvas, &stage.assets, &stage.game);
        panes::draw_save_menu(&mut canvas, &stage.game.save.panes, &stage.save_assets);
        ctx.set_image_rgba8(handle, 640, 480, canvas.pixels, true);
        stage.uploaded.set(stage.revision);
    }
    Image(Modifier::new().fill_max_size(), handle).image_fit(ImageFit::Contain)
}

fn intro_view(app: &App, ctx: &repose_core::RenderContext) -> repose_core::View {
    let Some(stage) = app.intro.as_ref() else {
        return Spacer();
    };
    if stage.canvas.pixels.is_empty() {
        return Spacer();
    }
    let handle = match stage.image.get() {
        Some(handle) => handle,
        None => {
            let handle = ctx.alloc_image_handle();
            stage.image.set(Some(handle));
            handle
        }
    };
    if stage.uploaded.get() != stage.revision {
        ctx.set_image_rgba8(handle, 640, 480, stage.canvas.pixels.clone(), true);
        stage.uploaded.set(stage.revision);
    }
    Image(Modifier::new().fill_max_size(), handle).image_fit(ImageFit::Contain)
}

fn title_screen_view(app: &App, ctx: &repose_core::RenderContext) -> repose_core::View {
    let Some(stage) = app.title_stage.as_ref() else {
        return Spacer();
    };
    if stage.canvas.pixels.is_empty() {
        return Spacer();
    }
    let handle = match stage.image.get() {
        Some(handle) => handle,
        None => {
            let handle = ctx.alloc_image_handle();
            stage.image.set(Some(handle));
            handle
        }
    };
    if stage.uploaded.get() != stage.revision {
        ctx.set_image_rgba8(handle, 640, 480, stage.canvas.pixels.clone(), true);
        stage.uploaded.set(stage.revision);
    }
    Image(Modifier::new().fill_max_size(), handle).image_fit(ImageFit::Contain)
}

/// The retail meter's heart row over the 3D view (`d_meter.cpp`'s `meter1_draw` layers build
/// the same screen; this composes the hearts it already ports).
fn meter_hud_view(app: &App, ctx: &repose_core::RenderContext) -> repose_core::View {
    let Some(stage) = app.meter.as_ref() else {
        return Spacer();
    };
    if stage.canvas.pixels.is_empty() {
        return Spacer();
    }
    let handle = match stage.image.get() {
        Some(handle) => handle,
        None => {
            let handle = ctx.alloc_image_handle();
            stage.image.set(Some(handle));
            handle
        }
    };
    if stage.uploaded.get() != stage.revision {
        ctx.set_image_rgba8(handle, 640, 480, stage.canvas.pixels.clone(), true);
        stage.uploaded.set(stage.revision);
    }
    Image(Modifier::new().fill_max_size(), handle).image_fit(ImageFit::Contain)
}

fn dialogue_view(app: &App, ctx: &repose_core::RenderContext) -> repose_core::View {
    if !app.dialogue_live || app.dialogue_canvas.pixels.is_empty() {
        return Spacer();
    }
    let handle = match app.dialogue_image.get() {
        Some(handle) => handle,
        None => {
            let handle = ctx.alloc_image_handle();
            app.dialogue_image.set(Some(handle));
            handle
        }
    };
    if app.dialogue_uploaded.get() != app.dialogue_revision {
        ctx.set_image_rgba8(handle, 640, 480, app.dialogue_canvas.pixels.clone(), true);
        app.dialogue_uploaded.set(app.dialogue_revision);
    }
    Image(Modifier::new().fill_max_size(), handle).image_fit(ImageFit::Contain)
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
    if let Some(dir) = android_app.internal_data_path() {
        game_utils::set_android_data_dir(dir);
    }
    match android::install_source(&android_app) {
        Ok(Some(summary)) => log::info!("clearsleeper: disc image {summary}"),
        Ok(None) => log::warn!(
            "clearsleeper: no disc image picked or found; {} is unset on a device, so the \
             session boots without assets",
            assets::ASSETS_ENV
        ),
        Err(error) => log::error!("clearsleeper: disc image rejected ({error})"),
    }
    let mut app = App::new_live();
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
        app.pump_audio(dt);
        root_view(&app, sched, ctx)
    }) {
        log::error!("clearsleeper: android run failed: {e:?}");
    }
}
