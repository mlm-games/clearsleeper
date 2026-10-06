use bevy_ecs::prelude::*;
use bevy_ecs::world::World;
use glam::Vec3;

use crate::j2d;
use crate::sim::collision::Collision;
use crate::sim::flags::StoryBits;
use crate::text::msgbox::{Pane, PaneState};
use crate::text::msgproc::fop_msgm_value_increase;

pub const PANE_TYPE_ID_PICTURE: u16 = 0x12;

pub const G_CM3D_F_INF: f32 = 1000000000.0;
pub const G_CM3D_F_ABS_MIN: f32 = 3.8146973e-06;
pub const S2RAD_CONSTANT: f32 = std::f32::consts::PI / 0x8000;

pub fn s2rad(units: i16) -> f32 {
    units as f32 * S2RAD_CONSTANT
}

pub fn s_cos(units: i16) -> f32 {
    s2rad(units).cos()
}

pub fn s_sin(units: i16) -> f32 {
    s2rad(units).sin()
}

pub fn m3d_is_zero(value: f32) -> bool {
    value.abs() < G_CM3D_F_ABS_MIN
}

pub fn c_atan2s(f0: f32, f1: f32) -> i16 {
    let angle = if m3d_is_zero(f0) {
        if f1 >= 0.0 { 0.0 } else { std::f32::consts::PI }
    } else if m3d_is_zero(f1) {
        if f0 >= 0.0 {
            std::f32::consts::FRAC_PI_2
        } else {
            -std::f32::consts::FRAC_PI_2
        }
    } else {
        f0.atan2(f1)
    };
    (angle * (0x8000 as f32 / std::f32::consts::PI))
        .round()
        .rem_euclid(65536.0) as i16
}

pub type Mtx = [[f32; 4]; 3];

pub const MTX_IDENTITY: Mtx = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
];

pub fn mtx_trans(x: f32, y: f32, z: f32) -> Mtx {
    [[1.0, 0.0, 0.0, x], [0.0, 1.0, 0.0, y], [0.0, 0.0, 1.0, z]]
}

pub fn mtx_scale(x: f32, y: f32, z: f32) -> Mtx {
    [[x, 0.0, 0.0, 0.0], [0.0, y, 0.0, 0.0], [0.0, 0.0, z, 0.0]]
}

pub fn mtx_xrot(degrees: i16) -> Mtx {
    let cos = s_cos(degrees);
    let sin = s_sin(degrees);
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, cos, -sin, 0.0],
        [0.0, sin, cos, 0.0],
    ]
}

pub fn mtx_yrot(degrees: i16) -> Mtx {
    let cos = s_cos(degrees);
    let sin = s_sin(degrees);
    [
        [cos, 0.0, sin, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-sin, 0.0, cos, 0.0],
    ]
}

