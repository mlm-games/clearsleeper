use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::event_manager::EventManager;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::lines;
use super::player::Player;
use super::sea::Sea;
use super::vibration::Vibration;
use super::water::Water;
use super::wlvtag::WaterLevel;

pub const OBJ_HHA_ACTORS: [&str; 1] = ["Hha"];

pub fn is_obj_hha(name: &str) -> bool {
    OBJ_HHA_ACTORS.contains(&name)
}

pub const ARC: &str = "Hha";

pub const EVENT_HHA_CLOSE: &str = "hha_close";
pub const EVENT_INFO_NONE: u8 = 0xFF;
pub const EVENT_IDX_NONE: i32 = -1;

pub const STATE_OPEN: u8 = 0;
pub const STATE_SWITCHING: u8 = 1;
pub const STATE_CLOSED: u8 = 2;

pub const WTR_STATE_IDLE: u8 = 0;
pub const WTR_STATE_CLOSING: u8 = 1;
pub const WTR_STATE_OPENING: u8 = 2;

pub const PART_COUNT: usize = 2;
pub const SPLASH_COUNT: usize = 2;

pub const BYTE_MASK: u32 = 0xFF;
pub const MIDDLE_SHIFT: u32 = 8;
pub const MIDDLE_FORCE_LOW: u8 = 0x0F;
pub const MIDDLE_MASK: u8 = 0x01;
pub const MIDDLE_NONE: u8 = 0;

pub const POS_Y_TABLE: [f32; 4] = [150.0, -150.0, 0.0, 0.0];
pub const TAR_Y_TABLE: [f32; 4] = [0.0, 0.0, 150.0, -150.0];
pub const MOVE_FRAME_TABLE: [u16; 2] = [55, 30];
pub const SPLASH_Y_TABLE: [f32; 2] = [-1400.0, -1450.0];
pub const SPLASH_Z_TABLE: [f32; 2] = [0.0, 100.0];

pub const POS_OFFSET: Vec3 = Vec3::new(0.0, -1400.0, 50.0);
pub const WTR_SCALE_INIT: f32 = 1.0;
pub const WTR_SCALE_STEP: f32 = 11.0 / 1800.0;
pub const WTR_SCALE_MIN_UNSET: f32 = 0.0;

pub const PART_MIDDLE_NONE: u8 = 0;

pub const YGUSH_Z_SCALE: f32 = 600.0;
pub const YGUSH_SCALE_MTX: Vec3 = Vec3::new(13.0, 1.0, 11.0);

pub const CO_SET: u32 = 0x01;
pub const CO_IS_OTHER: u32 = 0x08;
pub const CO_VS_ENEMY: u32 = 0x10;
pub const CO_VS_OTHER: u32 = 0x40;
pub const CYL_CO_SPRM: u32 = CO_SET | CO_IS_OTHER | CO_VS_ENEMY | CO_VS_OTHER;
pub const SPH_CO_SPRM: u32 = CO_SET | CO_IS_OTHER | CO_VS_ENEMY;
pub const TG_SPRM_NO_HIT_MARK: u32 = 1 << 2;
pub const CYL_RADIUS: f32 = 570.0;
pub const CYL_HEIGHT: f32 = 100.0;
pub const SPH_RADIUS: f32 = 220.0;
pub const STTS_WEIGHT: u8 = 0xFF;
pub const STTS_PARAM: i32 = 0xFF;

pub const CO_Y_OFFSET: f32 = -400.0;
pub const CO_Z_OFFSET: f32 = -110.0;
pub const CO_FORWARD: f32 = 460.0;

pub const WTR_CHK_FORWARD: f32 = 400.0;
pub const WTR_CHK_SCALE: f32 = 1.0;

pub const GET_R_WATER_SCALE: f32 = -190.0;

pub const STOP_R_SPAN: f32 = 75.0;
pub const STOP_R_TIMER_BASE: f32 = 15.0;
pub const STOP_R_SCALE: f32 = -300.0;

pub const CLOSE_TEX_FRAME: f32 = 37.0;
pub const OPEN_TEX_FRAME: f32 = 36.0;
pub const REST_TEX_FRAME: f32 = 0.0;
pub const CLOSE_TEX_SPEED: f32 = 1.0;
pub const OPEN_TEX_SPEED: f32 = 0.0;

pub const CLOSE_TEX_TIMER: u16 = 34;
pub const SPLASH_STOP_TIMER: u16 = 15;
pub const OPEN_SPLASH_TIMER: u16 = 35;
pub const OPEN_SPLASH_SCALE: f32 = 5.0;
pub const OPEN_SPLASH_BASE: f32 = 10.0;

pub const CLOSE_WTR_TIMER: u16 = 90;
pub const CLOSE_PART_TIMER: i16 = 65;
pub const OPEN_WTR_TIMER: u16 = 35;
pub const OPEN_PART_TIMER: i16 = 0;
pub const PART_TIMER_IDLE: i16 = -1;

pub const SHOCK_PATTERN: i32 = 4;
pub const SHOCK_FLAGS: i32 = -0x21;
pub const SHOCK_COORD: Vec3 = Vec3::new(0.0, 1.0, 0.0);

pub const SE_OBJ_F_WATERGATE_CL: u32 = 0x6A12;
pub const SE_ATM_WATER_GATE: u32 = 0x701F;
pub const SE_READ_RIDDLE_1: u32 = 0x0806;

