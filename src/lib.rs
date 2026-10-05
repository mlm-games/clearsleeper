//! Wind Waker recreation on repame: fixed-step sim + repose views, no bevy.

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
use wasm_bindgen::prelude::*;

pub mod assets;
pub mod audio;
pub mod bdl;
pub mod btk;
pub mod dzs;
pub mod hud;
pub mod input;
pub mod j2d;
pub mod menus;
pub mod movie;
pub mod save;
pub mod sim;
pub mod text;
pub mod view;

/// Fixed gameplay tick rate: 30 Hz (NTSC pacing).
pub const SIM_HZ: f64 = 30.0;

pub struct App {
    pub sim: Sim,
    pub cam: Signal<OrbitCamera>,
    pub audio: repame_audio::Audio,
    pub room: Option<bdl::RoomMesh>,
    pub menu: menus::Menu,
    stage: Option<dzs::StageTables>,
    rooms: Vec<u8>,
    room_tick: u64,
    room_actors: Vec<Entity>,
    sea_shade: f32,
    save_dir: PathBuf,
    name: String,
    menu_prev: HashSet<PhysicalKey>,
    movie: Option<movie::Movie>,
    movie_frame: usize,
    movie_clock: f32,
    movie_pixels: Vec<u8>,
    movie_size: (u32, u32),
    movie_image: Cell<Option<repose_core::ImageHandle>>,
    movie_uploaded: Cell<usize>,
    movie_sound: bool,
    movie_done: bool,
}

