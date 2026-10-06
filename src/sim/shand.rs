use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::collision::Collision;
use super::combat::{Combat, CutKind};
use super::damage::sword_power;
use super::damage_fx::DamageFxQueue;
use super::damage_resolve::{
    AT_SE_ARROW, AT_SE_NONE, AT_SPL_UNK0, AT_SPL_UNK1, AT_TYPE_FIRE_ARROW, AT_TYPE_ICE_ARROW,
    AT_TYPE_LIGHT_ARROW, AT_TYPE_NORMAL_ARROW, AT_TYPE_SWORD, Atk, Attacker, CcAtInfo, DamageFx,
    HitAudio, NAME_PLAYER, PlayerCut, at_power_check, def_se_set,
};
use super::events::SimEvents;
use super::item::Held;
use super::player::Player;
use super::walk;

pub const SHAND_ACTORS: [&str; 0] = [];

pub const SHAND_ACTOR: &str = "Shand";

pub fn is_shand(name: &str) -> bool {
    SHAND_ACTORS.contains(&name)
}

pub const PARAM_VHLIF_VINE: i32 = 53;
pub const PARAM_KOKIIE: i32 = 1;

const PARAM_LOW: u32 = 0xFF;
const SEG_COUNT: usize = 20;
const SEG_LAST: usize = SEG_COUNT - 1;
const CYL_COUNT: usize = 5;
const CYL_INDEX_LIMIT: usize = CYL_COUNT;

const TAPER_A: usize = 15;
const TAPER_A_SCALE: f32 = 1.0;
const TAPER_A_STEP: f32 = 0.2;
const TAPER_B: usize = 12;
const TAPER_B_STEP: f32 = 0.05;
const TAPER_B_BIAS: i32 = -10;
const TAPER_C_STEP: f32 = 0.03763158;
const TAPER_C_STEP_ALT: f32 = 0.08;
const TAPER_C_BIAS: i32 = 10;

const CYL_RADIUS: f32 = 15.0;
const CYL_HEIGHT: f32 = 200.0;
const SPH_RADIUS: f32 = 80.0;
const SEG_Y_DROP: f32 = -200.0;
const PARKED_Y: f32 = -20000.0;
const STTS_FIRST: u8 = 0xFF;
const STTS_SECOND: u8 = 0xFF;

const G_CM3D_F_INF: f32 = 1000000000.0;

const INIT_HEALTH: i16 = 2;
const HIT_HOLD: i16 = 10;
const HIT_HOLD_INIT: i16 = 30;
const TIMER_LINE_CHK: i16 = 10;
const CREATE_EXECUTIONS: usize = 3;

const HIO_NO: i8 = -1;
const HIO_FIELD_6: i16 = -3;
const HIO_FIELD_8: i16 = 50;
const HIO_FLAG_WINDOW: i16 = 90;

const THICK_KOKIIE: f32 = 15.75;
const THICK_VHLIF_VINE: f32 = 10.5;

const PHASE_COUNT_X: i16 = 1100;
const PHASE_COUNT_Z: i16 = 800;
const PHASE_SEG: i16 = 4000;
const PHASE_COUNT_CUT_X: i16 = 3500;
const PHASE_COUNT_CUT_Y: i16 = 4000;
const PHASE_COUNT_CUT_Z: i16 = 3800;

const ARRIVE_TOLERANCE: f32 = 10.0;
const LINCHK_START: f32 = 50.0;
const LINCHK_END: f32 = 4000.0;
const SPL_OFFSET: f32 = 10.0;
const GND_PROBE_DOWN: f32 = -100.0;
const GND_PROBE_UP: f32 = 200.0;

const SEG_PHASE_STRIDE: i32 = 4;
const SEG_PHASE_COUNT_MASK: i16 = 0b11;
const SEG_PHASE_MOD: i32 = SEG_COUNT as i32;

const LINE_POINTS: usize = SEG_COUNT;
const LINE_EYE_SLOT: i16 = 10;

const ATTN_LOCKON_MISC: u32 = 0x0000_0001;
const ATTN_NONE: u32 = 0;

const SE_OBJ_VINE_S_RECOVER: u32 = 0x5868;
const DEF_SE_PARAM: u32 = 33;

const PA_SIBOUBAKUEN: u16 = 0x0013;
const PA_SIBOUFLASH: u16 = 0x0016;
const HIT_PARTICLE_SCALE: f32 = 0.5;

