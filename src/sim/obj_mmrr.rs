use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::contact::{GObjInf, GStts, ObjSet};
use super::contact_run::{ContactBodies, ContactObj};
use super::damage_fx::{DamageParticle, DamageParticles};
use super::damage_resolve::AT_TYPE_LIGHT;
use super::events::{SimEvent, SimEvents};
use super::shape::{Shape, tri_line_cross, tri_plane};
use super::taglight::TagLight;
use super::walk;

pub const OBJ_MMRR_ACTOR: &str = "Obj_Mmrr";

pub const OBJ_MMRR_ACTORS: [&str; 0] = [];

pub fn is_obj_mmrr(name: &str) -> bool {
    OBJ_MMRR_ACTORS.contains(&name)
}

pub const ARC_NAME: &str = "Mmirror";

pub const PRM_WIDTH: u32 = 0;
pub const PRM_MASK: u32 = 0;

pub const HEAP_SIZE: u32 = 0x1a80;
pub const MODEL_HEAP: u32 = 0x80000;
pub const MODEL_FLAGS: u32 = 0x11000222;

pub const TRI_COUNT: usize = 5;

pub const L_ATTR_0XB4: f32 = 445.0;
pub const L_ATTR_0XB8: f32 = 0.0;
pub const L_ATTR_0XBC: i16 = -2000;
pub const L_ATTR_0XC0: Vec3 = Vec3::new(0.0, 320.0, 40.0);
pub const L_ATTR_RADIUS: f32 = 120.0;
pub const L_ATTR_0XD0: f32 = 10000.0;

pub const L_ATTR_TRI: [[Vec3; 3]; TRI_COUNT] = [
    [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(-50.0, -230.0, 0.0),
        Vec3::new(50.0, -230.0, 0.0),
    ],
    [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(-60.0, -20.0, 0.0),
        Vec3::new(-90.0, -80.0, 0.0),
    ],
    [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(90.0, -80.0, 0.0),
        Vec3::new(60.0, -20.0, 0.0),
    ],
    [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(-90.0, -80.0, 0.0),
        Vec3::new(-50.0, -230.0, 0.0),
    ],
    [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(50.0, -230.0, 0.0),
        Vec3::new(90.0, -80.0, 0.0),
    ],
];

pub const CPS_SRC_RADIUS: f32 = 50.0;
pub const CPS_LOCAL_END: Vec3 = Vec3::new(0.0, 0.0, L_ATTR_0XD0);

pub const STTS_TRI_MASK: u32 = 0xff;
pub const STTS_CPS_MASK: u32 = 0xff;

pub const TRI_AT_TYPE: u32 = 0;
pub const TRI_AT_SPRM: u32 = 0;
pub const TRI_TG_TYPE: u32 = AT_TYPE_LIGHT.0;
pub const TRI_TG_SPRM: u32 = 0x01 | 0x08;
pub const TRI_CO_SPRM: u32 = 0;

pub const CPS_AT_TYPE: u32 = AT_TYPE_LIGHT.0;
pub const CPS_AT_SPRM: u32 = 0x01 | 0x02 | 0x04 | 0x08;
pub const CPS_TG_TYPE: u32 = 0;
pub const CPS_TG_SPRM: u32 = 0;
pub const CPS_CO_SPRM: u32 = 0;

pub const SE_MIRROR_REFLECT: u32 = 0x6961;
pub const SE_MIRROR_LIGHT: u32 = 0x7028;

pub const PARTICLE_SASORIMIRROR00: u16 = 0x8294;
pub const PARTICLE_ALPHA: u8 = 0xff;
pub const PARTICLE_SETUP_INFO: i8 = -1;

pub const CHASE_LIT: f32 = 1.0;
pub const CHASE_UNLIT: f32 = 0.0;
pub const CHASE_RATE: f32 = 0.2;
pub const LIT_THRESHOLD: f32 = 0.999;

pub const ATTENTION_Y: f32 = 260.0;

pub const CULL_MIN: Vec3 = Vec3::new(-160.0, -1.0, -160.0);
pub const CULL_HALF_X: f32 = 160.0;
pub const CULL_HALF_Y: f32 = 680.0;
pub const CULL_HALF_Z: f32 = 160.0;

pub const LAMP_BOX: u8 = 0;
pub const LAMP_HALF_X: f32 = 50.0;
pub const LAMP_HALF_Z: f32 = 50.0;
pub const LAMP_FLOOR: f32 = 0.0;
pub const LAMP_TOP: f32 = 100.0;
pub const LAMP_UNSCALE: f32 = 0.5;

pub const NEAR_ZERO_SQUARED: f32 = 8e-11;