pub const PARTICLE_SIRENSUIRYU00: u32 = 0x810D;
pub const PARTICLE_SIRENSUIRYU01: u32 = 0x810E;
pub const SPLASH_PARTICLE_TABLE: [u32; 2] = [
    PARTICLE_SIRENSUIRYU00,
    PARTICLE_SIRENSUIRYU01,
];
pub const PARTICLE_LIFE: u8 = 255;

pub const CULL_MIN: [f32; 3] = [-700.0, -1600.0, -400.0];
pub const CULL_MAX: [f32; 3] = [700.0, 120.0, 1000.0];

pub const BTL_MODE_LOOP: u32 = 0;
pub const BTL_MODE_NONE: u32 = 1;
pub const BTK_MODE_TABLE: [u32; 2] = [BTL_MODE_LOOP, BTL_MODE_NONE];
pub const BTL_FRAME_STEP: f32 = 1.0;
pub const YGUSH_FRAME_STEP: f32 = 1.0;
pub const BTL_MAX_FRAME_TABLE: [f32; 2] = [672.0, 672.0];
pub const YGUSH_BTK_FRAME_MAX: f32 = 800.0;
pub const YGUSH_BCK_FRAME_MAX: f32 = 512.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BgwVolume {
    pub entity: Entity,
    pub slot: usize,
    pub used: bool,
    pub pos: Vec3,
    pub moved: bool,
}

#[derive(Resource, Debug, Default)]
pub struct BgwSets {
    pub live: Vec<BgwVolume>,
}

impl BgwSets {
    pub fn retain(&mut self, live: &[Entity]) {
        self.live.retain(|volume| live.contains(&volume.entity));
    }

    pub fn slot_of(&self, entity: Entity, slot: usize) -> Option<usize> {
        self.live
            .iter()
            .position(|volume| volume.entity == entity && volume.slot == slot)
    }

    pub fn regist(&mut self, entity: Entity, slot: usize, pos: Vec3) {
        match self.slot_of(entity, slot) {
            Some(index) => {
                let volume = &mut self.live[index];
                volume.used = true;
                volume.pos = pos;
                volume.moved = false;
            }
            None => self.live.push(BgwVolume {
                entity,
                slot,
                used: true,
                pos,
                moved: false,
            }),
        }
    }

    pub fn release(&mut self, entity: Entity, slot: usize) {
        if let Some(index) = self.slot_of(entity, slot) {
            self.live[index].used = false;
        }
    }

    pub fn move_volume(&mut self, entity: Entity, slot: usize, pos: Vec3) {
        if let Some(index) = self.slot_of(entity, slot) {
            let volume = &mut self.live[index];
            volume.pos = pos;
            volume.moved = true;
        }
    }

    pub fn chk_used(&self, entity: Entity, slot: usize) -> bool {
        match self.slot_of(entity, slot) {
            Some(index) => self.live[index].used,
            None => false,
        }
    }

    pub fn pos_of(&self, entity: Entity, slot: usize) -> Option<Vec3> {
        self.slot_of(entity, slot)
            .map(|index| self.live[index].pos)
    }

    pub fn contains(&self, point: Vec3) -> bool {
        self.live.iter().any(|volume| {
            volume.used
                && (point - volume.pos).length() <= BGWW_RADIUS
                && (point.y - volume.pos.y).abs() <= BGWW_HALF_HEIGHT
        })
    }
}

pub const BGWW_RADIUS: f32 = 570.0;
pub const BGWW_HALF_HEIGHT: f32 = 50.0;

#[derive(Resource, Debug, Default)]
pub struct SplashSets {
    pub live: Vec<SplashEmitter>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplashEmitter {
    pub entity: Entity,
    pub slot: usize,
    pub particle: u32,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub life: u8,
    pub active: bool,
}

impl SplashSets {
    pub fn retain(&mut self, live: &[Entity]) {
        self.live.retain(|entry| live.contains(&entry.entity));
    }

    pub fn slot_of(&self, entity: Entity, slot: usize) -> Option<usize> {
        self.live
            .iter()
            .position(|entry| entry.entity == entity && entry.slot == slot)
    }

    pub fn set(&mut self, entity: Entity, slot: usize, particle: u32, pos: Vec3, angle: [i16; 3]) {
        match self.slot_of(entity, slot) {
            Some(index) => {
                let entry = &mut self.live[index];
                entry.particle = particle;
                entry.pos = pos;
                entry.angle = angle;
                entry.life = PARTICLE_LIFE;
                entry.active = true;
            }
            None => self.live.push(SplashEmitter {
                entity,
                slot,
                particle,
                pos,
                angle,
                life: PARTICLE_LIFE,
                active: true,
            }),
        }
    }

    pub fn emitter_live(&self, entity: Entity, slot: usize) -> bool {
        match self.slot_of(entity, slot) {
            Some(index) => self.live[index].active,
            None => false,
        }
    }

    pub fn play(&mut self, entity: Entity, slot: usize) {
        if let Some(index) = self.slot_of(entity, slot) {
            self.live[index].active = true;
        }
    }

    pub fn stop(&mut self, entity: Entity, slot: usize) {
        if let Some(index) = self.slot_of(entity, slot) {
            self.live[index].active = false;
        }
    }

    pub fn remove(&mut self, entity: Entity, slot: usize) {
        if let Some(index) = self.slot_of(entity, slot) {
            self.live[index].active = false;
            self.live.remove(index);
        }
    }

