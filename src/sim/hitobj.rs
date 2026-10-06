use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::contact::{AT_SPRM_SET, GObjInf, GStts, ObjSet, TG_SPRM_SET};
use super::contact_run::{ContactBodies, ContactObj};
use super::shape::{Shape, ShapeKind};

pub const HITOBJ_ACTOR: &str = "Hitobj";

pub const HITOBJ_ACTORS: [&str; 0] = [];

pub fn is_hitobj(name: &str) -> bool {
    HITOBJ_ACTORS.contains(&name)
}

pub const RES: &str = "Hitobj";
pub const PARAM_MASK: u32 = 0xFF;
pub const PROC_NAME: u32 = 0x00B8;
pub const LIST_ID: u32 = 0x0007;
pub const LAYER_ID: u32 = 0xFFFF_FFFD;
pub const LIST_PRIO: u32 = 0x0000_FFFD;
pub const DRAW_PRIORITY: u32 = 0x0000_00AB;
pub const STATUS: u32 = 0x0004_0000;
pub const CULL_TYPE: u32 = 0;
pub const GROUP: u32 = 0;
pub const SIZE_OTHER: u32 = 0;
pub const PARAMETERS: u32 = 0;
pub const CLASS_SIZE: u32 = 0x0404;
pub const SRC_GOBJ_INF_SIZE: u32 = 0x30;
pub const SRC_SPH_SIZE: u32 = 0x40;
pub const MSTTS_OFFSET: u32 = 0x029C;
pub const MSPH_OFFSET: u32 = 0x02D8;
pub const MSTTS_SIZE: u32 = 0x3C;
pub const MSPH_SIZE: u32 = 0x12C;
pub const MUNUSED_PARAM_OFFSET: u32 = 0x0298;
pub const MTIMER_OFFSET: u32 = 0x029A;

pub const AT_SPRM_VS_ENEMY: u32 = 0x02;
pub const AT_SPRM_VS_OTHER: u32 = 0x08;
pub const TG_SPRM_IS_ENEMY: u32 = 0x02;
pub const AT_TYPE_WATER: u32 = 1 << 8;

pub const AT_SPRM_INIT: u32 = AT_SPRM_SET | AT_SPRM_VS_ENEMY | AT_SPRM_VS_OTHER;
pub const TG_SPRM_INIT: u32 = TG_SPRM_SET | TG_SPRM_IS_ENEMY;
pub const CO_SPRM_INIT: u32 = 0;

pub const SPH_CENTRE: Vec3 = Vec3::ZERO;
pub const SPH_RADIUS: f32 = 100.0;
pub const STTS_WEIGHT: u8 = 0xFF;
pub const STTS_HIT: u8 = 0xFF;
pub const TIMER_INIT: i16 = 3;
pub const SPH_BODY: usize = 0;
pub const CULL_HALF: f32 = SPH_RADIUS;
pub const CULL_MIN: [f32; 3] = [-CULL_HALF, -CULL_HALF, -CULL_HALF];
pub const CULL_MAX: [f32; 3] = [CULL_HALF, CULL_HALF, CULL_HALF];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ObjSrc {
    pub at_sprm: u32,
    pub at_type: u32,
    pub at_atp: u8,
    pub tg_sprm: u32,
    pub tg_type: u32,
    pub co_sprm: u32,
}

pub const CC_SPH_SRC: Sph = Sph {
    centre: SPH_CENTRE,
    radius: SPH_RADIUS,
    src: ObjSrc {
        at_sprm: AT_SPRM_INIT,
        at_type: AT_TYPE_WATER,
        at_atp: 0,
        tg_sprm: TG_SPRM_INIT,
        tg_type: 0,
        co_sprm: CO_SPRM_INIT,
    },
    tg_hit: false,
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sph {
    pub centre: Vec3,
    pub radius: f32,
    pub src: ObjSrc,
    pub tg_hit: bool,
}

impl Sph {
    fn set(&mut self, src: Sph) {
        self.centre = src.centre;
        self.radius = src.radius;
        self.src = src.src;
    }

    pub fn obj_set(&self) -> ObjSet {
        ObjSet {
            at_sprm: self.src.at_sprm,
            at_type: self.src.at_type,
            at_atp: self.src.at_atp,
            tg_sprm: self.src.tg_sprm,
            tg_type: self.src.tg_type,
            co_sprm: self.src.co_sprm,
            ..ObjSet::default()
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Hitobj {
    pub m_unused_param: u8,
    pub m_timer: i16,
    pub stts: GStts,
    pub sph: Sph,
    pub created: bool,
}

impl Hitobj {
    pub fn spawn(parameters: u32) -> Self {
        let mut hitobj = Hitobj {
            m_unused_param: 0,
            m_timer: 0,
            stts: GStts::default(),
            sph: Sph::default(),
            created: false,
        };
        hitobj.create(parameters);
        hitobj
    }

    pub fn create(&mut self, parameters: u32) {
        self.m_unused_param = (parameters & PARAM_MASK) as u8;
        self.stts = GStts::default();
        self.sph.set(CC_SPH_SRC);
        self.sph.tg_hit = false;
        self.m_timer = TIMER_INIT;
        self.created = true;
    }

    pub fn is_delete(&self) -> bool {
        true
    }

    pub fn set_sph(&self, bodies: &mut ContactBodies) {
        if let Some(body) = bodies.shapes.body_mut(SPH_BODY) {
            body.shape.kind = ShapeKind::Sphere;
            body.shape.pos = self.sph.centre;
            body.shape.radius = self.sph.radius;
        }
        if let Some(obj) = bodies.objs.get_mut(SPH_BODY) {
            obj.set = self.sph.obj_set();
            obj.g_stts = Some(self.stts);
        }
    }

    pub fn bodies(&self) -> ContactBodies {
        let mut bodies = ContactBodies::default();
        bodies.add(
            Shape {
                kind: ShapeKind::Sphere,
                pos: self.sph.centre,
                radius: self.sph.radius,
                ..Shape::default()
            },
            ContactObj {
                set: self.sph.obj_set(),
                g_stts: Some(self.stts),
                gobj_inf: Some(GObjInf::default()),
            },
        );
        bodies
    }

    pub fn execute(&mut self, actor: &Actor, bodies: &mut ContactBodies) -> bool {
        if self.m_timer != 0 {
            self.m_timer = self.m_timer.wrapping_sub(1);
            self.sph.centre = actor.pos;
            self.set_sph(bodies);
            false
        } else {
            true
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        false
    }
}

pub fn control(
    mut commands: Commands,
    fresh: Query<(Entity, &Hitobj), (With<Hitobj>, Without<ContactBodies>)>,
    mut proxies: Query<
        (Entity, &Actor, &mut Hitobj, &mut ContactBodies),
        (With<Hitobj>, With<ContactBodies>),
    >,
) {
    for (entity, hitobj) in &fresh {
        commands.entity(entity).insert(hitobj.bodies());
    }
    for (entity, actor, mut hitobj, mut bodies) in &mut proxies {
        let actor = &*actor;
        let hitobj = &mut *hitobj;
        let bodies = &mut *bodies;
        if !hitobj.created {
            hitobj.create(actor.parameters);
            hitobj.set_sph(bodies);
        }
        if hitobj.execute(actor, bodies) {
            commands.entity(entity).despawn();
        }
    }
}