impl App {
    /// Headless construction: silent audio, no device. Tests and tools.
    pub fn new() -> Self {
        let mut app = Self::with_audio(repame_audio::Audio::noop());
        app.menu.screen = menus::Screen::Playing;
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
            menu: menus::Menu::new(),
            stage: None,
            rooms: Vec::new(),
            room_tick: 0,
            room_actors: Vec::new(),
            sea_shade,
            save_dir: save::slots_dir(),
            name: save::DEFAULT_NAME.to_string(),
            menu_prev: HashSet::new(),
            movie: None,
            movie_frame: 0,
            movie_clock: 0.0,
            movie_pixels: Vec::new(),
            movie_size: (0, 0),
            movie_image: Cell::new(None),
            movie_uploaded: Cell::new(usize::MAX),
            movie_sound: false,
            movie_done: false,
        };
        app.movie = movie::Movie::open(&movie::title_path()).ok();
        app.reset_world();
        app
    }

    fn reset_world(&mut self) {
        self.sim = Sim::new(Duration::from_secs_f64(1.0 / SIM_HZ));
        self.stage = assets::load_stage_tables(assets::SEA_STAGE);
        sim::build(
            &mut self.sim,
            assets::load_stage(assets::SEA_STAGE),
            self.stage.as_ref(),
        );
        self.rooms = self.stage.as_ref().map(assets::start_set).unwrap_or_default();
        self.sim
            .world
            .insert_resource(sim::door::StayRoom(-1));
        self.room_tick = 0;
        self.room_actors.clear();
        let spawn = self.sim.world.resource::<sim::area::Area>().spawn;
        self.cam.set(OrbitCamera {
            target: spawn,
            ..OrbitCamera::default()
        });
        let stories = *self.sim.world.resource::<sim::flags::StoryBits>();
        let set = self
            .stage
            .as_ref()
            .and_then(|tables| load_rooms(tables, &self.rooms, stories));
        if let Some(set) = set {
            self.install_room_set(set, 0);
        }
    }

    /// Frame boundary: poll the shell into the sim's intent resources.
    pub fn stage_polled(&mut self, sched: &repose_core::runtime::Scheduler) {
        let held = &sched.held_keys;
        self.stage_axis(input::axis_from_keys(held, sched.window_focused));
        self.stage_damage(input::damage_from_keys(held, sched.window_focused));
        self.stage_do(input::do_from_keys(held, sched.window_focused));
        self.stage_target(input::target_from_keys(held, sched.window_focused));
        self.stage_item(input::item_from_keys(held, sched.window_focused));
        self.stage_use(input::use_from_keys(held, sched.window_focused));
        self.stage_menu(held, sched.window_focused);
    }

    fn stage_menu(&mut self, held: &HashSet<PhysicalKey>, window_focused: bool) {
        if self.menu.screen != menus::Screen::Playing {
            self.stage_axis(glam::Vec2::ZERO);
            self.stage_damage(0);
            self.stage_do(false);
            self.stage_target(false);
            self.stage_item(false);
            self.stage_use(false);
        }
        let mut cmds = if window_focused {
            menus::cmds_from(held, &self.menu_prev)
        } else {
            Vec::new()
        };
        if self.menu.screen == menus::Screen::Title && self.movie_done {
            cmds.push(menus::Cmd::Confirm);
            self.movie_done = false;
        }
        self.menu_prev.clone_from(held);
        if self.menu.screen == menus::Screen::Playing || cmds.is_empty() {
            return;
        }
        let before = self.menu.screen;
        let action = self.menu.step(&cmds);
        if before == menus::Screen::Title && self.menu.screen == menus::Screen::FileSelect {
            let slots = (0..save::SLOTS)
                .map(|slot| match self.read_slot(slot) {
                    Some(saved) => menus::SlotInfo::from_save(&saved),
                    None => menus::SlotInfo::Empty,
                })
                .collect();
            self.menu.slots = slots;
        }
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
            Some(menus::Action::GameOver(choice)) => {
                *self.sim.world.resource_mut::<sim::intent::GameOverChoice>() = choice;
                self.menu.screen = match choice {
                    sim::intent::GameOverChoice::Quit => menus::Screen::Title,
                    _ => menus::Screen::Playing,
                };
            }
            None => {}
        }
    }

    fn start_new(&mut self, slot: usize) {
        self.reset_world();
        if let Err(e) = self.write_slot(slot) {
            log::warn!("clearsleeper: slot {slot} write failed ({e})");
        }
        self.menu.screen = menus::Screen::Playing;
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
        if let Some((w, h, pixels)) = self.movie.as_mut().and_then(|m| m.frame(self.movie_frame).ok())
        {
            self.movie_size = (w, h);
            self.movie_pixels = movie::to_rgba(&pixels);
        }
    }

    fn movie_interval(&self) -> f32 {
        self.movie.as_ref().map_or(1.0 / 30.0, movie::Movie::interval)
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

    /// Take every sim event waiting for presentation.
    pub fn drain_events(&mut self) -> Vec<sim::events::SimEvent> {
        std::mem::take(&mut self.sim.world.resource_mut::<sim::events::SimEvents>().list)
    }

    /// Frame boundary for audio: advance voices, realize queued sim events.
    pub fn pump_audio(&mut self, dt: Duration) {
        self.audio.update(dt.as_secs_f32());
        self.audio.take_finished();
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
                | sim::events::SimEvent::Bgm(_) => None,
            };
            if let Some(cue) = cue {
                self.audio.play(cue);
            }
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
                let mut y = collision
                    .ground_at(player_pos.x, player_pos.z)
                    .unwrap_or(player_pos.y);
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
        }
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
                let size = sim::pickup::Pickup {
                    item: present.item,
                    timer: 0,
                }
                .size();
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
            } else if entity.contains::<sim::marlon::Marlon>()
                || entity.contains::<sim::shop::Shop>()
                || entity.contains::<sim::lenzo::Lenzo>()
                || entity.contains::<sim::zunari::Zunari>()
                || entity.contains::<sim::aryll::Aryll>()
                || entity.contains::<sim::medli::Medli>()
                || entity.contains::<sim::grammarian::Grammarian>()
            {
                (view::ActorKind::Talker, 0, 30.0, 12.0)
            } else if entity.contains::<sim::zelda::Zelda>() {
                if entity.get::<sim::zelda::Zelda>().is_some_and(|z| !z.drawn) {
                    continue;
                }
                (view::ActorKind::Talker, 0, 30.0, 12.0)
            } else if entity.contains::<sim::tingle::Tingle>() {
                (view::ActorKind::Scenery, 0, 24.0, 10.0)
            } else if entity.contains::<sim::chest::Chest>()
                || entity.contains::<sim::tbox::Tbox>()
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
            } else if entity.contains::<sim::wall::Wall>() {
                if entity
                    .get::<sim::wall::Wall>()
                    .is_some_and(|w| w.state == sim::wall::State::Break)
                {
                    continue;
                }
                (view::ActorKind::Scenery, 0, 40.0, 20.0)
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
                (
                    view::ActorKind::Fire,
                    0,
                    field.height(),
                    field.radius(true),
                )
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
                (view::ActorKind::Scenery, 0, max[1] - min[1], (max[0] - min[0]) * 0.5)
            } else if entity.contains::<sim::pbka::Pbka>() {
                let (min, max) = sim::pbka::Pbka::spawn().cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], (max[0] - min[0]) * 0.5)
            } else if entity.contains::<sim::dummy::Dummy>() {
                let (min, max) = sim::dummy::Dummy::spawn().cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], (max[0] - min[0]) * 0.5)
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
            } else if entity.contains::<sim::spotbox::SpotBox>() {
                lift = -(sim::spotbox::HALF);
                (
                    view::ActorKind::Scenery,
                    0,
                    sim::spotbox::HALF * 2.0,
                    sim::spotbox::HALF,
                )
            } else if let Some(cell) = entity.get::<sim::door::JailDoor>() {
                if cell.open() {
                    continue;
                }
                (
                    view::ActorKind::Scenery,
                    0,
                    sim::door::JAIL_HEIGHT,
                    40.0,
                )
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
        if self.menu.screen == menus::Screen::Title {
            self.step_movie(dt);
        } else if self.movie_sound {
            self.audio.stop_cue(audio::MOVIE_CUE);
        }
        let steps = self.sim.step(dt);
        let game_over = self.sim.world.resource::<sim::death::GameOver>().active;
        if game_over && self.menu.screen == menus::Screen::Playing {
            self.menu.screen = menus::Screen::GameOver;
            self.menu.slot = 0;
        }
        self.apply_exit();
        self.sync_rooms();
        steps
    }

    pub fn stage(&self) -> Option<&dzs::StageTables> {
        self.stage.as_ref()
    }

    fn apply_exit(&mut self) {
        let next =
            std::mem::take(&mut *self.sim.world.resource_mut::<sim::transition::NextStage>());
        let dest = match (next.target, next.exit) {
            (Some(target), _) => target,
            (None, Some(exit_id)) => {
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
            (None, None) => return,
        };
        let Some(tables) = assets::load_stage_tables(&dest.stage) else {
            return;
        };
        let rooms = assets::room_set_for(&tables, dest.room);
        let Some((spawn, yaw)) = assets::load_stage_spawn(&dest.stage, dest.room, dest.start)
        else {
            return;
        };
        self.swap_stage(tables, rooms, spawn, yaw);
    }

    pub fn swap_stage(
        &mut self,
        tables: dzs::StageTables,
        rooms: Vec<u8>,
        spawn: glam::Vec3,
        yaw: f32,
    ) {
        let has_sea = tables.has_sea_actor();
        let frame = self.sim.world.resource::<sim::frame::CurrentFrame>().0;
        let stories = *self.sim.world.resource::<sim::flags::StoryBits>();
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
        self.sim.world.resource_mut::<sim::flags::ZoneActors>().clear();
        *self.sim.world.resource_mut::<sim::talk::Dialogue>() = sim::talk::Dialogue::default();
        sim::spawn_actors(&mut self.sim.world, &tables);
        Self::install_tag_events(&mut self.sim.world, tables.actors());
        if !rooms.is_empty() {
            if let Some(set) = load_rooms(&tables, &rooms, stories) {
                self.install_room_set(set, frame);
            }
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
                let index = indices.get((actor.parameters >> 24) as usize).copied().flatten()?;
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

fn install_room_set(&mut self, set: assets::RoomSet, frame: u64) {
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
        self.room_tick = frame;
        self.install_room_actors(set.actors);
    }

    fn install_room_actors(&mut self, actors: Vec<dzs::StageActor>) {
        for entity in std::mem::take(&mut self.room_actors) {
            let _ = self.sim.world.despawn(entity);
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
        match load_rooms(tables, &rooms, stories) {
            Some(set) => {
                let frame = self.sim.world.resource::<sim::frame::CurrentFrame>().0;
                self.install_room_set(set, frame);
            }
            None => self.install_room_actors(Vec::new()),
        }
        self.rooms = rooms;
    }
}

fn load_rooms(
    tables: &dzs::StageTables,
    rooms: &[u8],
    stories: sim::flags::StoryBits,
) -> Option<assets::RoomSet> {
    assets::load_room_set(tables, rooms, |room| {
        sim::layer::no(tables.name(), room, &stories)
    })
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Platform entry: desktop window or, on the web, the `wasm_bindgen(start)`
/// hook; the shell loop stages input, steps the sim, pumps audio, composes.
#[cfg(not(target_os = "android"))]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    use std::time::Duration;
    use web_time::Instant;

    let mut app = App::new_live();
    let mut last = Instant::now();
    let root = move |sched: &mut repose_core::runtime::Scheduler,
                     ctx: &repose_core::RenderContext| {
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
    let frame = view::build_frame(&app.snapshot(), app.cam.get(), app.room.as_ref());
    let cam = app.cam.clone();
    let viewport = Viewport3d(
        frame,
        GeomHandle::new(),
        "scene.main",
        view::batch_desc(app.room.as_ref()),
        move |ev| match ev {
            View3dEvent::Orbit { dx, dy } => cam.update(|c| c.orbit(dx, dy)),
            View3dEvent::Zoom { factor } => cam.update(|c| c.zoom(factor)),
            _ => {}
        },
    );
    let overlay = match app.menu.screen {
        menus::Screen::Playing => {
            let held = app.held();
            hud::status_view(
                app.health(),
                app.rupees(),
                held.kind().is_some().then_some(held),
            )
        }
        menus::Screen::Title => ZStack(Modifier::new().fill_max_size())
            .child([title_movie(app, ctx), menus::title_view()]),
        menus::Screen::FileSelect => menus::file_select_view(&app.menu),
        menus::Screen::GameOver => menus::game_over_view(&app.menu),
    };
    ZStack(Modifier::new().fill_max_size()).child([viewport, overlay])
}

fn title_movie(app: &App, ctx: &repose_core::RenderContext) -> repose_core::View {
    if app.movie_pixels.is_empty() {
        return Spacer();
    }
    let handle = match app.movie_image.get() {
        Some(handle) => handle,
        None => {
            let handle = ctx.alloc_image_handle();
            app.movie_image.set(Some(handle));
            handle
        }
    };
    if app.movie_uploaded.get() != app.movie_frame {
        let (w, h) = app.movie_size;
        ctx.set_image_rgba8(handle, w, h, app.movie_pixels.clone(), true);
        app.movie_uploaded.set(app.movie_frame);
    }
    Image(Modifier::new().fill_max_size(), handle).image_fit(ImageFit::Cover)
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
