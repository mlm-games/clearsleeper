//! Wind Waker recreation on repame: fixed-step sim + repose views (no direct bevy).

use std::cell::Cell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{With, Without, World};
use repame_sim::Sim;
use repame_view3d::{GeomHandle, OrbitCamera, View3dEvent, Viewport3d};
use repose_core::input::PhysicalKey;
use repose_core::{ImageFit, Modifier, Signal, signal};
use repose_ui::{Image, ImageExt, Spacer, TextStyle, ViewExt, ZStack};

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
mod end_scene;
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
pub mod pad_source;
pub mod panes;
pub mod pause_chart;
pub mod pause_dmap;
pub mod pause_item;
pub mod pause_menu;
pub mod pause_seamap;
pub mod quit;
pub mod report;
pub mod save;
pub mod shot;
pub mod sim;
pub mod source;
pub mod text;
pub mod title_screen;
pub mod touch_controls;
pub mod ui_file_select;
pub mod ui_file_select_parts;
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

/// Body cut stem → blade-only bck stems. Retail swaps the blade's bck per cut direction while the
/// body plays its own bck (`tww/src/d/actor/d_a_player_sword.inc:569-571,671-673,773-775,874-876,973-975,1057-1059`;
/// HD `wwhd/wwhd_src/d/actor/d_a_player_main_01.cpp:2552-2572`).
const CUT_BLADES: [(&str, &str, &str); 7] = [
    ("cuta", "cutaa", "cutams"),
    ("cutf", "cutfa", "cutfms"),
    ("cutr", "cutra", "cutrms"),
    ("cutl", "cutla", "cutlms"),
    ("cutea", "cuteaa", "cuteams"),
    ("cuteb", "cuteba", "cutebms"),
    ("cutturnb", "cutturna", "cutturnms"),
];

/// BCK poses for the held parts. The blade bcks come from `LkAnm.arc` and are sampled at the
/// body cut's frame; the grip/shield bcks come from `Link.arc` (`tww/src/d/actor/d_a_player_main.cpp:11939-11950`,
/// HD `wwhd/wwhd_src/d/actor/d_a_player_main_01.cpp:3424-3439`).
#[derive(Default)]
pub struct PartAnims {
    pub blades: [Option<j3d::bck::Morph>; CUT_BLADES.len()],
    pub master_blades: [Option<j3d::bck::Morph>; CUT_BLADES.len()],
    pub grip: Option<j3d::bck::Morph>,
    pub shield: Option<j3d::bck::Morph>,
    pub leaf: Option<j3d::bck::Morph>,
    pub leaf_open: Option<j3d::bck::Morph>,
    pub hammer: Option<j3d::bck::Morph>,
    pub grip_brk: Option<j3d::brk::TevRegKey>,
    pub tip_bpk: Option<j3d::bpk::AnmColor>,
    pub master_tip_bpk: Option<j3d::bpk::AnmColor>,
    pub tip_btk: Option<j3d::btk::TextureAnimation>,
    pub tip_brk: Option<j3d::brk::TevRegKey>,
    pub master_tip_btk: Option<j3d::btk::TextureAnimation>,
    pub master_tip_brk: Option<j3d::brk::TevRegKey>,
    pub takt_brk: Option<j3d::brk::TevRegKey>,
    pub blade_btk: Option<j3d::btk::TextureAnimation>,
    pub glow_btk: Option<j3d::btk::TextureAnimation>,
    pub glow_brk: Option<j3d::brk::TevRegKey>,
    pub master_glow_btk: Option<j3d::btk::TextureAnimation>,
    pub master_glow_brk: Option<j3d::brk::TevRegKey>,
    pub bottle_btk: Option<j3d::btk::TextureAnimation>,
    pub liquid_brk: Option<j3d::brk::TevRegKey>,
    pub firefly_brk: Option<j3d::brk::TevRegKey>,
    pub grip_btk: Option<j3d::btk::TextureAnimation>,
    pub shield_btk: Option<j3d::btk::TextureAnimation>,
}

fn load_btk(anms: &assets::AnmArchive, id: u16) -> Option<j3d::btk::TextureAnimation> {
    j3d::btk::parse(&anms.bytes(assets::AnmRef::Id(id)).ok()?).ok()
}

fn load_brk(anms: &assets::AnmArchive, id: u16) -> Option<j3d::brk::TevRegKey> {
    j3d::brk::parse(&anms.bytes(assets::AnmRef::Id(id)).ok()?).ok()
}

fn first_uv(
    anim: Option<&j3d::btk::TextureAnimation>,
    frame: f32,
) -> Option<(u16, j3d::btk::UvMatrix)> {
    let anim = anim?;
    if anim.track_count() == 0 {
        return None;
    }
    Some((0, anim.uv_transform(0, frame)?))
}

fn first_color(anim: Option<&j3d::brk::TevRegKey>, frame: f32) -> Option<(u16, [f32; 3])> {
    let c = anim?.tev_color(0, frame)?;
    Some((
        0,
        [
            (1.0 + c[0] as f32 / 1024.0).max(0.0),
            (1.0 + c[1] as f32 / 1024.0).max(0.0),
            (1.0 + c[2] as f32 / 1024.0).max(0.0),
        ],
    ))
}

pub struct App {
    pub sim: Sim,
    pub cam: Signal<OrbitCamera>,
    pub audio: repame_audio::Audio,
    pub room: Option<j3d::bdl::RoomMesh>,
    pub actor_model: Option<j3d::bdl::RoomMesh>,
    /// `katsura.bdl`; retail attaches it to `CL_JNT_HEAD_JNT` rather than drawing the hair shape
    /// in `cl.bdl` (`d_a_player_main.cpp:11598`, `:11934`).
    pub hair_model: Option<j3d::bdl::RoomMesh>,
    /// The `hands.bdl` parts model, attached to `cl.bdl`'s hand joints.
    pub hands_model: Option<j3d::bdl::RoomMesh>,
    /// Held weapon/shield parts from `Link.arc`, attached to the hand joints.
    pub sword_model: Option<j3d::bdl::RoomMesh>,
    pub master_sword_model: Option<j3d::bdl::RoomMesh>,
    pub sword_grip_model: Option<j3d::bdl::RoomMesh>,
    pub master_grip_model: Option<j3d::bdl::RoomMesh>,
    pub shield_model: Option<j3d::bdl::RoomMesh>,
    pub mirror_shield_model: Option<j3d::bdl::RoomMesh>,
    /// Other held item bodies from `Link.arc`.
    pub leaf_model: Option<j3d::bdl::RoomMesh>,
    pub leaf_open_model: Option<j3d::bdl::RoomMesh>,
    pub hammer_model: Option<j3d::bdl::RoomMesh>,
    pub telescope_model: Option<j3d::bdl::RoomMesh>,
    pub takt_model: Option<j3d::bdl::RoomMesh>,
    pub bottle_model: Option<j3d::bdl::RoomMesh>,
    /// The blade's glow aura models, keyed by the sword table in `d_a_player_sword.inc:36-56`.
    pub sword_glow_model: Option<j3d::bdl::RoomMesh>,
    pub master_glow_model: Option<j3d::bdl::RoomMesh>,
    /// The forward-cut tip-stab models (`CUTFH`/`CUTFM`), attached at the blade's chest joint.
    pub tip_stab_model: Option<j3d::bdl::RoomMesh>,
    pub master_tip_stab_model: Option<j3d::bdl::RoomMesh>,
    /// Grandma's body and its BCKs from `Ba.arc`, drawn in place of her box.
    pub ba1_model: Option<assets::ActorModel>,
    pub chuchu_model: Option<assets::ActorModel>,
    pub armos_model: Option<assets::ActorModel>,
    pub floormaster_model: Option<assets::ActorModel>,
    pub amos_knight_model: Option<assets::ActorModel>,
    pub helmaroc_model: Option<assets::ActorModel>,
    pub phantom_ganon_model: Option<assets::ActorModel>,
    pub actor_models: std::collections::HashMap<&'static str, assets::ActorModel>,
    /// The 16x4 `blur.bti` swirl used for the sword trail (Link.arc id 0x70).
    pub blur_texture: Option<(Vec<u8>, u32, u32)>,
    pub weather_textures: Vec<Option<(Vec<u8>, u32, u32)>>,
    /// `b_sea_tex0and2.bti` from `Always.arc`, decoded to RGBA for the water surface
    /// (`d_a_sea.cpp:750-810` loads it as the sea's indirect/warp texture).
    pub sea_texture: Option<(Vec<u8>, u32, u32)>,
    /// Bottle contents models, shown inside the held bottle (`d_a_player_bottle.inc:32-78`).
    pub bottle_liquid_model: Option<j3d::bdl::RoomMesh>,
    pub bottle_fairy_model: Option<j3d::bdl::RoomMesh>,
    pub camera_model: Option<j3d::bdl::RoomMesh>,
    /// The player model's standing pose from `LkAnm.arc`; absent on a bind-pose-only source.
    pub player_anim: Option<assets::PlayerIdle>,
    pub part_anims: PartAnims,
    /// A `.dff` recording shown instead of the world, from `CS_DFF`. Its pages
    /// are its own, so it is presented alone rather than mixed into a room.
    dff: Option<j3d::present::Presented>,
    pub menu: menus::Menu,
    stage: Option<dzs::StageTables>,
    stage_name: String,
    stage_progress: std::collections::BTreeMap<u8, save::StageProgress>,
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
    /// Frames to ignore pause-close after opening; Escape is remapped to B in menus, so a
    /// slightly long hold would otherwise close the menu on the very next frame.
    pause_close_guard: u8,
    /// File-select action waiting for the retail slide-out animation to finish.
    pending_file_action: Option<menus::Action>,
    pad_prev: input::ButtonBits,
    sampled_pad: input::PadSnapshot,
    menu_clock: Duration,
    ending: Option<end_scene::Scene>,
    scene_image: end_scene::SceneImage,
    /// The hardware pad, polled every frame.
    pad_source: pad_source::PadSource,
    /// The on-screen controls a finger drives, and the overlay they are drawn on.
    touch: touch_controls::TouchControls,
    /// The input log the session writes, when `CS_INPUT_LOG` names a file. Each line is a
    /// tape line, so the session replays through `CS_INPUT`.
    input_log: Option<std::fs::File>,
    /// The keys the last poll held, as the log's bit mask.
    input_keys: u64,
    /// The frames the session has drawn, the retrace the input log numbers.
    frame: u64,
    /// The last input the web build logged, so the console gets a line per change.
    #[cfg(target_arch = "wasm32")]
    input_state: Option<(u64, u16, [i8; 4])>,
    intro_clock: f32,
    pub intro: Option<IntroStage>,
    /// True once an asset source (disc image or the development tree) is installed. The shell's
    /// own screens are pre-boot only: from here on every frame is the game's own rendering.
    assets_installed: bool,
    title_stage: Option<TitleStage>,
    meter: Option<MeterStage>,
    game_over: Option<GameOverStage>,
    album: Option<AlbumStage>,
    item_menu: Option<pause_item::ItemMenuStage>,
    chart_menu: Option<pause_chart::ChartStage>,
    dmap: Option<pause_dmap::DMapStage>,
    seamap: Option<pause_seamap::SeaMapStage>,
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

struct AlbumStage {
    return_screen: menus::Screen,
    album: crate::ui_menu_collect::AlbumBox,
    host: crate::ui_menu_collect::AlbumHost,
    tune: crate::ui_menu_collect::AlbumTune,
    input: crate::ui_menu_collect::AlbumInput,
    shutter_pending: bool,
    draw: Option<crate::ui_menu_collect::AlbumDraw>,
    canvas: text::raster::Canvas,
    image: Cell<Option<repose_core::ImageHandle>>,
    uploaded: Cell<u64>,
    revision: u64,
}

impl App {
    fn open_album(&mut self) {
        if self.album.is_some() {
            return;
        }
        let Ok(root) = assets::AssetRoot::from_env() else {
            return;
        };
        let Ok(cam) = root.read_archive("files/res/Msg/camres.arc") else {
            return;
        };
        let Ok(msgres) = root.read_archive("files/res/Msg/msgres.arc") else {
            return;
        };
        let (Some(main_bytes), Some(shutter_bytes)) = (
            cam.read("wipe_01_01.blo").ok(),
            cam.read("wipe_01_02.blo").ok(),
        ) else {
            return;
        };
        let (Some(main), Some(shutter)) = (j2d::parse(main_bytes), j2d::parse(shutter_bytes))
        else {
            return;
        };
        let message = msgres
            .read("hukidashi_08.blo")
            .ok()
            .and_then(j2d::parse)
            .unwrap_or_default();
        let mut album = crate::ui_menu_collect::AlbumBox::new(&main, &shutter, &message);
        let mut out = crate::ui_menu_collect::AlbumOutput::default();
        let input = self.album_input();
        album.create(&input, &mut out);
        let mut host = crate::ui_menu_collect::AlbumHost::default();
        let mut picture_num = input.picture_num;
        host.create(&mut picture_num, &mut out);
        self.album = Some(AlbumStage {
            return_screen: self.menu.screen,
            album,
            host,
            tune: crate::ui_menu_collect::AlbumTune::default(),
            input,
            shutter_pending: false,
            draw: None,
            canvas: text::raster::Canvas::new(640, 480),
            image: Cell::new(None),
            uploaded: Cell::new(0),
            revision: 1,
        });
        self.menu.screen = menus::Screen::PhotoAlbum;
    }

    fn album_input(&self) -> crate::ui_menu_collect::AlbumInput {
        use crate::ui_menu_collect::{AlbumInput, PhotoRecord};
        let mut input = AlbumInput {
            camera_attention: true,
            ..AlbumInput::default()
        };
        input = self.refresh_album_input(&input);
        let photo = *self.sim.world.resource::<sim::snap::PhotoCapture>();
        // The shell keeps one judged photo in memory; the album's slot count follows it until a
        // save is loaded (`d_picture_box.cpp:616-617`, the result/detail pair stored per picture).
        if photo.result != 0 || photo.detail != 0 {
            let mut record = PhotoRecord::from_texture(&[]);
            record.snap_result = photo.result;
            record.snap_result_detail = photo.detail;
            input.saved[0] = record;
            input.picture_num = 1;
        }
        input
    }

    fn refresh_album_input(
        &self,
        base: &crate::ui_menu_collect::AlbumInput,
    ) -> crate::ui_menu_collect::AlbumInput {
        use crate::ui_menu_collect::AlbumInput;
        let world = &self.sim.world;
        let mut input = base.clone();
        input.collect = *world.resource::<sim::flags::Collect>();
        input.story = *world.resource::<sim::flags::StoryBits>();
        input.camera_slot_item = world
            .resource::<sim::inventory::Equipped>()
            .item(sim::inventory::SLOT_PICTO_BOX);
        input.capture_from(world.resource::<sim::snap::SnapPacket>());
        input
    }