    pub fn chk_stop(&self, entity: Entity, slot: usize) -> bool {
        !self.emitter_live(entity, slot)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HhaCyl {
    pub centre: Vec3,
    pub radius: f32,
    pub height: f32,
    pub co_sprm: u32,
    pub tg_sprm: u32,
    pub at_type: u32,
    pub weight: u8,
    pub param: i32,
    pub registered: bool,
    pub co_hit: bool,
    pub co_no_hit_mark: bool,
    pub co_at_type: u32,
}

impl Default for HhaCyl {
    fn default() -> Self {
        Self::SRC
    }
}

impl HhaCyl {
    const SRC: HhaCyl = HhaCyl {
        centre: Vec3::ZERO,
        radius: CYL_RADIUS,
        height: CYL_HEIGHT,
        co_sprm: CYL_CO_SPRM,
        tg_sprm: TG_SPRM_NO_HIT_MARK,
        at_type: 0,
        weight: STTS_WEIGHT,
        param: STTS_PARAM,
        registered: false,
        co_hit: false,
        co_no_hit_mark: false,
        co_at_type: 0,
    };

    pub fn set(&mut self) {
        *self = HhaCyl::SRC;
    }

    pub fn stts_init(&mut self) {
        self.weight = STTS_WEIGHT;
        self.param = STTS_PARAM;
    }

    pub fn set_c(&mut self, centre: Vec3) {
        self.centre = centre;
    }

    pub fn get_c(&self) -> Vec3 {
        self.centre
    }

    pub fn co_set(&self) -> bool {
        self.co_sprm & CO_SET != 0
    }

    pub fn co_is_other(&self) -> bool {
        self.co_sprm & CO_IS_OTHER != 0
    }

    pub fn co_vs_enemy(&self) -> bool {
        self.co_sprm & CO_VS_ENEMY != 0
    }

    pub fn co_vs_other(&self) -> bool {
        self.co_sprm & CO_VS_OTHER != 0
    }

    pub fn tg_no_hit_mark(&self) -> bool {
        self.tg_sprm & TG_SPRM_NO_HIT_MARK != 0
    }

    pub fn register(&mut self) {
        self.registered = true;
    }

    pub fn unregister(&mut self) {
        self.registered = false;
    }

    pub fn covers(&self, point: Vec3) -> bool {
        let dx = point.x - self.centre.x;
        let dz = point.z - self.centre.z;
        if dx * dx + dz * dz > self.radius * self.radius {
            return false;
        }
        (point.y - self.centre.y).abs() <= self.height * 0.5
    }

    pub fn accepts_other(&self) -> bool {
        self.co_set() && (self.co_is_other() || self.co_vs_other())
    }

    pub fn probe_co(&mut self, player: Option<Vec3>) {
        let accepts = self.accepts_other();
        self.co_hit = match player {
            Some(point) => accepts && self.covers(point),
            None => false,
        };
        self.co_no_hit_mark = self.tg_no_hit_mark();
        self.co_at_type = self.at_type;
    }

    pub fn push_out(&self, point: Vec3) -> Option<Vec3> {
        if !self.accepts_other() || !self.covers(point) {
            return None;
        }
        let dx = point.x - self.centre.x;
        let dz = point.z - self.centre.z;
        let flat = (dx * dx + dz * dz).sqrt();
        if flat >= self.radius {
            return None;
        }
        if flat <= f32::EPSILON {
            return Some(Vec3::new(self.radius, 0.0, 0.0));
        }
        let push = self.radius - flat;
        Some(Vec3::new(dx / flat * push, 0.0, dz / flat * push))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HhaSph {
    pub centre: Vec3,
    pub radius: f32,
    pub co_sprm: u32,
    pub tg_sprm: u32,
    pub at_type: u32,
    pub weight: u8,
    pub param: i32,
    pub registered: bool,
    pub co_hit: bool,
    pub co_no_hit_mark: bool,
    pub co_at_type: u32,
}

impl Default for HhaSph {
    fn default() -> Self {
        Self::SRC
    }
}

impl HhaSph {
    const SRC: HhaSph = HhaSph {
        centre: Vec3::ZERO,
        radius: SPH_RADIUS,
        co_sprm: SPH_CO_SPRM,
        tg_sprm: TG_SPRM_NO_HIT_MARK,
        at_type: 0,
        weight: STTS_WEIGHT,
        param: STTS_PARAM,
        registered: false,
        co_hit: false,
        co_no_hit_mark: false,
        co_at_type: 0,
    };

    pub fn set(&mut self) {
        *self = HhaSph::SRC;
    }

    pub fn stts_init(&mut self) {
        self.weight = STTS_WEIGHT;
        self.param = STTS_PARAM;
    }

    pub fn set_c(&mut self, centre: Vec3) {
        self.centre = centre;
    }

    pub fn set_r(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn co_set(&self) -> bool {
        self.co_sprm & CO_SET != 0
    }

    pub fn co_is_other(&self) -> bool {
        self.co_sprm & CO_IS_OTHER != 0
    }

    pub fn co_vs_enemy(&self) -> bool {
        self.co_sprm & CO_VS_ENEMY != 0
    }

    pub fn tg_no_hit_mark(&self) -> bool {
        self.tg_sprm & TG_SPRM_NO_HIT_MARK != 0
    }

    pub fn register(&mut self) {
        self.registered = true;
    }

    pub fn unregister(&mut self) {
        self.registered = false;
    }

    pub fn covers(&self, point: Vec3) -> bool {
        (point - self.centre).length() <= self.radius
    }

    pub fn accepts_other(&self) -> bool {
        self.co_set() && self.co_is_other()
    }

    pub fn probe_co(&mut self, player: Option<Vec3>) {
        let accepts = self.accepts_other();
        self.co_hit = match player {
            Some(point) => accepts && self.covers(point),
            None => false,
        };
        self.co_no_hit_mark = self.tg_no_hit_mark();
        self.co_at_type = self.at_type;
    }

    pub fn push_out(&self, point: Vec3) -> Option<Vec3> {
        if !self.accepts_other() {
            return None;
        }
        let delta = point - self.centre;
        let len = delta.length();
        if len >= self.radius || len <= f32::EPSILON {
            return None;
        }
        Some(delta * ((self.radius - len) / len))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartCb {
    Normal,
    MoveState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HhaPart {
    pub pos: Vec3,
    pub pos_target: Vec3,
    pub pos_delta_dir: Vec3,
    pub delta_y: f32,
    pub part_idx: u8,
    pub mid: u8,
    pub cb: PartCb,
    pub has_mdl: bool,
    pub has_bgw: bool,
    pub bgw_moved: bool,
}

impl Default for HhaPart {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            pos_target: Vec3::ZERO,
            pos_delta_dir: Vec3::ZERO,
            delta_y: 0.0,
            part_idx: 0,
            mid: PART_MIDDLE_NONE,
            cb: PartCb::Normal,
            has_mdl: false,
            has_bgw: false,
            bgw_moved: false,
        }
    }
}

impl HhaPart {
    pub fn init_data(&mut self, y_pos: f32, y_tar: f32, speed: u16, index: u8, is_middle: u8) {
        self.pos = Vec3::new(0.0, y_pos, 0.0);
        self.pos_target = Vec3::new(0.0, y_tar, 0.0);
        self.part_idx = index;
        self.delta_y = (y_tar - y_pos) / speed as f32;
        self.pos_delta_dir = self.pos_target - self.pos;
        self.pos_delta_dir = match self.pos_delta_dir.length() {
            0.0 => Vec3::ZERO,
            len => self.pos_delta_dir / len,
        };
        self.cb = PartCb::Normal;
        self.mid = is_middle;
    }

    pub fn start_move(&mut self) {
        self.cb = PartCb::MoveState;
    }

    pub fn set_exe_proc(&mut self, cb: PartCb) {
        self.cb = cb;
    }

    pub fn exe_normal(
        &mut self,
        entity: Entity,
        bgws: &mut BgwSets,
        at: Vec3,
        shape_angle: [i16; 3],
    ) {
        let _ = self.model_translation(at, shape_angle);
        self.bgw_step(entity, bgws, at, shape_angle);
    }

    pub fn exe_move(
        &mut self,
        entity: Entity,
        bgws: &mut BgwSets,
        at: Vec3,
        shape_angle: [i16; 3],
        vibration: &mut Vibration,
    ) {
        self.pos.y += self.delta_y;
        let position_delta = self.pos_target - self.pos;
        if self.pos_delta_dir.dot(position_delta) <= 0.0 {
            self.pos = self.pos_target;
            self.set_exe_proc(PartCb::Normal);
            if self.mid == PART_MIDDLE_NONE && self.part_idx == 0 {
                vibration.start_shock(SHOCK_PATTERN, SHOCK_FLAGS, SHOCK_COORD);
            }
        }
        self.exe_normal(entity, bgws, at, shape_angle);
    }

    pub fn bgw_step(&mut self, entity: Entity, bgws: &mut BgwSets, at: Vec3, shape_angle: [i16; 3]) {
        if !self.has_bgw || !self.has_mdl {
            return;
        }
        if bgws.chk_used(entity, self.part_idx as usize) {
            bgws.move_volume(
                entity,
                self.part_idx as usize,
                self.model_translation(at, shape_angle),
            );
            self.bgw_moved = true;
        }
    }

    pub fn model_translation(&self, pos: Vec3, shape_angle: [i16; 3]) -> Vec3 {
        pos + rotate_zxy(self.pos, shape_angle)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HhaSplash {
    pub base_pos: Vec3,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub is_active: bool,
}

impl Default for HhaSplash {
    fn default() -> Self {
        Self {
            base_pos: Vec3::ZERO,
            pos: Vec3::ZERO,
            angle: [0; 3],
            is_active: false,
        }
    }
}

impl HhaSplash {
    pub fn create_s(
        &mut self,
        entity: Entity,
        slot: usize,
        particle: u32,
        p_pos: Vec3,
        offset_y: f32,
        offset_z: f32,
        p_angle: [i16; 3],
        sets: &mut SplashSets,
    ) {
        self.pos = Vec3::new(p_pos.x, p_pos.y + offset_y, p_pos.z);
        let calc_vec = base_z_rotated(p_angle[1]) * offset_z;
        self.pos += calc_vec;
        self.base_pos = self.pos;
        self.angle = p_angle;
        sets.set(entity, slot, particle, self.pos, self.angle);
        self.is_active = true;
    }

    pub fn get_base_pos(&self) -> Vec3 {
        self.base_pos
    }

    pub fn get_pos(&self) -> Vec3 {
        self.pos
    }

    pub fn get_pos_y(&self) -> f32 {
        self.pos.y
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        self.pos = pos;
    }

    pub fn set_pos_y(&mut self, y: f32) {
        self.pos.y = y;
    }

    pub fn play_particle(&mut self, entity: Entity, slot: usize, sets: &mut SplashSets) {
        if sets.emitter_live(entity, slot) {
            sets.play(entity, slot);
            self.is_active = true;
        }
    }

    pub fn stop_particle(&mut self, entity: Entity, slot: usize, sets: &mut SplashSets) {
        if sets.emitter_live(entity, slot) {
            sets.stop(entity, slot);
            self.is_active = false;
        }
    }

    pub fn delete_s(&mut self, entity: Entity, slot: usize, sets: &mut SplashSets) {
        if sets.emitter_live(entity, slot) {
            sets.remove(entity, slot);
            self.is_active = false;
        }
    }

    pub fn chk_stop(&self) -> bool {
        !self.is_active
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HhaYgush {
    pub base_pos: Vec3,
    pub pos: Vec3,
    pub scale: Vec3,
    pub rot: [i16; 3],
    pub visible: bool,
    pub btk_frame: f32,
    pub bck_frame: f32,
}

impl Default for HhaYgush {
    fn default() -> Self {
        Self {
            base_pos: Vec3::ZERO,
            pos: Vec3::ZERO,
            scale: Vec3::ONE,
            rot: [0; 3],
            visible: false,
            btk_frame: 0.0,
            bck_frame: 0.0,
        }
    }
}

impl HhaYgush {
    pub fn init_data(
        &mut self,
        p_pos: Vec3,
        z_scale: f32,
        p_angle: [i16; 3],
        p_scale: Vec3,
        is_visible: u8,
    ) {
        let calc_vec = base_z_rotated(p_angle[1]) * z_scale;
        self.base_pos = p_pos + calc_vec;
        self.pos = self.base_pos;
        self.rot = p_angle;
        self.scale = p_scale;
        self.visible = is_visible != 0;
        self.btk_frame = 0.0;
        self.bck_frame = 0.0;
    }

    pub fn check_draw(&self) -> bool {
        self.visible
    }

    pub fn disp_off(&mut self) {
        self.visible = false;
    }

    pub fn disp_on(&mut self) {
        self.visible = true;
    }

    pub fn get_base_pos(&self) -> Vec3 {
        self.base_pos
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        self.pos = pos;
    }

    pub fn anm_play(&mut self, pos: Vec3) {
        self.set_pos(pos);
        self.btk_frame += YGUSH_FRAME_STEP;
        self.bck_frame += YGUSH_FRAME_STEP;
        self.wrap();
    }

    pub fn wrap(&mut self) {
        if self.btk_frame > YGUSH_BTK_FRAME_MAX {
            self.btk_frame = 0.0;
        }
        if self.bck_frame > YGUSH_BCK_FRAME_MAX {
            self.bck_frame = 0.0;
        }
    }

    pub fn model_translation(&self) -> Vec3 {
        self.pos
    }

    pub fn model_scale(&self) -> Vec3 {
        self.scale * YGUSH_SCALE_MTX
    }
}

pub fn base_z_rotated(angle_y: i16) -> Vec3 {
    let (sin_y, cos_y) = super::stream::sin_cos(angle_y);
    Vec3::new(sin_y, 0.0, cos_y)
}

pub fn rotate_zxy(local: Vec3, shape_angle: [i16; 3]) -> Vec3 {
    let mut point = local;
    let ax = shape_angle[0];
    let ay = shape_angle[1];
    let az = shape_angle[2];
    let (sin_x, cos_x) = super::stream::sin_cos(ax);
    point = Vec3::new(point.x, point.y * cos_x + point.z * sin_x, -point.y * sin_x + point.z * cos_x);
    let (sin_y, cos_y) = super::stream::sin_cos(ay);
    point = Vec3::new(
        point.x * cos_y + point.z * sin_y,
        point.y,
        -point.x * sin_y + point.z * cos_y,
    );
    let (sin_z, cos_z) = super::stream::sin_cos(az);
    Vec3::new(
        point.x * cos_z - point.y * sin_z,
        point.x * sin_z + point.y * cos_z,
        point.z,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BtlAnm {
    pub frame: f32,
    pub speed: f32,
    pub mode: u32,
    pub max_frame: f32,
}

impl Default for BtlAnm {
    fn default() -> Self {
        Self {
            frame: 0.0,
            speed: 0.0,
            mode: BTL_MODE_LOOP,
            max_frame: 0.0,
        }
    }
}

impl BtlAnm {
    pub fn init(&mut self, mode: u32, max_frame: f32) {
        self.frame = 0.0;
        self.speed = 0.0;
        self.mode = mode;
        self.max_frame = max_frame;
    }

    pub fn set_frame(&mut self, frame: f32) {
        self.frame = frame;
    }

    pub fn set_play_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn play(&mut self) {
        if self.mode == BTL_MODE_NONE {
            return;
        }
        if self.max_frame <= 0.0 {
            return;
        }
        self.frame += BTL_FRAME_STEP * self.speed;
        if self.frame > self.max_frame {
            self.frame = 0.0;
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ObjHha {
    pub switch_no: u32,
    pub middle: u8,
    pub part_timer: i16,
    pub wtr_scale: f32,
    pub wtr_scale_min: f32,
    pub wtr_timer: u16,
    pub wtr_state: u8,
    pub event_idx: i32,
    pub state: u8,
    pub water_sound: bool,
    pub hitbox_active: bool,
    pub entered: bool,
    pub sw_on_create: bool,
    pub demo_accept: bool,
    pub water_h: f32,
    pub pos_offset: Vec3,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub shape_angle: [i16; 3],
    pub scale: Vec3,
    pub parts: [HhaPart; PART_COUNT],
    pub splashes: [HhaSplash; SPLASH_COUNT],
    pub ygush: HhaYgush,
    pub btk: [BtlAnm; 2],
    pub cyl: HhaCyl,
    pub sph: HhaSph,
    pub health: i32,
    pub max_health: i32,
}

impl ObjHha {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            switch_no: parameters & BYTE_MASK,
            middle: ((parameters >> MIDDLE_SHIFT) & BYTE_MASK) as u8,
            part_timer: PART_TIMER_IDLE,
            wtr_scale: WTR_SCALE_INIT,
            wtr_scale_min: WTR_SCALE_MIN_UNSET,
            wtr_timer: 0,
            wtr_state: WTR_STATE_IDLE,
            event_idx: EVENT_IDX_NONE,
            state: STATE_OPEN,
            water_sound: false,
            hitbox_active: false,
            entered: false,
            sw_on_create: false,
            demo_accept: false,
            water_h: 0.0,
            pos_offset: POS_OFFSET,
            pos: Vec3::ZERO,
            angle: [0; 3],
            shape_angle: [0; 3],
            scale: Vec3::ONE,
            parts: [HhaPart::default(); PART_COUNT],
            splashes: [HhaSplash::default(); SPLASH_COUNT],
            ygush: HhaYgush::default(),
            btk: [BtlAnm::default(); 2],
            cyl: HhaCyl::SRC,
            sph: HhaSph::SRC,
            health: 0,
            max_health: 0,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn check_sw(&self, switches: &Switches) -> bool {
        switches.test(self.switch_no as u8)
    }

    pub fn set_tex(&mut self, frame: f32, speed: f32, idx: i32) {
        let slot = usize::try_from(idx).unwrap_or(0);
        if let Some(btk) = self.btk.get_mut(slot) {
            btk.set_frame(frame);
            btk.set_play_speed(speed);
        }
    }

    pub fn get_water_h(&self, pos: Vec3, angle: [i16; 3], sea: Option<&Sea>, water: Option<&Water>) -> f32 {
        let mut ret = pos.y;
        let mut chk_pos = pos;
        let off_vec = base_z_rotated(angle[1]) * WTR_CHK_FORWARD;
        chk_pos += off_vec;
        if WTR_CHK_SCALE != 0.0 {
            if let Some(height) = lines::water_height(sea, water, chk_pos.x, chk_pos.z) {
                ret = height;
            }
        }
        ret
    }

    pub fn get_r(&self, angle_y: i16, level: &WaterLevel) -> Vec3 {
        base_z_rotated(angle_y) * (level.now * GET_R_WATER_SCALE)
    }

    pub fn init_co(&mut self, water_h: f32) {
        let sw = self.sw_on_create;
        let mut centre = base_z_rotated(self.angle[1]) * CO_FORWARD;
        centre.y = water_h;
        centre += self.pos;

        self.cyl.stts_init();
        self.cyl.set();
        self.cyl.set_c(centre);

        let do_sph_init = !sw && self.middle == 0;
        if do_sph_init {
            self.sph.stts_init();
            self.sph.set();
            self.sph.set_c(Vec3::new(
                self.pos.x,
                self.pos.y + CO_Y_OFFSET,
                self.pos.z + CO_Z_OFFSET,
            ));
            self.sph.set_r(SPH_RADIUS);
        }
    }

    pub fn create(&mut self, sw: bool) {
        if self.middle == MIDDLE_FORCE_LOW {
            self.middle = MIDDLE_NONE;
        }
        self.sw_on_create = sw;
        if sw {
            self.middle = (self.middle + 1) & MIDDLE_MASK;
            self.state = STATE_CLOSED;
        }
        for index in 0..self.btk.len() {
            let mode = BTK_MODE_TABLE.get(index).copied().unwrap_or(BTL_MODE_LOOP);
            let max_frame = BTL_MAX_FRAME_TABLE.get(index).copied().unwrap_or(0.0);
            if let Some(btk) = self.btk.get_mut(index) {
                btk.init(mode, max_frame);
            }
        }
        self.part_timer = PART_TIMER_IDLE;

        let mut pos_idx = 2 * (self.middle & MIDDLE_MASK) as usize;
        for index in 0..PART_COUNT {
            let slot = index;
            let y_pos = POS_Y_TABLE.get(pos_idx).copied().unwrap_or(0.0);
            let y_tar = TAR_Y_TABLE.get(pos_idx).copied().unwrap_or(0.0);
            let speed = MOVE_FRAME_TABLE
                .get(self.middle as usize)
                .copied()
                .unwrap_or(MOVE_FRAME_TABLE[0]);
            self.parts[slot].init_data(y_pos, y_tar, speed, index as u8, self.middle);
            pos_idx += 1;
        }

        self.pos_offset = POS_OFFSET;
        self.wtr_scale = WTR_SCALE_INIT;
    }

    pub fn init_data_middle(
        &mut self,
        scale: Vec3,
        entity: Entity,
        sets: &mut SplashSets,
    ) {
        if self.middle == 0 {
            self.set_tex(OPEN_TEX_FRAME, OPEN_TEX_SPEED, 1);
            self.hitbox_active = true;
            self.ygush
                .init_data(self.pos, YGUSH_Z_SCALE, self.angle, scale, 1);
            self.water_sound = true;
        } else {
            self.set_tex(REST_TEX_FRAME, OPEN_TEX_SPEED, 1);
            self.splashes[0].stop_particle(entity, 0, sets);
            self.splashes[1].stop_particle(entity, 1, sets);
            self.ygush
                .init_data(self.pos, YGUSH_Z_SCALE, self.angle, scale, 0);
            self.water_sound = false;
        }
    }

    pub fn set_splash_bottom_h(&mut self, water_h: f32) {
        self.splashes[0].set_pos_y(water_h);
    }

    pub fn set_splash_bottom_r(&mut self, level: &WaterLevel) {
        let mut pos = self.get_r(self.angle[1], level);
        pos += self.splashes[0].get_base_pos();
        pos.y = self.splashes[0].get_pos_y();
        self.splashes[0].set_pos(pos);
    }

    pub fn set_splash_bottom_stop_r(&mut self) {
        let timer = i32::from(self.wtr_timer);
        let scaling = (1.0 - (timer as f32 - STOP_R_TIMER_BASE) / STOP_R_SPAN) * STOP_R_SCALE;
        let mut calc_vec = base_z_rotated(self.angle[1]) * scaling;
        calc_vec += self.splashes[0].get_pos();
        calc_vec.y = self.splashes[0].get_pos_y();
        self.splashes[0].set_pos(calc_vec);
    }

    pub fn water_manager(&mut self, entity: Entity, level: &WaterLevel, sets: &mut SplashSets) {
        self.set_splash_bottom_h(self.water_h);
        self.set_splash_bottom_r(level);
        match self.wtr_state {
            WTR_STATE_IDLE => {}
            WTR_STATE_CLOSING => {
                self.wtr_scale -= WTR_SCALE_STEP;
                if self.wtr_scale < self.wtr_scale_min {
                    self.wtr_scale = self.wtr_scale_min;
                }
                if self.wtr_timer == CLOSE_TEX_TIMER {
                    self.set_tex(CLOSE_TEX_FRAME, CLOSE_TEX_SPEED, 1);
                }
                if self.wtr_timer == SPLASH_STOP_TIMER {
                    self.splashes[1].stop_particle(entity, 1, sets);
                    self.splashes[1].delete_s(entity, 1, sets);
                } else if self.wtr_timer > SPLASH_STOP_TIMER {
                    self.set_splash_bottom_stop_r();
                }
                self.wtr_timer = self.wtr_timer.wrapping_sub(1);
                if self.wtr_timer == 0 {
                    self.wtr_state = WTR_STATE_IDLE;
                    self.splashes[0].stop_particle(entity, 0, sets);
                    self.splashes[0].delete_s(entity, 0, sets);
                    self.hitbox_active = false;
                    self.ygush.disp_off();
                    self.water_sound = false;
                }
            }
            WTR_STATE_OPENING => {
                if self.wtr_timer == OPEN_SPLASH_TIMER {
                    self.splashes[1].play_particle(entity, 1, sets);
                    self.set_tex(REST_TEX_FRAME, CLOSE_TEX_SPEED, 1);
                }
                self.wtr_timer = self.wtr_timer.wrapping_sub(1);
                if self.splashes[0].chk_stop()
                    && f32::from(self.wtr_timer) <= level.now * OPEN_SPLASH_SCALE + OPEN_SPLASH_BASE
                {
                    self.splashes[0].play_particle(entity, 0, sets);
                    self.ygush.disp_on();
                    self.water_sound = true;
                }
                if self.wtr_timer == 0 {
                    self.set_tex(OPEN_TEX_FRAME, OPEN_TEX_SPEED, 1);
                    self.wtr_state = WTR_STATE_IDLE;
                    self.hitbox_active = true;
                }
            }
            _ => {}
        }

        for index in 0..self.btk.len() {
            if let Some(btk) = self.btk.get_mut(index) {
                btk.play();
            }
        }
    }

    pub fn part_manager(
        &mut self,
        entity: Entity,
        bgws: &mut BgwSets,
        at: Vec3,
        shape_angle: [i16; 3],
        vibration: &mut Vibration,
        events: &mut SimEvents,
    ) {
        if self.part_timer == 0 {
            self.parts[0].start_move();
            self.parts[1].start_move();
            self.part_timer = PART_TIMER_IDLE;
            if self.middle == 0 {
                events.list.push(SimEvent::Se(SE_OBJ_F_WATERGATE_CL));
            }
        } else if self.part_timer > 0 {
            self.part_timer -= 1;
        }

        for index in 0..PART_COUNT {
            let part = &mut self.parts[index];
            match part.cb {
                PartCb::Normal => part.exe_normal(entity, bgws, at, shape_angle),
                PartCb::MoveState => part.exe_move(entity, bgws, at, shape_angle, vibration),
            }
        }
    }

    pub fn ygush_manager(&mut self, level: &WaterLevel, water_h: f32) {
        if self.ygush.check_draw() {
            let mut ygush_pos = self.get_r(self.angle[1], level);
            ygush_pos += self.ygush.get_base_pos();
            ygush_pos.y = water_h;
            self.ygush.anm_play(ygush_pos);
        }
    }

    pub fn model_translation(&self) -> Vec3 {
        self.pos + rotate_zxy(self.pos_offset, self.shape_angle)
    }

    pub fn model_scale(&self) -> Vec3 {
        Vec3::new(self.scale.x, self.scale.y, self.scale.z * self.wtr_scale)
    }
}

#[derive(SystemParam)]
pub struct World<'w, 's> {
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub manager: ResMut<'w, EventManager>,
    pub switches: Res<'w, Switches>,
    pub events: ResMut<'w, SimEvents>,
    pub vibration: ResMut<'w, Vibration>,
    pub level: Res<'w, WaterLevel>,
    pub sea: Option<Res<'w, Sea>>,
    pub water: Option<Res<'w, Water>>,
    pub bgws: ResMut<'w, BgwSets>,
    pub splashes: ResMut<'w, SplashSets>,
    pub players: Query<'w, 's, &'static mut Player>,
}

pub fn control(
    mut statues: Query<(Entity, &mut Actor, &mut ObjHha), (With<ObjHha>, Without<Player>)>,
    world: World<'_, '_>,
) {
    let World {
        list,
        mut active,
        mut manager,
        switches,
        mut events,
        mut vibration,
        level,
        sea,
        water,
        mut bgws,
        mut splashes,
        mut players,
    } = world;
    let sea = sea.as_deref();
    let water = water.as_deref();
    let live: Vec<Entity> = statues.iter().map(|(entity, _, _)| entity).collect();
    bgws.retain(&live);
    splashes.retain(&live);
    for (entity, actor, mut statue) in &mut statues {
        statue.pos = actor.pos;
        statue.angle = actor.angle;
        statue.shape_angle = actor.shape_angle;
        statue.scale = actor.scale;
        let sw = statue.check_sw(&switches);
        if !statue.entered {
            statue.entered = true;
            statue.event_idx = manager.get_event_idx(Some(EVENT_HHA_CLOSE), EVENT_INFO_NONE);
            statue.create(sw);
            for index in 0..PART_COUNT {
                let slot = index;
                statue.parts[slot].has_mdl = true;
                statue.parts[slot].has_bgw = true;
                let world_pos = statue.parts[slot]
                    .model_translation(actor.pos, actor.shape_angle);
                bgws.regist(entity, slot, world_pos);
            }
            for index in 0..SPLASH_COUNT {
                let slot = index;
                let particle = SPLASH_PARTICLE_TABLE
                    .get(slot)
                    .copied()
                    .unwrap_or(PARTICLE_SIRENSUIRYU00);
                let y = SPLASH_Y_TABLE.get(slot).copied().unwrap_or(0.0);
                let z = SPLASH_Z_TABLE.get(slot).copied().unwrap_or(0.0);
                statue.splashes[slot].create_s(
                    entity, slot, particle, actor.pos, y, z, actor.angle, &mut splashes,
                );
            }
            statue.init_data_middle(actor.scale, entity, &mut splashes);
            let water_h = statue.get_water_h(actor.pos, actor.angle, sea, water);
            statue.water_h = water_h;
            statue.init_co(water_h);
        }

        statue.water_h = statue.get_water_h(actor.pos, actor.angle, sea, water);

        match statue.state {
            STATE_OPEN => {
                if statue.check_sw(&switches) {
                    if statue.middle == 0 {
                        statue.demo_accept = false;
                        if !statue.demo_accept {
                            order_other(&mut active, &list, statue.event_idx, entity);
                        } else {
                            events.list.push(SimEvent::Se(SE_READ_RIDDLE_1));
                            statue.wtr_state = WTR_STATE_CLOSING;
                            statue.wtr_timer = CLOSE_WTR_TIMER;
                            statue.part_timer = CLOSE_PART_TIMER;
                            statue.state = STATE_SWITCHING;
                        }
                    } else if manager.start_check(statue.event_idx) {
                        statue.wtr_state = WTR_STATE_OPENING;
                        statue.wtr_timer = OPEN_WTR_TIMER;
                        statue.part_timer = OPEN_PART_TIMER;
                        statue.state = STATE_CLOSED;
                    }
                }
            }
            STATE_SWITCHING => {
                if manager.end_check(statue.event_idx) {
                    manager.exception.init();
                    statue.state = STATE_CLOSED;
                }
            }
            STATE_CLOSED => {}
            _ => {}
        }

        statue.part_manager(
            entity,
            &mut bgws,
            actor.pos,
            actor.shape_angle,
            &mut vibration,
            &mut events,
        );
        statue.water_manager(entity, &level, &mut splashes);
        let water_h = statue.water_h;
        statue.ygush_manager(&level, water_h);

        if statue.water_sound {
            events.list.push(SimEvent::Se(SE_ATM_WATER_GATE));
        }

        let mut player = players.iter_mut().next();

        if statue.hitbox_active {
            let mut calc_vec = statue.cyl.get_c();
            calc_vec.y = statue.water_h;
            statue.cyl.set_c(calc_vec);
            statue.cyl.register();
            let is_sph_active = !sw && statue.middle == 0;
            if is_sph_active {
                statue.sph.register();
            } else {
                statue.sph.unregister();
            }
        } else {
            statue.cyl.unregister();
            statue.sph.unregister();
        }

        if let Some(player) = player.as_deref_mut() {
            if statue.cyl.registered {
                statue.cyl.probe_co(Some(player.pos));
                if statue.cyl.co_hit {
                    if let Some(step) = statue.cyl.push_out(player.pos) {
                        player.pos += step;
                    }
                }
            } else {
                statue.cyl.probe_co(None);
            }
            if statue.sph.registered {
                statue.sph.probe_co(Some(player.pos));
                if statue.sph.co_hit {
                    if let Some(step) = statue.sph.push_out(player.pos) {
                        player.pos += step;
                    }
                }
            } else {
                statue.sph.probe_co(None);
            }
        } else {
            statue.cyl.probe_co(None);
            statue.sph.probe_co(None);
        }
    }
}

fn order_other(active: &mut ActiveEvent, list: &EventList, index: i32, entity: Entity) {
    if index == EVENT_IDX_NONE || index < 0 {
        return;
    }
    let slot = index as usize;
    active.order(list, slot, entity);
}