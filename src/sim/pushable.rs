use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::collision::Collision;
use super::inventory::{Equipped, SLOT_COUNT as ITEM_SLOTS};
use super::movebox::Movebox;
use super::tag_hint::ITEM_POWER_BRACELETS;
use super::walk::{from_s16, to_s16};

/// `dBgS::PushPullCallBack`'s bound, `d_bg_s.cpp:531`, which is the size of the table it indexes,
/// `c_bg_s.h:40`, and the bound `cBgS::Regist` asserts while it hands a background its slot,
/// `c_bg_s.cpp:51`.
pub const BG_SLOTS: usize = 256;

/// `dBgW::PushPullLabel`, `d_bg_w.h:21-27`.
pub const PPLABEL_NONE: u32 = 0x00;
pub const PPLABEL_PUSH: u32 = 0x01;
pub const PPLABEL_PULL: u32 = 0x02;
pub const PPLABEL_UNK4: u32 = 0x04;
pub const PPLABEL_HEAVY: u32 = 0x08;
pub const PPLABEL_PUSH_PULL: u32 = PPLABEL_PUSH | PPLABEL_PULL;

/// `setFrontWallType`'s own verticality test, `main.cpp:4572-4574`: a face whose plane is steeper than
/// this is not a wall, and carries no `mPolyInfo`.
const FACE_FLAT: f32 = 0.05;
/// The slack a movebox's own cull box leaves under its top face, `M_ATTR`'s `cull_min.y` of `-1`,
/// which is the band a rider's feet have to fall in for its ground check to find that face.
const FACE_STAND: f32 = 1.0;

/// The solid one registered background stands on, in the owner's own frame: the actor's position,
/// the yaw its own `shape_angle.y` turns the model by, and the half extents and the two heights
/// its own cull box carries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Face {
    /// `current.pos`.
    pub pos: Vec3,
    /// `shape_angle.y`, which turns the background's own polygons about `pos`.
    pub yaw: i16,
    /// `attr()->mCullMinX` and `attr()->mCullMaxX`, halved, `movebox.cpp:1331`.
    pub half_x: f32,
    /// `attr()->mCullMinZ` and `attr()->mCullMaxZ`, halved, `movebox.cpp:1333`.
    pub half_z: f32,
    /// `attr()->mCullMinY`, `movebox.cpp:1332`.
    pub bottom: f32,
    /// `attr()->mCullMaxY`, `movebox.cpp:1333`.
    pub top: f32,
}

impl Face {
    /// The cull box a movebox hands the shell at `movebox.cpp:1331-1334`, which is the
    /// extent of the background it built for itself at `:84`.
    pub fn of(box_: &Movebox) -> Self {
        let (min, max) = box_.cull();
        Self {
            pos: box_.pos,
            yaw: box_.shape_angle[1],
            half_x: 0.5 * (max[0] - min[0]),
            half_z: 0.5 * (max[2] - min[2]),
            bottom: min[1],
            top: max[1],
        }
    }

    /// The turn `mDoMtx_stack_c::YrotS(home.angle.y)` gives before the owner's `transM`, which is
    /// the transform a movebox's own background is drawn and collided under, and which a plane's
    /// own normal takes without the translation.
    fn rot(&self, at: Vec3) -> Vec3 {
        let (s, c) = from_s16(self.yaw).sin_cos();
        Vec3::new(at.x * c + at.z * s, at.y, -at.x * s + at.z * c)
    }

    /// The same transform inverted, which is `RwgLineCheck`'s own frame at `c_bg_w.cpp:386`.
    fn to_local(&self, at: Vec3) -> Vec3 {
        let (s, c) = from_s16(self.yaw).sin_cos();
        let d = at - self.pos;
        Vec3::new(d.x * c - d.z * s, d.y, d.x * s + d.z * c)
    }