    fn step_album(&mut self, _dt: Duration) {
        use crate::ui_menu_collect::{AlbumOutput, CAPTURE_READY, PadInput};
        let refreshed = self
            .album
            .as_ref()
            .map(|stage| self.refresh_album_input(&stage.input));
        let Some(stage) = self.album.as_mut() else {
            return;
        };
        stage.input = refreshed.unwrap_or_default();
        // Discrete actions key on press edges; d-pad and the stick stay levels.
        let trig = self
            .sim
            .world
            .get_resource::<sim::intent::PadButtons>()
            .map(|pad| pad.trig)
            .unwrap_or_default();
        let pad = &self.sampled_pad;
        let menu_pad = PadInput {
            a: trig.a,
            b: trig.b,
            x: trig.x,
            y: trig.y,
            z: trig.z,
            l: trig.l,
            r: trig.r,
            start: trig.start,
            stick_value: pad.move_axis.length(),
            stick_angle: sim::walk::to_s16(pad.move_axis.x.atan2(-pad.move_axis.y)),
            ..PadInput::default()
        };
        let mut out = AlbumOutput::default();
        stage.host.execute(
            &mut stage.album,
            &stage.input,
            &menu_pad,
            &mut out,
            &stage.tune,
        );
        if let Some(step) = out.capture_step {
            stage.input.capture_step = step;
        }
        if let Some(num) = out.picture_num {
            stage.input.picture_num = num;
        }
        if let Some(status) = out.picture_status {
            stage.input.picture_status = status;
        }
        for (slot, record) in out.saved.iter().enumerate() {
            if let (0, Some(record)) = (slot, record) {
                let mut photo = self.sim.world.resource_mut::<sim::snap::PhotoCapture>();
                photo.result = record.snap_result;
                photo.detail = record.snap_result_detail;
            }
        }
        if out.release_shutter {
            self.sim
                .world
                .resource_mut::<sim::snap::SnapPacket>()
                .release_shutter();
            stage.shutter_pending = true;
        }
        if stage.shutter_pending {
            stage.shutter_pending = false;
            let camera = self
                .sim
                .world
                .get_resource::<sim::camera::Camera>()
                .copied();
            self.sim
                .world
                .resource_mut::<sim::snap::SnapPacket>()
                .judge_from_registry(camera.as_ref());
            stage
                .input
                .capture_from(self.sim.world.resource::<sim::snap::SnapPacket>());
            stage.input.capture_step = CAPTURE_READY;
        }
        if stage.album.get_status() == crate::ui_menu_collect::STATE_CLOSED || out.photo_box_cancel
        {
            // Return to the screen the album was opened from: the collection tab resumes
            // the menu (tww d_menu_collect.cpp:3897-3913), gameplay resumes the world.
            self.menu.screen = stage.return_screen;
            self.album = None;
            return;
        }
        stage.draw = Some(stage.album.draw(&stage.tune));
        stage.revision = stage.revision.wrapping_add(1);
    }

