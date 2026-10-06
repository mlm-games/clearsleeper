use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::damage_fx::{DamageParticle, DamageParticles};

pub const OBJ_EFF_ACTOR: &str = "d_a_obj_eff";

pub const OBJ_EFF_ACTORS: [&str; 0] = [];

pub fn is_obj_eff(name: &str) -> bool {
    OBJ_EFF_ACTORS.contains(&name)
}

pub const PRM_TYPE_W: u32 = 8;
pub const PRM_TYPE_S: u32 = 0;
pub const PRM_TYPE_MASK: u32 = (1 << PRM_TYPE_W) - 1;

pub const TYPE_BARREL_SMOKE: i32 = 0;
pub const TYPE_STOOL_SMOKE: i32 = 1;
pub const TYPE_SKULL_SMOKE: i32 = 2;
pub const TYPE_LAND_SMOKE: i32 = 3;
pub const TYPE_PINECONE_SMOKE: i32 = 4;
pub const TYPE_WOODBOX_SMOKE: i32 = 5;
pub const TYPE_COUNT: i32 = 6;

pub const HEAP_SIZE: [u32; 6] = [0x40, 0x40, 0x40, 0x20, 0x40, 0x40];
pub const HEAP_SIZE_OUT_OF_RANGE: u32 = 0x40;

pub const CALLBACK_LIFE_INIT: i32 = 0x3C;
pub const CALLBACK_FADE_LIMIT: i32 = 0x32;
pub const CALLBACK_FADE_STEP: f32 = 3.6;
pub const CALLBACK_CHASE_TARGET: i16 = 1;
pub const CALLBACK_CHASE_STEP: i16 = 4;

pub const PARTICLE_ALPHA_ARG: u8 = 0xB4;
pub const PARTICLE_SETUP_INFO: i8 = -1;

pub const PA_ID_AK_JT_ELEMENTSMOKE01: u16 = 0x2027;
pub const PA_ID_IT_ST_BOKKURI_SMOKE00: u16 = 0xA16B;

pub const BARREL_ORIGINAL_COLOR: [u8; 4] = [0xAF, 0x96, 0x64, 0xFF];
pub const STOOL_ORIGINAL_COLOR: [u8; 4] = [0xAF, 0x96, 0x64, 0xFF];
pub const SKULL_ORIGINAL_COLOR: [u8; 4] = [0xA0, 0xA0, 0x80, 0xFF];
pub const PINECONE_ORIGINAL_COLOR: [u8; 4] = [0x96, 0x78, 0x3C, 0xFF];
pub const WOODBOX_ORIGINAL_COLOR: [u8; 4] = [0xAF, 0x96, 0x64, 0xFF];
pub const SMOKE_BASE_COLOR: [u8; 4] = [0xA0, 0xA0, 0x80, 0xFF];

pub const PARTICLE_SCL_BARREL: [f32; 3] = [2.5, 2.5, 1.0];
pub const PARTICLE_SCL_STOOL: [f32; 3] = [2.5, 2.5, 1.0];
pub const PARTICLE_SCL_SKULL: [f32; 3] = [1.6, 1.6, 1.0];
pub const PARTICLE_SCL_WOODBOX: [f32; 3] = [2.5, 2.5, 1.0];
pub const LAND_EM_SCL: [f32; 3] = [1.0, 0.0, 1.0];
pub const LAND_EM_TRANS: [f32; 3] = [0.0, 5.0, 0.0];

pub const BARREL_VOLUME_SIZE: i32 = 0x28;
pub const STOOL_VOLUME_SIZE: i32 = 0x1E;
pub const LAND_VOLUME_SIZE: i32 = 0x1E;
pub const LAND_LIFE_TIME: i32 = 0x23;

pub const BARREL_RATE: f32 = 20.0;
pub const STOOL_RATE: f32 = 15.0;
pub const SKULL_RATE: f32 = 10.0;
pub const LAND_RATE: f32 = 20.0;
pub const WOODBOX_RATE: f32 = 30.0;

