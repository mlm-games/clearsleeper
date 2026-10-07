use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::contact::{
    self, AT_SPRM_GRP_ALL, AT_SPRM_SET, AtTgHit, CO_SPRM_I_GRP_ALL, CO_SPRM_NO_CRR, CO_SPRM_SET,
    CO_SPRM_VS_GRP_ALL, CcSystem, CoHit, DivideInfo, GObjInf, GSTTS_NO_ACTOR, GStts, ObjEntry,
    ObjSet, Stts, TG_SPRM_GRP_ALL, TG_SPRM_SET,
};
use super::player::Player;
use super::registry::{ActorRegistry, ERROR_PROC_ID};
use super::shape::{Shape, Shapes};

/// The `mType` word `cCcD_ObjAt::GetType` and `cCcD_ObjTg::GetType` share, which
/// `cCcS::ChkNoHitAtTg` (`c_cc_s.cpp:101-112`) ANDs: a hit survives only when both sides name one
/// common type bit. Reference processes register single-bit types, so one bit is the whole word.
pub const HIT_TYPE: u32 = 1;

/// One persistent `cCcD_Obj`: the three hit-info sub-objects plus the general status and general
/// object info the pipeline reads and writes across frames. The shape index, the status and the
/// divide info are rebuilt every step from the owning entity, so they are not held here.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ContactObj {
    /// `mObjAt`, `mObjTg`, `mObjCo` and `mFlags`.
    pub set: ObjSet,
    /// `GetStts()->GetGStts()`, which the reference's own status answers with itself.
    pub g_stts: Option<GStts>,
    /// `GetGObjInf()`.
    pub gobj_inf: Option<GObjInf>,
}

impl ContactObj {
    /// `cCcD_ObjAt::ChkAtSet` with every attack group and the shared type word, an attack body that
    /// is also its own correction body and its own target: the shape most processes register.
    pub fn at() -> Self {
        Self {
            set: ObjSet {
                at_sprm: AT_SPRM_SET | AT_SPRM_GRP_ALL,
                tg_sprm: TG_SPRM_SET | TG_SPRM_GRP_ALL,
                co_sprm: CO_SPRM_SET | CO_SPRM_I_GRP_ALL | CO_SPRM_VS_GRP_ALL,
                at_type: HIT_TYPE,
                tg_type: HIT_TYPE,
                ..ObjSet::default()
            },
            g_stts: Some(GStts::default()),
            gobj_inf: Some(GObjInf::default()),
        }
    }

    /// `cCcD_ObjAt::ChkAtSet` alone: a body that reaches out and is not itself a target.
    pub fn attack() -> Self {
        Self {
            set: ObjSet {
                at_sprm: AT_SPRM_SET | AT_SPRM_GRP_ALL,
                at_type: HIT_TYPE,
                ..ObjSet::default()
            },
            g_stts: Some(GStts::default()),
            gobj_inf: Some(GObjInf::default()),
        }
    }

    /// `cCcD_ObjTg::ChkTgSet` alone: a body that can be reached but reaches nothing.
    pub fn target() -> Self {
        Self {
            set: ObjSet {
                tg_sprm: TG_SPRM_SET | TG_SPRM_GRP_ALL,
                tg_type: HIT_TYPE,
                ..ObjSet::default()
            },
            g_stts: Some(GStts::default()),
            gobj_inf: Some(GObjInf::default()),
        }
    }

    /// `cCcD_ObjCo::ChkCoSet` with every collision group on both sides, which is what
    /// `cCcS::ChkNoHitCo` (`c_cc_s.cpp:154-165`) needs to keep a pair.
    pub fn co() -> Self {
        Self {
            set: ObjSet {
                co_sprm: CO_SPRM_SET | CO_SPRM_I_GRP_ALL | CO_SPRM_VS_GRP_ALL,
                ..ObjSet::default()
            },
            g_stts: Some(GStts::default()),
            gobj_inf: Some(GObjInf::default()),
        }
    }

    /// `dCcD_Stts::Init` (`d_cc_d.cpp:67-84`) read as the pipeline's one [`ObjEntry`]: the weight
    /// is the owner's, the process id is the owner's, and an entity the registry does not know is
    /// the error id with the no-actor flag raised, which is what the reference derives from a null
    /// actor pointer. `no_crr` is the owner's own `cCcD_CoSPrm_NoCrr_e`, which
    /// `cCcD_ObjCo::OnCoSPrmBit` sets on the collider rather than on the status, so it rides on
    /// the object's `co_sprm` and `cCcD_ObjCo::ChkNoCrr` (`c_cc_d.h:493`) answers it from there.
    fn entry(&self, shape: usize, weight_uc: u8, no_crr: bool, owner: Option<u32>) -> ObjEntry {
        ObjEntry {
            shape: Some(shape),
            set: ObjSet {
                co_sprm: self.set.co_sprm | if no_crr { CO_SPRM_NO_CRR } else { 0 },
                ..self.set
            },
            stts: Some(Stts {
                weight_uc,
                apid: owner.unwrap_or(ERROR_PROC_ID),
                actor: owner,
                cc_move: Vec3::ZERO,
                dmg: 0,
            }),
            g_stts: self.g_stts.map(|g_stts| match owner {
                Some(_) => g_stts,
                None => GStts {
                    flag: g_stts.flag | GSTTS_NO_ACTOR,
                    ..g_stts
                },
            }),
            gobj_inf: self.gobj_inf,
            divide: DivideInfo::default(),
        }
    }
}