    fn album_view(&self, ctx: &repose_core::RenderContext) -> repose_core::View {
        let Some(stage) = self.album.as_ref() else {
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
            canvas.fill_rect([0.0, 0.0, 640.0, 480.0], [8, 12, 20, 255]);
            let Some(draw) = stage.draw.as_ref() else {
                return Spacer();
            };
            if let Some(assets) = panes::load() {
                for text in draw.panes.texts.iter().filter(|text| text.visible) {
                    let size = text.font_size.1.max(12.0);
                    panes::draw_label(
                        &mut canvas,
                        assets,
                        &text.text,
                        text.x + text.shift_x,
                        text.y + text.shift_y,
                        size,
                        text.alpha as u32 * 0x0101_0101,
                    );
                }
                for icon in &draw.icons {
                    canvas.fill_rect(
                        [icon.x, icon.y, icon.x + icon.w, icon.y + icon.h],
                        [180, 180, 180, icon.alpha],
                    );
                }
            }
            for slot in &draw.photos {
                canvas.fill_rect(
                    [
                        slot.left as f32,
                        slot.top as f32,
                        slot.right as f32,
                        slot.bottom as f32,
                    ],
                    [40, 60, 90, 255],
                );
            }
            ctx.set_image_rgba8(handle, 640, 480, canvas.pixels, true);
            stage.uploaded.set(stage.revision);
        }
        Image(Modifier::new().fill_max_size(), handle).image_fit(ImageFit::Contain)
    }
}

fn letter_key(key: PhysicalKey) -> Option<char> {
    use PhysicalKey::*;
    Some(match key {
        KeyA => 'A',
        KeyB => 'B',
        KeyC => 'C',
        KeyD => 'D',
        KeyE => 'E',
        KeyF => 'F',
        KeyG => 'G',
        KeyH => 'H',
        KeyI => 'I',
        KeyJ => 'J',
        KeyK => 'K',
        KeyL => 'L',
        KeyM => 'M',
        KeyN => 'N',
        KeyO => 'O',
        KeyP => 'P',
        KeyQ => 'Q',
        KeyR => 'R',
        KeyS => 'S',
        KeyT => 'T',
        KeyU => 'U',
        KeyV => 'V',
        KeyW => 'W',
        KeyX => 'X',
        KeyY => 'Y',
        KeyZ => 'Z',
        _ => return None,
    })
}

fn key_pressed(held: &HashSet<PhysicalKey>, prev: &HashSet<PhysicalKey>, key: PhysicalKey) -> bool {
    held.contains(&key) && !prev.contains(&key)
}

/// The input log `CS_INPUT_LOG` names, emptied when the session opens it. A line is a tape
/// line, so the session replays through `CS_INPUT` exactly as it was played.
fn open_input_log() -> Option<std::fs::File> {
    let path = std::env::var_os("CS_INPUT_LOG").filter(|value| !value.is_empty())?;
    let file = std::fs::File::create(path).ok()?;
    Some(file)
}

/// The held keys with the on-screen controls' keys added, or nothing when the controls
/// are not standing in for a key.
fn virtual_keys(held: &HashSet<PhysicalKey>, keys: &[PhysicalKey]) -> Option<HashSet<PhysicalKey>> {
    if keys.is_empty() {
        return None;
    }
    let mut held = held.clone();
    for key in keys {
        held.insert(*key);
    }
    Some(held)
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

    /// The stage the pause menu's map tab names.
    pub fn stage_name(&self) -> &str {
        &self.stage_name
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
            hands_model: None,
            sword_model: None,
            master_sword_model: None,
            sword_grip_model: None,
            master_grip_model: None,
            shield_model: None,
            mirror_shield_model: None,
            leaf_model: None,
            leaf_open_model: None,
            hammer_model: None,
            telescope_model: None,
            takt_model: None,
            bottle_model: None,
            sword_glow_model: None,
            master_glow_model: None,
            tip_stab_model: None,
            master_tip_stab_model: None,
            ba1_model: None,
            chuchu_model: None,
            armos_model: None,
            floormaster_model: None,
            amos_knight_model: None,
            helmaroc_model: None,
            phantom_ganon_model: None,
            actor_models: std::collections::HashMap::new(),
            blur_texture: None,
            weather_textures: Vec::new(),
            sea_texture: assets::load_sea_texture(),
            bottle_liquid_model: None,
            bottle_fairy_model: None,
            camera_model: None,
            player_anim: None,
            part_anims: PartAnims::default(),
            dff: None,
            menu: menus::Menu::new(),
            stage: None,
            stage_name: String::new(),
            stage_progress: Default::default(),
            rooms: Vec::new(),
            room_exits: Vec::new(),
            start_layer: sim::transition::LAYER_NONE,
            room_tick: 0,
            room_actors: Vec::new(),
            sea_shade,
            save_dir: save::slots_dir(),
            name: save::DEFAULT_NAME.to_string(),
            menu_prev: HashSet::new(),
            pause_close_guard: 0,
            pending_file_action: None,
            pad_prev: input::ButtonBits::default(),
            sampled_pad: input::PadSnapshot::default(),
            menu_clock: Duration::ZERO,
            ending: None,
            scene_image: Default::default(),
            pad_source: pad_source::PadSource::new(),
            touch: touch_controls::TouchControls::new(),
            input_log: open_input_log(),
            input_keys: 0,
            frame: 0,
            #[cfg(target_arch = "wasm32")]
            input_state: None,
            intro_clock: 0.0,
            intro: None,
            assets_installed: assets::AssetRoot::from_env().is_ok(),
            title_stage: None,
            meter: None,
            game_over: None,
            album: None,
            item_menu: Some(pause_item::ItemMenuStage::default()),
            chart_menu: Some(pause_chart::ChartStage::default()),
            dmap: Some(pause_dmap::DMapStage::default()),
            seamap: Some(pause_seamap::SeaMapStage::default()),
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
        // `katsura.bdl` is a casual-clothes wig: both trees draw it only under the
        // CASUAL_CLOTHES flag (`tww/src/d/actor/d_a_player_main.cpp:1901`,
        // `wwhd/wwhd_src/d/actor/d_a_player_main_01.cpp:3729`). The port has no outfit
        // state, and `cl.bdl` shape 4 now supplies the real hair, so the wig is left
        // unloaded rather than drawn over it.
        app.hands_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_HANDS_ENTRY);
        if app.hands_model.is_none() {
            log::warn!("clearsleeper: hands model unavailable");
        }
        // Held weapon/shield parts. Every entry is present in Link.arc; the blade/grip/shield
        // choice is made per frame in root_view from the snapshot's equipment.
        app.sword_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_SWORD_ENTRY);
        app.master_sword_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_MASTER_SWORD_ENTRY);
        app.sword_grip_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_SWORD_GRIP_ENTRY);
        app.master_grip_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_MASTER_GRIP_ENTRY);
        app.shield_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_SHIELD_ENTRY);
        app.mirror_shield_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_MIRROR_SHIELD_ENTRY);
        app.leaf_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_DEKU_LEAF_ENTRY);
        app.leaf_open_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_DEKU_LEAF_OPEN_ENTRY);
        app.hammer_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_HAMMER_ENTRY);
        app.telescope_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_TELESCOPE_ENTRY);
        app.takt_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_WIND_WAKER_ENTRY);
        app.bottle_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_BOTTLE_ENTRY);
        app.bottle_liquid_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, "binliquid.bdl");
        app.bottle_fairy_model = assets::load_actor_model_entry(view::LINK_ARCHIVE, "binfairy.bdl");
        app.camera_model =
            assets::load_actor_model_entry(view::LINK_ARCHIVE, assets::LINK_CAMERA_ENTRY);
        // Blade glow auras are BDL entries 0x4A/0x4C of Link.arc, loaded by id the way retail
        // reads them (`d_a_player_sword.inc:36-56`; `tww/assets/GZLE01/res/Object/Link.h:71,73`).
        if let Some(archive) = assets::load_object_archive(view::LINK_ARCHIVE) {
            app.sword_glow_model = assets::object_model(&archive, 0x4A);
            app.master_glow_model = assets::object_model(&archive, 0x4C);
            app.tip_stab_model = assets::object_model(&archive, 0x3E);
            app.master_tip_stab_model = assets::object_model(&archive, 0x3F);
        }
        app.ba1_model = assets::load_ba_model();
        app.chuchu_model = assets::load_chuchu_model();
        app.armos_model = assets::load_armos_model();
        app.floormaster_model = assets::load_floormaster_model();
        app.amos_knight_model = assets::load_amos_knight_model();
        app.helmaroc_model = assets::load_helmaroc_model();
        app.phantom_ganon_model = assets::load_phantom_ganon_model();
        app.actor_models = assets::load_actor_models();
        app.player_anim = app.actor_model.as_ref().and_then(assets::load_link_idle);
        if app.player_anim.is_none() {
            log::warn!("clearsleeper: player idle animation unavailable");
        }
        // Part-local BCK poses. Blade bcks (LkAnm.arc) are keyed to the blade model's own joint
        // tree (tww/src/d/actor/d_a_player_sword.inc:60-67); the grip and shield bcks are entries
        // in Link.arc (tww/src/d/actor/d_a_player_main.cpp:11939-11950).
        if let Ok(anms) = assets::AnmArchive::open(assets::LINK_ANM_ARCHIVE) {
            if let Some(model) = app.sword_model.as_ref() {
                for (slot, (_, stem, _)) in CUT_BLADES.iter().enumerate() {
                    let entry = format!("bcks/{stem}.bck");
                    app.part_anims.blades[slot] =
                        anms.morph(assets::AnmRef::Entry(&entry), &model.joint_parents);
                }
            }
            if let Some(model) = app.master_sword_model.as_ref() {
                for (slot, (_, _, stem)) in CUT_BLADES.iter().enumerate() {
                    let entry = format!("bcks/{stem}.bck");
                    app.part_anims.master_blades[slot] =
                        anms.morph(assets::AnmRef::Entry(&entry), &model.joint_parents);
                }
            }
            // The leaf and hammer drive their own models with a single bck each
            // (`d_a_player_fan.inc:121,134`, `d_a_player_hammer.inc:21`; HD
            // `wwhd/wwhd_src/d/actor/d_a_player_main_02.cpp:1358`,
            // `wwhd/wwhd_src/d/actor/d_a_player_main_01.cpp:1112`).
            if let Some(model) = app.leaf_model.as_ref() {
                app.part_anims.leaf = anms.morph(
                    assets::AnmRef::Entry("bcks/usefanaa.bck"),
                    &model.joint_parents,
                );
            }
            if let Some(model) = app.leaf_open_model.as_ref() {
                app.part_anims.leaf_open = anms.morph(
                    assets::AnmRef::Entry("bcks/fanba.bck"),
                    &model.joint_parents,
                );
            }
            if let Some(model) = app.hammer_model.as_ref() {
                app.part_anims.hammer = anms.morph(
                    assets::AnmRef::Entry("bcks/hammerdam.bck"),
                    &model.joint_parents,
                );
            }
        }
        if let Ok(anms) = assets::AnmArchive::open("files/res/Object/Link.arc") {
            if let Ok(raw) = anms.bytes(assets::AnmRef::Id(0x70)) {
                if let Some(image) = bti::decode(&raw) {
                    app.blur_texture =
                        Some((image.pixels, image.width as u32, image.height as u32));
                }
            }
            if let Some(model) = app.master_grip_model.as_ref() {
                app.part_anims.grip = anms.morph(assets::AnmRef::Id(0x0F), &model.joint_parents);
            }
            // The master grip's shine is a two-frame TEV C-register change keyed to the final
            // Master Sword (`tww/src/d/actor/d_a_player_main.cpp:1222-1254`,
            // `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3141-3169`).
            if let Ok(raw) = anms.bytes(assets::AnmRef::Id(0x56)) {
                app.part_anims.grip_brk = j3d::brk::parse(&raw).ok();
            }
            // The grip's wing scroll and the mirror shield's face scroll
            // (`tww/src/d/actor/d_a_player_main.cpp:1261-1268,1283-1294`,
            // `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3167-3169,3191-3201`).
            if let Ok(raw) = anms.bytes(assets::AnmRef::Id(0x67)) {
                app.part_anims.grip_btk = j3d::btk::parse(&raw).ok();
            }
            if let Ok(raw) = anms.bytes(assets::AnmRef::Id(0x66)) {
                app.part_anims.shield_btk = j3d::btk::parse(&raw).ok();
            }
            // The forward-cut tip-stab colour animation, one per sword tier
            // (`tww/src/d/actor/d_a_player_sword.inc:36-56,99-119`;
            // `tww/assets/GZLE01/res/Object/LkAnm.h:334-335`).
            if let Ok(raw) = anms.bytes(assets::AnmRef::Id(0x14C)) {
                app.part_anims.tip_bpk = j3d::bpk::parse(&raw).ok();
            }
            if let Ok(raw) = anms.bytes(assets::AnmRef::Id(0x14D)) {
                app.part_anims.master_tip_bpk = j3d::bpk::parse(&raw).ok();
            }
            // Material animations for the tip-stab, Wind Waker, master blade and glow models
            // (`tww/assets/GZLE01/res/Object/LkAnm.h:337-346,451,473-474`).
            app.part_anims.tip_btk = load_btk(&anms, 0x159);
            app.part_anims.tip_brk = load_brk(&anms, 0x150);
            app.part_anims.master_tip_btk = load_btk(&anms, 0x15A);
            app.part_anims.master_tip_brk = load_brk(&anms, 0x151);
            app.part_anims.takt_brk = load_brk(&anms, 0x154);
            app.part_anims.blade_btk = load_btk(&anms, 0x1C3);
            app.part_anims.glow_btk = load_btk(&anms, 0x1D9);
            app.part_anims.glow_brk = load_brk(&anms, 0x155);
            app.part_anims.master_glow_btk = load_btk(&anms, 0x1DA);
            app.part_anims.master_glow_brk = load_brk(&anms, 0x156);
            app.part_anims.bottle_btk = load_btk(&anms, 0x16B);
            app.part_anims.liquid_brk = load_brk(&anms, 0x153);
            app.part_anims.firefly_brk = load_brk(&anms, 0x152);
        }
        if let (Ok(anms), Some(model)) = (
            assets::AnmArchive::open("files/res/Object/Link.arc"),
            app.shield_model.as_ref(),
        ) {
            app.part_anims.shield = anms.morph(assets::AnmRef::Id(0x0A), &model.joint_parents);
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
        self.stage_name = stage.to_owned();
        // Vrkumo cloud pages live in the stage archive (`tww/src/d/d_kankyo_wether.cpp:818-834`;
        // `wwhd/wwhd_src/d/d_kankyo_wether.cpp:2830-2846`), so reload them once the stage is known.
        self.weather_textures = assets::load_weather_textures(&self.stage_name);
        self.title_actor = None;
        self.stage_progress.clear();
        self.sim = Sim::new(Duration::from_secs_f64(1.0 / SIM_HZ));
        self.stage = assets::load_stage_tables(stage);
        // The stage's event-info table names the demo a PLYR row starts, and the new-game boot
        // path must carry it too. tww/src/d/d_stage.cpp:2246,
        // tww/src/d/d_event_manager.cpp:347-348; wwhd/wwhd_src/d/d_stage_3.cpp:172-175,
        // wwhd/wwhd_src/d/d_stage_4.cpp:226.
        self.sim.world.insert_resource(sim::event::StageEvents(
            self.stage
                .as_ref()
                .map(|tables| tables.event_names().to_vec())
                .unwrap_or_default(),
        ));
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
    }

    /// Frame boundary: poll the shell into the sim's intent resources.
    pub fn stage_polled(&mut self, sched: &repose_core::runtime::Scheduler) {
        let frame = touch_controls::read(sched.size, &sched.touch_points);
        self.touch.update(&frame, sched.size);
        let pad = self.pad_source.poll();
        match virtual_keys(&sched.held_keys, &frame.keys) {
            Some(held) => self.stage_keys_with(&held, sched.window_focused, pad, frame.pad),
            None => self.stage_keys_with(&sched.held_keys, sched.window_focused, pad, frame.pad),
        }
        self.stage_stylus(sched, frame.stylus);
    }

    /// The shell's keys as the game reads them: the move axis, the debug and action keys, the
    /// pad's face buttons and the menus' commands. The input log replays a session through
    /// this path, so a logged frame and a live frame stage the same thing.
    pub fn stage_keys(&mut self, held: &HashSet<PhysicalKey>, window_focused: bool) {
        self.stage_keys_with(
            held,
            window_focused,
            input::PadSnapshot::default(),
            input::PadSnapshot::default(),
        );
    }

    /// The same as [`App::stage_keys`], with the hardware pad and the on-screen controls
    /// folded in: a button held on any source is held, and a stick that moved on a pad
    /// wins over the keys' rest.
    pub fn stage_keys_with(
        &mut self,
        held: &HashSet<PhysicalKey>,
        window_focused: bool,
        pad: input::PadSnapshot,
        touch: input::PadSnapshot,
    ) {
        self.input_keys = if window_focused {
            input::key_mask(held)
        } else {
            0
        };
        let mut keyboard = input::pad_snapshot_from_keys(held, window_focused);
        let menu = self.menu.screen != menus::Screen::Playing || self.intro.is_some();
        if menu && window_focused {
            keyboard.hold.a |=
                held.contains(&PhysicalKey::Space) || held.contains(&PhysicalKey::Enter);
            if self.menu.screen != menus::Screen::Title && held.contains(&PhysicalKey::Escape) {
                keyboard.hold.b = true;
                keyboard.hold.start = false;
            }
        }
        // tww/src/m_Do/m_Do_controller_pad.cpp:20-54; tww/src/d/actor/d_a_player_main.cpp:10568-10632.
        let merged = if window_focused {
            input::merge_pad(keyboard, input::merge_pad(touch, pad))
        } else {
            self.sim
                .world
                .resource_mut::<sim::intent::PadButtons>()
                .trig = Default::default();
            input::PadSnapshot::default()
        };
        self.sampled_pad = merged;
        self.stage_axis(merged.move_axis);
        self.stage_damage(input::damage_from_keys(held, window_focused && !menu));
        self.stage_do(!menu && (merged.hold.a || input::do_from_keys(held, window_focused)));
        self.stage_target(
            !menu && (merged.hold.l || input::target_from_keys(held, window_focused)),
        );
        self.stage_item(!menu && (merged.hold.b || input::item_from_keys(held, window_focused)));
        // X/Y/Z each use the item assigned to that button; the pause menu writes the map.
        // `dComIfGp_getSelectItem` (tww/include/d/d_com_inf_game.h:2918-2939;
        // HD wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3180-3186).
        self.stage_use(
            !menu
                && (merged.hold.x
                    || merged.hold.y
                    || merged.hold.z
                    || input::use_from_keys(held, window_focused)),
        );
        self.stage_pad(merged.hold, merged.c_axis);
        let button = if merged.hold.x {
            0
        } else if merged.hold.y {
            1
        } else if merged.hold.z {
            2
        } else {
            self.sim.world.resource::<sim::intent::SelectedButton>().0
        };
        *self.sim.world.resource_mut::<sim::intent::SelectedButton>() =
            sim::intent::SelectedButton(button);
        self.stage_menu(held, window_focused);
    }

    /// The pointer or the finger that is not on a control, in the canvas's pixels.
    fn stage_stylus(&mut self, sched: &repose_core::runtime::Scheduler, touch: Option<glam::Vec2>) {
        let stylus = touch.map(|pos| (pos, true)).or_else(|| {
            input::stylus(
                [640.0, 480.0],
                sched.size,
                sched.pointer_pos_px,
                sched.mouse_primary,
            )
        });
        let mut intent = self
            .sim
            .world
            .get_resource_or_insert_with(sim::intent::Stylus::default);
        match stylus {
            Some((pos, down)) => {
                intent.pos = pos;
                intent.down = down;
            }
            None => intent.down = false,
        }
    }

    /// One tape line for the frame's input, written to the log `CS_INPUT_LOG` names. The
    /// line is the pad word, the stick and the key mask, which is what a replay needs to
    /// stage the same frame again.
    fn log_input(&mut self) {
        use std::io::Write;
        #[cfg(not(target_arch = "wasm32"))]
        if self.input_log.is_none() {
            return;
        }
        let tick = self.frame;
        let hold = self.sampled_pad;
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
        let axis = hold.move_axis;
        let stick_x = (axis.x * 127.0).round() as i8;
        let stick_y = (axis.y * 127.0).round() as i8;
        let c_x = (hold.c_axis.x * 127.0).round() as i8;
        let c_y = (hold.c_axis.y * 127.0).round() as i8;
        let line = format!(
            "retrace={tick} button={word:#06x} stick={stick_x},{stick_y} c={c_x},{c_y} l=0 r=0 keys={:#x}\n",
            self.input_keys
        );
        #[cfg(target_arch = "wasm32")]
        {
            // The console gets the line when the input changes, so a session's log stays
            // readable; the desktop's file gets every frame, so a replay is exact.
            let state = (self.input_keys, word, [stick_x, stick_y, c_x, c_y]);
            if Some(state) != self.input_state {
                self.input_state = Some(state);
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
        let sticks = |x: i8, y: i8| {
            (glam::Vec2::new(f32::from(x), f32::from(y)) / 127.0).clamp_length_max(1.0)
        };
        let pad = input::PadSnapshot {
            hold: state.hold(),
            move_axis: if state.keys_given {
                sticks(state.stick_x, state.stick_y)
            } else {
                state.move_axis()
            },
            c_axis: if state.keys_given {
                sticks(state.c_stick_x, state.c_stick_y)
            } else {
                state.c_axis()
            },
        };
        self.stage_keys_with(
            &input::keys_from_mask(state.keys),
            true,
            pad,
            Default::default(),
        );
    }

    /// Stage the pad's face buttons and C-stick. The trigger word is the edge against the
    /// previous poll, so it lasts one frame whatever the fixed step then does with it and a
    /// held button never reads as a fresh press.
    fn stage_pad(&mut self, held: input::ButtonBits, c_stick: glam::Vec2) {
        let prev = std::mem::replace(&mut self.pad_prev, held);
        let mut buttons = self.sim.world.resource_mut::<sim::intent::PadButtons>();
        buttons.trig = buttons.trig.union(input::pad_trig(held, prev));
        buttons.hold = held;
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
        let escape_pressed = window_focused
            && held.contains(&PhysicalKey::Escape)
            && !self.menu_prev.contains(&PhysicalKey::Escape);
        let album_down = held.contains(&PhysicalKey::KeyP);
        if window_focused
            && self.assets_installed
            && album_down
            && !self.menu_prev.contains(&PhysicalKey::KeyP)
        {
            match self.menu.screen {
                menus::Screen::Playing => self.open_album(),
                menus::Screen::PhotoAlbum => {
                    self.menu.screen = menus::Screen::Playing;
                    self.album = None;
                }
                _ => {}
            }
        }
        if self.menu.screen == menus::Screen::NameEntry && window_focused {
            for key in held
                .iter()
                .copied()
                .filter(|key| !self.menu_prev.contains(key))
            {
                if let Some(letter) = letter_key(key) {
                    if self.menu.name_entry.len() < 8 {
                        self.menu.name_entry.push(letter);
                    }
                } else if key == PhysicalKey::Backspace {
                    self.menu.name_entry.pop();
                } else if key == PhysicalKey::Enter || key == PhysicalKey::Space {
                    let name = self.menu.name_entry.clone();
                    let slot = self.menu.name_slot;
                    if !name.is_empty() {
                        self.name = name;
                    }
                    self.menu.screen = menus::Screen::Playing;
                    self.reset_world_at(assets::PROLOGUE_STAGE, assets::PROLOGUE_SPAWN_POINT);
                    self.open_new_slot(slot);
                    self.menu_prev.clone_from(held);
                    return;
                } else if key == PhysicalKey::Escape {
                    self.menu.screen = menus::Screen::Title;
                    self.menu_prev.clone_from(held);
                    return;
                }
            }
        }
        // `d_menu_window.cpp:1479-1539` opens the pause menu on Start; B/Start closes it and
        // L/R move between pages. Escape is both the shell's Start and its B, so this reads
        // the physical keys directly rather than the remapped pad bits.
        if self.assets_installed && window_focused {
            let pad_start = self
                .sim
                .world
                .resource::<sim::intent::PadButtons>()
                .trig
                .start;
            let open_tab = if key_pressed(held, &self.menu_prev, PhysicalKey::KeyI) {
                Some(0)
            } else if key_pressed(held, &self.menu_prev, PhysicalKey::KeyM) {
                Some(2)
            } else if key_pressed(held, &self.menu_prev, PhysicalKey::KeyN) {
                Some(1)
            } else if key_pressed(held, &self.menu_prev, PhysicalKey::KeyG) {
                Some(3)
            } else if key_pressed(held, &self.menu_prev, PhysicalKey::KeyE) {
                Some(4)
            } else if key_pressed(held, &self.menu_prev, PhysicalKey::KeyC) {
                Some(5)
            } else {
                None
            };
            match self.menu.screen {
                menus::Screen::Playing
                    if escape_pressed
                        || pad_start
                        || key_pressed(held, &self.menu_prev, PhysicalKey::Tab) =>
                {
                    self.menu.pause = menus::PauseState::default();
                    self.pause_close_guard = 2;
                    if let Some(stage) = self.item_menu.as_mut() {
                        stage.opened = false;
                    }
                    self.menu.screen = menus::Screen::Pause;
                }
                menus::Screen::Playing if open_tab.is_some() => {
                    self.menu.pause = menus::PauseState {
                        tab: open_tab.unwrap_or_default(),
                        ..menus::PauseState::default()
                    };
                    self.pause_close_guard = 2;
                    if let Some(stage) = self.item_menu.as_mut() {
                        stage.opened = false;
                    }
                    self.menu.screen = menus::Screen::Pause;
                }
                menus::Screen::Pause => {
                    let prev = self.menu_prev.clone();
                    self.stage_pause(held, &prev, window_focused);
                }
                _ => {}
            }
        }
        let cmds = if !self.assets_installed && window_focused {
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
            menus::Screen::Settings | menus::Screen::QuitConfirm => {
                // Only the pre-boot shell enters these; return to the retail title instead of
                // drawing the shell over a session that has the disc.
                self.menu.screen = menus::Screen::Title;
            }
            _ => {}
        }
    }

    /// One frame of pause-menu input. The arrow keys and WASD move the cursor, L/R change
    /// page, A selects an item, X/Y/Z assign it to that button, and Escape or B closes.
    /// `d_menu_window.cpp:1680-1710` uses the same button set for the item page.
    fn stage_pause(
        &mut self,
        held: &HashSet<PhysicalKey>,
        prev: &HashSet<PhysicalKey>,
        window_focused: bool,
    ) {
        let pressed =
            |key: PhysicalKey| window_focused && held.contains(&key) && !prev.contains(&key);
        if self.pause_close_guard > 0 {
            self.pause_close_guard -= 1;
            return;
        }
        let keys = self.sim.world.resource::<sim::intent::PadButtons>().trig;
        // Retail gates close on the item page having no note or subwindow open
        // (d_menu_window.cpp:1682, d_menu_item.cpp:2145-2147); let the ported state
        // machine consume B in that case instead of dropping the whole menu.
        let ported_busy = self.menu.pause.tab == 0
            && self
                .sim
                .world
                .get_resource::<crate::ui_menu_item::MenuWindow>()
                .is_some_and(|window| {
                    window.state != crate::ui_menu_item::STATE_NO_MENU
                        && window
                            .item
                            .as_ref()
                            .is_some_and(|item| item.note_check() || item.sub_win)
                });
        if !ported_busy && (pressed(PhysicalKey::Escape) || keys.b || keys.start) {
            if self.menu.pause.assigning {
                self.menu.pause.assigning = false;
            } else {
                self.menu.screen = menus::Screen::Playing;
            }
            return;
        }
        // Retail's L/R flip between the item and collection pages
        // (d_menu_window.cpp:1692-1743), not a linear cycle through every tab.
        if pressed(PhysicalKey::KeyO) || keys.l || pressed(PhysicalKey::KeyR) || keys.r {
            let pause = &mut self.menu.pause;
            pause.tab = if pause.tab == 0 { 5 } else { 0 };
            pause.cursor = 0;
            pause.assigning = false;
        }
        if pressed(PhysicalKey::ArrowUp) || pressed(PhysicalKey::KeyW) {
            self.menu.pause.cursor = self.menu.pause.cursor.saturating_sub(1);
        }
        if pressed(PhysicalKey::ArrowDown) || pressed(PhysicalKey::KeyS) {
            self.menu.pause.cursor = self.menu.pause.cursor.saturating_add(1).min(15);
        }
        let a = keys.a
            || pressed(PhysicalKey::KeyQ)
            || pressed(PhysicalKey::Space)
            || pressed(PhysicalKey::Enter);
        if a && self.menu.pause.tab == 5 {
            // The collection tab opens the photo album (`d_menu_window.cpp:813-890`).
            self.open_album();
            return;
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
    /// The retail file select is a pad-driven state machine (`d_file_select.cpp`), so the
    /// shell's keys are folded into the pad it reads: Space and Enter are A, Escape is B and
    /// the main stick is the cursor.
    pub(crate) fn menu_pad(&self) -> ui_menu_save::PadInput {
        let buttons = *self.sim.world.resource::<sim::intent::PadButtons>();
        // tww/src/d/d_file_select.cpp:498-505: the cursor reads the main stick.
        let axis = self.sampled_pad.move_axis;
        ui_menu_save::PadInput {
            trig_a: buttons.trig.a,
            trig_b: buttons.trig.b,
            trig_x: buttons.trig.x,
            trig_y: buttons.trig.y,
            trig_z: buttons.trig.z,
            trig_l: buttons.trig.l,
            trig_r: buttons.trig.r,
            trig_start: buttons.trig.start,
            stick_value: axis.length(),
            stick_angle: sim::walk::to_s16(axis.x.atan2(-axis.y)),
        }
    }

    pub(crate) fn file_select_pad(&self) -> ui_file_select::PadInput {
        let mut pad = ui_file_select::PadInput::from(&self.menu_pad());
        pad.hold_a = self.sampled_pad.hold.a;
        pad.hold_b = self.sampled_pad.hold.b;
        pad
    }

    /// One frame of the retail file select: the ported state machine (`d_file_select.cpp`)
    /// moves the panes and reads the pad, and the slot it picks starts the game or loads the
    /// save.
    fn stage_file_select(&mut self) {
        if let Some(action) = self.pending_file_action {
            // Retail slides the panes out before the chosen file starts
            // (tww d_file_select.cpp:294-357, HD d_file_select.cpp:1141-1150).
            if self.menu.fs.close() {
                self.pending_file_action = None;
                self.apply_menu_action(Some(action));
            }
            self.menu
                .revision
                .set(self.menu.revision.get().wrapping_add(1));
            return;
        }
        if !self.menu.tick_file_select() {
            return;
        }
        let pad = self.file_select_pad();
        let menus::Menu { fs, card, .. } = &mut self.menu;
        fs.move_step(&pad, card, None);
        if let Some(slot) = self.menu.fs.slot_picked.take() {
            let action = self.menu.file_select_pick(slot);
            self.pending_file_action = Some(action);
            return;
        }
        if self.menu.fs.is_select_end() == crate::ui_file_select::SELECT_END_RETURN {
            self.menu.screen = menus::Screen::Title;
        }
        self.menu
            .revision
            .set(self.menu.revision.get().wrapping_add(1));
    }

    fn apply_menu_action(&mut self, action: Option<menus::Action>) {
        match action {
            Some(menus::Action::Load(slot)) => {
                match self.menu.card.slots.get(slot).cloned().flatten() {
                    Some(saved) => match self.load(&saved) {
                        Ok(()) => {
                            // tww/src/d/d_s_name.cpp:1165-1175,1213-1219: a fresh extra file enters the opening.
                            if saved.progress.clear_count != 0
                                && !self
                                    .sim
                                    .world
                                    .resource::<sim::flags::StoryBits>()
                                    .test(0x3510)
                            {
                                self.open_new_slot(slot);
                            } else {
                                self.menu.screen = menus::Screen::Playing;
                                self.order_start_event();
                            }
                        }
                        Err(e) => log::warn!("clearsleeper: slot {slot} load failed ({e})"),
                    },
                    None => log::warn!("clearsleeper: selected slot {slot} could not be read"),
                }
            }
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
        // A new file goes through the naming step before the opening scene
        // (tww d_s_name.cpp:1165-1171, HD d_file_select.cpp:669-678).
        self.menu.name_entry = save::DEFAULT_NAME.to_owned();
        self.menu.name_slot = slot;
        self.menu.screen = menus::Screen::NameEntry;
    }

    fn open_new_slot(&mut self, slot: usize) {
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
        let new_game = self.pending_slot.is_some();
        self.menu.screen = menus::Screen::Playing;
        if let Some(slot) = self.pending_slot.take() {
            if let Err(e) = self.write_slot(slot) {
                log::warn!("clearsleeper: slot {slot} write failed ({e})");
            }
        }
        if new_game {
            self.order_start_event();
        }
    }

    /// `daPy_lk_c::create`'s start demo (`d_a_player_main.cpp:12328`,
    /// `dComIfGp_evmng_startDemo(getStartEvent())`) through `dEvent_exception_c::setStartDemo`
    /// (`d_event_manager.cpp:24-54`): the row the player spawned on names an `EVNT` index, and
    /// that row's spawn switch is what keeps a one-shot cutscene from replaying. `sea` room 44's
    /// point-206 `PLYR` row carries start event 29, `awake`, on switch 0x5C - the sister's
    /// opening cutscene - and the room's `LBNK` demo archive (`Demo02`) is what the event's
    /// `PACKAGE` staff mounts.
    fn order_start_event(&mut self) {
        let start = *self
            .sim
            .world
            .get_resource_or_insert_with(sim::event::StartEvent::default);
        if start.index == sim::event::NO_START_EVENT {
            return;
        }
        let stage_type = self
            .stage
            .as_ref()
            .and_then(|tables| tables.stag())
            .map_or(0, |stag| stag.stage_type);
        let Some(name) = self
            .sim
            .world
            .get_resource::<sim::event::StageEvents>()
            .and_then(|names| names.name_with_stage(start.index, stage_type))
            .map(str::to_owned)
        else {
            return;
        };
        // `dComIfGs_isSwitch(switchNo, room)` first (`d_event_manager.cpp:44-48`): a switch that
        // is already on means the row's demo has played, and the exception answers with no event.
        if start.switch != sim::event::NO_START_EVENT {
            let mut switches = self.sim.world.resource_mut::<sim::flags::Switches>();
            if switches.test(start.switch) {
                return;
            }
            switches.set(start.switch);
        }
        let list = self.sim.world.resource::<sim::event::EventList>().clone();
        let Some(index) = list.event_index(&name) else {
            return;
        };
        let player = {
            let mut players = self
                .sim
                .world
                .query_filtered::<Entity, With<sim::player::Player>>();
            players.iter(&self.sim.world).next()
        };
        let Some(player) = player else {
            return;
        };
        self.sim
            .world
            .resource_mut::<sim::event::ActiveEvent>()
            .order(&list, index, player);
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
        // tww/src/d/actor/d_a_title.cpp:651-675; tww/src/d/d_s_title.cpp:85-89.
        if request.scene_proc == sim::title::FPC_NM_TITLE_SCENE {
            self.begin_movie(ui_file_select::SceneName::Title);
            return;
        }
        if request.scene_proc != sim::title::FPC_NM_NAME_SCENE {
            return;
        }
        self.sim
            .world
            .resource_mut::<sim::title::TitleScene>()
            .request = None;
        self.menu.screen = menus::Screen::FileSelect;
        self.menu.open_file_select();
        self.menu.sync_file_select_cursor();
    }

    /// The actor and its scene request are rebuilt on every entry, as the title scene is
    /// recreated after `dComIfG_resetToOpening` (`d_s_title.cpp:48`, `d_s_play.cpp:305`).
    pub(crate) fn sync_title_actor(&mut self) {
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
        title_screen::render(&mut stage.canvas, &stage.assets, &title);
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

    /// Stage the orbit view yaw into the sim's camera-relative walk frame.
    pub fn stage_camera(&mut self) {
        // tww/src/d/actor/d_a_player_main.cpp:10568: movement follows the active camera.
        let cam_yaw = self.game_camera().unwrap_or_else(|| self.cam.get()).yaw;
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
        save_assets.bind(&mut game.save);
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
        let pad = self.menu_pad();
        let mut vibration = *self.sim.world.resource::<sim::vibration::Vibration>();
        let mut health = *self.sim.world.resource::<sim::health::Health>();
        // The save the game-over screen writes is the state at the death, not the save the
        // session loaded (`d_gameover.cpp:109-166` writes the current state).
        let pending_save = Some(self.save());
        let clears = self
            .sim
            .world
            .resource::<text::driver::TextFlags>()
            .clear_count;
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
                pending_save,
                vibration: &mut vibration,
                health: &mut health,
                life_on_open: life,
            };
            outcome = stage.game.step(&mut env);
            stage.save_assets.sync_text(&mut stage.game.save, clears);
            stage.revision = stage.revision.wrapping_add(1);
        } else if self.game_over_failed {
            // The retail BLO could not bind, so there is no state machine to produce an
            // outcome. Read the same pad directly rather than leaving the player on a
            // frozen world with no way out (`d_gameover.cpp:109-166`).
            if pad.trig_a {
                outcome = Some(game_over_screen::Outcome::Continue);
            } else if pad.trig_b {
                outcome = Some(game_over_screen::Outcome::Title);
            }
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
                | sim::events::SimEvent::Bgm(_) => None,
                // Consumed by `pump_damage_fx`, which spawns the dropped item before the
                // event is ever drained here.
                sim::events::SimEvent::StealItem(_) => None,
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
        for steal in steals {
            // `fopAcM_createStealItem` (`f_op_actor_mng.cpp:1142-1178`,
            // `wwhd/wwhd_src/f_op/f_op_actor_mng_4.cpp:25-44`): the struck actor drops a
            // re-grabbable item at its own point with retail's -6.0 gravity. The life-ball
            // table this resolves through is absent in the port, so a table index falls back
            // to a green rupee; a literal id is used as-is (same rule as `switem::created_item`).
            let item = if (0x20..=0x3E).contains(&steal.table) || steal.table == 0x3F {
                0x01
            } else {
                (steal.table & 0xFF) as u8
            };
            let source = crate::dzs::StageActor {
                name: "item".to_owned(),
                parameters: u32::from(item),
                pos: steal.pos.to_array(),
                angle: [0; 3],
                set_id: 0,
                room: steal.room,
                scale: [1.0; 3],
            };
            let entity = sim::spawn_actor(&mut self.sim.world, &source);
            if let Some(mut pickup) = self.sim.world.get_mut::<sim::pickup::Pickup>(entity) {
                *pickup = sim::pickup::Pickup::thrown(u32::from(item), 0.0, 0.0, -6.0);
            }
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
        let player_speed = player.map_or(0.0, |player| player.speed);
        let item_in_use = self
            .sim
            .world
            .get_resource::<sim::item_use::ItemUse>()
            .and_then(|state| state.stem)
            .is_some_and(|stem| !matches!(stem, "grabup" | "grabwait"))
            || self
                .sim
                .world
                .get_resource::<sim::hammer::Cut>()
                .is_some_and(|cut| cut.kind != sim::hammer::CUT_NONE)
            || self
                .sim
                .world
                .get_resource::<sim::scope::Scope>()
                .is_some_and(|scope| scope.active);
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
            player_anm: self
                .sim
                .world
                .get_resource::<sim::link_demo::PlayerAnm>()
                .and_then(|anm| {
                    anm.stem.map(|stem| view::PlayerAnm {
                        stem,
                        frame: anm.frame,
                        looping: false,
                    })
                })
                .or_else(|| {
                    self.sim
                        .world
                        .get_resource::<sim::player_pose::PlayerPose>()
                        .and_then(|pose| {
                            pose.stem.map(|stem| view::PlayerAnm {
                                stem,
                                frame: pose.frame,
                                looping: matches!(
                                    stem,
                                    "walk"
                                        | "dash"
                                        | "swimp"
                                        | "swimwait"
                                        | "crouch"
                                        | "lieforward"
                                        | "hanging"
                                        | "grabwait"
                                        | "walkpush"
                                        | "walkpull"
                                        | "hookshotjmp"
                                ),
                            })
                        })
                }),
            player_speed,
            item_in_use,
            player_no_draw: player.is_some_and(|player| player.no_draw)
                || self
                    .sim
                    .world
                    .get_resource::<sim::camera::Camera>()
                    .is_some_and(|camera| camera.attention(sim::camera::ATTN_SUBJECT)),
            equipment: {
                let collect = self.sim.world.resource::<sim::flags::Collect>();
                view::PlayerEquipment {
                    sword: sim::inventory::sword(collect),
                    shield: sim::inventory::shield(collect),
                }
            },
            // The shield sits in hand while wielding the sword or guarding, otherwise on the back
            // (`tww/src/d/actor/d_a_player_main.cpp:1301-1319`,
            // `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3206-3234`).
            shield_on_back: !self
                .sim
                .world
                .get_resource::<sim::combat::Combat>()
                .is_some_and(|combat| combat.active)
                && !self
                    .sim
                    .world
                    .get_resource::<sim::guard::Guard>()
                    .is_some_and(|guard| guard.raised)
                && !self
                    .sim
                    .world
                    .get_resource::<sim::link_proc::LinkProc>()
                    .is_some_and(|proc| proc.current == sim::link_proc::Proc::GuardCrash),
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
            billboards: self.billboard_views(),
            oriented_weather: {
                let mut quads = self.oriented_weather_views();
                quads.extend(self.hookshot_chain_views());
                quads
            },
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

    /// Rain drops and the sun disc, from the same `Weather` packet the decomp draws
    /// (`tww/src/d/d_kankyo_rain.cpp:3946-4117`,
    /// `wwhd/wwhd_src/d/d_kankyo_rain_drawRain.cpp:9-156`).
    /// Weather draw state, from the same packets the decomp's `dKyr_draw*` pass reads
    /// (`tww/src/d/d_kankyo_rain.cpp:3947`, HD `wwhd_src/d/d_kankyo_rain_drawRain.cpp:9`).
    fn billboard_views(&self) -> Vec<view::BillboardView> {
        let Some(weather) = self.sim.world.get_resource::<sim::weather::Weather>() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let board =
            |pos, half, color, alpha, roll, page, textured, depth_test| view::BillboardView {
                pos,
                half,
                color,
                alpha,
                roll,
                page,
                textured,
                depth_test,
            };
        // Rain: the four stacked quads per drop are collapsed into one tall streak.
        let count = weather.rain.rain_count.max(0) as usize;
        for (index, drop) in weather.rain.eff.iter().take(count).enumerate() {
            if drop.alpha <= 0.0 || drop.timer > 0 {
                continue;
            }
            let size = 2.5 + index as f32 / 250.0 * 3.0;
            out.push(board(
                drop.base_pos + drop.pos + glam::Vec3::Y * 160.0,
                glam::Vec2::new(size, 240.0 + size * 2.0),
                [1.0, 1.0, 1.0],
                (drop.alpha * 14.0).clamp(0.0, 1.0),
                0.0,
                assets::WeatherPage::Rain,
                true,
                true,
            ));
        }
        // Snow: half-size and droop from `draw_snow` (`d_kankyo_rain.cpp:4956-4999`).
        let snow_count = weather.snow.eff_count.max(0) as usize;
        for (index, flake) in weather.snow.eff.iter().take(snow_count).enumerate() {
            if flake.status == 0 || flake.scale <= 0.0 {
                continue;
            }
            let p = flake.scale + flake.scale - flake.scale * flake.scale;
            let h = p * (8.0 + index as f32 / 250.0);
            let droop = 2.0 * (index as f32 / 250.0) * weather.snow.m36d0;
            out.push(board(
                flake.base_pos + flake.pos - glam::Vec3::Y * (droop * 0.5),
                glam::Vec2::new(h, h + droop * 0.5),
                [1.0, 1.0, 1.0],
                120.0 / 255.0,
                0.0,
                assets::WeatherPage::Rain,
                true,
                true,
            ));
        }
        let sun = &weather.sun;
        if sun.visibility > 0.0 && (sun.sun_alpha > 0.0 || sun.moon_alpha > 0.0) {
            let alpha = sun.sun_alpha.max(sun.moon_alpha) * sun.visibility;
            out.push(board(
                sun.pos[0],
                glam::Vec2::splat(30.0),
                [
                    sun.color.r as f32 / 127.0,
                    sun.color.g as f32 / 127.0,
                    sun.color.b as f32 / 127.0,
                ],
                alpha.clamp(0.0, 1.0),
                0.0,
                assets::WeatherPage::Rain,
                true,
                true,
            ));
        }
        // Lens flare: eight quads with per-index scale/alpha tables, depth test off
        // (`d_kankyo_rain.cpp:3692-3732,3678`).
        let lens = &weather.sun_lenz;
        if sun.visibility > 0.1 {
            let scales = [
                8000.0, 10000.0, 1600.0, 4800.0, 1200.0, 5600.0, 2400.0, 7200.0,
            ];
            let alphas = [255.0, 80.0, 140.0, 255.0, 125.0, 140.0, 170.0, 140.0];
            for (index, &position) in lens.positions.iter().enumerate() {
                if !lens.draw_lenz_in_sky && index != 0 {
                    continue;
                }
                let page = match index {
                    1 => assets::WeatherPage::LenzB,
                    2 => assets::WeatherPage::LenzA,
                    _ => assets::WeatherPage::Rain,
                };
                let fade = if index >= 2 { lens.dist_falloff } else { 1.0 };
                out.push(board(
                    position,
                    glam::Vec2::splat(scales[index] / 8000.0),
                    [1.0, 1.0, 241.0 / 255.0],
                    (alphas[index] / 255.0) * sun.visibility * sun.visibility * fade,
                    lens.angle_deg as f32,
                    page,
                    true,
                    false,
                ));
            }
        }
        // Stars: untextured, no page (`d_kankyo_rain.cpp:5080-5087`).
        for i in 0..weather.star.effect_num.max(0) as usize {
            let size = if i < 8 {
                190.0
            } else if i < 16 {
                290.0
            } else {
                120.0
            };
            let color = match i {
                6 | 8 => [1.0, 200.0 / 255.0, 200.0 / 255.0],
                _ => [220.0 / 255.0, 230.0 / 255.0, 1.0],
            };
            out.push(board(
                sun.pos[0]
                    + glam::Vec3::new((i % 16) as f32 * 40.0 - 300.0, (i / 16) as f32 * 40.0, 0.0),
                glam::Vec2::splat(size),
                color,
                1.0,
                0.0,
                assets::WeatherPage::Rain,
                false,
                true,
            ));
        }
        // Smoke puffs (housi) and poison.
        let housi = &weather.housi;
        for puff in housi.effect.iter().take(housi.count.max(0) as usize) {
            if puff.alpha <= 0.0 {
                continue;
            }
            let sway = puff.scale.x * 10.0;
            out.push(board(
                puff.base_pos + puff.pos,
                glam::Vec2::new(
                    3.0 - 0.22 * sway.sin(),
                    3.0 - 0.22 * (puff.scale.y * 10.0).sin(),
                ),
                [229.0 / 255.0, 1.0, 200.0 / 255.0],
                puff.alpha * housi.m5ddc,
                0.0,
                assets::WeatherPage::Smoke,
                true,
                true,
            ));
        }
        let poison = &weather.poison;
        for eff in poison.eff.iter().take(poison.count.max(0) as usize) {
            if eff.alpha <= 1e-6 || eff.status == 0 {
                continue;
            }
            out.push(board(
                poison.base_pos + eff.pos,
                glam::Vec2::splat(eff.size),
                [95.0 / 255.0, 186.0 / 255.0, 226.0 / 255.0],
                eff.alpha,
                0.0,
                assets::WeatherPage::Poison,
                true,
                true,
            ));
        }
        // Cloud shadows: depth test off (`d_kankyo_rain.cpp:5465`).
        let clouds = &weather.cloud;
        for cloud in clouds.eff.iter().take(clouds.count.max(0) as usize) {
            if cloud.alpha <= 0.0 {
                continue;
            }
            out.push(board(
                cloud.base_pos + cloud.pos,
                glam::Vec2::splat(cloud.size),
                [
                    weather.bg0_k0.r as f32 / 255.0,
                    weather.bg0_k0.g as f32 / 255.0,
                    weather.bg0_k0.b as f32 / 255.0,
                ],
                cloud.alpha,
                cloud.anim_counter as f32 * -0.02617994,
                assets::WeatherPage::CloudShadow,
                true,
                false,
            ));
        }
        // Fire has no J3D model: `d_a_fire.cpp:222-224` draws only particle emitters. Give the
        // lit source a warm untextured quad so the brazier reads as burning rather than a box.
        for entity in self.sim.world.iter_entities() {
            let Some(fire) = entity.get::<sim::fire::Fire>() else {
                continue;
            };
            if !fire.lit {
                continue;
            }
            let Some(actor) = entity.get::<sim::actor::Actor>() else {
                continue;
            };
            out.push(board(
                actor.pos + glam::Vec3::Y * 8.0,
                glam::Vec2::new(10.0, 18.0),
                [1.0, 0.55, 0.18],
                0.9,
                0.0,
                assets::WeatherPage::Rain,
                false,
                false,
            ));
        }
        // The Windfall music cue is an `ID_AK_SN_MACOREMUSIC00` emitter in the decomp
        // (`d_a_mmusic.cpp:151-153`), not a model; show a small untextured note while active.
        for entity in self.sim.world.iter_entities() {
            let Some(music) = entity.get::<sim::mmusic::Mmusic>() else {
                continue;
            };
            if !music.want {
                continue;
            }
            let Some(actor) = entity.get::<sim::actor::Actor>() else {
                continue;
            };
            out.push(board(
                actor.pos + glam::Vec3::Y * 30.0,
                glam::Vec2::splat(7.0),
                [1.0, 0.92, 0.45],
                0.8,
                0.0,
                assets::WeatherPage::Rain,
                false,
                false,
            ));
        }
        out
    }

    /// Splash crests and sea-surface caps, drawn on world-space planes instead of camera-facing
    /// billboards (`tww/src/d/d_kankyo_rain.cpp:4220-4234,5332-5370`; HD
    /// `wwhd_src/d/d_kankyo_rain_drawSibuki.cpp:68-81`,
    /// `wwhd/wwhd_src/d/d_kankyo_rain_drawWave.cpp:71-112`).
    fn oriented_weather_views(&self) -> Vec<view::OrientedWeatherView> {
        let Some(weather) = self.sim.world.get_resource::<sim::weather::Weather>() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let eye = weather.rain.cam_eye_pos;
        let center = weather.rain.cam_center_pos;
        let forward = (center - eye).normalize_or(glam::Vec3::Z);
        let right = glam::Vec3::Y.cross(forward).normalize_or(glam::Vec3::X);
        let up = forward.cross(right).normalize_or(glam::Vec3::Y);
        // Sibuki: world XZ ground quads scattered in front of the camera.
        if weather.rain.sibuki_alpha > 1.0 {
            let count = weather.rain.rain_count.max(0) as usize / 2;
            let base = eye + forward * 7000.0;
            for i in 0..count {
                let jitter = |k: u32| {
                    let v = (i as u32 + 1)
                        .wrapping_mul(1664525)
                        .wrapping_add(k.wrapping_mul(1013904223));
                    ((v >> 8) % 2000) as f32 / 1000.0 - 1.0
                };
                let pos = base
                    + glam::Vec3::new(jitter(1) * 3600.0, jitter(2) * 1500.0, jitter(3) * 3600.0);
                let size = 20.0 + jitter(4).abs() * 25.0;
                out.push(view::OrientedWeatherView {
                    corners: [
                        [pos.x - size, pos.y, pos.z - size],
                        [pos.x + size, pos.y, pos.z - size],
                        [pos.x + size, pos.y, pos.z + size],
                        [pos.x - size, pos.y, pos.z + size],
                    ],
                    color: [180.0 / 255.0, 200.0 / 255.0, 220.0 / 255.0],
                    alpha: (weather.rain.sibuki_alpha / 255.0 * weather.rain.fwd_fade1)
                        .clamp(0.0, 1.0),
                    page: assets::WeatherPage::LenzB,
                    textured: true,
                    depth_test: true,
                });
            }
        }
        // Wave: camera-plane caps, top edge sheared by wind across the crest.
        let wave_count = weather.wave_chan.wave_count.max(0) as usize;
        if wave_count > 0 {
            for (index, crest) in weather.wave.eff.iter().take(wave_count).enumerate() {
                if crest.alpha <= 0.0 || crest.status == 0 {
                    continue;
                }
                let height = crest.counter.sin() * 250.0 * crest.scale * crest.strength_env;
                let width = 5.0 * 250.0 * crest.scale;
                if width <= 0.0 || height <= 0.0 {
                    continue;
                }
                let shear =
                    width * 1.2 * crest.speed * weather.wave.skew_width * weather.wave.skew_dir;
                let base = crest.base_pos + crest.pos;
                let corner = |x: f32, y: f32| {
                    let p = base + right * x + up * y;
                    [p.x, p.y, p.z]
                };
                out.push(view::OrientedWeatherView {
                    corners: [
                        corner(-width, 0.0),
                        corner(width, 0.0),
                        corner(width + shear, height),
                        corner(-width + shear, height),
                    ],
                    color: [0.12, 0.35, 0.45],
                    alpha: crest.alpha.clamp(0.0, 1.0),
                    page: assets::WeatherPage::Rain,
                    textured: true,
                    depth_test: true,
                });
                let _ = index;
            }
        }
        // Vrkumo: cloud layers on a world-space dome, drawn with depth test off the way HD does
        // (`wwhd/wwhd_src/d/d_kankyo_rain_drawVrkumo.cpp:152-166`).
        let vrkumo_count = weather.vrkumo_count.max(0) as usize;
        if vrkumo_count > 0 {
            for (index, cloud) in weather.vrkumo.inst.iter().take(vrkumo_count).enumerate() {
                if cloud.alpha <= 0.0 {
                    continue;
                }
                let yaw = cloud.position.x.atan2(cloud.position.z);
                let horizontal = (cloud.position.x * cloud.position.x
                    + cloud.position.z * cloud.position.z)
                    .sqrt();
                let pitch = cloud.position.y.atan2(horizontal);
                let radius = 8000.0;
                let direction = glam::Vec3::new(
                    pitch.cos() * yaw.sin(),
                    pitch.sin(),
                    pitch.cos() * yaw.cos(),
                );
                let center = eye + direction * radius;
                let right = glam::Vec3::Y.cross(direction).normalize_or(glam::Vec3::X);
                let up = direction.cross(right).normalize_or(glam::Vec3::Y);
                let size = 400.0 + cloud.height + cloud.dist_falloff * 200.0;
                let corner = |x: f32, y: f32| {
                    let p = center + right * x + up * y;
                    [p.x, p.y, p.z]
                };
                out.push(view::OrientedWeatherView {
                    corners: [
                        corner(-size, -size),
                        corner(size, -size),
                        corner(size, size),
                        corner(-size, size),
                    ],
                    color: [0.85, 0.87, 0.92],
                    alpha: cloud.alpha.clamp(0.0, 1.0),
                    page: assets::WeatherPage::vrkumo(index % 3),
                    textured: true,
                    depth_test: false,
                });
            }
        }
        out
    }

    /// The hookshot chain has no J3D model in either tree: GC draws a hard-coded GX display
    /// list (`tww/src/d/actor/d_a_hookshot.cpp:21,68`) and HD builds a runtime primitive from
    /// guest-memory vertices (`wwhd_src/d/actor/d_a_hookshot_shape.cpp:580-612`). The sim keeps
    /// the two endpoints and the link count, so draw the chain as one world-space ribbon.
    fn hookshot_chain_views(&self) -> Vec<view::OrientedWeatherView> {
        let mut out = Vec::new();
        for entity in self.sim.world.iter_entities() {
            let Some(hook) = entity.get::<sim::hookshot::Hookshot>() else {
                continue;
            };
            if hook.phase == sim::hookshot::Phase::Wait || hook.chain <= 0.0 {
                continue;
            }
            let from = hook.root;
            let to = hook.pos;
            let along = to - from;
            let length = along.length();
            if length < 1e-3 {
                continue;
            }
            let dir = along / length;
            let eye = self
                .sim
                .world
                .get_resource::<sim::camera::Camera>()
                .and_then(|c| c.view)
                .map(|v| v.lookat.eye)
                .unwrap_or(from + glam::Vec3::Y * 100.0);
            let side = dir.cross(eye - from).normalize_or(glam::Vec3::X) * 1.6;
            let corner = |p: glam::Vec3| [p.x, p.y, p.z];
            out.push(view::OrientedWeatherView {
                corners: [
                    corner(from - side),
                    corner(from + side),
                    corner(to + side),
                    corner(to - side),
                ],
                color: [0.62, 0.64, 0.68],
                alpha: 1.0,
                page: assets::WeatherPage::Rain,
                textured: false,
                depth_test: true,
            });
        }
        out
    }

    fn actor_views(&self) -> Vec<view::ActorView> {
        let mut out = Vec::new();
        let anims: std::collections::HashMap<bevy_ecs::entity::Entity, view::ActorAnm> = self
            .sim
            .world
            .get_resource::<sim::actor_anm::ActorAnms>()
            .map(|anms| anms.0.iter().map(|anm| (anm.entity, anm.view())).collect())
            .unwrap_or_default();
        for entity in self.sim.world.iter_entities() {
            if let Some(bomb) = entity.get::<sim::item::Bomb>() {
                let anm = anims.get(&entity.id()).copied();
                let modeled = anm
                    .as_ref()
                    .and_then(|a| actor_model(self, a.archive))
                    .is_some();
                out.push(view::ActorView {
                    pos: bomb.pos,
                    kind: view::ActorKind::Bomb,
                    item: 0,
                    height: 12.0,
                    half: 6.0,
                    anm,
                    modeled,
                });
                continue;
            }
            if let Some(arrow) = entity.get::<sim::arrow::Arrow>() {
                if arrow.visible {
                    let anm = anims.get(&entity.id()).copied();
                    let modeled = anm
                        .as_ref()
                        .and_then(|a| actor_model(self, a.archive))
                        .is_some();
                    out.push(view::ActorView {
                        pos: arrow.pos,
                        kind: view::ActorKind::Arrow,
                        item: 0,
                        height: 3.0,
                        half: 3.0,
                        anm,
                        modeled,
                    });
                }
                continue;
            }
            if let Some(boom) = entity.get::<sim::boomerang::Boomerang>() {
                let anm = anims.get(&entity.id()).copied();
                let modeled = anm
                    .as_ref()
                    .and_then(|a| actor_model(self, a.archive))
                    .is_some();
                out.push(view::ActorView {
                    pos: boom.pos,
                    kind: view::ActorKind::Boomerang,
                    item: 0,
                    height: 10.0,
                    half: 10.0,
                    anm,
                    modeled,
                });
                continue;
            }
            if let Some(spear) = entity.get::<sim::enemy::Spear>() {
                let anm = anims.get(&entity.id()).copied();
                let modeled = anm
                    .as_ref()
                    .and_then(|a| actor_model(self, a.archive))
                    .is_some();
                out.push(view::ActorView {
                    pos: spear.pos,
                    kind: view::ActorKind::Spear,
                    item: 0,
                    height: 4.0,
                    half: 4.0,
                    anm,
                    modeled,
                });
                continue;
            }
            if let Some(shot) = entity.get::<sim::warship::Cannon>() {
                let anm = anims.get(&entity.id()).copied();
                let modeled = anm
                    .as_ref()
                    .and_then(|a| actor_model(self, a.archive))
                    .is_some();
                out.push(view::ActorView {
                    pos: shot.pos,
                    kind: view::ActorKind::Cannon,
                    item: 0,
                    height: 24.0,
                    half: 24.0,
                    anm,
                    modeled,
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
                    anm: None,
                    modeled: false,
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
            } else if let Some(pillar) = entity.get::<sim::hcbh::Hcbh>() {
                if !pillar.drawn() {
                    continue;
                }
                let (_, max) = pillar.cull();
                (view::ActorKind::Scenery, 0, max[1], max[0])
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
            } else if let Some(npc) = entity.get::<sim::npc_ac1::NpcAc1>() {
                let (min, max) = npc.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(npc) = entity.get::<sim::npc_co1::NpcCo1>() {
                let (min, max) = npc.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(npc) = entity.get::<sim::npc_gp1::NpcGp1>() {
                let (min, max) = npc.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(npc) = entity.get::<sim::npc_pf1::NpcPf1>() {
                let (min, max) = npc.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(npc) = entity.get::<sim::npc_uk::NpcUk>() {
                let (min, max) = npc.cull();
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
            } else if entity.contains::<sim::ashut::Ashut>() {
                let (min, max) =
                    sim::ashut::Ashut::spawn(0, &sim::flags::Switches::default()).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::aygr::Aygr>() {
                let (min, max) = sim::aygr::Aygr::spawn(0).cull();
                lift = min[1];
                (
                    view::ActorKind::Scenery,
                    0,
                    max[1] - min[1],
                    (max[0] - min[0]) * 0.5,
                )
            } else if entity.contains::<sim::hami2::Hami2>() {
                let (min, max) =
                    sim::hami2::Hami2::spawn(0, &sim::flags::Switches::default()).cull();
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
            } else if let Some(x) = entity.get::<sim::swflat::Swflat>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::balancelift::Balancelift>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::light::Light>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::vtil::Vtil>() {
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
            } else if let Some(x) = entity.get::<sim::adnno::Adnno>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::daioeye::DaioEye>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::demodk::DemoDk>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::esekh::Esekh>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::eskban::Eskban>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::ikari::Ikari>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::kaji::Kaji>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Fire, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::kamome2::Kamome2>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Gull, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::kanat::Kanat>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::kanban::Kanban>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::kantera::Kantera>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::kmon::Kmon>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::kui::Kui>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::npc_km1::NpcKm1>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Talker, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::obj_akabe::Akabe>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::obj_eff::ObjEff>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::obj_ekskz::Ekskz>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::obj_htetu1::Htetu1>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::obj_itnak::Itnak>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::obj_nest::ObjNest>() {
                let (bottom, height, half) = x.cull();
                lift = bottom;
                (view::ActorKind::Scenery, 0, height, half)
            } else if let Some(x) = entity.get::<sim::obj_paper::Paper>() {
                let (bottom, height, half) = x.cull();
                lift = bottom;
                (view::ActorKind::Scenery, 0, height, half)
            } else if let Some(x) = entity.get::<sim::otble::Otble>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::qtkhd::Qtkhd>() {
                let (half, top) = x.cull();
                lift = 0.0;
                (view::ActorKind::Scenery, 0, top, half)
            } else if let Some(x) = entity.get::<sim::ritem::Ritem>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Pickup, x.item, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::sbox::Sbox>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::sie_flag::SieFlag>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::ssturu::SsTuru>() {
                if !x.drawn() {
                    continue;
                }
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::table::Table>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::tenmado::Tenmado>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::tide::Tide>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::touseki::Touseki>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::usovmc::Usovmc>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::vdora::Vdora>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::warpgn::Warpgn>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::warpt::Warpt>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::ygcwp::Ygcwp>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::ygush::Ygush>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::zouk::Zouk>() {
                let (min, max) = x.cull();
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if entity.contains::<sim::bemos::Bemos>() {
                // Cull boxes: `d_a_obj_bemos.cpp:1388` (CULLBOX_4), `d_a_mbdoor.cpp:611`
                // (CULLBOX_6), `d_a_obj_hfuck1.cpp:277` (CULLBOX_3), `d_a_jbo.cpp:277` and
                // `d_a_nh.cpp:534` (CULLBOX_0), `d_a_obj_gaship2.cpp:157` (CULLBOX_0),
                // `d_a_obj_bscurtain.cpp:149` / `d_a_obj_roten.cpp:203` (CULLBOX_4),
                // `d_a_ep.cpp:560-561` for the 160 cube; `swlight` draws the lamp box from the
                // arrival-state eeprom table (`d_a_obj_swlight.cpp:130` is its 240 cull sphere).
                lift = 0.0;
                (view::ActorKind::Enemy, 0, 800.0, 100.0)
            } else if entity.contains::<sim::mbdoor::Mbdoor>() {
                let (min, max) = ([-150.0_f32, 0.0, -150.0], [150.0_f32, 300.0, 150.0]);
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if entity.contains::<sim::hfuck1::Hfuck1>() {
                let (min, max) = ([-75.0_f32, 0.0, -75.0], [75.0_f32, 150.0, 75.0]);
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if entity.contains::<sim::jbaba::Baba>() {
                let (min, max) = ([-40.0_f32, 0.0, -40.0], [40.0_f32, 125.0, 40.0]);
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if entity.contains::<sim::nh::Nh>() {
                let (min, max) = ([-40.0_f32, 0.0, -40.0], [40.0_f32, 125.0, 40.0]);
                lift = min[1];
                (view::ActorKind::Enemy, 0, max[1] - min[1], max[0])
            } else if entity.contains::<sim::gaship2::Gaship2>() {
                let (min, max) = ([-40.0_f32, 0.0, -40.0], [40.0_f32, 125.0, 40.0]);
                lift = min[1];
                (view::ActorKind::Ship, 0, max[1] - min[1], max[0])
            } else if entity.contains::<sim::bscurtain::BsCurtain>() {
                let (min, max) = ([-100.0_f32, 0.0, -100.0], [100.0_f32, 800.0, 100.0]);
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if entity.contains::<sim::roten::Roten>() {
                let (min, max) = ([-100.0_f32, 0.0, -100.0], [100.0_f32, 800.0, 100.0]);
                lift = min[1];
                (view::ActorKind::Scenery, 0, max[1] - min[1], max[0])
            } else if let Some(x) = entity.get::<sim::ep::Ep>() {
                if !x.drawn() {
                    continue;
                }
                let (base, height, half) = x.hull();
                lift = base;
                (view::ActorKind::Fire, 0, height, half)
            } else if entity.contains::<sim::swlight::Swlight>() {
                lift = 0.0;
                (
                    view::ActorKind::Scenery,
                    0,
                    sim::swlight::LAMP_TOP,
                    sim::swlight::LAMP_HALF_X,
                )
            } else if let Some(x) = entity.get::<sim::canon2::Canon2>() {
                if !x.drawn() {
                    continue;
                }
                let (base, height, half) = x.hull();
                lift = base;
                (view::ActorKind::Cannon, 0, height, half)
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
            } else if entity.contains::<sim::woodbox::WoodBox>() {
                // `d_wood.cpp` builds these from GX display lists, not a J3D model, so the
                // port keeps its box primitive but gives it the wood colour.
                (view::ActorKind::Wood, 0, 36.0, 18.0)
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
                anm: anims.get(&entity.id()).copied(),
                modeled: anims.get(&entity.id()).is_some(),
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
        // The sea texture scrolls with the wave phase; one UV unit per frame at 60 Hz, the
        // rate `d_a_sea.cpp:113-130` advances its scroll offsets with.
        let phase = self.frame as f32 * 0.01;
        Some(view::SeaView {
            origin: sea.origin(),
            side: sim::sea::GRID_CELLS,
            heights: sea.heights().to_vec(),
            color,
            scroll: [phase, phase * 0.5],
        })
    }

    pub fn advance(&mut self, dt: Duration) -> u32 {
        self.frame += 1;
        self.log_input();
        if self.intro.is_some() {
            self.step_intro(dt);
            self.clear_pad_edges();
            return 0;
        }
        // tww/src/d/d_s_name.cpp:1130-1153; tww/src/d/d_gameover.cpp:109-166.
        if self.menu.screen != menus::Screen::Playing {
            if self.assets_installed {
                self.advance_scene(dt);
            } else {
                self.clear_pad_edges();
            }
            return 0;
        }
        if sim::pause::advance(&mut self.sim.world, dt, self.sim.step) {
            return 0;
        }
        let steps = self.sim.step_with(dt, |world| {
            world.resource_mut::<sim::intent::PadButtons>().trig = Default::default();
        });
        if steps == 0 {
            return 0;
        }
        self.drain_demo_creates();
        self.drain_demo_gaps();
        let game_over = self.sim.world.resource::<sim::death::GameOver>().active;
        if game_over && self.menu.screen == menus::Screen::Playing {
            self.menu.screen = menus::Screen::GameOver;
            self.game_over = None;
            self.game_over_failed = false;
            return steps;
        }
        self.apply_exit();
        if self.menu.screen != menus::Screen::Playing {
            return steps;
        }
        self.sync_rooms();
        self.pump_dialogue();
        self.step_meter();
        steps
    }

    /// Step the ported retail item menu for one frame and feed its button assignments into
    /// the sim's X/Y/Z map. `menu_window_step` is the same system the retail schedule uses.
    fn step_item_menu(&mut self) {
        use crate::ui_menu_item::PadState;
        let trig = self.sim.world.resource::<sim::intent::PadButtons>().trig;
        let hold = self.sampled_pad.hold;
        let axis = self.sampled_pad.move_axis;
        // Retail's menu reads press edges for face buttons (`d_menu_item.cpp:2148-2182`,
        // `d_menu_fmap2.cpp:769,1674`); levels would repeat assignment and note close.
        let pad = PadState {
            a: trig.a,
            b: trig.b,
            z: trig.z,
            x: trig.x,
            y: trig.y,
            start: trig.start,
            up: axis.y > 0.5,
            down: axis.y < -0.5,
            left: axis.x < -0.5,
            right: axis.x > 0.5,
            lock_l: hold.l,
            lock_r: hold.r,
            stick_value: axis.length(),
            stick_angle: sim::walk::to_s16(axis.x.atan2(-axis.y)),
        };
        let stage = self
            .item_menu
            .get_or_insert_with(pause_item::ItemMenuStage::default);
        let writes = stage.step(&mut self.sim.world, pad);
        for (button, slot) in writes {
            self.sim
                .world
                .resource_mut::<sim::inventory::ItemSelect>()
                .assign(usize::from(button), slot);
        }
        stage.draw = pause_item::ItemMenuStage::draw_window(&mut self.sim.world);
        stage.revision = stage.revision.wrapping_add(1);
    }

    /// One frame of the ported sea-chart state machine. `d_menu_fmap2.cpp` reads the
    /// face buttons and the main stick, both already staged into the sim's pad resources.
    fn step_chart_menu(&mut self) {
        use crate::ui_menu_fmap2::ChartPadInput;
        let hold = self.sampled_pad.hold;
        let axis = self.sampled_pad.move_axis;
        let trig = self.sim.world.resource::<sim::intent::PadButtons>().trig;
        let pad = ChartPadInput {
            trig_a: trig.a,
            trig_b: trig.b,
            lock_l: hold.l,
            lock_r: hold.r,
            stick_value: axis.length(),
            stick_angle: sim::walk::to_s16(axis.x.atan2(-axis.y)),
            opt_ruby: false,
        };
        let stage = self
            .chart_menu
            .get_or_insert_with(pause_chart::ChartStage::default);
        pause_chart::ChartStage::step(&mut self.sim.world, pad);
        stage.draw = pause_chart::ChartStage::draw(&mut self.sim.world);
        stage.revision = stage.revision.wrapping_add(1);
    }

    /// One frame of the ported dungeon map. The stage's own `DungeonMap` state is created
    /// lazily from the disc's `dmapres.arc` the first time the tab is opened.
    fn step_dmap_menu(&mut self) {
        use crate::ui_menu_dmap::PadInput;
        let trig = self.sim.world.resource::<sim::intent::PadButtons>().trig;
        let axis = self.sampled_pad.move_axis;
        let mut pad = PadInput {
            trig_a: trig.a,
            trig_b: trig.b,
            up: axis.y > 0.5,
            down: axis.y < -0.5,
            left: axis.x < -0.5,
            right: axis.x > 0.5,
            stick_value: axis.length(),
            stick_angle: sim::walk::to_s16(axis.x.atan2(-axis.y)),
        };
        // Feed the map the items the player actually owns and where they stand
        // (tww d_menu_dmap.cpp:434-458, HD hd_menu_button_mapping.cpp:47-51).
        let dungeon_item = self.sim.world.resource::<sim::flags::MemBits>().byte();
        let player_pos = self
            .sim
            .world
            .iter::<sim::player::Player>()
            .next()
            .map_or([0.0; 3], |p| [p.pos.x, p.pos.y, p.pos.z]);
        let visited = self.stage_progress.values().next().map_or(
            [0; crate::ui_menu_dmap::VISITED_ROOM_WORDS],
            |entry| {
                let mut words = [0; crate::ui_menu_dmap::VISITED_ROOM_WORDS];
                for (index, open) in entry.opened.iter().take(16).enumerate() {
                    if *open {
                        words[index / 8] |= 1 << (index % 8);
                    }
                }
                words
            },
        );
        let facts = pause_dmap::DMapFacts {
            dungeon_item,
            player_pos,
            visited_room: visited,
            stay_room: -1,
        };
        let stage = self.dmap.get_or_insert_with(pause_dmap::DMapStage::default);
        if !stage.created {
            stage.created = pause_dmap::DMapStage::create(&mut stage.map, &facts);
        }
        stage.draw = pause_dmap::DMapStage::step(&mut stage.map, &facts, &mut pad);
        stage.revision = stage.revision.wrapping_add(1);
    }

    /// One frame of the ported sea chart (`d_menu_fmap.cpp`).
    fn step_seamap_menu(&mut self) {
        use crate::ui_menu_fmap::PadInput;
        let trig = self.sim.world.resource::<sim::intent::PadButtons>().trig;
        let axis = self.sampled_pad.move_axis;
        let pad = PadInput {
            trig_a: trig.a,
            trig_b: trig.b,
            trig_x: trig.x,
            trig_y: trig.y,
            trig_z: trig.z,
            trig_l: trig.l,
            trig_r: trig.r,
            trig_start: trig.start,
            trig_left: axis.x < -0.5,
            trig_down: axis.y < -0.5,
            stick_value: axis.length(),
            stick_angle: sim::walk::to_s16(axis.x.atan2(-axis.y)),
        };
        let stage = self
            .seamap
            .get_or_insert_with(pause_seamap::SeaMapStage::default);
        stage.step(&mut self.sim.world, pad);
        stage.revision = stage.revision.wrapping_add(1);
    }

    pub(crate) fn clear_pad_edges(&mut self) {
        self.sim
            .world
            .resource_mut::<sim::intent::PadButtons>()
            .trig = Default::default();
    }

    fn advance_scene(&mut self, dt: Duration) {
        if self.menu.screen == menus::Screen::Movie {
            self.step_movie(dt);
            self.clear_pad_edges();
            return;
        }
        if self.menu.screen == menus::Screen::PhotoAlbum {
            self.step_album(dt);
            self.clear_pad_edges();
            return;
        }
        if self.menu.screen == menus::Screen::Pause {
            // The pause menu freezes the simulation: no sim tick, no dialogue or meter
            // advancement, while its own keys are handled once per drawn frame by stage_menu.
            if self.menu.pause.tab == 0 {
                self.step_item_menu();
            } else if self.menu.pause.tab == 2 {
                self.step_dmap_menu();
            } else if self.menu.pause.tab == 3 {
                self.step_chart_menu();
            } else if self.menu.pause.tab == 4 {
                self.step_seamap_menu();
            }
            self.clear_pad_edges();
            return;
        }
        let step = self.sim.step;
        self.menu_clock = (self.menu_clock + dt).min(step * self.sim.max_steps.max(1));
        while self.menu_clock >= step {
            self.menu_clock -= step;
            let before = self.menu.screen;
            match before {
                menus::Screen::Title => {
                    self.sync_title_actor();
                    self.sim
                        .world
                        .run_system_cached(sim::title::control)
                        .expect("registered title resources");
                    self.handle_title_request();
                    self.step_title();
                }
                menus::Screen::FileSelect => self.stage_file_select(),
                menus::Screen::GameOver => self.step_game_over(),
                menus::Screen::ClearSave => self.step_clear_save(),
                _ => {}
            }
            self.clear_pad_edges();
            if self.menu.screen != before || self.intro.is_some() {
                self.menu_clock = Duration::ZERO;
                self.sync_title_actor();
                break;
            }
        }
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
        // tww/src/d/d_s_play.cpp:325-329: ENDING is a movie scene, not a stage archive.
        if dest.stage == "ENDING" {
            self.begin_movie(ui_file_select::SceneName::Ending);
            return;
        }
        let Some(tables) = assets::load_stage_tables(&dest.stage) else {
            return;
        };
        let rooms = assets::room_set_for(&tables, dest.room);
        let Some(spawn) = assets::load_stage_spawn(&dest.stage, dest.room, dest.start) else {
            return;
        };
        // The destination row's start event. `daPy_lk_c` reads it off the row it is created on
        // (`d_a_player_main.cpp:12328`), so a stage entered through an `EVNT` spawn point asks for
        // that event as its own player comes up, not before.
        let start = sim::event::StartEvent::from_player(spawn.player, |index| {
            tables.event_spawn_switch(index)
        });
        self.swap_stage(tables, rooms, spawn.pos, spawn.yaw, layer);
        self.sim.world.insert_resource(start);
        self.order_start_event();
    }

    pub fn swap_stage(
        &mut self,
        tables: dzs::StageTables,
        rooms: Vec<u8>,
        spawn: glam::Vec3,
        yaw: f32,
        layer: i8,
    ) {
        self.store_stage_progress();
        self.restore_stage_progress(tables.stag().map_or(0, |s| s.save_table));
        self.mount_stage(tables, rooms, spawn, yaw, layer);
    }

    fn mount_stage(
        &mut self,
        tables: dzs::StageTables,
        rooms: Vec<u8>,
        spawn: glam::Vec3,
        yaw: f32,
        layer: i8,
    ) {
        // A stage mount with no spawn row of its own - a save load - must not inherit the start
        // event the previous mount asked for. The entry paths that do have a row replace this.
        self.sim
            .world
            .insert_resource(sim::event::StartEvent::default());
        // `dSv_zoneActor_c::init` (`d_save.cpp:1318-1323`) clears the zone actor bits, and it
        // runs from `dStage_roomControl_c::init` (`d_stage.cpp:122-152`), which only happens
        // when a stage is created. A room transition inside the same stage keeps them.
        let same_stage = self
            .sim
            .world
            .get_resource::<sim::area::StageName>()
            .is_some_and(|name| name.0 == tables.name());
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
        // tww/src/d/d_stage.cpp:2246; tww/src/d/d_event_manager.cpp:158-176.
        let manager = assets::AssetRoot::from_env()
            .ok()
            .and_then(|root| sim::event_manager::load_stage_event_list(&root, tables.name()))
            .unwrap_or_default();
        self.sim.world.insert_resource(manager);
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
        sim::transition::carry_zone_actors(
            &mut *self.sim.world.resource_mut::<sim::flags::ZoneActors>(),
            same_stage,
        );
        *self.sim.world.resource_mut::<sim::talk::Dialogue>() = sim::talk::Dialogue::default();
        sim::weight::init(&mut self.sim.world);
        sim::link_proc::init(&mut self.sim.world);
        sim::anim_clock::init(&mut self.sim.world);
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
        // `dStage_RoomCheck` (`d_stage.cpp:2265-2292`) takes the room set from the room read
        // table by the read id the ground polygon's group names, and leaves the room set alone
        // when that id is the reference's own `0xFF` (`d_stage.cpp:2278-2280`).
        let read_id = self
            .sim
            .world
            .get_resource::<sim::transition::RoomSetRequest>()
            .and_then(|request| request.read_id);
        let requested = match read_id {
            Some(id) => tables.load_set(id),
            None => tables.load_set(room),
        };
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

/// The model an `ActorAnm` resolves to, shared by the actor view list and `root_view` so a
/// body only counts as modelled when the archive and its morph are actually loaded.
fn actor_model<'a>(app: &'a App, archive: &str) -> Option<&'a assets::ActorModel> {
    if archive == sim::actor_anm::BA_ARCHIVE {
        app.ba1_model.as_ref()
    } else if archive == sim::actor_anm::CC_ARCHIVE {
        app.chuchu_model.as_ref()
    } else if archive == sim::actor_anm::AM_ARCHIVE {
        app.armos_model.as_ref()
    } else if archive == sim::actor_anm::FM_ARCHIVE {
        app.floormaster_model.as_ref()
    } else if archive == sim::actor_anm::AM2_ARCHIVE {
        app.amos_knight_model.as_ref()
    } else if archive == sim::actor_anm::BDK_ARCHIVE {
        app.helmaroc_model.as_ref()
    } else if archive == sim::actor_anm::FGANON_ARCHIVE {
        app.phantom_ganon_model.as_ref()
    } else {
        app.actor_models.get(archive)
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
    if app.menu.screen == menus::Screen::NameEntry {
        return menus::name_entry_view(&app.menu);
    }
    if app.menu.screen == menus::Screen::Title {
        return ZStack(Modifier::new().fill_max_size())
            .child([title_screen_view(app, ctx), app.touch.view(ctx)]);
    }
    if matches!(
        app.menu.screen,
        menus::Screen::Movie | menus::Screen::ClearSave
    ) {
        return ZStack(Modifier::new().fill_max_size())
            .child([app.ending_view(ctx), app.touch.view(ctx)]);
    }
    let snapshot = app.snapshot();
    let mut camera = app.game_camera().unwrap_or_else(|| app.cam.get());
    if app.camera_underground(&camera, &snapshot) {
        camera = app.fallback_camera(&snapshot);
    }
    // The held sword/grip/shield follow `getSelectEquip(0/1)` (tww/src/d/actor/d_a_player_main.cpp:1945-1964).
    // The blade/grip use the left hand, the shield the right or the back. A missing model or
    // NO_ITEM simply contributes nothing.
    let body_node = |name: &[u8], fallback: usize| {
        app.actor_model.as_ref().map_or(fallback, |m| {
            m.find_node_id(name).unwrap_or(fallback as u32) as usize
        })
    };
    let lhand = body_node(b"cl_LhandA", 8);
    let rhand = body_node(b"cl_RhandA", 12);
    let poda = body_node(b"cl_podA", 13);
    // Retail swaps the blade's bck with the cut direction and samples it at the body cut's frame;
    // at rest it holds the first cut's bck at frame 0 (`tww/src/d/actor/d_a_player_sword.inc:569-571`,
    // `tww/src/d/actor/d_a_player_main.cpp:5877`).
    let blade_slot = snapshot
        .player_anm
        .and_then(|anm| CUT_BLADES.iter().position(|(stem, _, _)| *stem == anm.stem));
    let blade_frame = if blade_slot.is_some() {
        snapshot.player_anm.map_or(0.0, |anm| anm.frame)
    } else {
        0.0
    };
    // The sword goes to its back scabbard whenever another item is selected
    // (`tww/src/d/actor/d_a_player_main.cpp:1269-1277`,
    // `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3180-3186`). The port has no button-to-item
    // selection, so an item animation standing in for it is the closest available signal.
    let sword_on_back = snapshot.item_in_use;
    // The stowed mirror shield sways with the walk/dash cycle; the phase is the under-animation
    // frame off 22 and the amplitude the speed ratio (`tww/src/d/actor/d_a_player_main.cpp:1320-1337`,
    // `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3235-3255`).
    let shield_sway = if snapshot.shield_on_back
        && snapshot.equipment.shield == sim::inventory::ITEM_MIRROR_SHIELD
        && matches!(
            snapshot.player_anm.map(|anm| anm.stem),
            Some("walk" | "dash")
        ) {
        let end = app
            .player_anim
            .as_ref()
            .map_or(30.0, |idle| idle.stem_frame_max("walk"));
        let frame = snapshot.player_anm.map_or(0.0, |anm| anm.frame);
        let phase = std::f32::consts::TAU * (frame - 22.0) / end;
        -5500.0
            * (snapshot.player_speed / sim::walk::MAX_NORMAL_SPEED).abs()
            * phase.sin()
            * std::f32::consts::TAU
            / 65536.0
    } else {
        0.0
    };
    // The master grip's wing bck takes frame 1 for the final two sword tiers
    // (`tww/src/d/actor/d_a_player_main.cpp:1243-1248`,
    // `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3153-3159`).
    let grip_frame = if matches!(
        snapshot.equipment.sword,
        sim::damage::MASTER_SWORD_2 | sim::damage::MASTER_SWORD_3
    ) {
        1.0
    } else {
        0.0
    };
    // The BRK shine is gated on the final Master Sword alone, unlike the wing BCK above
    // (`tww/src/d/actor/d_a_player_main.cpp:1232-1235`,
    // `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3145-3151`). The renderer
    // has no MDL3/TEV C-register path, so the sampled s10 register is used as a vertex tint.
    let grip_tint = if snapshot.equipment.sword == sim::damage::MASTER_SWORD_3 {
        app.part_anims.grip_brk.as_ref().and_then(|brk| {
            let c = brk.tev_color(0, 1.0)?;
            Some([
                (1.0 + c[0] as f32 / 1024.0).max(0.0),
                (1.0 + c[1] as f32 / 1024.0).max(0.0),
                (1.0 + c[2] as f32 / 1024.0).max(0.0),
            ])
        })
    } else {
        None
    };
    // Texture-scroll matrices for the grip and mirror shield, keyed to their material slots
    // (`tww/src/d/actor/d_a_player_main.cpp:1261-1268,1283-1294`,
    // `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3167-3169,3191-3201`).
    let part_btk = app.sim.world.get_resource::<sim::part_btk::PartBtk>();
    let grip_uv = app.part_anims.grip_btk.as_ref().and_then(|anim| {
        let track = anim.track("SWGripMSBMAT", 0)?;
        Some((0u16, anim.uv_transform(track, part_btk?.grip)?))
    });
    let shield_uv = app.part_anims.shield_btk.as_ref().and_then(|anim| {
        let track = anim.track("SHMSBMAT", 0)?;
        Some((1u16, anim.uv_transform(track, part_btk?.shield)?))
    });
    // Material animations on the other held parts, sampled at the body cut's frame where the
    // reference drives them alongside the transform bck (`d_a_player_sword.inc:99-172`,
    // `d_a_player_tact.inc:42-48`,
    // `wwhd/wwhd_src/d/actor/d_a_player_main_01.cpp:2621-2633`).
    let tip_uv = first_uv(app.part_anims.tip_btk.as_ref(), blade_frame);
    let tip_color = first_color(app.part_anims.tip_brk.as_ref(), blade_frame);
    let master_tip_uv = first_uv(app.part_anims.master_tip_btk.as_ref(), blade_frame);
    let master_tip_color = first_color(app.part_anims.master_tip_brk.as_ref(), blade_frame);
    let blade_uv = first_uv(app.part_anims.blade_btk.as_ref(), blade_frame);
    let takt_phase = app.sim.world.get_resource::<sim::tact::Tact>();
    let takt_color = first_color(
        app.part_anims.takt_brk.as_ref(),
        if takt_phase.is_some_and(|tact| {
            matches!(
                tact.phase,
                sim::tact::TactPhase::Play | sim::tact::TactPhase::Direction
            )
        }) {
            1.0
        } else {
            0.0
        },
    );
    let soup_frame = if app
        .sim
        .world
        .get_resource::<sim::combat::Combat>()
        .is_some_and(|c| c.soup)
    {
        1.0
    } else {
        0.0
    };
    let glow_uv = first_uv(app.part_anims.glow_btk.as_ref(), soup_frame);
    let glow_color = first_color(app.part_anims.glow_brk.as_ref(), soup_frame);
    let bottle_uv = first_uv(app.part_anims.bottle_btk.as_ref(), snapshot.tick as f32);
    let bottle_item = app
        .sim
        .world
        .get_resource::<sim::inventory::Equipped>()
        .and_then(|equipped| {
            (sim::ba1::ITEM_BOTTLE_SLOT_0
                ..sim::ba1::ITEM_BOTTLE_SLOT_0 + sim::ba1::ITEM_BOTTLE_SLOTS)
                .map(|slot| equipped.item(slot))
                .find(|&item| item != sim::inventory::NO_ITEM)
        });
    // The shield bck follows the under-body frame while guarding, frame 0 at rest
    // (`tww/src/d/actor/d_a_player_main.cpp:1341,6469-6471`,
    // `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3259-3260`).
    let is_guard = matches!(
        snapshot.player_anm.map(|anm| anm.stem),
        Some("difence" | "difencea" | "atng")
    );
    let shield_frame = if is_guard {
        snapshot.player_anm.map_or(0.0, |anm| anm.frame % 12.0)
    } else {
        0.0
    };
    let mut held: Vec<view::AttachedPart<'_>> = Vec::new();
    if !snapshot.player_no_draw {
        let sword = snapshot.equipment.sword;
        if sword == sim::damage::ITEM_SWORD {
            if let Some(model) = app.sword_model.as_ref() {
                let mut part = if sword_on_back {
                    view::AttachedPart::sword_back(model, poda)
                } else {
                    view::AttachedPart::hand(model, lhand)
                };
                if let Some(morph) = app.part_anims.blades[blade_slot.unwrap_or(0)].as_ref() {
                    part.anim = Some((morph, blade_frame));
                }
                held.push(part);
            }
            if let Some(model) = app.sword_grip_model.as_ref() {
                held.push(if sword_on_back {
                    view::AttachedPart::sword_back(model, poda)
                } else {
                    view::AttachedPart::hand(model, lhand)
                });
            }
        } else if sword != sim::inventory::NO_ITEM {
            if let Some(model) = app.master_sword_model.as_ref() {
                let mut part = if sword_on_back {
                    view::AttachedPart::sword_back(model, poda)
                } else {
                    view::AttachedPart::hand(model, lhand)
                };
                if let Some(morph) = app.part_anims.master_blades[blade_slot.unwrap_or(0)].as_ref()
                {
                    part.anim = Some((morph, blade_frame));
                }
                part.uv = blade_uv;
                held.push(part);
            }
            if let Some(model) = app.master_grip_model.as_ref() {
                let mut part = if sword_on_back {
                    view::AttachedPart::sword_back(model, poda)
                } else {
                    view::AttachedPart::hand(model, lhand)
                };
                if let Some(morph) = app.part_anims.grip.as_ref() {
                    part.anim = Some((morph, grip_frame));
                }
                part.color = grip_tint.map(|tint| (0, tint));
                part.uv = grip_uv;
                held.push(part);
            }
        }
        // The forward cut adds a tip-stab model at the blade's chest joint, coloured by the
        // sword tier's BPK (`d_a_player_main.cpp:11604-11609`, `d_a_player_sword.inc:36-56`).
        if snapshot.player_anm.map(|anm| anm.stem) == Some("cutf") {
            let normal = sword == sim::damage::ITEM_SWORD;
            let model = if normal {
                app.tip_stab_model.as_ref()
            } else if sword != sim::inventory::NO_ITEM {
                app.master_tip_stab_model.as_ref()
            } else {
                None
            };
            if let Some(model) = model {
                let mut part = if sword_on_back {
                    view::AttachedPart::sword_back(model, poda)
                } else {
                    view::AttachedPart::hand(model, lhand)
                };
                part.uv = if normal { tip_uv } else { master_tip_uv };
                part.color = if normal { tip_color } else { master_tip_color }.or_else(|| {
                    let bpk = if normal {
                        app.part_anims.tip_bpk.as_ref()
                    } else {
                        app.part_anims.master_tip_bpk.as_ref()
                    };
                    bpk.and_then(|anm| anm.color(0, blade_frame)).map(|rgba| {
                        (
                            0,
                            [
                                rgba[0] as f32 / 255.0,
                                rgba[1] as f32 / 255.0,
                                rgba[2] as f32 / 255.0,
                            ],
                        )
                    })
                });
                held.push(part);
            }
        }
        // The final Master Sword carries a glow aura over the blade
        // (`d_a_player_sword.inc:158-204`), attached at the same hand joint.
        if sword == sim::damage::MASTER_SWORD_3 {
            if let Some(model) = app.master_glow_model.as_ref() {
                let mut part = if sword_on_back {
                    view::AttachedPart::sword_back(model, poda)
                } else {
                    view::AttachedPart::hand(model, lhand)
                };
                part.uv = glow_uv;
                part.color = glow_color;
                held.push(part);
            }
        }
        let shield_anim = app.part_anims.shield.as_ref().map(|m| (m, shield_frame));
        match snapshot.equipment.shield {
            sim::inventory::ITEM_SHIELD => {
                if let Some(model) = app.shield_model.as_ref() {
                    let mut part = if snapshot.shield_on_back {
                        view::AttachedPart::back(model, poda, false)
                    } else {
                        view::AttachedPart::hand(model, rhand)
                    };
                    part.anim = shield_anim;
                    held.push(part);
                }
            }
            sim::inventory::ITEM_MIRROR_SHIELD => {
                if let Some(model) = app.mirror_shield_model.as_ref() {
                    let mut part = if snapshot.shield_on_back {
                        view::AttachedPart::back(model, poda, true)
                    } else {
                        view::AttachedPart::hand(model, rhand)
                    };
                    part.anim = shield_anim;
                    part.sway = shield_sway;
                    part.uv = shield_uv;
                    held.push(part);
                }
            }
            _ => {}
        }
        // Held item bodies. Retail attaches these to lHandA with identity, except the telescope's
        // fixed offset (`tww/src/d/actor/d_a_player_main.cpp:1139-1159`,
        // `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:2981-3021`).
        let fan_phase = app
            .sim
            .world
            .get_resource::<sim::deku_leaf::Fan>()
            .map(|fan| fan.phase);
        if fan_phase.is_some_and(|phase| phase != sim::deku_leaf::FanPhase::Idle) {
            let gliding = fan_phase == Some(sim::deku_leaf::FanPhase::Glide);
            let model = if gliding {
                app.leaf_open_model.as_ref()
            } else {
                app.leaf_model.as_ref()
            };
            if let Some(model) = model {
                let mut part = view::AttachedPart::hand(model, lhand);
                let morph = if gliding {
                    app.part_anims.leaf_open.as_ref()
                } else {
                    app.part_anims.leaf.as_ref()
                };
                if let Some(morph) = morph {
                    part.anim = Some((morph, 0.0));
                }
                held.push(part);
            }
        }
        if app
            .sim
            .world
            .get_resource::<sim::hammer::Cut>()
            .is_some_and(|cut| cut.kind != sim::hammer::CUT_NONE)
        {
            if let Some(model) = app.hammer_model.as_ref() {
                let mut part = view::AttachedPart::hand(model, lhand);
                if let Some(morph) = app.part_anims.hammer.as_ref() {
                    part.anim = Some((morph, 0.0));
                }
                held.push(part);
            }
        }
        if app
            .sim
            .world
            .get_resource::<sim::scope::Scope>()
            .is_some_and(|scope| scope.active)
        {
            if let Some(model) = app.telescope_model.as_ref() {
                let mut part = view::AttachedPart::hand(model, lhand);
                let deg = std::f32::consts::TAU / 65536.0;
                part.local = glam::Mat4::from_translation(glam::Vec3::new(8.0, 15.0, 1.0))
                    * glam::Mat4::from_euler(
                        glam::EulerRot::XYZ,
                        -0x4000 as f32 * deg,
                        0x4444 as f32 * deg,
                        -0x41F as f32 * deg,
                    );
                held.push(part);
            }
        }
        if app
            .sim
            .world
            .get_resource::<sim::tact::Tact>()
            .is_some_and(|tact| tact.phase != sim::tact::TactPhase::Idle)
        {
            if let Some(model) = app.takt_model.as_ref() {
                let mut part = view::AttachedPart::hand(model, lhand);
                part.color = takt_color;
                held.push(part);
            }
        }
        if app
            .sim
            .world
            .get_resource::<sim::item_use::ItemUse>()
            .and_then(|state| state.stem)
            .is_some_and(|stem| matches!(stem, "binopena" | "bindrinking"))
        {
            if let Some(model) = app.bottle_model.as_ref() {
                let mut part = view::AttachedPart::hand(model, lhand);
                part.uv = bottle_uv;
                held.push(part);
            }
            // The contents sit inside the bottle body: a coloured liquid for potions, a
            // pulsating fairy for the fairy bottle (`d_a_player_bottle.inc:39-78`).
            let contents = match bottle_item {
                Some(sim::ba1::ITEM_FAIRY_BOTTLE) => app.bottle_fairy_model.as_ref(),
                Some(0x51..=0x55) => app.bottle_liquid_model.as_ref(),
                _ => None,
            };
            if let Some(model) = contents {
                let mut part = view::AttachedPart::hand(model, lhand);
                let (anim, frame) = if bottle_item == Some(sim::ba1::ITEM_FAIRY_BOTTLE) {
                    (app.part_anims.firefly_brk.as_ref(), snapshot.tick as f32)
                } else {
                    (
                        app.part_anims.liquid_brk.as_ref(),
                        f32::from(bottle_item.unwrap_or(0x51) - 0x51),
                    )
                };
                part.color = first_color(anim, frame);
                held.push(part);
            }
        }
        if app
            .sim
            .world
            .get_resource::<sim::item_use::ItemUse>()
            .and_then(|state| state.stem)
            == Some("photo")
        {
            if let Some(model) = app.camera_model.as_ref() {
                held.push(view::AttachedPart::hand(model, lhand));
            }
        }
    }
    // Grandma is the one actor whose animation is already published to the view. Place her model
    // with the ECS actor's own transform and pose it with the BCK its cursor named
    // (tww/src/d/actor/d_a_npc_ba1.cpp:273-299, HD `d_a_npc_ba1.cpp:393-423`).
    let npcs: Vec<view::NpcDraw<'_>> = app
        .sim
        .world
        .get_resource::<sim::actor_anm::ActorAnms>()
        .map(|anms| {
            anms.0
                .iter()
                .filter_map(|anm| {
                    let model = actor_model(app, anm.archive)?;
                    // A negative bck is the static-pose sentinel for bodies with no animation
                    // (`NpcDraw::anim == None` draws the bind pose).
                    let anim = if anm.bck < 0 {
                        None
                    } else {
                        let (_, morph) = model.morphs.iter().find(|(id, _)| *id == anm.bck)?;
                        let end = morph.frame_max().max(1.0);
                        let frame = anm.frame * 2.0;
                        let frame = if anm.looping {
                            frame % end
                        } else {
                            frame.min(end)
                        };
                        Some((morph, frame))
                    };
                    let actor = app.sim.world.get::<sim::actor::Actor>(anm.entity)?;
                    let ba1 = if anm.archive == sim::actor_anm::BA_ARCHIVE {
                        app.sim.world.get::<sim::ba1::Ba1>(anm.entity)
                    } else {
                        None
                    };
                    let joint_control = match (
                        model.mesh.find_node_id(b"head"),
                        model.mesh.find_node_id(b"backbone"),
                        ba1,
                    ) {
                        (Some(head), Some(backbone), Some(ba1)) => Some((
                            head as u16,
                            -sim::walk::from_s16(ba1.joint_head_y),
                            backbone as u16,
                            sim::walk::from_s16(ba1.joint_backbone_y),
                        )),
                        _ => None,
                    };
                    let cloth = model
                        .mesh
                        .find_node_id(b"footL")
                        .zip(model.cloth.as_ref())
                        .map(|(joint, mesh)| (joint as u16, mesh));
                    Some(view::NpcDraw {
                        mesh: &model.mesh,
                        anim,
                        pos: actor.pos,
                        yaw: sim::walk::from_s16(actor.shape_angle[1]),
                        joint_control,
                        cloth,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    // The slash trail is the blade tip sampled at past cut frames, ribboned by the view with
    // `blur.bti` (`tww/src/d/actor/d_a_player_particle.inc:193-309`).
    let blur = app.blur_texture.as_ref().and_then(|(rgba, width, height)| {
        let stem = snapshot.player_anm.map(|anm| anm.stem)?;
        if !matches!(
            stem,
            "cuta" | "cutf" | "cutr" | "cutl" | "cutea" | "cuteb" | "cutturnb"
        ) {
            return None;
        }
        let morph = if snapshot.equipment.sword == sim::damage::ITEM_SWORD {
            app.part_anims.blades[blade_slot.unwrap_or(0)].as_ref()
        } else {
            app.part_anims.master_blades[blade_slot.unwrap_or(0)].as_ref()
        }?;
        let frame = snapshot.player_anm.map_or(0.0, |anm| anm.frame);
        let to_world = glam::Mat4::from_translation(snapshot.player_pos)
            * glam::Mat4::from_rotation_y(-snapshot.player_yaw);
        let points = (0..16u32)
            .map(|k| {
                let past = (frame - k as f32 * 2.0).max(0.0);
                let joints = morph.joint_matrices(past);
                let tip = joints
                    .last()
                    .copied()
                    .unwrap_or(glam::Mat4::IDENTITY)
                    .w_axis
                    .truncate();
                to_world.transform_point3(tip)
            })
            .collect();
        Some(view::SwordBlurDraw {
            points,
            rgba,
            width: *width,
            height: *height,
        })
    });
    let frame = match &app.dff {
        Some(presented) => j3d::present::frame(presented, camera),
        None => view::build_frame_with_actor(
            &snapshot,
            camera,
            app.room.as_ref(),
            app.actor_model.as_ref(),
            app.player_anim.as_ref(),
            app.hair_model.as_ref(),
            app.hands_model.as_ref(),
            &held,
            &npcs,
            blur.as_ref(),
            &app.weather_textures,
            app.sea_texture.as_ref(),
        ),
    };
    let desc = match &app.dff {
        Some(_) => view::batch_desc(app.room.as_ref()),
        None => view::batch_desc_with_actor(
            app.room.as_ref(),
            app.actor_model.as_ref(),
            app.hair_model.as_ref(),
            app.hands_model.as_ref(),
            &held,
            &npcs,
            blur.is_some(),
            if snapshot.billboards.is_empty() && snapshot.oriented_weather.is_empty() {
                0
            } else {
                app.weather_textures.len()
            },
            usize::from(app.sea_texture.is_some()),
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
        menus::Screen::PhotoAlbum => app.album_view(ctx),
        menus::Screen::Pause => match app.menu.pause.tab {
            0 => item_menu_view(app, ctx),
            2 => dmap_menu_view(app, ctx),
            3 => chart_menu_view(app, ctx),
            4 => seamap_menu_view(app, ctx),
            _ => pause_menu::view(app),
        },
        // The shell's own screens belong to the pre-boot session; a session that somehow
        // lands on one of them draws the retail frame alone rather than the shell over it.
        menus::Screen::Title
        | menus::Screen::Settings
        | menus::Screen::QuitConfirm
        | menus::Screen::NameEntry
        | menus::Screen::Movie
        | menus::Screen::ClearSave => Spacer(),
    };
    ZStack(Modifier::new().fill_max_size()).child([
        viewport,
        overlay,
        dialogue_view(app, ctx),
        app.touch.view(ctx),
    ])
}

/// The ported item menu's canvas, composited over the live view while the pause menu's
/// ITEM tab is open. The fallback is the shell-level tab view when the BLO is unavailable.
fn item_menu_view(app: &App, ctx: &repose_core::RenderContext) -> repose_core::View {
    let Some(stage) = app.item_menu.as_ref() else {
        return pause_menu::view(app);
    };
    let Some(assets) = panes::load() else {
        return pause_menu::view(app);
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
        canvas.fill_rect([0.0, 0.0, 640.0, 480.0], [8, 12, 20, 255]);
        if let Some(draw) = stage.draw.as_ref() {
            pause_item::ItemMenuStage::render(&mut canvas, assets, draw);
        }
        ctx.set_image_rgba8(handle, 640, 480, canvas.pixels, true);
        stage.uploaded.set(stage.revision);
    }
    Image(Modifier::new().fill_max_size(), handle).image_fit(ImageFit::Contain)
}

/// The ported dungeon map's canvas, composited over the live view on the MAP tab.
fn dmap_menu_view(app: &App, ctx: &repose_core::RenderContext) -> repose_core::View {
    let Some(stage) = app.dmap.as_ref() else {
        return pause_menu::view(app);
    };
    let Some(assets) = panes::load() else {
        return pause_menu::view(app);
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
        canvas.fill_rect([0.0, 0.0, 640.0, 480.0], [8, 12, 20, 255]);
        if let Some(draw) = stage.draw.as_ref() {
            pause_dmap::DMapStage::render(&mut canvas, assets, draw);
        }
        ctx.set_image_rgba8(handle, 640, 480, canvas.pixels, true);
        stage.uploaded.set(stage.revision);
    }
    Image(Modifier::new().fill_max_size(), handle).image_fit(ImageFit::Contain)
}

/// The ported sea map's canvas, composited over the live view on the SEA tab.
fn seamap_menu_view(app: &App, ctx: &repose_core::RenderContext) -> repose_core::View {
    let Some(stage) = app.seamap.as_ref() else {
        return pause_menu::view(app);
    };
    let Some(assets) = panes::load() else {
        return pause_menu::view(app);
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
        canvas.fill_rect([0.0, 0.0, 640.0, 480.0], [8, 12, 20, 255]);
        if let Some(draw) = stage.draw.as_ref() {
            pause_seamap::SeaMapStage::render(&mut canvas, assets, draw);
        }
        ctx.set_image_rgba8(handle, 640, 480, canvas.pixels, true);
        stage.uploaded.set(stage.revision);
    }
    Image(Modifier::new().fill_max_size(), handle).image_fit(ImageFit::Contain)
}

/// The ported sea chart's canvas, composited over the live view on the CHART tab.
fn chart_menu_view(app: &App, ctx: &repose_core::RenderContext) -> repose_core::View {
    let Some(stage) = app.chart_menu.as_ref() else {
        return pause_menu::view(app);
    };
    let Some(assets) = panes::load() else {
        return pause_menu::view(app);
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
        canvas.fill_rect([0.0, 0.0, 640.0, 480.0], [8, 12, 20, 255]);
        if let Some(draw) = stage.draw.as_ref() {
            pause_chart::ChartStage::render(&mut canvas, assets, draw);
        }
        ctx.set_image_rgba8(handle, 640, 480, canvas.pixels, true);
        stage.uploaded.set(stage.revision);
    }
    Image(Modifier::new().fill_max_size(), handle).image_fit(ImageFit::Contain)
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
        menus::Screen::FileSelect
        | menus::Screen::Playing
        | menus::Screen::GameOver
        | menus::Screen::PhotoAlbum
        | menus::Screen::Pause
        | menus::Screen::NameEntry
        | menus::Screen::Movie
        | menus::Screen::ClearSave => Spacer(),
    }
}

/// `dGameover_c::_draw` (`d_gameover.cpp:168-175`): the letters screen, then the
/// save menu's `_draw2`, composited over the live view.
fn game_over_overlay(app: &App, ctx: &repose_core::RenderContext) -> repose_core::View {
    let Some(stage) = app.game_over.as_ref() else {
        if app.game_over_failed {
            // No BLO bound, so show a minimal prompt instead of a frozen world with no
            // exit (`d_gameover.cpp:168-175`).
            return repose_ui::Center(repose_core::Modifier::new().fill_max_size()).child(
                repose_ui::Text("GAME OVER\nA: Continue   B: Title")
                    .size(repose_core::Sp(28.0))
                    .color(repose_core::Color::from_hex("#F4EAD5")),
            );
        }
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
        stage.save_assets.draw(&mut canvas, &stage.game.save);
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
    if !app.dialogue_live
        || app.menu.screen == menus::Screen::Pause
        || app.dialogue_canvas.pixels.is_empty()
    {
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