pub const BARREL_CENTER_SPEED: f32 = 15.0;
pub const STOOL_CENTER_SPEED: f32 = 15.0;
pub const SKULL_CENTER_SPEED: f32 = 8.0;
pub const LAND_CENTER_SPEED: f32 = 10.0;

pub const BARREL_AXIS_SPEED: f32 = 10.0;
pub const STOOL_AXIS_SPEED: f32 = 10.0;

pub const EMITTER_MAX_FRAME: i32 = 1;

pub const LAND_SCALE_MUL: f32 = 5.0 / 3.0;

pub const CALLBACK_CLASS: [CallbackClass; 6] = [
    CallbackClass::BarrelSmoke,
    CallbackClass::StoolSmoke,
    CallbackClass::SkullSmoke,
    CallbackClass::LandSmoke,
    CallbackClass::PineconeSmoke,
    CallbackClass::WoodBoxSmoke,
];

pub const CALLBACK_ORIGINAL_COLOR: [[u8; 4]; 6] = [
    BARREL_ORIGINAL_COLOR,
    STOOL_ORIGINAL_COLOR,
    SKULL_ORIGINAL_COLOR,
    SMOKE_BASE_COLOR,
    PINECONE_ORIGINAL_COLOR,
    WOODBOX_ORIGINAL_COLOR,
];

pub const CALLBACK_RATE_OFF: [u8; 6] = [0, 0, 0, 0, 0, 0];

pub const CALLBACK_LIFE: [i32; 6] = [
    CALLBACK_LIFE_INIT,
    CALLBACK_LIFE_INIT,
    CALLBACK_LIFE_INIT,
    0,
    CALLBACK_LIFE_INIT,
    CALLBACK_LIFE_INIT,
];

pub const EFF_SET: [EffSet; 6] = [
    EffSet {
        particle_id: PA_ID_AK_JT_ELEMENTSMOKE01,
        alpha: PARTICLE_ALPHA_ARG,
        setup_info: PARTICLE_SETUP_INFO,
        scale: None,
        scale_2d: Some(PARTICLE_SCL_BARREL),
        volume_size: BARREL_VOLUME_SIZE,
        life_time: 0,
        rate: BARREL_RATE,
        max_frame: EMITTER_MAX_FRAME,
        away_from_center_speed: BARREL_CENTER_SPEED,
        away_from_axis_speed: BARREL_AXIS_SPEED,
        emitter_scale: None,
        emitter_translation: None,
    },
    EffSet {
        particle_id: PA_ID_AK_JT_ELEMENTSMOKE01,
        alpha: PARTICLE_ALPHA_ARG,
        setup_info: PARTICLE_SETUP_INFO,
        scale: None,
        scale_2d: Some(PARTICLE_SCL_STOOL),
        volume_size: STOOL_VOLUME_SIZE,
        life_time: 0,
        rate: STOOL_RATE,
        max_frame: EMITTER_MAX_FRAME,
        away_from_center_speed: STOOL_CENTER_SPEED,
        away_from_axis_speed: STOOL_AXIS_SPEED,
        emitter_scale: None,
        emitter_translation: None,
    },
    EffSet {
        particle_id: PA_ID_AK_JT_ELEMENTSMOKE01,
        alpha: PARTICLE_ALPHA_ARG,
        setup_info: PARTICLE_SETUP_INFO,
        scale: None,
        scale_2d: Some(PARTICLE_SCL_SKULL),
        volume_size: 0,
        life_time: 0,
        rate: SKULL_RATE,
        max_frame: EMITTER_MAX_FRAME,
        away_from_center_speed: SKULL_CENTER_SPEED,
        away_from_axis_speed: 0.0,
        emitter_scale: None,
        emitter_translation: None,
    },
    EffSet {
        particle_id: PA_ID_AK_JT_ELEMENTSMOKE01,
        alpha: PARTICLE_ALPHA_ARG,
        setup_info: PARTICLE_SETUP_INFO,
        scale: Some([0.0, 0.0, 0.0]),
        scale_2d: None,
        volume_size: LAND_VOLUME_SIZE,
        life_time: LAND_LIFE_TIME,
        rate: LAND_RATE,
        max_frame: EMITTER_MAX_FRAME,
        away_from_center_speed: LAND_CENTER_SPEED,
        away_from_axis_speed: 0.0,
        emitter_scale: Some(LAND_EM_SCL),
        emitter_translation: Some(LAND_EM_TRANS),
    },
    EffSet {
        particle_id: PA_ID_IT_ST_BOKKURI_SMOKE00,
        alpha: PARTICLE_ALPHA_ARG,
        setup_info: PARTICLE_SETUP_INFO,
        scale: None,
        scale_2d: None,
        volume_size: 0,
        life_time: 0,
        rate: 0.0,
        max_frame: 0,
        away_from_center_speed: 0.0,
        away_from_axis_speed: 0.0,
        emitter_scale: None,
        emitter_translation: None,
    },
    EffSet {
        particle_id: PA_ID_AK_JT_ELEMENTSMOKE01,
        alpha: PARTICLE_ALPHA_ARG,
        setup_info: PARTICLE_SETUP_INFO,
        scale: None,
        scale_2d: Some(PARTICLE_SCL_WOODBOX),
        volume_size: 0,
        life_time: 0,
        rate: WOODBOX_RATE,
        max_frame: EMITTER_MAX_FRAME,
        away_from_center_speed: 0.0,
        away_from_axis_speed: 0.0,
        emitter_scale: None,
        emitter_translation: None,
    },
];