pub fn mtx_zrot(degrees: i16) -> Mtx {
    let cos = s_cos(degrees);
    let sin = s_sin(degrees);
    [
        [cos, -sin, 0.0, 0.0],
        [sin, cos, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ]
}

pub fn mtx_concat(a: &Mtx, b: &Mtx) -> Mtx {
    let mut out = MTX_IDENTITY;
    for row in 0..3 {
        for col in 0..3 {
            out[row][col] = a[row][0] * b[0][col] + a[row][1] * b[1][col] + a[row][2] * b[2][col];
        }
    }
    for row in 0..3 {
        out[row][3] = a[row][0] * b[0][3] + a[row][1] * b[1][3] + a[row][2] * b[2][3] + a[row][3];
    }
    out
}

pub fn mtx_zxy_rot_m(mtx: Mtx, x: i16, y: i16, z: i16) -> Mtx {
    let mut out = mtx;
    if y != 0 {
        let tmp = mtx_yrot(y);
        out = mtx_concat(&out, &tmp);
    }
    if x != 0 {
        let tmp = mtx_xrot(x);
        out = mtx_concat(&out, &tmp);
    }
    if z != 0 {
        let tmp = mtx_zrot(z);
        out = mtx_concat(&out, &tmp);
    }
    out
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum Phase {
    #[default]
    Init = 0,
    Loading = 1,
    Next = 2,
    Stop = 3,
    Complete = 4,
    Error = 5,
}

pub const PLACE_NAME_STAGE_MAX: u32 = 19;
pub const PLACE_NAME_ARCHIVE: &str = "PName";
pub const PLACE_NAME_LAYOUT: &str = "place_name.blo";
pub const PLACE_NAME_HEAP_LOCK: u8 = 10;
pub const PLACE_NAME_TEX_BYTES: usize = 0x3C00;
pub const PLACE_NAME_TEX_ALIGN: usize = 0x20;
pub const DEFAULT_MOUNT_DIRECTION: i32 = 0;
pub const CARD_TAG: &str = "pn";
pub const CARD_BAR_TAGS: [&str; 2] = ["blc1", "blc2"];
pub const CARD_FADE_FRAMES: i32 = 10;

pub const NAME_TEXTURE: [&str; 19] = [
    "/res/placename/pn_01.bti",
    "/res/placename/pn_02.bti",
    "/res/placename/pn_03.bti",
    "/res/placename/pn_04.bti",
    "/res/placename/pn_05.bti",
    "/res/placename/pn_06.bti",
    "/res/placename/pn_07.bti",
    "/res/placename/pn_08.bti",
    "/res/placename/pn_09.bti",
    "/res/placename/pn_10.bti",
    "/res/placename/pn_11.bti",
    "/res/placename/pn_12.bti",
    "/res/placename/pn_13.bti",
    "/res/placename/pn_14.bti",
    "/res/placename/pn_15.bti",
    "/res/placename/pn_16.bti",
    "/res/placename/pn_17.bti",
    "/res/placename/pn_18.bti",
    "/res/placename/pn_19.bti",
];

pub fn name_texture(stage: u32) -> Option<&'static str> {
    if stage >= PLACE_NAME_STAGE_MAX {
        return None;
    }
    NAME_TEXTURE.get(stage as usize).copied()
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaneQuad {
    pub name: String,
    pub kind: j2d::Kind,
    pub bounds: j2d::Bounds,
    pub alpha: u8,
    pub visible: bool,
    pub texture: Option<String>,
    pub text: String,
}

#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct PlaceName {
    pub screen: Option<j2d::Screen>,
    pub pane: PaneState,
    pub image: Vec<u8>,
}

fn picture_pane(pane: &j2d::ScreenPane) -> Pane {
    Pane {
        type_id: PANE_TYPE_ID_PICTURE,
        bounds: [
            pane.bounds.x,
            pane.bounds.y,
            pane.bounds.right(),
            pane.bounds.bottom(),
        ],
        alpha: pane.alpha,
        visible: pane.visible,
        connect_parent: false,
        influenced_alpha: true,
        parent_draw_alpha: 0xFF,
        white: 0xFFFF_FFFF,
    }
}

impl PlaceName {
    pub fn set_screen(&mut self, mut screen: j2d::Screen) {
        for tag in CARD_BAR_TAGS {
            if let Some(pane) = screen.panes.iter_mut().find(|pane| pane.name == tag) {
                pane.visible = false;
            }
        }
        if let Some(pane) = screen.find(CARD_TAG) {
            self.pane.set_pane_data(picture_pane(pane));
        }
        self.screen = Some(screen);
    }

    pub fn add_counter(&mut self) {
        self.pane.user_area += 1;
    }

    pub fn dec_counter(&mut self) {
        self.pane.user_area -= 1;
    }

    pub fn get_counter(&self) -> i16 {
        self.pane.user_area
    }

    pub fn change_texture(&mut self, image: &[u8]) {
        self.image = image.to_vec();
    }

    pub fn delete_screen(&mut self) {
        self.screen = None;
        self.image = Vec::new();
    }

    pub fn open_anime(&mut self) -> bool {
        if self.get_counter() < CARD_FADE_FRAMES as i16 {
            self.add_counter();
            let alpha = fop_msgm_value_increase(CARD_FADE_FRAMES, i32::from(self.get_counter()), 0)
                .unwrap_or_default();
            self.pane.set_now_alpha(alpha);
        }
        true
    }

    pub fn close_anime(&mut self) -> bool {
        let mut ret = false;
        if self.get_counter() > 0 {
            self.dec_counter();
            let alpha = fop_msgm_value_increase(CARD_FADE_FRAMES, i32::from(self.get_counter()), 0)
                .unwrap_or_default();
            self.pane.set_now_alpha(alpha);
        }
        if self.get_counter() <= 0 {
            ret = true;
        }
        ret
    }

    pub fn draw(&mut self) -> Vec<PaneQuad> {
        self.pane.set_alpha();
        let alpha = self.pane.now_alpha;
        let Some(screen) = self.screen.as_ref() else {
            return Vec::new();
        };
        screen
            .panes
            .iter()
            .map(|pane| PaneQuad {
                name: pane.name.clone(),
                kind: pane.kind,
                bounds: pane.bounds,
                alpha,
                visible: pane.visible,
                texture: pane.texture.clone(),
                text: pane.text.clone(),
            })
            .collect()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DvdThread {
    pub path: String,
    pub mount: i32,
    pub bytes: Vec<u8>,
    pub ready: bool,
}

impl DvdThread {
    pub fn create(path: &str) -> Self {
        Self {
            path: path.to_string(),
            mount: DEFAULT_MOUNT_DIRECTION,
            bytes: Vec::new(),
            ready: false,
        }
    }

    pub fn sync(&self) -> bool {
        self.ready
    }

    pub fn finish(&mut self, bytes: Vec<u8>) {
        self.bytes = bytes;
        self.ready = true;
    }

    pub fn mem_address(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlaceNameEnv {
    pub menu_flag: bool,
    pub heap_lock: u8,
    pub mesg_status: u8,
    pub now_stage: u32,
    pub res_load: Phase,
}

#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct PlaceNameProcess {
    pub card: PlaceName,
    pub dvd: Option<DvdThread>,
    pub state: u8,
    pub res_load: Phase,
    pub heap: bool,
    pub heap_lock: Option<u8>,
    pub timg: Vec<u8>,
    pub stage_name: u8,
    pub listed: bool,
    pub deleted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcSlot {
    Create = 0,
    Delete = 1,
    Execute = 2,
    IsDelete = 3,
    Draw = 4,
}

pub const PROC_METHODS: [ProcSlot; 5] = [
    ProcSlot::Create,
    ProcSlot::Delete,
    ProcSlot::Execute,
    ProcSlot::IsDelete,
    ProcSlot::Draw,
];

pub const PLACE_NAME_PROFILE_SIZE: usize = 0x120;
pub const PLACE_NAME_PROFILE_NAME: u16 = 0x01EE;
pub const PLACE_NAME_PROFILE_DRAW_PRIO: u16 = 0x01DD;
pub const PLACE_NAME_PROFILE_LIST: u32 = 12;

pub const WATER_MARK_PROFILE_SIZE: usize = 0x138;
pub const WATER_MARK_PROFILE_NAME: u16 = 0x01DA;
pub const WATER_MARK_PROFILE_DRAW_PRIO: u16 = 0x01CA;
pub const WATER_MARK_PROFILE_LIST: u32 = 9;

impl PlaceNameProcess {
    pub fn create(&mut self, env: &PlaceNameEnv, layout: &j2d::Screen) -> Phase {
        let rt = env.res_load;
        self.res_load = rt;

        if env.menu_flag
            || (env.heap_lock != 0 && env.heap_lock != PLACE_NAME_HEAP_LOCK)
            || env.mesg_status != 0
        {
            return Phase::Init;
        }

        if self.state == 0 {
            if rt == Phase::Complete {
                self.heap = true;
                self.heap_lock = Some(PLACE_NAME_HEAP_LOCK);
                self.card.set_screen(layout.clone());
                self.timg = vec![0; PLACE_NAME_TEX_BYTES];
                self.state = 1;
                self.dvd = None;

                if env.now_stage != 0 {
                    return Phase::Init;
                }
            } else {
                return rt;
            }
        } else if self.state == 1 {
            if env.now_stage >= PLACE_NAME_STAGE_MAX {
                return Phase::Init;
            }
            self.dvd = Some(DvdThread::create(name_texture(env.now_stage).unwrap_or("")));
            self.state = 2;
            return Phase::Init;
        } else if let Some(dvd) = self.dvd.as_ref() {
            if dvd.sync() {
                self.timg = dvd
                    .mem_address()
                    .iter()
                    .take(PLACE_NAME_TEX_BYTES)
                    .copied()
                    .collect();
                self.timg.resize(PLACE_NAME_TEX_BYTES, 0);
                let timg = self.timg.clone();
                self.card.change_texture(&timg);
                self.state = 3;
            }
            if self.state != 3 {
                return Phase::Init;
            }
        } else {
            return Phase::Init;
        }

        Phase::Complete
    }

    pub fn execute(&mut self, stage_name: u8) -> bool {
        if stage_name == 2 {
            self.card.open_anime();
        } else if stage_name == 1 && self.card.close_anime() {
            self.deleted = true;
        }
        true
    }

    pub fn draw(&mut self) -> bool {
        self.listed = true;
        true
    }

    pub fn delete(&mut self) -> bool {
        self.stage_name = 0;
        self.card = PlaceName::default();
        self.dvd = None;
        self.timg = Vec::new();
        self.heap = false;
        self.heap_lock = None;
        self.listed = false;
        self.res_load = Phase::Init;
        true
    }

    pub fn is_delete(&self) -> bool {
        true
    }
}

pub fn proc_create(card: &mut PlaceNameProcess, env: &PlaceNameEnv, layout: &j2d::Screen) -> Phase {
    card.create(env, layout)
}

pub fn proc_delete(card: &mut PlaceNameProcess) -> bool {
    card.delete()
}

pub fn proc_execute(card: &mut PlaceNameProcess, stage_name: u8) -> bool {
    card.execute(stage_name)
}

pub fn proc_is_delete(_card: &PlaceNameProcess) -> bool {
    true
}

pub fn proc_draw(card: &mut PlaceNameProcess) -> bool {
    card.draw()
}

pub const WATER_MARK_ARCHIVE: &str = "Always";
pub const WATER_MARK_MODEL_RES: u32 = 0x2F;
pub const WATER_MARK_BRK_RES: u32 = 0x4D;
pub const WATER_MARK_BTP_RES: u32 = 0x63;
pub const WATER_MARK_HEAP_SIZE: usize = 0x12A0;
pub const WATER_MARK_HEAP_ALIGN: usize = 0x20;
pub const WATER_MARK_MODEL_SIZE: u32 = 0x80000;
pub const WATER_MARK_MODEL_FLAGS: u32 = 0x11020022;
pub const WATER_MARK_CIRCLE_MAX: i16 = 10;
pub const WATER_MARK_FOOT_ID_MAX: i16 = 40;
pub const WATER_MARK_FOOT_SPAN: i16 = 20;
pub const WATER_MARK_PROBE_LIFT: f32 = 10.0;
pub const WATER_MARK_BASE_LIFT: f32 = 0.1;
pub const WATER_MARK_CLIP_SCALE: f32 = 50.0;
pub const WATER_MARK_CULL_RADIUS: f32 = 50.0;

pub const EMODE_NONE: u8 = 0;
pub const EMODE_LOOP: u8 = 1;
pub const EMODE_ONCE: u8 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Anm {
    pub frame: f32,
    pub rate: f32,
    pub start: f32,
    pub end: f32,
    pub loop_frame: f32,
    pub mode: u8,
    pub playing: bool,
    pub stopped: bool,
}

impl Anm {
    pub fn init(&mut self, frame_max: f32, play: bool, mode: u8, rate: f32) {
        self.frame = 0.0;
        self.rate = rate;
        self.start = 0.0;
        self.end = frame_max;
        self.loop_frame = 0.0;
        self.mode = mode;
        self.playing = play;
        self.stopped = false;
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.stopped = false;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn get_frame(&self) -> f32 {
        self.frame
    }

    pub fn set_frame(&mut self, frame: f32) {
        self.frame = frame;
    }

    pub fn get_rate(&self) -> f32 {
        self.rate
    }

    pub fn get_end_frame(&self) -> f32 {
        self.end
    }

    pub fn is_loop(&self) -> bool {
        self.mode == EMODE_LOOP
    }

    pub fn is_stop(&self) -> bool {
        self.stopped || self.rate == 0.0
    }

    pub fn update(&mut self) {
        if !self.playing {
            return;
        }
        if self.mode == EMODE_LOOP {
            let span = self.end - self.start + 1.0;
            if span > 0.0 {
                self.frame = (self.frame - self.start).rem_euclid(span) + self.start;
            }
        } else {
            self.frame += self.rate;
            if self.frame >= self.end {
                self.frame = self.end;
                self.stopped = true;
            }
        }
        if self.frame < self.start {
            self.frame = self.start;
        }
    }
}

pub trait Ground {
    fn ground_cross(&self, pos: [f32; 3]) -> f32;
    fn tri_normal(&self, pos: [f32; 3]) -> Option<[f32; 3]>;
    fn chk_move_bg(&self, pos: [f32; 3]) -> bool;
}

pub struct GroundProbe<'a> {
    pub collision: &'a Collision,
    pub move_bg: bool,
}

impl Ground for GroundProbe<'_> {
    fn ground_cross(&self, pos: [f32; 3]) -> f32 {
        self.collision
            .ground_at(pos[0], pos[2])
            .unwrap_or(-G_CM3D_F_INF)
    }

    fn tri_normal(&self, pos: [f32; 3]) -> Option<[f32; 3]> {
        let normal = self.collision.ground_normal(pos[0], pos[2])?;
        Some([normal.x, normal.y, normal.z])
    }

    fn chk_move_bg(&self, _pos: [f32; 3]) -> bool {
        self.move_bg
    }
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WaterMarkShared {
    pub circle_cnt: i16,
    pub player_foot_now_id: i16,
    pub ground_check: [f32; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct WaterMarkDraw {
    pub base: Mtx,
    pub scale: [f32; 3],
    pub brk_frame: f32,
    pub btp_frame: i32,
    pub cull_radius: f32,
    pub listed: bool,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct WaterMark {
    pub pos: [f32; 3],
    pub scale: [f32; 3],
    pub base_scale: [f32; 3],
    pub base: Mtx,
    pub param: u16,
    pub angle: i16,
    pub moving: i16,
    pub window_start: i16,
    pub window_end: i16,
    pub foot_id: i16,
    pub brk: Anm,
    pub btp: Anm,
    pub model: bool,
    pub heap: bool,
    pub deleted: bool,
}

impl Default for WaterMark {
    fn default() -> Self {
        Self {
            pos: [0.0, 0.0, 0.0],
            scale: [0.0, 0.0, 0.0],
            base_scale: [0.0, 0.0, 0.0],
            base: MTX_IDENTITY,
            param: 0,
            angle: 0,
            moving: 0,
            window_start: 0,
            window_end: 0,
            foot_id: 0,
            brk: Anm::default(),
            btp: Anm::default(),
            model: false,
            heap: false,
            deleted: false,
        }
    }
}

impl WaterMark {
    pub fn spawn(
        params: u32,
        pos: Vec3,
        scale: Vec3,
        shared: &mut WaterMarkShared,
        ground: &dyn Ground,
        res: WaterMarkRes,
    ) -> (Self, Phase) {
        let mut mark = Self {
            pos: [pos.x, pos.y, pos.z],
            scale: [scale.x, scale.y, scale.z],
            ..Self::default()
        };
        let phase = mark.create(params, shared, ground, res);
        (mark, phase)
    }

    pub fn create(
        &mut self,
        params: u32,
        shared: &mut WaterMarkShared,
        ground: &dyn Ground,
        res: WaterMarkRes,
    ) -> Phase {
        self.angle = (params >> 0x10) as i16;
        self.param = (params & 0xFFFF) as u16;

        if self.param != 0 && self.param != 1 && self.param != 2 {
            return Phase::Error;
        }

        if self.param == 1 {
            shared.circle_cnt += 1;
            if shared.circle_cnt > WATER_MARK_CIRCLE_MAX {
                return Phase::Error;
            }
        }

        self.heap = res.heap;

        if self.heap {
            self.model = res.model;
            self.brk.init(res.brk_frames, true, EMODE_NONE, 1.0);
            self.btp.init(res.btp_frames, false, EMODE_NONE, 1.0);
        } else {
            return Phase::Error;
        }

        if !self.model || !(res.brk_ok && res.btp_ok) {
            return Phase::Error;
        }

        self.base_scale = self.scale;

        if !self.set_matrix(shared, ground) {
            return Phase::Error;
        }

        if self.param == 2 {
            self.foot_id = shared.player_foot_now_id;
            shared.player_foot_now_id += 1;
            if shared.player_foot_now_id == WATER_MARK_FOOT_ID_MAX {
                shared.player_foot_now_id = 0;
            }

            self.window_start = self.foot_id + WATER_MARK_FOOT_SPAN;
            if self.window_start >= WATER_MARK_FOOT_ID_MAX {
                self.window_start -= WATER_MARK_FOOT_ID_MAX;
            }

            self.window_end = self.window_start + WATER_MARK_FOOT_SPAN;
            if self.window_end >= WATER_MARK_FOOT_ID_MAX {
                self.window_end -= WATER_MARK_FOOT_ID_MAX;
            }

            self.param = 0;
        } else {
            self.foot_id = -1;
        }

        Phase::Complete
    }

    pub fn set_matrix(&mut self, shared: &mut WaterMarkShared, ground: &dyn Ground) -> bool {
        let check_pos = [
            self.pos[0],
            self.pos[1] + WATER_MARK_PROBE_LIFT,
            self.pos[2],
        ];
        shared.ground_check = check_pos;

        self.pos[1] = ground.ground_cross(check_pos);
        if self.pos[1] != -G_CM3D_F_INF {
            let Some(normal) = ground.tri_normal(check_pos) else {
                return false;
            };
            let tri_angle = c_atan2s(normal[0], normal[2]).wrapping_sub(self.angle);
            let norm_xz_mag = (normal[0] * normal[0] + normal[2] * normal[2]).sqrt();

            let trans = mtx_trans(self.pos[0], self.pos[1] + WATER_MARK_BASE_LIFT, self.pos[2]);
            self.base = mtx_zxy_rot_m(
                trans,
                c_atan2s(norm_xz_mag * s_cos(tri_angle), normal[1]),
                self.angle,
                c_atan2s(norm_xz_mag * -s_sin(tri_angle), normal[1]),
            );

            self.moving = i16::from(ground.chk_move_bg(check_pos));
        } else {
            return false;
        }

        true
    }

    pub fn execute(&mut self, shared: &mut WaterMarkShared, ground: Option<&dyn Ground>) -> bool {
        if self.foot_id != -1 {
            let now = shared.player_foot_now_id;
            if self.window_start < self.window_end {
                if self.window_start <= now && self.window_end > now {
                    self.foot_id = -1;
                }
            } else if self.window_start <= now || self.window_end > now {
                self.foot_id = -1;
            }
        }

        if self.foot_id == -1 {
            self.brk.play();
        }

        if self.brk.is_stop() {
            self.deleted = true;
        } else if self.moving == 1 {
            if let Some(ground) = ground {
                if !self.set_matrix(shared, ground) {
                    self.deleted = true;
                }
            }
        }

        true
    }

    pub fn delete(&mut self, shared: &mut WaterMarkShared) -> bool {
        if self.heap {
            self.heap = false;
        }
        if i32::from(self.param) == 1 {
            shared.circle_cnt -= 1;
        }
        self.model = false;
        true
    }

    pub fn draw(&mut self, culled: bool) -> Option<WaterMarkDraw> {
        if culled {
            return None;
        }
        let draw = WaterMarkDraw {
            base: self.base,
            scale: self.scale,
            brk_frame: self.brk.get_frame(),
            btp_frame: i32::from(self.param),
            cull_radius: self.scale[0] * WATER_MARK_CLIP_SCALE,
            listed: true,
        };
        Some(draw)
    }

    pub fn cull_radius(&self) -> f32 {
        self.scale[0] * WATER_MARK_CLIP_SCALE
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WaterMarkRes {
    pub heap: bool,
    pub model: bool,
    pub brk_ok: bool,
    pub btp_ok: bool,
    pub brk_frames: f32,
    pub btp_frames: f32,
}

pub fn proc_water_mark_create(
    mark: &mut WaterMark,
    params: u32,
    shared: &mut WaterMarkShared,
    ground: &dyn Ground,
    res: WaterMarkRes,
) -> Phase {
    mark.create(params, shared, ground, res)
}

pub fn proc_water_mark_delete(mark: &mut WaterMark, shared: &mut WaterMarkShared) -> bool {
    mark.delete(shared)
}

pub fn proc_water_mark_execute(
    mark: &mut WaterMark,
    shared: &mut WaterMarkShared,
    ground: Option<&dyn Ground>,
) -> bool {
    mark.execute(shared, ground)
}

pub fn proc_water_mark_is_delete(_mark: &WaterMark) -> bool {
    true
}

pub fn proc_water_mark_draw(mark: &mut WaterMark, culled: bool) -> Option<WaterMarkDraw> {
    mark.draw(culled)
}

pub const LETTER_NO_SEND: u8 = 0;
pub const LETTER_SEND: u8 = 1;
pub const LETTER_STOCK: u8 = 2;
pub const LETTER_READ: u8 = 3;

pub const LETTER_ROCK_SPIRE_SHOP_AD: u16 = 0x7A03;
pub const LETTER_ORCA: u16 = 0x7B03;
pub const LETTER_BAITO: u16 = 0x7C03;
pub const LETTER_BOMBS_AD: u16 = 0x7D03;
pub const LETTER_ARYLL: u16 = 0x8B03;
pub const LETTER_GRANDMA: u16 = 0x9D03;
pub const LETTER_BAITOS_MOM: u16 = 0xAC03;
pub const LETTER_HOSKITS_GIRLFRIEND: u16 = 0xAE03;
pub const LETTER_GOLD_MEMBERSHIP: u16 = 0xAF03;
pub const LETTER_SILVER_MEMBERSHIP: u16 = 0xB003;
pub const LETTER_TINGLE: u16 = 0xB203;
pub const LETTER_KOMALIS_FATHER: u16 = 0xB503;

pub fn letter_is_no_send(stories: &StoryBits, idx: u16) -> bool {
    stories.reg(idx) == LETTER_NO_SEND
}

pub fn letter_send(stories: &mut StoryBits, idx: u16) {
    stories.set_reg(idx, LETTER_SEND);
}

pub fn letter_is_send(stories: &StoryBits, idx: u16) -> bool {
    stories.reg(idx) == LETTER_SEND
}

pub fn letter_stock(stories: &mut StoryBits, idx: u16) {
    stories.set_reg(idx, LETTER_STOCK);
}

pub fn letter_is_stock(stories: &StoryBits, idx: u16) -> bool {
    stories.reg(idx) == LETTER_STOCK
}

pub fn letter_read(stories: &mut StoryBits, idx: u16) {
    stories.set_reg(idx, LETTER_READ);
}

pub fn letter_is_read(stories: &StoryBits, idx: u16) -> bool {
    stories.reg(idx) == LETTER_READ
}

pub fn letter_delivery(stories: &mut StoryBits, idx: u16) {
    if letter_is_send(stories, idx) {
        letter_stock(stories, idx);
    }
}

pub fn letter_auto_stock(stories: &mut StoryBits, idx: u16) {
    if letter_is_no_send(stories, idx) {
        letter_stock(stories, idx);
    }
}

pub fn letter_is_delivery(stories: &StoryBits, idx: u16) -> bool {
    !letter_is_no_send(stories, idx)
}

pub fn water_mark_control(
    mut marks: Query<&mut WaterMark>,
    mut shared: ResMut<WaterMarkShared>,
    collision: Option<Res<Collision>>,
) {
    let probe = collision.as_deref().map(|collision| GroundProbe {
        collision,
        move_bg: false,
    });
    for mut mark in &mut marks {
        mark.brk.update();
        mark.btp.update();
        let ground = probe.as_ref().map(|probe| probe as &dyn Ground);
        mark.execute(&mut shared, ground);
    }
}

pub fn init(world: &mut World) {
    world.init_resource::<WaterMarkShared>();
    world.init_resource::<PlaceNameProcess>();
}