    /// Where a horizontal segment first runs into the solid, and the outward normal of the face it
    /// ran into: the answer `dBgS::LineCross`, `bg_s.h:117`, gives for a moving background's own
    /// polygons, which the port's [`Collision`] does not hold.
    pub fn cross(&self, from: Vec3, to: Vec3) -> Option<(f32, Vec3)> {
        if from.y < self.bottom || from.y > self.top {
            return None;
        }
        let start = self.to_local(from);
        let end = self.to_local(to);
        let mut enter = f32::NEG_INFINITY;
        let mut leave = f32::INFINITY;
        let mut normal = Vec3::ZERO;
        for (lo, hi, at, along, axis) in [
            (-self.half_x, self.half_x, start.x, end.x - start.x, Vec3::X),
            (-self.half_z, self.half_z, start.z, end.z - start.z, Vec3::Z),
        ] {
            if along.abs() <= f32::EPSILON {
                if at < lo || at > hi {
                    return None;
                }
                continue;
            }
            let mut near = (lo - at) / along;
            let mut far = (hi - at) / along;
            let mut side = -1.0;
            if near > far {
                std::mem::swap(&mut near, &mut far);
                side = 1.0;
            }
            if near > enter {
                enter = near;
                normal = axis * side;
            }
            leave = leave.min(far);
            if enter > leave {
                return None;
            }
        }
        if normal == Vec3::ZERO || !(0.0..=1.0).contains(&enter) {
            return None;
        }
        Some((enter, self.rot(normal)))
    }

    /// Whether a rider's feet stand on this background's own top face, which is what
    /// `dBgS_Acch::GroundCheck`, `d_bg_s_acch.cpp:122-145`, answers when its `m_gnd` came off one
    /// of its polygons.
    pub fn carries(&self, feet: Vec3) -> bool {
        let local = self.to_local(feet);
        local.x.abs() <= self.half_x
            && local.z.abs() <= self.half_z
            && (feet.y - self.top).abs() <= FACE_STAND
    }
}

/// Which background a polygon belongs to, which is the index a `cBgS_PolyInfo` carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Background {
    /// A polygon of a room's own static background. `cBgS::Regist` gave that background a slot of
    /// its own, `c_bg_s.cpp:42-66`, and no room publishes a push and pull callback on it.
    Room(usize),
    /// A polygon of the background a movebox registered for itself, named by the slot
    /// `cBgS::Regist` handed it.
    Registered(usize),
}

impl Background {
    /// `cBgS_PolyInfo::GetBgIndex`, `c_bg_s_poly_info.h`.
    pub fn index(self) -> usize {
        match self {
            Self::Room(tri) => tri,
            Self::Registered(slot) => slot,
        }
    }
}

/// One polygon as the trace that found it left it: the background it belongs to, the cross point,
/// and the plane's own normal, which `setFrontWallType` tests at `main.cpp:4572-4574` and turns into
/// its wall facing at `main.cpp:4576`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Poly {
    pub bg: Background,
    pub point: Vec3,
    pub normal: Vec3,
}

impl Poly {
    /// `setFrontWallType`'s own verticality test, `main.cpp:4572-4574`, which is the last thing that
    /// stands between a trace and the `mPolyInfo` the push and pull states dispatch on.
    pub fn is_face(&self) -> bool {
        self.normal.y.abs() <= FACE_FLAT
    }

    /// `m352C`, `main.cpp:4576`: the wall's own facing.
    pub fn wall_yaw(&self) -> i16 {
        to_s16(self.normal.x.atan2(self.normal.z))
    }
}

/// `cBgS_ChkElm`, `c_bg_s.h:15-32`: one slot of the table the dispatch indexes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Element {
    /// `bgwp->ChkUsed()`, which `d_bg_s.cpp:534` refuses on.
    pub used: bool,
    /// `m_chk_element[bg_index].m_actor_ptr`, which `d_bg_s.cpp:536` refuses on.
    pub actor: Option<Entity>,
    /// `bgwp->GetPushPullCallback()`, which `d_bg_s.cpp:538` refuses on.
    pub push_pull: bool,
    /// `bgwp->GetRideCallback()`, which `d_bg_s.cpp:524` refuses on.
    pub ride: bool,
}

/// `cBgS::m_chk_element`, `c_bg_s.h:40`: the table every push and pull and ride callback is
/// dispatched through, keyed on the background a polygon belongs to. The player asks
/// `dBgS::PushPullCallBack` with the polygon its own wall trace landed on, `pushpull.inc:28`, and
/// every actor's ground check asks `dBgS::RideCallBack` with the polygon it landed on,
/// `d_bg_s_acch.cpp:144`, so one entry per registered background holds the background itself, the
/// actor that owns it, and the two callbacks that actor published. A movebox registers a background
/// of its own at `movebox.cpp:1321-1322` and takes both callbacks back at `:1882-1883`.
///
/// The port's [`Collision`](super::collision::Collision) holds the static rooms' triangles only and
/// a movebox's own background contributes none of them, so a movebox registers its solid here
/// instead of a polygon list: [`Face`] is the rectangle its own cull box hands the shell at
/// `movebox.cpp:1331`, and every trace that would run against that background is offered it.
#[derive(Resource, Debug, Clone)]
pub struct PushPulls {
    slots: Vec<Element>,
    faces: Vec<Option<Face>>,
    /// `l_SetCounter`, `c_bg_s.cpp:13`, which is where `cBgS::Regist` resumes its search from.
    next: usize,
    /// The slots this table holds, which is every element of `cBgS::m_chk_element` that a trace
    /// has something to ask.
    live: Vec<usize>,
}

