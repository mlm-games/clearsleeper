use std::cell::Cell;
use std::time::Duration;

use anyhow::Context;
use repame_audio::{Audio, CueRequest};
use repose_core::{Color, ImageFit, Modifier, RenderContext, View};
use repose_ui::{Center, Image, ImageExt, Spacer, Text, ViewExt};

use crate::{
    App, assets, audio, menus, movie, panes, save, sim, text, ui_file_select as files,
    ui_menu_save as saves,
};

pub(crate) enum Scene {
    Movie(MovieScene),
    Save(Box<ClearSave>),
    Failed(String),
}

#[derive(Default)]
pub(crate) struct SceneImage {
    handle: Cell<Option<repose_core::ImageHandle>>,
    pixels: Cell<Option<(u32, u32, Vec<u8>)>>,
}

impl SceneImage {
    pub(crate) fn set(&self, width: u32, height: u32, rgba: Vec<u8>) {
        self.pixels.set(Some((width, height, rgba)));
    }

    fn view(&self, ctx: &RenderContext) -> View {
        if let Some((w, h, rgba)) = self.pixels.take() {
            let handle = self
                .handle
                .get()
                .unwrap_or_else(|| ctx.alloc_image_handle());
            ctx.set_image_rgba8(handle, w, h, rgba, true);
            self.handle.set(Some(handle));
        }
        self.handle.get().map_or_else(Spacer, |handle| {
            Image(
                Modifier::new().fill_max_size().background(Color::BLACK),
                handle,
            )
            .image_fit(ImageFit::Contain)
        })
    }
}

enum Sound {
    Decode { next: usize, pcm: Vec<i16> },
    Prepare(Vec<u8>),
    Playing,
}

pub(crate) struct MovieScene {
    movie: movie::Movie,
    script: files::TitleScene,
    sound: Sound,
    cue: &'static str,
    elapsed: Duration,
    displayed: usize,
}

impl MovieScene {
    fn open(
        scene: files::SceneName,
        count: u8,
        audio: &Audio,
        image: &SceneImage,
    ) -> anyhow::Result<Self> {
        // tww/src/d/actor/d_a_movie_player.cpp:4225-4228; tww/src/d/d_s_title.cpp:116-122.
        let path = if scene == files::SceneName::Ending {
            movie::ENDING_PATH
        } else {
            movie::TITLE_PATH
        };
        let root = assets::AssetRoot::from_env()?;
        let mut movie = movie::Movie::parse(root.stream(path)?).with_context(|| path)?;
        let (w, h, rgb) = movie.frame(0)?;
        image.set(w, h, movie::to_rgba(&rgb));
        let mut script = files::TitleScene {
            clear_count: count,
            ..Default::default()
        };
        script.create(scene);
        let sound = if audio.is_live() {
            Sound::Decode {
                next: 0,
                pcm: Vec::new(),
            }
        } else {
            Sound::Playing
        };
        Ok(Self {
            movie,
            script,
            sound,
            cue: path,
            elapsed: Duration::ZERO,
            displayed: 0,
        })
    }

    fn prepare_audio(&mut self, audio: &mut Audio) -> anyhow::Result<bool> {
        // tww/src/d/actor/d_a_movie_player.cpp:2979-3005: decode each frame's own stereo block.
        if let Sound::Decode { next, pcm } = &mut self.sound {
            let end = (*next + 128).min(self.movie.len());
            for n in *next..end {
                self.movie.audio(n)?.decode(pcm);
            }
            *next = end;
            if end == self.movie.len() {
                self.sound = Sound::Prepare(movie::wave(pcm));
            }
            return Ok(false);
        }
        if let Sound::Prepare(wav) = &self.sound {
            match audio.request_cue(self.cue, audio::movie_cue(), &[wav]) {
                CueRequest::Queued => return Ok(false),
                CueRequest::Ready => {
                    audio.play(self.cue);
                }
                CueRequest::Failed(e) => log::warn!("clearsleeper: movie audio unavailable: {e}"),
            }
            self.sound = Sound::Playing;
            return Ok(false);
        }
        Ok(true)
    }