const FLAG_RAISED: u8 = 1;
const FLAG_RECOVERED: u8 = 2;

const STATE_NORMAL: i16 = 0;
const STATE_STOW: i16 = 1;
const STATE_CUT: i16 = 2;
const SUB_EXTEND: i16 = 0;
const SUB_FOLLOW: i16 = 1;

const HOST_FOUND: i16 = -1;
const HOST_MISSING: i16 = -2;

const DEAD_HEALTH: i16 = 0;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const SWORD_REACH: f32 = 20.0;
const SWORD_RADIUS: f32 = 20.0;
const ARROW_RADIUS: f32 = 5.0;
const ARROW_STRETCH: f32 = 1.25;
const AT_WINDOW_FIRST: u32 = 1;
const AT_WINDOW_LAST: u32 = 6;
const CULL_HALF: f32 = 200.0;

const GROW_LENGTH_RATE: f32 = 0.05;
const GROW_LENGTH_SCALE: f32 = 0.1;
const GROW_LENGTH_MAX: f32 = 1.0;
const GROW_WOBBLE_TARGET: f32 = 10.0;
const GROW_WOBBLE_SCALE: f32 = 0.1;
const GROW_WOBBLE_MAX: f32 = 0.5;
const CUT_LENGTH_TARGET: f32 = 20.0;
const CUT_LENGTH_SCALE: f32 = 0.1;
const CUT_LENGTH_MAX: f32 = 0.5;
const SWAY_Y_MAX: f32 = 0.1;
const SWAY_Z_MAX: f32 = 1.0;
const TIP_APPROACH_SCALE: f32 = 1.0;
const TIP_APPROACH_FACTOR: f32 = 50.0;
const TIP_RATE_MAX: f32 = 0.01;
const TIP_RATE_TARGET: f32 = 1.0;
const TIP_RATE_SCALE: f32 = 1.0;
const RETRACT_Y_TARGET: f32 = -5.0;
const RETRACT_Y_STEP: f32 = 1.0;
const RETRACT_Y_RATE: f32 = 0.1;
const RETRACT_Z_TARGET: f32 = 20.0;
const RETRACT_Z_STEP: f32 = 1.0;
const RETRACT_Z_RATE: f32 = 0.2;
const RETRACT_WOBBLE_SCALE: f32 = 1.0;
const RETRACT_WOBBLE_MAX: f32 = 1.0;
const HIT_SWAY_Y: f32 = 3.0;
const HIT_SWAY_Z: f32 = 40.0;
const HIT_WOBBLE_LO: f32 = 30.0;
const HIT_WOBBLE_SPAN: f32 = 20.0;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct ShandStatic {
    pub hio_set: bool,
    pub hio_no: i8,
    pub field_6: i16,
    pub field_8: i16,
}