impl Default for PushPulls {
    fn default() -> Self {
        Self::new()
    }
}

impl PushPulls {
    /// `cBgS::Ct`, `c_bg_s.cpp:94-98`: the table starts empty, every slot unused, and the
    /// hand-out counter back at its first slot.
    pub fn new() -> Self {
        Self {
            slots: vec![Element::default(); BG_SLOTS],
            faces: vec![None; BG_SLOTS],
            next: 0,
            live: Vec::new(),
        }
    }

    /// `cBgS_ChkElm::Release`, `c_bg_s.cpp:32-37`, over the whole table as `cBgS::Ct` leaves it at
    /// `:94-98`: every slot goes back to unused, which is what an actor's `Delete` does to the
    /// background it registered.
    pub fn reset(&mut self) {
        for slot in &mut self.slots {
            *slot = Element::default();
        }
        for face in &mut self.faces {
            *face = None;
        }
        self.next = 0;
        self.live.clear();
    }

    /// `cBgS::Regist`, `c_bg_s.cpp:40-71`: take the first unused slot at or above the hand-out
    /// counter and register the background against it, which is the index every one of its
    /// polygons then carries. An actor already in the table keeps the slot it holds, as the
    /// reference's early return on `bgw->ChkUsed()` at `:44` does.
    pub fn claim(
        &mut self,
        actor: Entity,
        face: Option<Face>,
        push_pull: bool,
        ride: bool,
    ) -> usize {
        if let Some(slot) = self.slots.iter().position(|slot| slot.actor == Some(actor)) {
            if let Some(entry) = self.slots.get_mut(slot) {
                entry.used = true;
                entry.push_pull = entry.push_pull || push_pull;
                entry.ride = entry.ride || ride;
            }
            return slot;
        }
        let start = self.next;
        let slot = (0..BG_SLOTS)
            .map(|offset| (start + offset) % BG_SLOTS)
            .find(|index| !self.slots.get(*index).is_some_and(|slot| slot.used))
            .unwrap_or(start % BG_SLOTS);
        self.next = (slot + 1) % BG_SLOTS;
        if let Some(entry) = self.slots.get_mut(slot) {
            *entry = Element {
                used: true,
                actor: Some(actor),
                push_pull,
                ride,
            };
        }
        if let Some(entry) = self.faces.get_mut(slot) {
            *entry = face;
        }
        self.live.push(slot);
        slot
    }

    /// `dBgS::PushPullCallBack`, `d_bg_s.cpp:529-542`, its `JUT_ASSERT` at `:531` kept as the guard
    /// it can only be in a build that cannot assert. A background index outside the table answers
    /// null, an unused slot answers null, a slot with no actor answers null, a slot whose actor
    /// published no push and pull callback answers null, and what is left is the actor the
    /// dispatch hands the label to at `:542`.
    pub fn push_pull_actor(&self, poly: Poly) -> Option<Entity> {
        if poly.bg.index() >= BG_SLOTS {
            return None;
        }
        match poly.bg {
            Background::Room(_) => None,
            Background::Registered(slot) => self.element(slot, true),
        }
    }

    /// `dBgS::RideCallBack`, `d_bg_s.cpp:519-527`: the same table and the same refusals, keyed on
    /// the background a rider's own ground polygon belongs to.
    pub fn ride_actor(&self, poly: Poly) -> Option<Entity> {
        if poly.bg.index() >= BG_SLOTS {
            return None;
        }
        match poly.bg {
            Background::Room(_) => None,
            Background::Registered(slot) => self.element(slot, false),
        }
    }

    fn element(&self, slot: usize, push_pull: bool) -> Option<Entity> {
        let element = self.slots.get(slot)?;
        if !element.used || element.actor.is_none() {
            return None;
        }
        if push_pull && !element.push_pull {
            return None;
        }
        if !push_pull && !element.ride {
            return None;
        }
        element.actor
    }

