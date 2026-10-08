use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};

use super::actor::{Actor, atan2s};
use super::pirateship::Pirate;
use super::stream::add_calc_angle_s2;
use super::walk;

pub const PIRATE_FLAG_ACTORS: [&str; 0] = [];

pub const PIRATE_FLAG_ACTOR: &str = "Pirate_flag";

pub fn is_pirate_flag(name: &str) -> bool {
    PIRATE_FLAG_ACTORS.contains(&name)
}

pub const GRID: i32 = 5;
pub const VERTS: usize = 25;
pub const LAST_COL: i32 = 4;
pub const LAST_ROW: i32 = 4;

pub const HIO_MNO: i8 = -1;
pub const HIO_M05: u8 = 0;
pub const HIO_M06: u8 = 0;
pub const HIO_M07: u8 = 0;
pub const HIO_M08: i32 = 0x40;
pub const HIO_M0C: f32 = 13.0;
pub const HIO_M10: f32 = 7.0;
pub const HIO_M14: f32 = -3.5;
pub const HIO_M18: f32 = 0.45;
pub const HIO_M1C: f32 = 0.0;

pub const RND_RANGE: f32 = 200.0;
pub const RND_BASE: i32 = 900;
pub const NRM_WOBBLE_SCALE: f32 = 300.0;
pub const HALF_TURN: i16 = -0x8000;
pub const NRM_SNAP_SCALE: f32 = 1.25;
pub const CALC_SCALE: i16 = 5;
pub const CALC_MAX_STEP: i16 = 0xC0;
pub const SHAPE_CALC_SCALE: i16 = 8;
pub const SHAPE_CALC_MAX_STEP: i16 = 0x400;

pub const WIND_TIP_Z: f32 = 0.064;
pub const VELOCITY_DAMP: f32 = 0.875;
pub const NRM_VTX_GAIN: f32 = 900.0;
pub const NRM_VTX_ANGLE_STEP: i32 = -800;

pub const SUB_DIST_COL: f32 = 200.0;
pub const SUB_DIST_ROW: f32 = 100.0;
pub const SUB_DIST_DIAG: f32 = 223.6068;

pub const FLAG_OFFSET: Vec3 = Vec3::new(0.0, 1000.0, 100.0);
pub const DRAW_TRANS_Z: f32 = 30.0;

pub const CULL_MIN: [f32; 3] = [0.0, -400.0, -30.0];
pub const CULL_MAX: [f32; 3] = [800.0, 400.0, 30.0];

pub const DEG2S_CONSTANT: f32 = 32768.0 / 180.0;
pub const NEAR_ZERO_SQUARE: f32 = 8e-11;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
pub const RND_SEED: i32 = 100;

pub const L_POS: [Vec3; VERTS] = [Vec3::ZERO; VERTS];

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct PirateFlagHio {
    pub m_no: i8,
    pub m05: u8,
    pub m06: u8,
    pub m07: u8,
    pub m08: i32,
    pub m0c: f32,
    pub m10: f32,
    pub m14: f32,
    pub m18: f32,
    pub m1c: f32,
    pub wind_vec: Vec3,
    pub light_vec: Vec3,
}