pub const TRI_BODY: usize = 0;
pub const CPS_BODY: usize = TRI_COUNT;

fn turned(yaw: i16, v: Vec3) -> Vec3 {
    if yaw == 0 {
        return v;
    }
    let (sin_y, cos_y) = walk::from_s16(yaw).sin_cos();
    Vec3::new(v.x * cos_y + v.z * sin_y, v.y, v.z * cos_y - v.x * sin_y)
}

fn rot_x(v: Vec3, units: i16) -> Vec3 {
    if units == 0 {
        return v;
    }
    let (sin_x, cos_x) = walk::from_s16(units).sin_cos();
    Vec3::new(v.x, v.y * cos_x - v.z * sin_x, v.y * sin_x + v.z * cos_x)
}

fn rot_z(v: Vec3, units: i16) -> Vec3 {
    if units == 0 {
        return v;
    }
    let (sin_z, cos_z) = walk::from_s16(units).sin_cos();
    Vec3::new(v.x * cos_z - v.y * sin_z, v.x * sin_z + v.y * cos_z, v.z)
}

fn zxy_rot_m(v: Vec3, angle: [i16; 3]) -> Vec3 {
    turned(angle[1], rot_x(rot_z(v, angle[2]), angle[0]))
}

fn lamp_inside(actor_pos: Vec3, angle: [i16; 3], point: Vec3) -> bool {
    let mut v = point - actor_pos;
    v = turned(angle[1].wrapping_neg(), v);
    v = rot_x(v, angle[0].wrapping_neg());
    v = rot_z(v, angle[2].wrapping_neg());
    v *= LAMP_UNSCALE;
    v.x.abs() <= LAMP_HALF_X && v.y >= LAMP_FLOOR && v.y <= LAMP_TOP && v.z.abs() <= LAMP_HALF_Z
}