/// The shapes and hit infos one entity registers with the contact system. The reference keeps all
/// three on the process itself, so the registration rides on the owning actor here.
#[derive(Component, Debug, Default)]
pub struct ContactBodies {
    /// The `cCcD_ShapeAttr` list this entity owns.
    pub shapes: Shapes,
    /// One [`ContactObj`] per body, in body order, so the two lists stay index-parallel.
    pub objs: Vec<ContactObj>,
}

impl ContactBodies {
    /// `cCcS::MoveObj`: append a shape with its hit infos and answer the one list position both are
    /// addressed by. A body with no [`ContactObj`] is a plain volume and takes no part in the
    /// contact pass.
    pub fn add(&mut self, shape: Shape, obj: ContactObj) -> usize {
        self.objs.push(obj);
        self.shapes.add(shape)
    }

    /// Keep one body on its owner, the way a process re-registers its colliders from the transform
    /// it just computed. Only the position is touched; radius, height, kind and yaw are the
    /// author's.
    pub fn follow(&mut self, body: usize, at: Vec3) {
        self.shapes.set_vec(body, at);
    }
}

/// One recorded AT-by-TG crossing, with the entities and bodies behind the two object slots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactAtTg {
    pub at: Entity,
    pub at_body: usize,
    pub tg: Entity,
    pub tg_body: usize,
    /// `SetAtTgCommonHitInf`'s `i_setAt`.
    pub set_at: bool,
    /// `SetAtTgCommonHitInf`'s `i_setTg`.
    pub set_tg: bool,
    /// The point `CrossAtTg` reported.
    pub cross: Vec3,
    /// `dCcD_GObjTg.mRVec` (`d_cc_d.h:362`), which `SetAtTgGObjInf` fills from the attacker's own
    /// velocity at `d_cc_s.cpp:428`. The player's own `getDamageVec` (`main.cpp:5558-5586`) reads this
    /// word and leaves it either as it stands or as a fixed ten-unit push, and the guard slip takes
    /// its facing and its speed from what comes back.
    pub r_vec: Vec3,
}

/// One recorded CO contact, with the entities and bodies behind the two object slots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactCo {
    pub obj1: Entity,
    pub body1: usize,
    pub obj2: Entity,
    pub body2: usize,
    /// The first body's `GetCoCP`, before any correction.
    pub pos1: Vec3,
    /// The second body's `GetCoCP`, before any correction.
    pub pos2: Vec3,
    /// The `CrossCo` overlap depth the correction was split from.
    pub cross_len: f32,
}

/// Which entity and which of its bodies one `CcSystem::objs` slot came from.
#[derive(Debug, Clone, Copy)]
struct Slot {
    entity: Entity,
    body: usize,
}

/// What one contact pass produced this step. The correction is already committed to the actors that
/// filled this, so only the crossing records are left for readers.
#[derive(Resource, Debug, Default)]
pub struct ContactResults {
    pub at_tg: Vec<ContactAtTg>,
    pub co: Vec<ContactCo>,
    slots: Vec<Slot>,
    system: CcSystem,
    flat: Shapes,
}

impl ContactResults {
    fn begin(&mut self) {
        self.at_tg.clear();
        self.co.clear();
        self.slots.clear();
        self.system.clear_registrations();
        self.flat.clear();
    }

    /// Append one entity's bodies to the flat shape list and its hit infos to the object list, and
    /// push the two sublists the object belongs to. The whole entity's body list is copied so the
    /// bodies that carry no hit infos still hold the next entity's base index right. `velocity` is
    /// the owner's own `speed`, which `SetAtTgGObjInf` copies onto the reflect vector of whatever
    /// this entity's attack objects reach (`d_cc_s.cpp:428`); an owner the registry does not know
    /// still has one.
    fn register(
        &mut self,
        entity: Entity,
        bodies: &ContactBodies,
        weight_uc: u8,
        no_crr: bool,
        owner: Option<u32>,
        velocity: Vec3,
    ) {
        if bodies.objs.is_empty() || bodies.objs.len() > bodies.shapes.bodies.len() {
            return;
        }
        let base = self.flat.bodies.len();
        self.flat.bodies.extend_from_slice(&bodies.shapes.bodies);
        for (body, obj) in bodies.objs.iter().enumerate() {
            let index = self.system.objs.len();
            self.system
                .objs
                .push(obj.entry(base + body, weight_uc, no_crr, owner));
            self.system.at_vec.push(velocity);
            if obj.set.chk_at_set() {
                self.system.at.push(index);
            }
            if obj.set.chk_tg_set() {
                self.system.tg.push(index);
            }
            if obj.set.chk_co_set() {
                self.system.co.push(index);
            }
            self.slots.push(Slot { entity, body });
        }
    }