impl Default for PirateFlagHio {
    fn default() -> Self {
        PirateFlagHio {
            m_no: HIO_MNO,
            m05: HIO_M05,
            m06: HIO_M06,
            m07: HIO_M07,
            m08: HIO_M08,
            m0c: HIO_M0C,
            m10: HIO_M10,
            m14: HIO_M14,
            m18: HIO_M18,
            m1c: HIO_M1C,
            wind_vec: Vec3::ZERO,
            light_vec: Vec3::ZERO,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct PirateFlagShip {
    pub ship: Option<Entity>,
    pub ready: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PirateFlag {
    pub m2a8: i16,
    pub pos: [[Vec3; VERTS]; 2],
    pub nrm: [[Vec3; VERTS]; 2],
    pub back_nrm: [[Vec3; VERTS]; 2],
    pub offset_vec: [Vec3; VERTS],
    pub m878: i16,
    pub m87a: i16,
    pub m87c: i16,
    pub m87e: u8,
    pub m87f: u8,
    pub seed: [i32; 3],
    pub mtx: Mat4,
    pub ship: Option<Entity>,
    pub visible: bool,
    pub created: bool,
    pub health: i32,
    pub max_health: i32,
}

impl PirateFlag {
    pub fn spawn(_parameters: u32) -> Self {
        let mut pos = [[Vec3::ZERO; VERTS]; 2];
        for (i, slot) in pos[0].iter_mut().enumerate() {
            *slot = L_POS[i];
        }
        PirateFlag {
            m2a8: 0,
            pos,
            nrm: [[Vec3::ZERO; VERTS]; 2],
            back_nrm: [[Vec3::ZERO; VERTS]; 2],
            offset_vec: [Vec3::ZERO; VERTS],
            m878: 0,
            m87a: 0,
            m87c: 0,
            m87e: 0,
            m87f: 1,
            seed: [RND_SEED; 3],
            mtx: Mat4::IDENTITY,
            ship: None,
            visible: false,
            created: false,
            health: 0,
            max_health: 0,
        }
    }

    pub fn cur(&self) -> usize {
        (self.m87e & 1) as usize
    }

    pub fn change_current_pos(&mut self) {
        self.m87e ^= 1;
    }

    pub fn escape_nrm_angle(&self) -> i16 {
        self.m87a
    }

    pub fn is_delete(&self) -> bool {
        true
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.visible
    }

    pub fn set_ship(&mut self, ship: Entity) {
        self.ship = Some(ship);
    }

    pub fn set_nrm_mtx(&self) -> Mat4 {
        rot_y(self.m878)
    }

    pub fn set_back_nrm(&mut self) {
        let cur = self.cur();
        let nrm = self.nrm[cur];
        let mut back = self.back_nrm[cur];
        for i in 0..VERTS {
            back[i] = Vec3::ZERO - nrm[i];
        }
        self.back_nrm[cur] = back;
    }

    fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = self.seed[0] as f32 / RND_SCALE[0]
            + self.seed[1] as f32 / RND_SCALE[1]
            + self.seed[2] as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        self.rnd() * range
    }

    fn set_correct_nrm_angle(&mut self, param_0: i16, param_1: f32, hio: &PirateFlagHio) {
        let sample = self.rnd_f(RND_RANGE);
        self.m87c = self.m87c.wrapping_add((sample as i32 + RND_BASE) as i16);
        self.m878 = (NRM_WOBBLE_SCALE * ssin(i32::from(self.m87c))) as i16;

        let temp_r0 = param_0.wrapping_add(HALF_TURN);
        let param_0_int = i32::from(param_0);
        let temp_r26 = (hio.m1c * (1.0 - param_1 * 0.5)) as i16;
        let limit = deg2s(f32::from(temp_r26) * NRM_SNAP_SCALE);

        if i32::from(temp_r0).abs() < i32::from(limit) {
            let var_r4 = if temp_r0 > 0 {
                deg2s(-f32::from(temp_r26))
            } else {
                deg2s(f32::from(temp_r26))
            };
            add_calc_angle_s2(&mut self.m87a, var_r4, CALC_SCALE, CALC_MAX_STEP);
        } else if param_0_int.abs() < i32::from(limit) {
            let var_r4 = if param_0 > 0 {
                deg2s(-f32::from(temp_r26))
            } else {
                deg2s(f32::from(temp_r26))
            };
            add_calc_angle_s2(&mut self.m87a, var_r4, CALC_SCALE, CALC_MAX_STEP);
        } else {
            add_calc_angle_s2(&mut self.m87a, 0, CALC_SCALE, CALC_MAX_STEP);
        }

        self.m878 = self.m878.wrapping_add(self.m87a);
    }

    fn set_nrm_vtx(&self, pos: &[Vec3; VERTS], out: &mut Vec3, col: i32, row: i32, mtx: Mat4) {
        let idx = col + row * GRID;
        let sp_c8 = pos[idx as usize];
        let mut sp_d4 = Vec3::ZERO;

        if col != 0 {
            let tmp = pos[(idx - 1) as usize] - sp_c8;
            if row != 0 {
                let tmp2 = pos[(col + (row - 1) * GRID) as usize] - sp_c8;
                sp_d4 += norm_zp(tmp.cross(tmp2));
            }
            if row != LAST_ROW {
                let tmp2 = pos[(col + (row + 1) * GRID) as usize] - sp_c8;
                sp_d4 += norm_zp(tmp2.cross(tmp));
            }
        }

        if col != LAST_COL {
            let tmp = pos[(1 + idx) as usize] - sp_c8;
            if row != 0 {
                let tmp2 = pos[(col + (row - 1) * GRID) as usize] - sp_c8;
                sp_d4 += norm_zp(tmp2.cross(tmp));
            }
            if row != LAST_ROW {
                let tmp2 = pos[(col + (row + 1) * GRID) as usize] - sp_c8;
                sp_d4 += norm_zp(tmp.cross(tmp2));
            }
        }

        sp_d4 = norm_zp(sp_d4);
        let deg = ssin((col + row) * NRM_VTX_ANGLE_STEP);
        let calc = mtx * rot_y((NRM_VTX_GAIN * deg) as i16);
        let tmp3 = calc.transform_point3(sp_d4);
        *out = norm_zp(tmp3);
    }

    pub fn flag_move(&mut self, angle_y: i16, shape_angle_y: i16, hio: &PirateFlagHio) {
        let from = self.cur();
        let pos = self.pos[from];
        let nrm = self.nrm[from];
        self.change_current_pos();
        let to = self.cur();

        let wind_angle = atan2s(hio.wind_vec.x, hio.wind_vec.z);
        let sum = angle_y.wrapping_add(shape_angle_y);
        let rot = sum.wrapping_sub(wind_angle).wrapping_neg();
        let calc = rot_y(rot);

        let _sp = calc.transform_point3(Vec3::new(0.0, 0.0, WIND_TIP_Z));
        let dest = calc.transform_point3(Vec3::new(1.0, 0.0, 0.0));
        let abs_z = dest.z.abs();

        self.m2a8 = self.m2a8.wrapping_add(hio.m08 as i16);

        let temp_f3 = 0.5 + ssin(i32::from(self.m2a8)) * 0.5;
        let sp2c = Vec3::new(
            0.0,
            0.0,
            hio.m0c * temp_f3 + hio.m10 * (1.0 - temp_f3),
        );

        for i in 0..GRID {
            for j in 0..GRID {
                let idx = (i + j * GRID) as usize;
                self.pos[to][idx] = pos[idx];
                let sp14 = get_cloth_anim_factor(&pos, &nrm, &sp2c, i, j, hio);
                self.offset_vec[idx] += sp14;
                self.offset_vec[idx] *= VELOCITY_DAMP;
                self.pos[to][idx] += self.offset_vec[idx];
            }
        }

        let light_angle = atan2s(hio.light_vec.x, hio.light_vec.z);
        self.set_correct_nrm_angle(light_angle.wrapping_sub(angle_y), abs_z, hio);

        let nrm_mtx = self.set_nrm_mtx();
        let pos_to = self.pos[to];
        let mut nrm_new = self.nrm[to];
        for i in 0..GRID {
            for j in 0..GRID {
                let out = &mut nrm_new[(i + j * GRID) as usize];
                self.set_nrm_vtx(&pos_to, out, j, i, nrm_mtx);
            }
        }
        self.nrm[to] = nrm_new;
        self.set_back_nrm();
    }

    pub fn execute(
        &mut self,
        actor: &mut Actor,
        ship_pos: Vec3,
        ship_shape_angle: [i16; 3],
        hio: &PirateFlagHio,
    ) -> bool {
        if hio.m07 != 0 {
            return true;
        }

        actor.pos = base_tr(ship_pos, ship_shape_angle).transform_point3(FLAG_OFFSET);
        actor.angle = ship_shape_angle;

        let target = atan2s(hio.wind_vec.x, hio.wind_vec.z);
        let target = target.wrapping_sub(actor.angle[1]);
        add_calc_angle_s2(
            &mut actor.shape_angle[1],
            target,
            SHAPE_CALC_SCALE,
            SHAPE_CALC_MAX_STEP,
        );
        self.flag_move(actor.angle[1], actor.shape_angle[1], hio);
        true
    }

    pub fn draw_mtx(&self, actor: &Actor) -> Mat4 {
        let mut mtx = Mat4::from_translation(actor.pos);
        mtx *= rot_y(actor.angle[1].wrapping_add(actor.shape_angle[1]));
        mtx *= rot_x(actor.angle[0]);
        mtx *= rot_z(actor.angle[2]);
        mtx *= Mat4::from_translation(Vec3::new(0.0, 0.0, DRAW_TRANS_Z));
        mtx
    }
}

fn get_cloth_anim_sub_factor(
    p_pos: Vec3,
    p_other: Vec3,
    p_dst: &mut Vec3,
    dist_ideal: f32,
    stiffness: f32,
) {
    let tmp = p_other - p_pos;
    let mut unit = norm_zp(tmp);
    let mut gap = tmp.length() - dist_ideal;
    gap *= stiffness;
    unit *= gap;
    *p_dst += unit;
}

fn get_cloth_anim_factor(
    pos: &[Vec3; VERTS],
    nrm: &[Vec3; VERTS],
    wind: &Vec3,
    col: i32,
    row: i32,
    hio: &PirateFlagHio,
) -> Vec3 {
    let r30 = row * GRID;
    let idx = (col + r30) as usize;
    let sp20 = pos[idx];
    let mut sp14 = nrm[idx] * wind.dot(nrm[idx]);
    sp14.y += hio.m14;

    if col != 0 {
        get_cloth_anim_sub_factor(
            sp20,
            pos[(col - 1 + r30) as usize],
            &mut sp14,
            SUB_DIST_COL,
            hio.m18,
        );
        if row != 0 {
            get_cloth_anim_sub_factor(
                sp20,
                pos[((row - 1) * GRID + col) as usize],
                &mut sp14,
                SUB_DIST_ROW,
                hio.m18,
            );
            get_cloth_anim_sub_factor(
                sp20,
                pos[((row - 1) * GRID + (col - 1)) as usize],
                &mut sp14,
                SUB_DIST_DIAG,
                hio.m18,
            );
        }
        if row != LAST_ROW {
            get_cloth_anim_sub_factor(
                sp20,
                pos[((row + 1) * GRID + col) as usize],
                &mut sp14,
                SUB_DIST_ROW,
                hio.m18,
            );
            get_cloth_anim_sub_factor(
                sp20,
                pos[((row + 1) * GRID + (col - 1)) as usize],
                &mut sp14,
                SUB_DIST_DIAG,
                hio.m18,
            );
        }
        if col != LAST_COL {
            get_cloth_anim_sub_factor(
                sp20,
                pos[(1 + col + r30) as usize],
                &mut sp14,
                SUB_DIST_COL,
                hio.m18,
            );
            if row != 0 {
                get_cloth_anim_sub_factor(
                    sp20,
                    pos[(1 + col + (row - 1) * GRID) as usize],
                    &mut sp14,
                    SUB_DIST_DIAG,
                    hio.m18,
                );
            }
            if row != LAST_ROW {
                get_cloth_anim_sub_factor(
                    sp20,
                    pos[(1 + col + (row + 1) * GRID) as usize],
                    &mut sp14,
                    SUB_DIST_DIAG,
                    hio.m18,
                );
            }
        }
        return sp14;
    }

    if row != 0 && row != LAST_ROW {
        get_cloth_anim_sub_factor(
            sp20,
            pos[(r30 + (col + 1)) as usize],
            &mut sp14,
            SUB_DIST_COL,
            hio.m18,
        );
        if row != 0 {
            get_cloth_anim_sub_factor(
                sp20,
                pos[((row - 1) * GRID + col) as usize],
                &mut sp14,
                SUB_DIST_ROW,
                hio.m18,
            );
            get_cloth_anim_sub_factor(
                sp20,
                pos[((row - 1) * GRID + (col + 1)) as usize],
                &mut sp14,
                SUB_DIST_DIAG,
                hio.m18,
            );
        }
        if row != LAST_ROW {
            get_cloth_anim_sub_factor(
                sp20,
                pos[((row + 1) * GRID + col) as usize],
                &mut sp14,
                SUB_DIST_ROW,
                hio.m18,
            );
            get_cloth_anim_sub_factor(
                sp20,
                pos[((row + 1) * GRID + (col + 1)) as usize],
                &mut sp14,
                SUB_DIST_DIAG,
                hio.m18,
            );
        }
        return sp14;
    }

    Vec3::ZERO
}

fn base_tr(pos: Vec3, shape_angle: [i16; 3]) -> Mat4 {
    let mut mtx = Mat4::from_translation(pos);
    if shape_angle[1] != 0 {
        mtx *= rot_y(shape_angle[1]);
    }
    if shape_angle[0] != 0 {
        mtx *= rot_x(shape_angle[0]);
    }
    if shape_angle[2] != 0 {
        mtx *= rot_z(shape_angle[2]);
    }
    mtx
}

fn deg2s(deg: f32) -> i16 {
    (deg * DEG2S_CONSTANT) as i32 as i16
}

fn ssin(angle: i32) -> f32 {
    walk::from_s16(angle as i16).sin()
}

fn rot_x(angle: i16) -> Mat4 {
    Mat4::from_rotation_x(walk::from_s16(angle))
}

fn rot_y(angle: i16) -> Mat4 {
    Mat4::from_rotation_y(walk::from_s16(angle))
}

fn rot_z(angle: i16) -> Mat4 {
    Mat4::from_rotation_z(walk::from_s16(angle))
}

fn norm_zp(v: Vec3) -> Vec3 {
    if v.length_squared() < NEAR_ZERO_SQUARE {
        return Vec3::ZERO;
    }
    v.normalize()
}

pub fn control(
    hio_res: Res<PirateFlagHio>,
    mut ship_res: ResMut<PirateFlagShip>,
    mut flags: Query<(&mut Actor, &mut PirateFlag), (With<PirateFlag>, Without<Pirate>)>,
    ships: Query<&Actor, (With<Pirate>, Without<PirateFlag>)>,
) {
    let hio = *hio_res;
    for (mut actor, mut flag) in &mut flags {
        let ship = if flag.ship.is_some() {
            flag.ship
        } else {
            ship_res.ship
        };
        let Some(ship_entity) = ship else {
            continue;
        };
        if !flag.created {
            ship_res.ship = Some(ship_entity);
            flag.flag_move(actor.angle[1], actor.shape_angle[1], &hio);
            flag.created = true;
        }
        flag.visible = ship_res.ready;
        let Some(ship_actor) = ships.get(ship_entity).ok() else {
            continue;
        };
        flag.execute(
            &mut actor,
            ship_actor.pos,
            ship_actor.shape_angle,
            &hio,
        );
        flag.mtx = flag.draw_mtx(&actor);
    }
}