fn cs_to_vec(angle: [i16; 3]) -> Vec3 {
    Vec3::new(angle[0] as f32, angle[1] as f32, angle[2] as f32)
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ObjMmrr {
    pub tris: [[Vec3; 3]; TRI_COUNT],
    pub tri_registered: bool,
    pub tri_tg_on: [bool; TRI_COUNT],
    pub cps_start: Vec3,
    pub cps_end: Vec3,
    pub cps_radius: f32,
    pub cps_at_vec: Vec3,
    pub cps_registered: bool,
    pub eye_pos: Vec3,
    pub bf8: bool,
    pub bf9: bool,
    pub bfc: f32,
    pub c00: f32,
    pub emit_live: bool,
    pub started: bool,
}

impl ObjMmrr {
    pub fn spawn(parameters: u32) -> Self {
        let _reserved = parameters & PRM_MASK;
        Self {
            tris: L_ATTR_TRI,
            tri_registered: false,
            tri_tg_on: [false; TRI_COUNT],
            cps_start: Vec3::ZERO,
            cps_end: CPS_LOCAL_END,
            cps_radius: CPS_SRC_RADIUS,
            cps_at_vec: Vec3::ZERO,
            cps_registered: false,
            eye_pos: Vec3::ZERO,
            bf8: false,
            bf9: false,
            bfc: 0.0,
            c00: 0.0,
            emit_live: false,
            started: false,
        }
    }

    pub fn setup(&mut self, actor: &mut Actor, pos: Vec3) {
        actor.pos = pos;
        self.bf8 = true;
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let (sin_val, cos_val) = if self.bf9 {
            (
                -L_ATTR_0XD0 * walk::from_s16(L_ATTR_0XBC).sin(),
                L_ATTR_0XD0 * walk::from_s16(L_ATTR_0XBC).cos(),
            )
        } else {
            (0.0, 0.0)
        };
        (
            [CULL_MIN.x, CULL_MIN.y, CULL_MIN.z],
            [CULL_HALF_X, sin_val + CULL_HALF_Y, cos_val + CULL_HALF_Z],
        )
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn lit(&self) -> bool {
        self.bf9
    }

    pub fn emitting(&self) -> bool {
        self.emit_live
    }

    pub fn beam_scale(&self, scale: Vec3) -> Vec3 {
        Vec3::new(scale.x, scale.y, scale.z * self.c00)
    }

    pub fn beam_axis(&self) -> Vec3 {
        self.cps_at_vec
    }

    pub fn beam(&self) -> Option<(Vec3, Vec3, f32)> {
        if self.cps_registered {
            Some((self.cps_start, self.cps_end, self.cps_radius))
        } else {
            None
        }
    }

    pub fn beam_at_type(&self) -> u32 {
        CPS_AT_TYPE
    }

    pub fn beam_at_sprm(&self) -> u32 {
        CPS_AT_SPRM
    }

    pub fn occluders(&self) -> &[[Vec3; 3]; TRI_COUNT] {
        &self.tris
    }

    pub fn occluder_tg_on(&self) -> [bool; TRI_COUNT] {
        self.tri_tg_on
    }

    pub fn tri_stts_mask(&self) -> u32 {
        STTS_TRI_MASK
    }

    pub fn cps_stts_mask(&self) -> u32 {
        STTS_CPS_MASK
    }

    pub fn tri_attr(&self) -> (u32, u32, u32, u32) {
        (TRI_AT_TYPE, TRI_AT_SPRM, TRI_TG_TYPE, TRI_TG_SPRM)
    }

    pub fn tri_co_sprm(&self) -> u32 {
        TRI_CO_SPRM
    }

    pub fn cps_attr(&self) -> (u32, u32, u32, u32) {
        (CPS_AT_TYPE, CPS_AT_SPRM, CPS_TG_TYPE, CPS_TG_SPRM)
    }

    pub fn cps_co_sprm(&self) -> u32 {
        CPS_CO_SPRM
    }

    pub fn registered(&self) -> bool {
        self.tri_registered
    }

    pub fn init_cc(&mut self, pos: Vec3, angle: [i16; 3], collision: Option<&Collision>) {
        for i in 0..TRI_COUNT {
            self.tri_tg_on[i] = true;
        }
        self.set_cc_rec_pos(pos, angle);
        self.set_cc_trans_pos(pos, angle, collision);
    }

    pub fn set_cc_rec_pos(&mut self, pos: Vec3, angle: [i16; 3]) {
        for slot in self.tris.iter_mut() {
            for corner in slot.iter_mut() {
                let local = *corner;
                let lifted = rot_x(local, L_ATTR_0XBC) + Vec3::new(0.0, L_ATTR_0XB4, L_ATTR_0XB8);
                *corner = zxy_rot_m(lifted, angle) + pos;
            }
        }
    }

    pub fn set_cc_trans_pos(&mut self, pos: Vec3, angle: [i16; 3], collision: Option<&Collision>) {
        let mut start = zxy_rot_m(L_ATTR_0XC0, angle);
        let mut end = zxy_rot_m(L_ATTR_0XC0 + CPS_LOCAL_END, angle);
        if let Some(collision) = collision {
            let cross = collision.slide(pos + start, pos + end);
            if cross != pos + end {
                end = cross - pos;
            }
        }
        start += pos;
        end += pos;
        self.cps_start = start;
        self.cps_end = end;
        self.cps_radius = L_ATTR_RADIUS;
        let delta = end - start;
        if delta.length_squared() >= NEAR_ZERO_SQUARED {
            self.cps_at_vec = delta / delta.length();
        }
        self.c00 = (start - end).length() / L_ATTR_0XD0;
    }

    pub fn chk_light(&self, lamps: &[(Vec3, [i16; 3])]) -> bool {
        let mut ret = false;
        if lamps
            .iter()
            .any(|(pos, angle)| lamp_inside(*pos, *angle, self.eye_pos))
        {
            ret = true;
        } else {
            for (pos, _) in lamps.iter() {
                for tri in self.tris.iter() {
                    if let Some(hit) = tri_line_cross(self.eye_pos, *pos, *tri) {
                        let to_light = *pos - hit;
                        let reflect = if to_light.length_squared() < NEAR_ZERO_SQUARED {
                            Vec3::ZERO
                        } else {
                            to_light / to_light.length()
                        };
                        let (normal, _) = tri_plane(*tri);
                        if normal.dot(reflect) < 0.0 {
                            ret = true;
                        }
                    }
                }
            }
        }
        ret
    }

    pub fn eff_start(&mut self, pos: Vec3, angle: [i16; 3]) -> DamageParticle {
        self.emit_live = true;
        DamageParticle {
            id: PARTICLE_SASORIMIRROR00,
            pos,
            angle: Some(cs_to_vec(angle)),
            scale: None,
        }
    }

    pub fn eff_stop(&mut self) {
        self.emit_live = false;
    }

    pub fn eff_remove(&mut self) {
        self.emit_live = false;
    }

    pub fn register_cc(&mut self) {
        self.tri_registered = true;
        self.cps_registered = self.bf9;
    }

    fn tri_obj(&self) -> ContactObj {
        let (at_type, at_sprm, tg_type, tg_sprm) = self.tri_attr();
        ContactObj {
            set: ObjSet {
                at_sprm,
                at_type,
                at_atp: 0,
                tg_sprm,
                tg_type,
                co_sprm: self.tri_co_sprm(),
                ..ObjSet::default()
            },
            g_stts: Some(GStts::default()),
            gobj_inf: Some(GObjInf::default()),
        }
    }

    fn cps_obj(&self) -> ContactObj {
        let (at_type, at_sprm, tg_type, tg_sprm) = self.cps_attr();
        ContactObj {
            set: ObjSet {
                at_sprm,
                at_type,
                at_atp: 0,
                tg_sprm,
                tg_type,
                co_sprm: self.cps_co_sprm(),
                ..ObjSet::default()
            },
            g_stts: Some(GStts::default()),
            gobj_inf: Some(GObjInf::default()),
        }
    }

    pub fn init_cc_bodies(&self, bodies: &mut ContactBodies) {
        bodies.shapes.clear();
        bodies.objs.clear();
        for index in 0..TRI_COUNT {
            let tri = self.tris[index];
            bodies.add(Shape::triangle(tri[0], tri[1], tri[2]), self.tri_obj());
        }
        bodies.add(self.cps_shape(), self.cps_obj());
    }

    pub fn sync_cc_bodies(&self, bodies: &mut ContactBodies) {
        if bodies.shapes.bodies.len() != TRI_COUNT + 1 || bodies.objs.len() != TRI_COUNT + 1 {
            self.init_cc_bodies(bodies);
        }
        for (index, slot) in self.tris.iter().enumerate() {
            if let Some(body) = bodies.shapes.body_mut(TRI_BODY + index) {
                body.shape.set_verts(slot[0], slot[1], slot[2]);
            }
        }
        if self.cps_registered {
            if let Some(body) = bodies.shapes.body_mut(CPS_BODY) {
                body.shape = self.cps_shape();
            }
        } else if bodies.shapes.bodies.len() == TRI_COUNT + 1 {
            bodies.shapes.remove(CPS_BODY);
            bodies.objs.pop();
        }
    }

    fn cps_shape(&self) -> Shape {
        let delta = self.cps_end - self.cps_start;
        Shape::capsule(
            (self.cps_start + self.cps_end) * 0.5,
            self.cps_radius,
            delta.length(),
        )
    }

    pub fn execute(
        &mut self,
        actor: &Actor,
        collision: Option<&Collision>,
        lamps: &[(Vec3, [i16; 3])],
        events: &mut SimEvents,
        particles: &mut DamageParticles,
    ) {
        self.eye_pos = Vec3::new(actor.pos.x, actor.pos.y + ATTENTION_Y, actor.pos.z);
        if self.chk_light(lamps) {
            walk::chase_f(&mut self.bfc, CHASE_LIT, CHASE_RATE);
        } else {
            walk::chase_f(&mut self.bfc, CHASE_UNLIT, CHASE_RATE);
        }
        let old_bf9 = self.bf9;
        self.bf9 = self.bfc > LIT_THRESHOLD;
        if self.bf8 {
            self.set_cc_rec_pos(actor.pos, actor.shape_angle);
        }
        if self.bf8 || self.bf9 {
            self.set_cc_trans_pos(actor.pos, actor.shape_angle, collision);
        }
        if self.bf9 {
            if !old_bf9 {
                events.list.push(SimEvent::Se(SE_MIRROR_REFLECT));
                let particle = self.eff_start(actor.pos, actor.shape_angle);
                particles.items.push(particle);
            }
            events.list.push(SimEvent::Se(SE_MIRROR_LIGHT));
        } else if old_bf9 {
            self.eff_stop();
        }
        self.register_cc();
        self.bf8 = false;
    }
}

pub fn control(
    // Mirrors are driven, then their contact bodies are synced, so the two
    // passes never run at the same time. `ParamSet` states that; two plain
    // queries would be a B0001 because both touch `ObjMmrr`.
    mut passes: ParamSet<(
        Query<(&Actor, &mut ObjMmrr)>,
        Query<(&ObjMmrr, &mut ContactBodies)>,
    )>,
    lamps: Query<(&Actor, &TagLight)>,
    collision: Option<Res<Collision>>,
    mut events: ResMut<SimEvents>,
    mut particles: ResMut<DamageParticles>,
) {
    let lit_lamps: Vec<(Vec3, [i16; 3])> = lamps
        .iter()
        .filter(|(_, spot)| spot.lit && spot.kind == LAMP_BOX)
        .map(|(actor, _)| (actor.pos, actor.angle))
        .collect();
    let bg = collision.as_deref();
    for (actor, mut mirror) in passes.p0() {
        if !mirror.started {
            mirror.started = true;
            mirror.init_cc(actor.pos, actor.shape_angle, bg);
        }
        mirror.execute(actor, bg, &lit_lamps, &mut events, &mut particles);
    }
    for (mirror, mut bodies) in passes.p1() {
        mirror.sync_cc_bodies(&mut bodies);
    }
}