    /// Translate the two recorded lists into entity-addressable records.
    fn collect(&mut self, at_tg: Vec<AtTgHit>, co: Vec<CoHit>) {
        for hit in at_tg {
            let (Some(at), Some(tg)) = (
                self.slots.get(hit.at).copied(),
                self.slots.get(hit.tg).copied(),
            ) else {
                continue;
            };
            self.at_tg.push(ContactAtTg {
                at: at.entity,
                at_body: at.body,
                tg: tg.entity,
                tg_body: tg.body,
                set_at: hit.set_at,
                set_tg: hit.set_tg,
                cross: hit.cross,
                r_vec: hit.r_vec,
            });
        }
        for hit in co {
            let (Some(one), Some(two)) = (
                self.slots.get(hit.obj1).copied(),
                self.slots.get(hit.obj2).copied(),
            ) else {
                continue;
            };
            self.co.push(ContactCo {
                obj1: one.entity,
                body1: one.body,
                obj2: two.entity,
                body2: two.body,
                pos1: hit.pos1,
                pos2: hit.pos2,
                cross_len: hit.cross_len,
            });
        }
    }

    /// Commit `dCcS::SetPosCorrect`'s output. The correction it accumulated into each object's
    /// status is what the reference actor reads back through `GetCCMoveP` and moves itself by;
    /// here it lands on the owning actor's position and on the body the same object list slot
    /// carried, so the two agree at the next step.
    fn commit(&mut self, bodies: &mut Query<(Entity, &mut Actor, &mut ContactBodies)>) {
        for (index, obj) in self.system.objs.iter().enumerate() {
            let Some(slot) = self.slots.get(index).copied() else {
                continue;
            };
            let Some(stts) = obj.stts else {
                continue;
            };
            let step = stts.cc_move;
            if step == Vec3::ZERO {
                continue;
            }
            if let Ok((_, mut actor, mut contact)) = bodies.get_mut(slot.entity) {
                actor.pos += step;
                contact.shapes.move_body(slot.body, step);
            }
        }
    }

    /// Keep the half of `cCcD_Obj` the reference carries across steps: the hit bits the three clear
    /// calls take down at the top of the next pass, the general status apids
    /// `dCcS::ChkAtTgHitAfterCross` writes, and the general object info the contact bookkeeping
    /// stamps. `dCcD_GStts::Move`, which is what slides those apids into the old slots, has no
    /// located caller in the reference, so the old fields stay as the author set them.
    fn sync_back(&mut self, bodies: &mut Query<(Entity, &mut Actor, &mut ContactBodies)>) {
        for (index, obj) in self.system.objs.iter().enumerate() {
            let Some(slot) = self.slots.get(index).copied() else {
                continue;
            };
            let Ok((_, _, mut contact)) = bodies.get_mut(slot.entity) else {
                continue;
            };
            let Some(record) = contact.objs.get_mut(slot.body) else {
                continue;
            };
            record.set = obj.set;
            record.g_stts = obj.g_stts;
            record.gobj_inf = obj.gobj_inf;
        }
    }
}

/// The player's own velocity vector. An [`Actor`] carries one whole, but the player entity is a bare
/// [`Player`] — `mod.rs` spawns it with no [`Actor`] and no [`ContactBodies`], so it never reaches
/// the query below and this is the shape the port's own code already used wherever it needed the
/// player's velocity whole: the sword's [`super::damage_resolve::Attacker`] is stamped with it at
/// `combat.rs:330-334`, and `combat.rs:312-318` turns the same body facing for the blade's offset.
pub fn player_velocity(player: &Player) -> Vec3 {
    Vec3::new(
        player.speed * player.yaw.sin(),
        player.speed_y,
        player.speed * player.yaw.cos(),
    )
}

/// `cCcS::Move` (`c_cc_s.cpp:432-441`) for the whole world, once per step: assemble every
/// entity's registration, calc the area, check AT against TG, check CO, run the retail move hook,
/// then drop the registrations.
///
/// This is `contact::move_pipeline` with one difference: the four lists go down separately, after
/// the correction has been committed. `move_pipeline` drops them itself, and `Stts::cc_move`, which
/// is what `dCcS::SetPosCorrect` accumulated for this step, goes down with `objs` before a caller
/// can read it.
pub fn run(
    mut bodies: Query<(Entity, &mut Actor, &mut ContactBodies)>,
    registry: Res<ActorRegistry>,
    mut results: ResMut<ContactResults>,
) {
    let state = &mut *results;
    state.begin();
    for (entity, actor, contact) in &mut bodies {
        state.register(
            entity,
            &*contact,
            actor.weight_uc,
            actor.no_crr,
            registry.proc_id(entity),
            actor.speed,
        );
    }
    contact::calc_area(&state.flat, &mut state.system);
    let at_tg = contact::chk_at_tg(&state.flat, &mut state.system);
    let co = contact::chk_co(&state.flat, &mut state.system);
    contact::move_after_check();
    state.collect(at_tg, co);
    state.commit(&mut bodies);
    state.sync_back(&mut bodies);
    state.system.clear_registrations();
}