pub fn prm_get_type(parameters: u32) -> i32 {
    ((parameters >> PRM_TYPE_S) & PRM_TYPE_MASK) as i32
}

pub fn make_prm(m_type: i32) -> u32 {
    ((m_type as u32) << PRM_TYPE_S) & PRM_TYPE_MASK
}

pub fn make_barrel_smoke() -> u32 {
    make_prm(TYPE_BARREL_SMOKE)
}

pub fn make_stool_smoke() -> u32 {
    make_prm(TYPE_STOOL_SMOKE)
}

pub fn make_skull_smoke() -> u32 {
    make_prm(TYPE_SKULL_SMOKE)
}

pub fn make_land_smoke() -> u32 {
    make_prm(TYPE_LAND_SMOKE)
}

pub fn make_pinecone_smoke() -> u32 {
    make_prm(TYPE_PINECONE_SMOKE)
}

pub fn make_woodBox_smoke() -> u32 {
    make_prm(TYPE_WOODBOX_SMOKE)
}

pub fn land_smoke_scale(i_scale: f32) -> [f32; 3] {
    [
        i_scale * LAND_SCALE_MUL,
        i_scale * LAND_SCALE_MUL,
        i_scale * LAND_SCALE_MUL,
    ]
}

pub fn eff_set_for_type(m_type: i32) -> Option<&'static EffSet> {
    if m_type < 0 {
        return None;
    }
    EFF_SET.get(m_type as usize)
}

pub fn heap_size_for_type(m_type: i32) -> u32 {
    if m_type < 0 {
        return HEAP_SIZE_OUT_OF_RANGE;
    }
    HEAP_SIZE
        .get(m_type as usize)
        .copied()
        .unwrap_or(HEAP_SIZE_OUT_OF_RANGE)
}

pub fn callback_for_type(m_type: i32) -> Option<CallbackClass> {
    if m_type < 0 || m_type >= TYPE_COUNT {
        return None;
    }
    Some(CALLBACK_CLASS[m_type as usize])
}