    fn advance(
        &mut self,
        dt: Duration,
        pad: &files::PadInput,
        audio: &mut Audio,
        image: &SceneImage,
    ) -> anyhow::Result<Option<files::SceneName>> {
        // tww/src/d/d_s_title.cpp:69-89; tww/src/d/actor/d_a_movie_player.cpp:3682-3687.
        if self.prepare_audio(audio)? {
            self.elapsed += dt;
        }
        let count = self.movie.len();
        let frame = (self.elapsed.as_secs_f64() * f64::from(self.movie.rate())) as usize;
        let index = frame.min(count - 1);
        if self.displayed != index {
            let (w, h, rgb) = self.movie.frame(index)?;
            image.set(w, h, movie::to_rgba(&rgb));
            self.displayed = index;
        }
        self.script.movie_rest_frame = count.saturating_sub(frame) as i32;
        self.script.execute(pad, false, false, true);
        Ok(self.script.requests.change_to.take())
    }
}

#[derive(Clone, Copy)]
enum Phase {
    Open(u8),
    Question,
    Close,
    Pick,
    ClosePick,
    Reset(u8),
}

pub(crate) struct ClearSave {
    phase: Phase,
    save: saves::SaveMenu,
    assets: panes::SaveAssets,
    extra: save::SaveGame,
    flags: saves::MenuFlags,
    system: saves::SystemRequest,
    particles: saves::Particles,
}

impl App {
    pub(crate) fn begin_movie(&mut self, scene: files::SceneName) {
        let count = self
            .sim
            .world
            .resource::<text::driver::TextFlags>()
            .clear_count;
        self.menu.screen = menus::Screen::Movie;
        self.ending = Some(
            match MovieScene::open(scene, count, &self.audio, &self.scene_image) {
                Ok(movie) => {
                    if scene == files::SceneName::Ending {
                        self.audio.stop_music(30.0 / crate::SIM_HZ as f32);
                        self.sim
                            .world
                            .resource_mut::<sim::events::SimEvents>()
                            .list
                            .push(sim::events::SimEvent::Credits);
                    }
                    Scene::Movie(movie)
                }
                Err(err) => Scene::Failed(format!("Could not open the movie: {err:#}")),
            },
        );
        self.clear_pad_edges();
    }

    pub(crate) fn step_movie(&mut self, dt: Duration) {
        let pad = self.file_select_pad();
        let Some(Scene::Movie(reel)) = self.ending.as_mut() else {
            if pad.any_confirm() {
                self.return_to_title();
            }
            return;
        };
        let result = reel.advance(dt, &pad, &mut self.audio, &self.scene_image);
        if matches!(result, Ok(None)) {
            return;
        }
        self.audio.stop_cue(reel.cue);
        if reel.script.scene == files::SceneName::Ending && result.is_ok() {
            self.sim
                .world
                .resource_mut::<sim::events::SimEvents>()
                .list
                .push(sim::events::SimEvent::CreditsDone);
        }
        match result {
            Ok(Some(files::SceneName::NameSelect)) => {
                if let Err(err) = self.begin_clear_save() {
                    self.ending = Some(Scene::Failed(format!(
                        "Could not open the clear-data menu: {err:#}"
                    )));
                }
            }
            Ok(_) => self.return_to_title(),
            Err(err) => {
                self.ending = Some(Scene::Failed(format!("Could not read the movie: {err:#}")))
            }
        }
    }

    pub(crate) fn return_to_title(&mut self) {
        // tww/src/d/d_s_title.cpp:48,110-132: returning to the title recreates the title actor.
        self.ending = None;
        self.menu.screen = menus::Screen::Title;
        if let Some(actor) = self.title_actor.take() {
            self.sim.world.despawn(actor);
        }
        self.sim
            .world
            .insert_resource(sim::title::TitleScene::default());
        self.sim
            .world
            .insert_resource(sim::title::TitleKirakira::default());
        self.clear_pad_edges();
        self.sync_title_actor();
    }

    fn begin_clear_save(&mut self) -> anyhow::Result<()> {
        // tww/src/d/d_s_name.cpp:171-179,236-249: first clear opens save use type 3.
        let root = assets::AssetRoot::from_env()?;
        let assets = panes::SaveAssets::load(&root).context(saves::SCREEN_ARCHIVE)?;
        let mut menu = saves::SaveMenu::default();
        menu.set_use_type(3);
        assets.bind(&mut menu);
        let previous = *self.sim.world.resource::<sim::flags::StoryBits>();
        self.reset_world_at(assets::PROLOGUE_STAGE, assets::PROLOGUE_SPAWN_POINT);
        // tww/src/d/d_s_name.cpp:171-174: NAMEEX sets the clear count for the rest of the session.
        self.sim
            .world
            .resource_mut::<text::driver::TextFlags>()
            .clear_count = 1;
        self.sim
            .world
            .resource_mut::<text::driver::TextDriver>()
            .flags
            .clear_count = 1;
        let mut extra = self.save();
        extra.prepare_second_quest(&previous);
        self.menu.card.restore();
        self.menu.screen = menus::Screen::ClearSave;
        self.ending = Some(Scene::Save(Box::new(ClearSave {
            phase: Phase::Open(20),
            save: menu,
            assets,
            extra,
            flags: Default::default(),
            system: Default::default(),
            particles: Default::default(),
        })));
        self.step_clear_save();
        Ok(())
    }