impl Default for ShandStatic {
    fn default() -> Self {
        ShandStatic {
            hio_set: false,
            hio_no: HIO_NO,
            field_6: HIO_FIELD_6,
            field_8: HIO_FIELD_8,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct ShandHost {
    pub id: i32,
    pub anchor: Vec3,
    pub flag: u8,
    pub pos: Vec3,
    pub shape_angle_y: i16,
    pub health: i16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShandAt {
    pub obj: Atk,
    pub pos: Vec3,
    pub radius: f32,
}

impl ShandAt {
    pub fn hits(&self, centre: Vec3, radius: f32) -> bool {
        walk::within(self.pos, centre, self.radius + radius)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShandSeg {
    pub pos: Vec3,
    pub field_18: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShandSph {
    pub centre: Vec3,
    pub radius: f32,
    pub at_set: bool,
    pub co_set: bool,
    pub tg_set: bool,
}

impl Default for ShandSph {
    fn default() -> Self {
        ShandSph {
            centre: Vec3::ZERO,
            radius: SPH_RADIUS,
            at_set: true,
            co_set: false,
            tg_set: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShandCyl {
    pub centre: Vec3,
    pub radius: f32,
    pub height: f32,
    pub at_set: bool,
    pub co_set: bool,
    pub tg_set: bool,
}

impl Default for ShandCyl {
    fn default() -> Self {
        ShandCyl {
            centre: Vec3::ZERO,
            radius: CYL_RADIUS,
            height: CYL_HEIGHT,
            at_set: false,
            co_set: true,
            tg_set: true,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Shand {
    pub health: i16,
    pub max_health: i16,
    pub m_execute_count: i16,
    pub m_state: i16,
    pub field_2ba: i16,
    pub field_2bc: [i16; 4],
    pub field_2c4: i16,
    pub field_2c8: Vec3,
    pub field_2d4: Vec3,
    pub field_2f0: f32,
    pub field_2f4: f32,
    pub field_2f8: f32,
    pub field_2fc: f32,
    pub field_300: f32,
    pub field_304: f32,
    pub field_308: i32,
    pub field_310: Vec3,
    pub field_314: u8,
    pub field_318: i16,
    pub field_31c: [ShandSeg; SEG_COUNT],
    pub line_pos: [Vec3; LINE_POINTS],
    pub line_size: [f32; LINE_POINTS],
    pub sph: ShandSph,
    pub cyl: [ShandCyl; CYL_COUNT],
    pub stts_csprm: u8,
    pub stts_ccol: u8,
    pub ground_y: f32,
    pub eye_pos: Vec3,
    pub attention_flags: u32,
    pub attention_pos: Vec3,
    pub has_hio: bool,
    pub host_state: i16,
    pub low: i32,
    pub seed: [i32; 3],
}

impl Shand {
    pub fn spawn(parameters: u32) -> Self {
        let low = (parameters & PARAM_LOW) as i32;
        let field_304 = if low == PARAM_KOKIIE {
            THICK_KOKIIE
        } else {
            THICK_VHLIF_VINE
        };
        let mut shand = Shand {
            health: INIT_HEALTH,
            max_health: INIT_HEALTH,
            m_execute_count: 0,
            m_state: STATE_NORMAL,
            field_2ba: SUB_FOLLOW,
            field_2bc: [0; 4],
            field_2c4: HIT_HOLD_INIT,
            field_2c8: Vec3::ZERO,
            field_2d4: Vec3::ZERO,
            field_2f0: 1.0,
            field_2f4: 0.0,
            field_2f8: 0.0,
            field_2fc: 0.0,
            field_300: 0.0,
            field_304,
            field_308: 0,
            field_310: Vec3::ZERO,
            field_314: 0,
            field_318: TIMER_LINE_CHK,
            field_31c: [ShandSeg::default(); SEG_COUNT],
            line_pos: [Vec3::ZERO; LINE_POINTS],
            line_size: [0.0; LINE_POINTS],
            sph: ShandSph::default(),
            cyl: [ShandCyl::default(); CYL_COUNT],
            stts_csprm: STTS_FIRST,
            stts_ccol: STTS_SECOND,
            ground_y: 0.0,
            eye_pos: Vec3::ZERO,
            attention_flags: ATTN_NONE,
            attention_pos: Vec3::ZERO,
            has_hio: false,
            host_state: HOST_MISSING,
            low,
            seed: [RND_SEED; 3],
        };
        shand.m_execute_count = shand.rnd_f(10000.0) as i16;
        let mut actor = Actor::default();
        let mut env = Env::bare();
        for _ in 0..CREATE_EXECUTIONS {
            shand.execute(&mut actor, &mut env);
        }
        shand
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let mut lo = Vec3::splat(f32::MAX);
        let mut hi = Vec3::splat(f32::MIN);
        for seg in self.field_31c.iter() {
            lo = lo.min(seg.pos);
            hi = hi.max(seg.pos);
        }
        lo = lo.min(self.sph.centre);
        hi = hi.max(self.sph.centre);
        for cyl in self.cyl.iter() {
            lo = lo.min(cyl.centre);
            hi = hi.max(cyl.centre);
        }
        let pad = CYL_RADIUS + SPH_RADIUS + CULL_HALF;
        (
            [lo.x - pad, lo.y - pad, lo.z - pad],
            [hi.x + pad, hi.y + pad, hi.z + pad],
        )
    }

    pub fn drawn(&self) -> bool {
        true
    }

    fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = (self.seed[0]) as f32 / RND_SCALE[0]
            + (self.seed[1]) as f32 / RND_SCALE[1]
            + (self.seed[2]) as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        self.rnd() * range
    }

    pub fn execute(&mut self, actor: &mut Actor, env: &mut Env<'_>) {
        self.m_execute_count = self.m_execute_count.wrapping_add(1);
        for j in 0..4 {
            if self.field_2bc[j] != 0 {
                self.field_2bc[j] -= 1;
            }
        }
        if self.field_2c4 != 0 {
            self.field_2c4 -= 1;
        }
        self.hand_move(actor, env);
    }

    fn hand_move(&mut self, actor: &mut Actor, env: &mut Env<'_>) {
        let host = env.host;
        self.host_state = if host.is_some() {
            HOST_FOUND
        } else {
            HOST_MISSING
        };
        if let Some(host) = host {
            self.field_310 = host.anchor;
            actor.pos = self.field_310;
            actor.angle[1] = actor.old.angle[1].wrapping_add(host.shape_angle_y);
            match self.m_state {
                STATE_NORMAL => {
                    self.normal(actor, env);
                    self.attention_flags = ATTN_LOCKON_MISC;
                    if host.health == DEAD_HEALTH {
                        self.m_state = STATE_STOW;
                        self.field_2ba = SUB_EXTEND;
                    }
                }
                STATE_STOW => {
                    if self.low != PARAM_KOKIIE {
                        let mut chk = actor.pos;
                        chk.y += GND_PROBE_DOWN;
                        self.ground_y = env.ground_at(chk);
                        let mut spl = actor.pos;
                        spl.y += GND_PROBE_UP;
                        let spl_ground_y = env.ground_at(spl) + SPL_OFFSET;
                        if spl_ground_y != -G_CM3D_F_INF {
                            self.ground_y = spl_ground_y;
                        }
                    } else {
                        self.ground_y = -G_CM3D_F_INF;
                    }
                    self.m_state = STATE_CUT;
                    self.cut(actor, env);
                }
                STATE_CUT => {
                    self.cut(actor, env);
                }
                _ => {}
            }
        }

        for i in 0..LINE_POINTS {
            self.line_pos[i] = self.field_31c[i].pos;
            self.line_size[i] = self.field_31c[i].field_18;
        }

        let eye_index = (HIO_FIELD_6 + LINE_EYE_SLOT) as usize;
        self.eye_pos = self.line_pos[eye_index];
        self.attention_pos = self.eye_pos;

        let mut is_hit = false;
        let mut hit_obj: Option<Atk> = None;
        let mut hit_pos: Option<Vec3> = None;
        if self.field_2bc[1] == 0 && self.m_state == STATE_NORMAL {
            self.sph.centre = self.eye_pos;
            self.cyl[0].centre = actor.pos;
            for i in 0..CYL_INDEX_LIMIT {
                if self.field_2c4 == 0
                    && let Some((atk, pos)) = self.cyl[i].tg_hit(env.hits)
                {
                    hit_obj = Some(atk);
                    hit_pos = Some(pos);
                    is_hit = true;
                    break;
                }
                if i > 0 {
                    let raw = i32::from(self.m_execute_count & SEG_PHASE_COUNT_MASK)
                        + i as i32 * SEG_PHASE_STRIDE;
                    let seg_idx = (raw % SEG_PHASE_MOD) as usize;
                    let mut centre = self.line_pos[seg_idx];
                    centre.y += SEG_Y_DROP;
                    self.cyl[i].centre = centre;
                }
            }
        } else {
            let centre = Vec3::new(0.0, PARKED_Y, 0.0);
            self.sph.centre = centre;
            for i in 0..CYL_INDEX_LIMIT {
                self.cyl[i].centre = centre;
            }
        }

        let sph_hit = if is_hit {
            None
        } else if self.field_2c4 == 0 {
            self.sph.tg_hit(env.hits)
        } else {
            None
        };
        if is_hit || sph_hit.is_some() {
            if !is_hit {
                hit_obj = sph_hit.as_ref().map(|(atk, _)| atk.clone());
                hit_pos = sph_hit.map(|(_, pos)| pos);
            }
            let fallback = PlayerCut::default();
            let cut = env.cut.unwrap_or(&fallback);
            let mut at_info = CcAtInfo {
                mp_obj: hit_obj.clone(),
                mp_actor: None,
                m_damage: 0,
                mb_dead: false,
                m_resulting_attack_type: 0,
                m0c: [0; 3],
                m_pl_cut_bit: 0,
                p_particle_pos: hit_pos,
                m_hit_sound_id: 0,
            };
            at_info.mp_actor = at_power_check(&mut at_info, cut);
            self.field_2c4 = HIT_HOLD;
            if let Some(attacker) = at_info.mp_actor.clone() {
                let obj = at_info.mp_obj.clone();
                if let Some(audio) = env.audio.as_deref_mut()
                    && let Some(events) = env.events.as_deref_mut()
                    && let Some(obj) = obj.as_ref()
                {
                    def_se_set(obj, DEF_SE_PARAM, audio, events);
                }
                if self.host_state == HOST_FOUND {
                    let stat = env.stat;
                    self.m_state = STATE_STOW;
                    self.field_2ba = SUB_EXTEND;
                    self.field_2bc[0] = stat.field_8;
                    self.field_2bc[1] = stat.field_8.wrapping_add(HIO_FLAG_WINDOW);
                    self.field_2f8 = HIT_SWAY_Y;
                    self.field_2fc = HIT_SWAY_Z;
                    self.field_300 = self.rnd_f(HIT_WOBBLE_SPAN) + HIT_WOBBLE_LO;
                    self.field_314 = FLAG_RAISED;
                    let scale = Vec3::splat(HIT_PARTICLE_SCALE);
                    if let Some(fx) = env.fx.as_deref_mut() {
                        fx.push(DamageFx::Particle {
                            id: PA_SIBOUBAKUEN,
                            pos: attacker.pos,
                            angle: None,
                            scale: Some(scale),
                        });
                        fx.push(DamageFx::Particle {
                            id: PA_SIBOUFLASH,
                            pos: attacker.pos,
                            angle: None,
                            scale: Some(scale),
                        });
                    }
                }
            }
        }
    }

    fn normal(&mut self, actor: &Actor, env: &mut Env<'_>) {
        if self.field_318 != 0 {
            let mut chk_start = actor.pos;
            let mut chk_end = actor.pos;
            chk_start.y += LINCHK_START;
            chk_end.y += LINCHK_END;
            if let Some(cross) = env.line_cross(chk_start, chk_end) {
                self.field_31c[SEG_LAST].pos = cross;
                self.field_2d4 = cross;
                self.field_2c8 = cross;
            }
            self.field_318 -= 1;
        }

        match self.field_2ba {
            SUB_EXTEND => {
                if (self.field_31c[SEG_LAST].pos.y - self.field_2d4.y).abs() < ARRIVE_TOLERANCE {
                    self.field_2ba = SUB_FOLLOW;
                    self.field_314 = FLAG_RECOVERED;
                }
                self.field_2d4 = self.field_2c8;
            }
            SUB_FOLLOW => {
                self.field_2d4 = self.field_2c8;
            }
            _ => {}
        }

        let y_diff = (actor.old.pos.y - self.field_2c8.y).abs();
        add_calc2(
            &mut self.field_2f4,
            y_diff * GROW_LENGTH_RATE,
            GROW_LENGTH_SCALE,
            GROW_LENGTH_MAX,
        );
        add_calc2(
            &mut self.field_300,
            GROW_WOBBLE_TARGET,
            GROW_WOBBLE_SCALE,
            GROW_WOBBLE_MAX,
        );
        self.control1(actor);
        self.control2();
        self.control3();
    }

    fn cut(&mut self, actor: &Actor, env: &mut Env<'_>) {
        add_calc2(
            &mut self.field_2f4,
            CUT_LENGTH_TARGET,
            CUT_LENGTH_SCALE,
            CUT_LENGTH_MAX,
        );
        self.cut_control(actor);
        self.cut_control3();
        if self.field_2bc[0] == 0 {
            let live = env.host.is_some_and(|h| h.health != DEAD_HEALTH);
            if live {
                self.m_state = STATE_NORMAL;
                self.field_2ba = SUB_EXTEND;
                self.field_2f0 = 0.0;
                self.field_300 = 0.0;
                if let Some(audio) = env.audio.as_deref_mut()
                    && let Some(events) = env.events.as_deref_mut()
                {
                    audio.se_start(events, SE_OBJ_VINE_S_RECOVER, 0);
                }
            }
        }
    }

    fn control1(&mut self, actor: &Actor) {
        self.field_31c[0].pos = actor.pos;
        let base = Vec3::new(0.0, self.field_2f8, self.field_2fc);
        let sway = turned(actor.angle[1], base);
        add_calc2(&mut self.field_2f8, 0.0, 1.0, SWAY_Y_MAX);
        add_calc2(&mut self.field_2fc, 0.0, 1.0, SWAY_Z_MAX);
        let seg_local = Vec3::new(0.0, self.field_2f8, self.field_2f4);
        let amplitude = self.field_300;
        let count = self.m_execute_count;
        for i in 1..SEG_COUNT {
            let idx = i as i16;
            let phase_x = count
                .wrapping_mul(PHASE_COUNT_X)
                .wrapping_add(idx.wrapping_mul(PHASE_SEG));
            let phase_z = count
                .wrapping_mul(PHASE_COUNT_Z)
                .wrapping_add(idx.wrapping_mul(PHASE_SEG));
            let wobble_x = amplitude * ssin(phase_x);
            let wobble_z = amplitude * scos(phase_z);
            let factor = if i < TAPER_A {
                TAPER_A_SCALE
            } else {
                TAPER_A_SCALE - (i as f32 - TAPER_A as f32) * TAPER_A_STEP
            };
            let prev = self.field_31c[i - 1].pos;
            let cur = self.field_31c[i].pos;
            let fx = cur.x - prev.x + sway.x * factor + wobble_x * factor;
            let fy = cur.y - prev.y + sway.y;
            let fz = cur.z - prev.z + sway.z * factor + wobble_z * factor;
            let yaw = atan2s(fx, fz);
            let pitch = atan2s(fy, (fx * fx + fz * fz).sqrt()).wrapping_neg();
            let step = link_pos(yaw, pitch, seg_local);
            self.field_31c[i].pos = prev + step;
        }
    }

    fn control2(&mut self) {
        let rel = Vec3::new(0.0, 0.0, self.field_2f4);
        let tip_rate = self.field_2f0 * TIP_APPROACH_FACTOR;
        let tip = self.field_31c[SEG_LAST].pos;
        self.field_31c[SEG_LAST].pos = Vec3::new(
            chase(tip.x, self.field_2d4.x, TIP_APPROACH_SCALE, tip_rate),
            chase(tip.y, self.field_2d4.y, TIP_APPROACH_SCALE, tip_rate),
            chase(tip.z, self.field_2d4.z, TIP_APPROACH_SCALE, tip_rate),
        );
        add_calc2(
            &mut self.field_2f0,
            TIP_RATE_TARGET,
            TIP_RATE_SCALE,
            TIP_RATE_MAX,
        );
        for i in (1..SEG_LAST).rev() {
            let ahead = self.field_31c[i + 1].pos;
            let delta = self.field_31c[i].pos - ahead;
            let yaw = atan2s(delta.x, delta.z);
            let pitch = atan2s(delta.y, (delta.x * delta.x + delta.z * delta.z).sqrt())
                .wrapping_neg();
            let abs_offset = link_pos(yaw, pitch, rel);
            self.field_31c[i].pos = ahead + abs_offset;
        }
    }

    fn control3(&mut self) {
        let thickness = self.field_304;
        for counter in 0..SEG_COUNT {
            self.field_31c[counter].field_18 = if counter < TAPER_B {
                thickness
            } else {
                thickness
                    * (1.0 - (counter as i32 + TAPER_B_BIAS) as f32 * TAPER_B_STEP)
            };
        }
    }

    fn cut_control(&mut self, actor: &Actor) {
        self.field_31c[0].pos = actor.pos;
        let base = Vec3::new(0.0, self.field_2f8, self.field_2fc);
        let sway = turned(actor.angle[1], base);
        add_calc2(
            &mut self.field_2f8,
            RETRACT_Y_TARGET,
            RETRACT_Y_STEP,
            RETRACT_Y_RATE,
        );
        add_calc2(
            &mut self.field_2fc,
            RETRACT_Z_TARGET,
            RETRACT_Z_STEP,
            RETRACT_Z_RATE,
        );
        add_calc0(&mut self.field_300, RETRACT_WOBBLE_SCALE, RETRACT_WOBBLE_MAX);
        let seg_local = Vec3::new(0.0, self.field_2f8, self.field_2f4);
        let amplitude = self.field_300;
        let count = self.m_execute_count;
        for i in 1..SEG_COUNT {
            let idx = i as i16;
            let phase_x = count
                .wrapping_mul(PHASE_COUNT_CUT_X)
                .wrapping_add(idx.wrapping_mul(PHASE_SEG));
            let phase_y = count
                .wrapping_mul(PHASE_COUNT_CUT_Y)
                .wrapping_add(idx.wrapping_mul(PHASE_SEG));
            let phase_z = count
                .wrapping_mul(PHASE_COUNT_CUT_Z)
                .wrapping_add(idx.wrapping_mul(PHASE_SEG));
            let wobble = Vec3::new(
                amplitude * ssin(phase_x),
                amplitude * scos(phase_y),
                amplitude * scos(phase_z),
            );
            let factor = 1.0 - i as f32 * TAPER_C_STEP;
            let scaled = Vec3::new(sway.x * factor, sway.y, sway.z * factor);
            let prev = self.field_31c[i - 1].pos;
            let cur = self.field_31c[i].pos;
            let fx = cur.x - prev.x + scaled.x + wobble.x;
            let mut fy = cur.y + sway.y + wobble.y;
            if fy < self.ground_y {
                fy = self.ground_y;
            }
            let delta_y = fy - prev.y;
            let fz = cur.z - prev.z + scaled.z + wobble.z;
            let yaw = atan2s(fx, fz);
            let pitch = atan2s(delta_y, (fx * fx + fz * fz).sqrt()).wrapping_neg();
            let step = link_pos(yaw, pitch, seg_local);
            self.field_31c[i].pos = prev + step;
        }
    }

    fn cut_control3(&mut self) {
        let thickness = self.field_304;
        for counter in 0..SEG_COUNT {
            self.field_31c[counter].field_18 = if counter < TAPER_B {
                thickness
            } else {
                thickness * (1.0 - (counter as i32 - TAPER_C_BIAS) as f32 * TAPER_C_STEP_ALT)
            };
        }
    }
}

impl ShandCyl {
    fn tg_hit(&self, hits: &[ShandAt]) -> Option<(Atk, Vec3)> {
        if !self.tg_set {
            return None;
        }
        for at in hits {
            if at.hits(self.centre, self.radius) {
                return Some((at.obj.clone(), at.pos));
            }
        }
        None
    }
}

impl ShandSph {
    fn tg_hit(&self, hits: &[ShandAt]) -> Option<(Atk, Vec3)> {
        if !self.tg_set {
            return None;
        }
        for at in hits {
            if at.hits(self.centre, self.radius) {
                return Some((at.obj.clone(), at.pos));
            }
        }
        None
    }
}

fn ssin(angle: i16) -> f32 {
    walk::from_s16(angle).sin()
}

fn scos(angle: i16) -> f32 {
    walk::from_s16(angle).cos()
}

fn atan2s(y: f32, x: f32) -> i16 {
    if y == 0.0 && x == 0.0 {
        return 0;
    }
    walk::to_s16(y.atan2(x))
}

fn turned(yaw: i16, v: Vec3) -> Vec3 {
    let (s, c) = walk::from_s16(yaw).sin_cos();
    Vec3::new(v.x * c + v.z * s, v.y, v.z * c - v.x * s)
}

fn link_pos(yaw: i16, pitch: i16, v: Vec3) -> Vec3 {
    turned(pitch, turned(yaw, v))
}

fn chase(value: f32, target: f32, scale: f32, max_step: f32) -> f32 {
    let mut out = value;
    add_calc2(&mut out, target, scale, max_step);
    out
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let mut step = scale * (target - *value);
        if step > max_step {
            step = max_step;
        } else if step < -max_step {
            step = -max_step;
        }
        *value += step;
    }
}

fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let mut step = *value * scale;
    if step > max_step {
        step = max_step;
    } else if step < -max_step {
        step = -max_step;
    }
    *value -= step;
}

fn cut_spl(kind: CutKind) -> u8 {
    match kind {
        CutKind::EA | CutKind::EB => AT_SPL_UNK1,
        _ => AT_SPL_UNK0,
    }
}

pub struct Env<'a> {
    pub host: Option<ShandHost>,
    pub hits: &'a [ShandAt],
    pub collision: Option<&'a Collision>,
    pub cut: Option<&'a PlayerCut>,
    pub stat: ShandStatic,
    pub audio: Option<&'a mut HitAudio>,
    pub events: Option<&'a mut SimEvents>,
    pub fx: Option<&'a mut DamageFxQueue>,
}

impl Env<'_> {
    fn bare() -> Self {
        Env {
            host: None,
            hits: &[],
            collision: None,
            cut: None,
            stat: ShandStatic::default(),
            audio: None,
            events: None,
            fx: None,
        }
    }

    fn ground_at(&self, pos: Vec3) -> f32 {
        match self.collision {
            Some(collision) => collision.ground_at(pos.x, pos.z).unwrap_or(-G_CM3D_F_INF),
            None => -G_CM3D_F_INF,
        }
    }

    fn line_cross(&self, _start: Vec3, _end: Vec3) -> Option<Vec3> {
        None
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    combat: Res<'w, Combat>,
    held: Res<'w, Held>,
    players: Query<'w, 's, (Entity, &'static Player), Without<Shand>>,
    arrows: Query<'w, 's, (&'static Actor, &'static arrow::Arrow), Without<Shand>>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut stat: ResMut<ShandStatic>,
    mut audio: ResMut<HitAudio>,
    mut events: ResMut<SimEvents>,
    mut fx: ResMut<DamageFxQueue>,
    mut hosts: Query<&mut ShandHost, Without<Shand>>,
    mut shands: Query<(Entity, &mut Actor, &mut Shand), (With<Shand>, Without<Player>)>,
) {
    let mut hits: Vec<ShandAt> = Vec::new();
    if watch.combat.active
        && watch.combat.frame >= AT_WINDOW_FIRST
        && watch.combat.frame < AT_WINDOW_LAST
    {
        for (_, body) in watch.players.iter() {
            hits.push(ShandAt {
                obj: Atk {
                    at_type: AT_TYPE_SWORD,
                    at_se: AT_SE_NONE,
                    at_spl: cut_spl(watch.combat.kind),
                    at_atp: sword_power(watch.held.item, watch.combat.soup) as u8,
                    ac: Some(Attacker {
                        name: NAME_PLAYER.to_owned(),
                        pos: body.pos,
                        speed: Vec3::ZERO,
                        bomb_state: 0,
                        tsubo_type: 0,
                    }),
                },
                pos: body.pos + Vec3::new(0.0, 0.0, SWORD_REACH),
                radius: SWORD_RADIUS,
            });
        }
    }
    for (_, shaft) in watch.arrows.iter() {
        if shaft.phase != arrow::Phase::Flying {
            continue;
        }
        let at_type = match shaft.kind {
            arrow::TYPE_LIGHT => AT_TYPE_LIGHT_ARROW,
            arrow::TYPE_FIRE => AT_TYPE_FIRE_ARROW,
            arrow::TYPE_ICE => AT_TYPE_ICE_ARROW,
            _ => AT_TYPE_NORMAL_ARROW,
        };
        let tip = shaft.pos + shaft.speed * ARROW_STRETCH;
        let centre = (shaft.pos + tip) * 0.5;
        let reach = ARROW_RADIUS + (tip - shaft.pos).length() * 0.5;
        hits.push(ShandAt {
            obj: Atk {
                at_type,
                at_se: AT_SE_ARROW,
                at_spl: AT_SPL_UNK0,
                at_atp: arrow::POWER as u8,
                ac: Some(Attacker {
                    name: NAME_PLAYER.to_owned(),
                    pos: shaft.pos,
                    speed: shaft.speed,
                    bomb_state: 0,
                    tsubo_type: 0,
                }),
            },
            pos: centre,
            radius: reach,
        });
    }

    for (_, mut actor, mut shand) in shands.iter_mut() {
        let actor = &mut *actor;
        let shand = &mut *shand;
        if !stat.hio_set {
            stat.hio_set = true;
            shand.has_hio = true;
        }
        let host = hosts
            .iter_mut()
            .find(|h| h.id == shand.field_308)
            .map(|h| *h);
        let mut env = Env {
            host,
            hits: &hits,
            collision: watch.collision.as_deref(),
            cut: None,
            stat: *stat,
            audio: Some(&mut *audio),
            events: Some(&mut *events),
            fx: Some(&mut *fx),
        };
        shand.execute(actor, &mut env);
        if shand.host_state == HOST_FOUND {
            for mut h in hosts.iter_mut() {
                if h.id == shand.field_308 {
                    h.flag = shand.field_314;
                }
            }
        }
    }
}
