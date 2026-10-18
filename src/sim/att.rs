use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::contact::{
    AT_SPRM_SET, CO_SPRM_SET, CO_SPRM_VS_GRP_ALL, GObjInf, GStts, ObjSet, TG_SPRM_SET,
};
use super::contact_run::{ContactBodies, ContactObj, ContactResults};
use super::events::{SimEvent, SimEvents};
use super::registry::{ActorRegistry, STATUS_UNK10000000};
use super::shape::{Shape, ShapeKind};

pub const ATT_ACTOR: &str = "att";

pub const ATT_ACTORS: [&str; 0] = [];

pub fn is_att(name: &str) -> bool {
    ATT_ACTORS.contains(&name)
}

pub const BOSS_ACTOR: &str = "Bgn";

pub const PARAM_MASK: u32 = 0xFF;
pub const PARAM_SITATION: u8 = 101;
pub const PARAM_LOCKON: u8 = 100;
pub const PARAM_ROPE_MAX: u8 = 10;
pub const ATTN_TYPE_BATTLE: usize = 0x2;
pub const ATTN_SLOTS: usize = 8;
pub const ATTN_DISTANCE_BATTLE: u8 = 4;
pub const ATTN_NONE: u32 = 0x0000_0000;
pub const ATTN_LOCKON_BATTLE: u32 = 0x0000_0004;
pub const OFF_STATUS_NONE: u32 = 0;
pub const STTS_WEIGHT: u8 = 0xFF;
pub const STTS_SECOND: u8 = 0xFF;
pub const AT_SPRM_VS_PLAYER: u32 = 0x04;
pub const TG_SPRM_IS_ENEMY: u32 = 0x02;
pub const CO_SPRM_IS_PLAYER: u32 = 0x04;
pub const AT_TYPE_HOOKSHOT: u32 = 1 << 15;
pub const TG_TYPE_ALL_BUT: u32 = !((1 << 8) | (1 << 17) | (1 << 21) | (1 << 22) | (1 << 23));
pub const REG0_F16: f32 = 0.0;
pub const REG6_F3: f32 = 0.0;
pub const REG6_S7: i16 = 0;
pub const REG13_F3: f32 = 0.0;
pub const HOLD_FRAMES: i8 = 10;
pub const SE_LK_W_WEP_HIT: u32 = 0x2833;
pub const SE_CM_BGN_D_STRING_PLINK: u32 = 0x5970;
pub const MOVE_FLASH: i16 = 15;
pub const MOVE_TIMER: f32 = 6000.0;
pub const MOVE_RISE: f32 = 1.0;
pub const NEIGHBOUR_MAX: usize = 1;
pub const NEIGHBOUR_SLOT: usize = 2;
pub const NEIGHBOUR_TIMER: i16 = 600;
pub const ROPE_INDEX_7: usize = 7;
pub const CYL_RADIUS_LIVE: f32 = 200.0;
pub const SPH_LIFT: f32 = 1000.0;
pub const NON_POS: f32 = -30000.0;
pub const MOVE_COUNT: usize = 8;
pub const MOVE_STRIDE: usize = 0x30C;
pub const MOVE_BASE: usize = 0xAAA8;
pub const ROPE_COUNT: usize = 8;
pub const BOSS_SLOT_COUNT: usize = 5;
pub const CYL_BODY: usize = 0;
pub const SPH_BODY: usize = 1;
pub const SITATION_RADIUS: f32 = 90.0;
pub const CC_CYL_RADIUS: f32 = 200.0;
pub const CC_CYL_HEIGHT: f32 = 2000.0;
pub const BM_SPH_RADIUS: f32 = 200.0;
pub const CULL_HALF: f32 = CC_CYL_RADIUS;
pub const CULL_HALF_HEIGHT: f32 = CC_CYL_HEIGHT * 0.5;

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AttBoss {
    pub entity: Option<Entity>,
}