    pub(crate) fn step_clear_save(&mut self) {
        let pad = self.menu_pad();
        let picker_pad = self.file_select_pad();
        if !matches!(self.ending, Some(Scene::Save(_))) {
            if picker_pad.any_confirm() {
                self.return_to_title();
            }
            return;
        }
        let Some(Scene::Save(clear)) = self.ending.as_mut() else {
            return;
        };
        // tww/src/d/d_s_name.cpp:1390-1430,1230-1252.
        match clear.phase {
            Phase::Open(n) if n > 0 => clear.phase = Phase::Open(n - 1),
            Phase::Open(_) | Phase::Question | Phase::Close => {
                let mut health = *self.sim.world.resource::<sim::health::Health>();
                let mut vibration = self.sim.world.resource_mut::<sim::vibration::Vibration>();
                let mut env = saves::SaveEnv {
                    pad,
                    card: &mut self.menu.card,
                    menu: &mut clear.flags,
                    system: &mut clear.system,
                    particles: &mut clear.particles,
                    pending_save: None,
                    life_on_open: u16::from(health.units),
                    health: &mut health,
                    vibration: &mut vibration,
                };
                match clear.phase {
                    Phase::Open(_) if clear.save.open(&mut env) => clear.phase = Phase::Question,
                    Phase::Question => {
                        clear.save.move_step(&mut env);
                        if clear.save.end_status == 0 || env.system.is_reset() {
                            clear.phase = Phase::Reset(45);
                        } else if clear.save.save_status == 3 {
                            clear.phase = Phase::Close;
                        }
                    }
                    Phase::Close if clear.save.close(&mut env) => {
                        self.menu.open_file_select_mode(files::MODE_EXTRA);
                        self.menu.sync_file_select_cursor();
                        clear.phase = Phase::Pick;
                    }
                    _ => {}
                }
            }
            Phase::Pick => {
                if !self.menu.tick_file_select() {
                    return;
                }
                let fs = &mut self.menu.fs;
                let payload = (fs.proc == files::proc_ex_data_save).then(|| clear.extra.clone());
                fs.move_step(&picker_pad, &mut self.menu.card, payload);
                match fs.is_select_end() {
                    files::SELECT_END_START => clear.phase = Phase::Reset(45),
                    files::SELECT_END_CANCEL | files::SELECT_END_RETURN => {
                        clear.phase = Phase::ClosePick
                    }
                    _ => {}
                }
                self.menu
                    .revision
                    .set(self.menu.revision.get().wrapping_add(1));
            }
            Phase::ClosePick => {
                // tww/src/d/d_s_name.cpp:1255-1271,1285-1289.
                self.menu
                    .revision
                    .set(self.menu.revision.get().wrapping_add(1));
                if self.menu.fs.close() {
                    clear.save.initialize();
                    if self.menu.fs.is_select_end() == files::SELECT_END_CANCEL {
                        clear.save.set_error_flag(1);
                        clear.save.set_error_type(self.menu.fs.err_type);
                    }
                    clear.phase = Phase::Open(0);
                }
            }
            Phase::Reset(0) => {
                self.return_to_title();
                return;
            }
            Phase::Reset(n) => {
                clear.phase = Phase::Reset(n - 1);
                return;
            }
        }
        if !matches!(
            clear.phase,
            Phase::Pick | Phase::ClosePick | Phase::Reset(_)
        ) {
            clear.assets.sync_text(&mut clear.save, 1);
            let mut canvas = text::raster::Canvas::new(640, 480);
            clear.assets.draw(&mut canvas, &clear.save);
            self.scene_image.set(640, 480, canvas.pixels);
        }
    }

    pub(crate) fn ending_view(&self, ctx: &RenderContext) -> View {
        match &self.ending {
            Some(Scene::Failed(error)) => {
                Center(Modifier::new().fill_max_size().background(Color::BLACK)).child(Text(
                    format!("{error}\nPress A, B or Start to return to the title."),
                ))
            }
            Some(Scene::Save(clear)) if matches!(clear.phase, Phase::Pick | Phase::ClosePick) => {
                menus::file_select_view(&self.menu, ctx)
            }
            _ => self.scene_image.view(ctx),
        }
    }
}