pub fn solid_heap_cb(eff: &mut ObjEff) -> bool {
    eff.create_heap()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackClass {
    BarrelSmoke,
    StoolSmoke,
    SkullSmoke,
    LandSmoke,
    PineconeSmoke,
    WoodBoxSmoke,
}

impl CallbackClass {
    pub fn index(self) -> usize {
        match self {
            Self::BarrelSmoke => 0,
            Self::StoolSmoke => 1,
            Self::SkullSmoke => 2,
            Self::LandSmoke => 3,
            Self::PineconeSmoke => 4,
            Self::WoodBoxSmoke => 5,
        }
    }

    pub fn original_color(self) -> [u8; 4] {
        CALLBACK_ORIGINAL_COLOR[self.index()]
    }

    pub fn rate_off(self) -> u8 {
        CALLBACK_RATE_OFF[self.index()]
    }

    pub fn life_init(self) -> i32 {
        CALLBACK_LIFE[self.index()]
    }

    pub fn overrides_execute(self) -> bool {
        match self {
            Self::BarrelSmoke => true,
            Self::StoolSmoke => true,
            Self::SkullSmoke => true,
            Self::LandSmoke => false,
            Self::PineconeSmoke => true,
            Self::WoodBoxSmoke => true,
        }
    }

    pub fn cb_execute(self, cb_life: &mut i32, global_alpha: &mut i16, ended: &mut bool) {
        if !self.overrides_execute() {
            return;
        }
        *cb_life -= 1;
        if *cb_life <= 0 {
            *ended = true;
        } else if *cb_life < CALLBACK_FADE_LIMIT {
            *global_alpha = (*cb_life as f32 * CALLBACK_FADE_STEP) as i16;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffSet {
    pub particle_id: u16,
    pub alpha: u8,
    pub setup_info: i8,
    pub scale: Option<[f32; 3]>,
    pub scale_2d: Option<[f32; 3]>,
    pub volume_size: i32,
    pub life_time: i32,
    pub rate: f32,
    pub max_frame: i32,
    pub away_from_center_speed: f32,
    pub away_from_axis_speed: f32,
    pub emitter_scale: Option<[f32; 3]>,
    pub emitter_translation: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Complete,
    Error,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ObjEff {
    pub m_type: i32,
    pub phase: Phase,
    pub scale: [f32; 3],
    pub callback: CallbackClass,
    pub callback_live: bool,
    pub cb_life: i32,
    pub cb_ended: bool,
    pub global_alpha: i16,
    pub off_draw: bool,
    pub emitter_live: bool,
    pub emitter_dead: bool,
    pub emitted: bool,
}

impl ObjEff {
    pub fn spawn(parameters: u32) -> Self {
        Self::spawn_scaled(parameters, [1.0, 1.0, 1.0])
    }

    pub fn spawn_land_smoke(scale: f32) -> Self {
        Self::spawn_scaled(make_land_smoke(), land_smoke_scale(scale))
    }

    pub fn spawn_scaled(parameters: u32, scale: [f32; 3]) -> Self {
        let mut eff = Self {
            m_type: prm_get_type(parameters),
            phase: Phase::Complete,
            scale,
            callback: CallbackClass::BarrelSmoke,
            callback_live: false,
            cb_life: CALLBACK_LIFE_INIT,
            cb_ended: false,
            global_alpha: PARTICLE_ALPHA_ARG as i16,
            off_draw: false,
            emitter_live: false,
            emitter_dead: false,
            emitted: false,
        };
        eff.create();
        eff
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0])
    }

    pub fn drawn(&self) -> bool {
        !self.off_draw
    }

    pub fn heap_size(&self) -> u32 {
        heap_size_for_type(self.m_type)
    }

    pub fn create(&mut self) -> Phase {
        self.phase = Phase::Complete;
        match callback_for_type(self.m_type) {
            Some(class) => self.callback = class,
            None => {
                self.phase = Phase::Error;
                return self.phase;
            }
        }
        if self.heap_size() == 0 {
            self.phase = Phase::Error;
            return self.phase;
        }
        if self.create_heap() {
            if self.eff_set() != 0 {
                self.off_draw = true;
            } else {
                self.phase = Phase::Error;
            }
        } else {
            self.phase = Phase::Error;
        }
        self.phase
    }

    pub fn create_heap(&mut self) -> bool {
        match self.callback {
            CallbackClass::BarrelSmoke => self.create_heap_barrel_smoke(),
            CallbackClass::StoolSmoke => self.create_heap_stool_smoke(),
            CallbackClass::SkullSmoke => self.create_heap_skull_smoke(),
            CallbackClass::LandSmoke => self.create_heap_land_smoke(),
            CallbackClass::PineconeSmoke => self.create_heap_pinecone_smoke(),
            CallbackClass::WoodBoxSmoke => self.create_heap_woodBox_smoke(),
        }
    }

    pub fn create_heap_barrel_smoke(&mut self) -> bool {
        self.new_callback(CallbackClass::BarrelSmoke)
    }

    pub fn create_heap_stool_smoke(&mut self) -> bool {
        self.new_callback(CallbackClass::StoolSmoke)
    }

    pub fn create_heap_skull_smoke(&mut self) -> bool {
        self.new_callback(CallbackClass::SkullSmoke)
    }

    pub fn create_heap_land_smoke(&mut self) -> bool {
        self.new_callback(CallbackClass::LandSmoke)
    }

    pub fn create_heap_pinecone_smoke(&mut self) -> bool {
        self.new_callback(CallbackClass::PineconeSmoke)
    }

    pub fn create_heap_woodBox_smoke(&mut self) -> bool {
        self.new_callback(CallbackClass::WoodBoxSmoke)
    }

    fn new_callback(&mut self, class: CallbackClass) -> bool {
        self.callback = class;
        self.cb_life = class.life_init();
        self.callback_live = true;
        self.callback_live
    }

    pub fn eff_set(&mut self) -> u8 {
        match self.callback {
            CallbackClass::BarrelSmoke => self.eff_set_barrel_smoke(),
            CallbackClass::StoolSmoke => self.eff_set_stool_smoke(),
            CallbackClass::SkullSmoke => self.eff_set_skull_smoke(),
            CallbackClass::LandSmoke => self.eff_set_land_smoke(),
            CallbackClass::PineconeSmoke => self.eff_set_pinecone_smoke(),
            CallbackClass::WoodBoxSmoke => self.eff_set_woodBox_smoke(),
        }
    }

    pub fn eff_set_barrel_smoke(&mut self) -> u8 {
        self.eff_set_emit(TYPE_BARREL_SMOKE)
    }

    pub fn eff_set_stool_smoke(&mut self) -> u8 {
        self.eff_set_emit(TYPE_STOOL_SMOKE)
    }

    pub fn eff_set_skull_smoke(&mut self) -> u8 {
        self.eff_set_emit(TYPE_SKULL_SMOKE)
    }

    pub fn eff_set_land_smoke(&mut self) -> u8 {
        self.eff_set_emit(TYPE_LAND_SMOKE)
    }

    pub fn eff_set_pinecone_smoke(&mut self) -> u8 {
        self.eff_set_emit(TYPE_PINECONE_SMOKE)
    }

    pub fn eff_set_woodBox_smoke(&mut self) -> u8 {
        self.eff_set_emit(TYPE_WOODBOX_SMOKE)
    }

    fn eff_set_emit(&mut self, m_type: i32) -> u8 {
        self.emitter_live = eff_set_for_type(m_type).is_some();
        u8::from(self.emitter_live)
    }

    pub fn emitter(&self) -> EffSet {
        let mut set = eff_set_for_type(self.m_type).copied().unwrap_or(EFF_SET[0]);
        if self.m_type == TYPE_LAND_SMOKE {
            set.scale = Some(self.scale);
        }
        set
    }

    pub fn particle(&self, pos: Vec3) -> DamageParticle {
        let set = self.emitter();
        DamageParticle {
            id: set.particle_id,
            pos,
            angle: None,
            scale: set
                .scale
                .or(set.scale_2d)
                .map(Vec3::from_array),
        }
    }

    pub fn remove(&mut self) {
        match self.callback {
            CallbackClass::BarrelSmoke => self.remove_barrel_smoke(),
            CallbackClass::StoolSmoke => self.remove_stool_smoke(),
            CallbackClass::SkullSmoke => self.remove_skull_smoke(),
            CallbackClass::LandSmoke => self.remove_land_smoke(),
            CallbackClass::PineconeSmoke => self.remove_pinecone_smoke(),
            CallbackClass::WoodBoxSmoke => self.remove_woodBox_smoke(),
        }
    }

    pub fn remove_barrel_smoke(&mut self) {
        self.remove_common();
    }

    pub fn remove_stool_smoke(&mut self) {
        self.remove_common();
    }

    pub fn remove_skull_smoke(&mut self) {
        self.remove_common();
    }

    pub fn remove_land_smoke(&mut self) {
        self.remove_common();
    }

    pub fn remove_pinecone_smoke(&mut self) {
        self.remove_common();
    }

    pub fn remove_woodBox_smoke(&mut self) {
        self.remove_common();
    }

    fn remove_common(&mut self) {
        if self.callback_live {
            self.end();
        }
    }

    pub fn delete(&mut self) -> bool {
        self.remove();
        true
    }

    pub fn die(&mut self) -> bool {
        match self.callback {
            CallbackClass::BarrelSmoke => self.die_barrel_smoke(),
            CallbackClass::StoolSmoke => self.die_stool_smoke(),
            CallbackClass::SkullSmoke => self.die_skull_smoke(),
            CallbackClass::LandSmoke => self.die_land_smoke(),
            CallbackClass::PineconeSmoke => self.die_pinecone_smoke(),
            CallbackClass::WoodBoxSmoke => self.die_woodBox_smoke(),
        }
    }

    pub fn die_barrel_smoke(&mut self) -> bool {
        self.die_common()
    }

    pub fn die_stool_smoke(&mut self) -> bool {
        self.die_common()
    }

    pub fn die_skull_smoke(&mut self) -> bool {
        self.die_common()
    }

    pub fn die_land_smoke(&mut self) -> bool {
        self.die_common()
    }

    pub fn die_pinecone_smoke(&mut self) -> bool {
        self.die_common()
    }

    pub fn die_woodBox_smoke(&mut self) -> bool {
        self.die_common()
    }

    fn die_common(&self) -> bool {
        self.cb_ended
    }

    pub fn execute(&mut self) -> bool {
        self.die()
    }

    pub fn end(&mut self) {
        if !self.callback_live {
            return;
        }
        self.cb_ended = true;
        self.emitter_live = false;
    }

    pub fn is_end(&self) -> bool {
        self.cb_ended
    }

    pub fn set_emitter_dead(&mut self, dead: bool) {
        self.emitter_dead = dead;
    }

    pub fn cb_execute(&mut self) {
        if !self.callback_live {
            return;
        }
        if self.callback.overrides_execute() {
            self.callback.cb_execute(
                &mut self.cb_life,
                &mut self.global_alpha,
                &mut self.cb_ended,
            );
            return;
        }
        if self.emitter_dead && self.emitter_live {
            self.end();
        }
        if self.cb_ended {
            chase_s(
                &mut self.global_alpha,
                CALLBACK_CHASE_TARGET,
                CALLBACK_CHASE_STEP,
            );
        }
    }
}

fn chase_s(value: &mut i16, target: i16, step: i16) -> bool {
    if *value < target {
        *value += step;
        if *value > target {
            *value = target;
        }
        true
    } else if *value > target {
        *value -= step;
        if *value < target {
            *value = target;
        }
        true
    } else {
        false
    }
}

pub fn control(
    mut commands: Commands,
    mut particles: ResMut<DamageParticles>,
    mut effects: Query<(Entity, &Actor, &mut ObjEff)>,
) {
    for (entity, actor, mut eff) in &mut effects {
        if eff.phase == Phase::Error {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if !eff.emitted {
            eff.emitted = true;
            particles.items.push(eff.particle(actor.pos));
        }
        eff.cb_execute();
        if eff.execute() {
            let _ = commands.entity(entity).despawn();
        }
    }
}