    /// `dBgS::LineCross`, `bg_s.h:117`: the nearest polygon the segment runs into, offered the
    /// registered backgrounds and the room's own triangles and answered with whichever it meets
    /// first. A room with no registered background in it answers off [`Collision`] alone, which is
    /// what every trace in the port did before this table existed.
    pub fn line_cross(&self, collision: Option<&Collision>, from: Vec3, to: Vec3) -> Option<Poly> {
        let travel = to - from;
        let length = travel.length_squared();
        let room = collision
            .and_then(|collision| collision.line_cross(from, to))
            .map(|hit| {
                let along = if length > 0.0 {
                    (hit.point - from).dot(travel) / length
                } else {
                    0.0
                };
                (
                    along,
                    Poly {
                        bg: Background::Room(hit.tri),
                        point: hit.point,
                        normal: hit.normal,
                    },
                )
            });
        let mut nearest = room;
        for &slot in &self.live {
            let Some(face) = self.faces.get(slot).copied().flatten() else {
                continue;
            };
            let Some((along, normal)) = face.cross(from, to) else {
                continue;
            };
            if nearest.as_ref().is_none_or(|(best, _)| along < *best) {
                nearest = Some((
                    along,
                    Poly {
                        bg: Background::Registered(slot),
                        point: from + travel * along,
                        normal,
                    },
                ));
            }
        }
        nearest.map(|(_, poly)| poly)
    }

    /// `dBgS_Acch::GroundCheck`, `d_bg_s_acch.cpp:122-145`: the background the rider's own feet
    /// stand on, which is the ride a box publishes for whoever is riding it.
    pub fn standing(&self, feet: Vec3) -> Option<(Entity, Face)> {
        for &slot in &self.live {
            let Some(face) = self.faces.get(slot).copied().flatten() else {
                continue;
            };
            if !face.carries(feet) {
                continue;
            }
            let element = self.slots.get(slot)?;
            if !element.used {
                continue;
            }
            if let Some(actor) = element.actor {
                return Some((actor, face));
            }
        }
        None
    }
}

/// What `dBgS::PushPullCallBack`, `d_bg_s.cpp:542`, hands back: the actor its callback ran on,
/// with the two fields `setPushPullKeepData` reads off it the moment it returns, `pushpull.inc:34`
/// and `pushpull.inc:43-47`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Answer {
    pub actor: Entity,
    /// `pp_actor->current.pos`.
    pub pos: Vec3,
    /// `pp_actor->shape_angle.y`.
    pub shape_yaw: i16,
}

/// Everything a push and pull state needs to hand a label over: the C-button item the heavy bit
/// comes off, and the actors that published a callback. The table itself is a resource of its own,
/// which the guards read without taking this.
#[derive(SystemParam)]
pub struct Call<'w, 's> {
    /// `dComIfGs_getSelectEquip`, `d_com_inf_game.h:968`.
    pub equipped: Res<'w, Equipped>,
    /// `m_chk_element[bg_index].m_actor_ptr`, which the dispatch hands the label to.
    pub actors: Query<'w, 's, &'static mut Movebox>,
}

impl Call<'_, '_> {
    /// `setPushPullKeepData`, `pushpull.inc:28`: hand the label to the actor the table named behind
    /// the polygon. The box's half of the call is `Movebox::pp_call_back`,
    /// `movebox.cpp:1259-1297`, and what comes back is the two fields the caller reads off
    /// it immediately after.
    pub fn send(
        &mut self,
        polys: &PushPulls,
        poly: Poly,
        angle: i16,
        label: u32,
    ) -> Option<Answer> {
        let actor = polys.push_pull_actor(poly)?;
        let mut box_ = self.actors.get_mut(actor).ok()?;
        box_.pp_call_back(angle, label);
        Some(Answer {
            actor,
            pos: box_.pos,
            shape_yaw: box_.shape_angle[1],
        })
    }

    /// `checkPowerGloveEquip`, `player_main.h:1874`, which reads the C-button item the player
    /// has selected. The port keeps no button-to-item selection, so the item is read off the whole
    /// inventory, which is the only place the port holds it.
    pub fn power_glove(&self) -> bool {
        (0..ITEM_SLOTS).any(|slot| self.equipped.item(slot) == ITEM_POWER_BRACELETS)
    }
}