impl AttBoss {
    pub fn boss(&self) -> Option<Entity> {
        self.entity
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct AttNonPos {
    pub pos: Vec3,
}

impl Default for AttNonPos {
    fn default() -> Self {
        Self {
            pos: Vec3::new(NON_POS, NON_POS, NON_POS),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ObjSrc {
    pub at_sprm: u32,
    pub at_type: u32,
    pub at_atp: u8,
    pub tg_sprm: u32,
    pub tg_type: u32,
    pub co_sprm: u32,
}

pub const SITATION_SPH_SRC: Sph = Sph {
    centre: Vec3::ZERO,
    radius: SITATION_RADIUS,
    src: ObjSrc {
        at_sprm: AT_SPRM_SET | AT_SPRM_VS_PLAYER,
        at_type: 0,
        at_atp: 0,
        tg_sprm: TG_SPRM_SET | TG_SPRM_IS_ENEMY,
        tg_type: AT_TYPE_HOOKSHOT,
        co_sprm: 0,
    },
    tg_hit: false,
};

pub const CC_CYL_SRC: Cyl = Cyl {
    centre: Vec3::ZERO,
    radius: CC_CYL_RADIUS,
    height: CC_CYL_HEIGHT,
    src: ObjSrc {
        at_sprm: 0,
        at_type: 0,
        at_atp: 0,
        tg_sprm: TG_SPRM_SET | TG_SPRM_IS_ENEMY,
        tg_type: TG_TYPE_ALL_BUT,
        co_sprm: CO_SPRM_SET | CO_SPRM_IS_PLAYER | CO_SPRM_VS_GRP_ALL,
    },
    tg_hit: false,
};

pub const BM_SPH_SRC: Sph = Sph {
    centre: Vec3::ZERO,
    radius: BM_SPH_RADIUS,
    src: ObjSrc {
        at_sprm: AT_SPRM_SET | AT_SPRM_VS_PLAYER,
        at_type: 0,
        at_atp: 0,
        tg_sprm: TG_SPRM_SET | TG_SPRM_IS_ENEMY,
        tg_type: TG_TYPE_ALL_BUT,
        co_sprm: 0,
    },
    tg_hit: false,
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Cyl {
    pub centre: Vec3,
    pub radius: f32,
    pub height: f32,
    pub src: ObjSrc,
    pub tg_hit: bool,
}

impl Cyl {
    fn set(&mut self, src: Cyl) {
        self.centre = src.centre;
        self.radius = src.radius;
        self.height = src.height;
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MoveSlot {
    pub m2d0: u8,
    pub m2ec: f32,
    pub m300: i16,
    pub m304: f32,
    pub m308: i8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct BgnBody {
    pub m02b5: i8,
    pub moves: [MoveSlot; MOVE_COUNT],
    pub rope_pos: [Vec3; ROPE_COUNT],
    pub m_c748: i16,
    pub m_c74c: i16,
    pub m_c7ac: [i16; BOSS_SLOT_COUNT],
}

impl Default for BgnBody {
    fn default() -> Self {
        Self {
            m02b5: 0,
            moves: [MoveSlot::default(); MOVE_COUNT],
            rope_pos: [Vec3::ZERO; ROPE_COUNT],
            m_c748: 0,
            m_c74c: 0,
            m_c7ac: [0; BOSS_SLOT_COUNT],
        }
    }
}

impl BgnBody {
    pub fn slot_offset(index: usize) -> Option<usize> {
        let byte = index * MOVE_STRIDE;
        if MOVE_BASE + byte < MOVE_BASE + MOVE_COUNT * MOVE_STRIDE {
            Some(byte / MOVE_STRIDE)
        } else {
            None
        }
    }

    pub fn slot(&self, index: usize) -> Option<&MoveSlot> {
        Self::slot_offset(index).and_then(|slot| self.moves.get(slot))
    }

    pub fn slot_mut(&mut self, index: usize) -> Option<&mut MoveSlot> {
        match Self::slot_offset(index) {
            Some(slot) => self.moves.get_mut(slot),
            None => None,
        }
    }

    pub fn rope(&self, index: usize) -> Option<Vec3> {
        self.rope_pos.get(index).copied()
    }

    pub fn slot_neighbour(&mut self, index: usize) -> Option<&mut i16> {
        self.m_c7ac.get_mut(index)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Att {
    pub m2b5: u8,
    pub m550: i8,
    pub attn_distance: [u8; ATTN_SLOTS],
    pub attn_flags: u32,
    pub attn_pos: Vec3,
    pub eye_pos: Vec3,
    pub cyl: Cyl,
    pub sph: Sph,
    pub created: bool,
}

impl Att {
    pub fn spawn(parameters: u32) -> Self {
        let m2b5 = (parameters & PARAM_MASK) as u8;
        let mut attn_distance = [0; ATTN_SLOTS];
        attn_distance[ATTN_TYPE_BATTLE] = ATTN_DISTANCE_BATTLE;
        let mut att = Att {
            m2b5,
            m550: 0,
            attn_distance,
            attn_flags: ATTN_NONE,
            attn_pos: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            cyl: Cyl::default(),
            sph: Sph::default(),
            created: false,
        };
        if m2b5 == PARAM_SITATION {
            att.sph.set(SITATION_SPH_SRC);
        }
        if m2b5 < PARAM_ROPE_MAX {
            att.cyl.set(CC_CYL_SRC);
            att.sph.set(BM_SPH_SRC);
        }
        att
    }

    pub fn bodies(&self) -> ContactBodies {
        let mut bodies = ContactBodies::default();
        bodies.add(
            Shape {
                kind: ShapeKind::Cylinder,
                radius: self.cyl.radius,
                height: self.cyl.height,
                ..Shape::default()
            },
            ContactObj {
                set: self.cyl.obj_set(),
                g_stts: Some(GStts::default()),
                gobj_inf: Some(GObjInf::default()),
            },
        );
        bodies.add(
            Shape {
                kind: ShapeKind::Sphere,
                radius: self.sph.radius,
                ..Shape::default()
            },
            ContactObj {
                set: self.sph.obj_set(),
                g_stts: Some(GStts::default()),
                gobj_inf: Some(GObjInf::default()),
            },
        );
        bodies
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (
            [-CULL_HALF, -CULL_HALF_HEIGHT, -CULL_HALF],
            [CULL_HALF, CULL_HALF_HEIGHT, CULL_HALF],
        )
    }

    pub fn drawn(&self) -> bool {
        false
    }

    pub fn create(
        &mut self,
        registry: &mut ActorRegistry,
        cache: &mut AttBoss,
        proc_id: Option<u32>,
    ) {
        self.created = true;
        if self.m2b5 == PARAM_SITATION {
            if let Some(proc_id) = proc_id {
                registry.on_status(proc_id, STATUS_UNK10000000);
            }
        }
        if self.m2b5 < PARAM_ROPE_MAX {
            cache.entity = None;
        }
    }
}

fn set_cyl(bodies: &mut ContactBodies, cyl: Cyl, origin: Vec3) {
    if let Some(body) = bodies.shapes.body_mut(CYL_BODY) {
        body.shape.kind = ShapeKind::Cylinder;
        body.shape.pos = cyl.centre - origin;
        body.shape.radius = cyl.radius;
        body.shape.height = cyl.height;
    }
    if let Some(obj) = bodies.objs.get_mut(CYL_BODY) {
        obj.set = cyl.obj_set();
    }
}

fn set_sph(bodies: &mut ContactBodies, sph: Sph, origin: Vec3) {
    if let Some(body) = bodies.shapes.body_mut(SPH_BODY) {
        body.shape.kind = ShapeKind::Sphere;
        body.shape.pos = sph.centre - origin;
        body.shape.radius = sph.radius;
    }
    if let Some(obj) = bodies.objs.get_mut(SPH_BODY) {
        obj.set = sph.obj_set();
    }
}

fn execute(
    att: &mut Att,
    actor: &mut Actor,
    bodies: &mut ContactBodies,
    boss: Option<&mut BgnBody>,
    boss_entity: Option<Entity>,
    cache: &mut AttBoss,
    hit_cyl: bool,
    hit_sph: bool,
    non_pos: Vec3,
    proc_id: Option<u32>,
    registry: &mut ActorRegistry,
    events: &mut SimEvents,
) {
    att.cyl.tg_hit = hit_cyl;
    att.sph.tg_hit = hit_sph;

    if att.m2b5 == PARAM_SITATION {
        att.attn_pos = actor.pos;
        att.eye_pos = actor.pos;
        if let Some(proc_id) = proc_id {
            registry.off_status(proc_id, OFF_STATUS_NONE);
        }
        att.attn_flags = ATTN_NONE;
        att.sph.centre = actor.pos;
        set_sph(bodies, att.sph, actor.pos);
    }

    if att.m2b5 == PARAM_LOCKON {
        att.attn_pos = actor.pos;
        att.eye_pos = actor.pos;
        return;
    }

    cache.entity = boss_entity;
    let Some(boss) = boss else {
        return;
    };

    let r30 = att.m2b5 as usize;

    if att.m550 != 0 {
        att.m550 = att.m550.wrapping_sub(1);
    }

    if att.m550 == 0 && (att.cyl.tg_hit || att.sph.tg_hit) {
        att.m550 = HOLD_FRAMES;
        let broken = match boss.slot_mut(r30) {
            Some(slot) => {
                slot.m308 = slot.m308.wrapping_sub(1);
                if slot.m308 <= 0 {
                    slot.m2d0 = 1;
                    slot.m304 = MOVE_TIMER + REG6_F3;
                    true
                } else {
                    slot.m300 = MOVE_FLASH + REG6_S7;
                    false
                }
            }
            None => false,
        };
        if broken {
            events.list.push(SimEvent::Se(SE_LK_W_WEP_HIT));
            if r30 <= NEIGHBOUR_MAX
                && boss
                    .slot(NEIGHBOUR_MAX - r30)
                    .is_some_and(|slot| slot.m2d0 != 0)
            {
                if let Some(slot) = boss.slot_neighbour(NEIGHBOUR_SLOT) {
                    *slot = NEIGHBOUR_TIMER;
                }
            }
        } else {
            events.list.push(SimEvent::Se(SE_CM_BGN_D_STRING_PLINK));
        }
    }

    let mut sp08 = boss.rope(r30).unwrap_or(actor.pos);
    actor.pos = sp08;

    let live = boss.m02b5 == 0
        && (boss.m_c748 != 0 || boss.m_c74c != 0 || r30 != ROPE_INDEX_7)
        && boss.slot(r30).map_or(true, |slot| slot.m2d0 == 0)
        && boss.slot(r30).map_or(true, |slot| slot.m2ec < MOVE_RISE);

    if live {
        att.attn_flags = ATTN_LOCKON_BATTLE;
        att.cyl.radius = CYL_RADIUS_LIVE + REG0_F16;
        att.cyl.centre = sp08;
        sp08.y += SPH_LIFT + REG13_F3;
        att.sph.centre = sp08;
        att.eye_pos = sp08;
        att.attn_pos = sp08;
    } else {
        if let Some(proc_id) = proc_id {
            registry.off_status(proc_id, OFF_STATUS_NONE);
        }
        att.attn_flags = ATTN_NONE;
        att.cyl.centre = non_pos;
        att.sph.centre = non_pos;
    }

    set_cyl(bodies, att.cyl, actor.pos);
    set_sph(bodies, att.sph, actor.pos);
}

pub fn control(
    mut proxies: Query<(Entity, &mut Actor, &mut Att, &mut ContactBodies)>,
    boss_ids: Query<(Entity, &Actor), (With<BgnBody>, Without<Att>)>,
    mut boss_bodies: Query<&mut BgnBody, (With<BgnBody>, Without<Att>)>,
    mut boss_cache: ResMut<AttBoss>,
    non_pos: Res<AttNonPos>,
    mut events: ResMut<SimEvents>,
    mut registry: ResMut<ActorRegistry>,
    results: Res<ContactResults>,
) {
    let boss_entity = boss_ids
        .iter()
        .find(|(_, actor)| actor.name == BOSS_ACTOR)
        .map(|(entity, _)| entity);

    for (entity, mut actor, mut att, mut bodies) in &mut proxies {
        let actor = &mut *actor;
        let att = &mut *att;
        let bodies = &mut *bodies;
        let proc_id = registry.proc_id(entity);

        if !att.created {
            att.create(&mut registry, &mut boss_cache, proc_id);
        }

        let hit = |body: usize| {
            results
                .at_tg
                .iter()
                .any(|rec| rec.tg == entity && rec.tg_body == body && rec.set_tg)
        };
        let hit_cyl = hit(CYL_BODY);
        let hit_sph = hit(SPH_BODY);

        let mut boss = boss_entity.and_then(|id| boss_bodies.get_mut(id).ok());
        let boss = boss.as_mut().map(|body| &mut **body);
        execute(
            att,
            actor,
            bodies,
            boss,
            boss_entity,
            &mut boss_cache,
            hit_cyl,
            hit_sph,
            non_pos.pos,
            proc_id,
            &mut registry,
            &mut events,
        );
    }
}
