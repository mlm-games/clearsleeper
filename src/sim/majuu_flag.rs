use bevy_ecs::prelude::*;
use glam::{Mat3, Mat4, Vec3};

use super::actor::Actor;
use super::stream::add_calc_angle_s2;
use super::walk;

pub const MAJUU_FLAG_ACTORS: [&str; 3] = ["HcFlag", "MjFlag", "MtFlag"];

pub fn is_majuu_flag(name: &str) -> bool {
    MAJUU_FLAG_ACTORS.contains(&name)
}

pub const PRM_FLAG_SHIFT: u32 = 0;
pub const PRM_FLAG_MASK: u32 = 0xFF;
pub const PRM_TEX_SHIFT: u32 = 0x18;
pub const PRM_TEX_MASK: u32 = 0xFF;

pub const VERT_COUNT: usize = 21;

const HIO_MB_USE_TOON_TEX: bool = true;
const HIO_M05: i8 = 0;
const HIO_M06: i8 = 0;
const HIO_M07: i8 = 0;
const HIO_M08: i8 = 0;
const HIO_MWAVE_SPEED: i16 = 0x400;
const HIO_MWIND_SPEED1: f32 = 20.0;
const HIO_MWIND_SPEED2: f32 = 10.0;
const HIO_MGRAVITY: f32 = -1.25;
const HIO_MSPRING: f32 = 0.45;
const HIO_MFLAG_SCALE: f32 = 1.0;
const HIO_MB_HAS_CUSTOM_ROTATION: bool = false;
const HIO_MROT_X: i32 = -0x5DC;
const HIO_MTRANS_Z: f32 = 50.0;

const CLOTH_NRM_X: f32 = 1.0;
const CLOTH_NRM_Y: f32 = 0.0;
const CLOTH_NRM_Z: f32 = 0.0;
const CLOTH_WAVE_BIAS: f32 = 0.5;
const CLOTH_WIND_POW: f32 = 2.0;
const CLOTH_SPEED_DAMP: f32 = 0.85;
const CLOTH_IDEAL_LONG: f32 = 264.9547;
const CLOTH_IDEAL_SHORT: f32 = 102.0;
const CLOTH_RND_JITTER: f32 = 10.0;
const CLOTH_SET_NRM_VTX_DEG: f32 = -800.0;
const CLOTH_SET_NRM_VTX_GAIN: f32 = 900.0;

const RND_SEED: i32 = 100;
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];

const DEG2S_CONSTANT: f32 = 32768.0 / 180.0;
const NEAR_ZERO_SQUARE: f32 = 8e-11;

const YAW_SCALE: i16 = 8;
const YAW_MAX_STEP: i16 = 0x400;
const PARENT_TRANS_Z: f32 = 6.0;
const TEX_FLAG_TRANS_Z: f32 = 5.0;
const TEX_TYPE_3_SETTLE: u32 = 20;

const FLAG_TYPE_2: u8 = 2;
const FLAG_TYPE_3: u8 = 3;
const FLAG_TYPE_4: u8 = 4;
const FLAG_TYPE_0xFF: u8 = 0xFF;

const SCALE_TEX_TYPE_2: f32 = 2.0;
const SCALE_TEX_TYPE_3: f32 = 1.27;
const SCALE_TEX_TYPE_4: f32 = 0.3;
const SCALE_TEX_TYPE_0xFF: f32 = 0.3;
const SCALE_STEP: f32 = 0.05;
const SCALE_BASE: f32 = 0.3;

const CULL_MIN: [f32; 3] = [-300.0, -1500.0, -100.0];
const CULL_MAX: [f32; 3] = [300.0, 100.0, 1200.0];

const BAND_HIGH_INDEX: i32 = 3;
const BAND_INDEX_6: i32 = 6;
const BAND_INDEX_10: i32 = 10;
const BAND_INDEX_15: i32 = 0xF;
const BAND_COUNT: usize = 6;

const REL_POS_IDX: [[i32; 7]; VERT_COUNT] = [
    [1, 2, -1, -1, -1, -1, -1],
    [3, 4, 2, 0, -1, -1, -1],
    [0, 1, 4, 5, -1, -1, -1],
    [6, 7, 4, 1, -1, -1, -1],
    [1, 3, 7, 8, 5, 2, 1],
    [2, 4, 8, 9, -1, -1, -1],
    [10, 11, 7, 3, -1, -1, -1],
    [3, 6, 11, 12, 8, 4, 3],
    [4, 7, 12, 13, 9, 5, 4],
    [5, 8, 13, 14, -1, -1, -1],
    [15, 16, 11, 6, -1, -1, -1],
    [6, 10, 16, 17, 12, 7, 6],
    [7, 11, 17, 18, 13, 8, 7],
    [8, 12, 18, 19, 14, 9, 8],
    [9, 13, 19, 20, -1, -1, -1],
    [16, 10, -1, -1, -1, -1, -1],
    [17, 11, 10, 15, -1, -1, -1],
    [18, 12, 11, 16, -1, -1, -1],
    [19, 13, 12, 17, -1, -1, -1],
    [20, 14, 13, 18, -1, -1, -1],
    [14, 19, -1, -1, -1, -1, -1],
];

const MAJUU_FLAG_POS: [Vec3; VERT_COUNT] = [Vec3::ZERO; VERT_COUNT];

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Statics {
    pub mb_use_toon_tex: bool,
    pub m05: i8,
    pub m06: i8,
    pub m07: i8,
    pub m08: i8,
    pub wave_speed: i16,
    pub wind_speed1: f32,
    pub wind_speed2: f32,
    pub gravity: f32,
    pub spring: f32,
    pub flag_scale: f32,
    pub has_custom_rotation: bool,
    pub custom_rotation: Vec3,
    pub rot_x: i32,
    pub trans_z: f32,
    pub wind_vec: Vec3,
    pub wind_pow: f32,
    pub monotone: bool,
}

impl Default for Statics {
    fn default() -> Self {
        Statics {
            mb_use_toon_tex: HIO_MB_USE_TOON_TEX,
            m05: HIO_M05,
            m06: HIO_M06,
            m07: HIO_M07,
            m08: HIO_M08,
            wave_speed: HIO_MWAVE_SPEED,
            wind_speed1: HIO_MWIND_SPEED1,
            wind_speed2: HIO_MWIND_SPEED2,
            gravity: HIO_MGRAVITY,
            spring: HIO_MSPRING,
            flag_scale: HIO_MFLAG_SCALE,
            has_custom_rotation: HIO_MB_HAS_CUSTOM_ROTATION,
            custom_rotation: Vec3::ZERO,
            rot_x: HIO_MROT_X,
            trans_z: HIO_MTRANS_Z,
            wind_vec: Vec3::ZERO,
            wind_pow: 0.0,
            monotone: false,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct MajuuFlag {
    pub flag_type: u8,
    pub tex_type: u8,
    pub use_player_tev_str: u8,
    pub flag_scale: f32,
    pub wave: i16,
    pub rotate_y: i16,
    pub m796: i16,
    pub m798: i16,
    pub cur_arr: u8,
    pub m79b: u8,
    pub pos_arr: [[Vec3; VERT_COUNT]; 2],
    pub nrm_arr: [[Vec3; VERT_COUNT]; 2],
    pub nrm_arr_back: [[Vec3; VERT_COUNT]; 2],
    pub speed: [Vec3; VERT_COUNT],
    pub mtx: Mat4,
    pub parent_mtx: Option<Mat4>,
    pub parent_pos: Option<Vec3>,
    pub seed: [i32; 3],
    pub created: bool,
}

impl MajuuFlag {
    pub fn spawn(parameters: u32) -> Self {
        let flag_type = ((parameters >> PRM_FLAG_SHIFT) & PRM_FLAG_MASK) as u8;
        let tex_type = ((parameters >> PRM_TEX_SHIFT) & PRM_TEX_MASK) as u8;
        let mut seed = [RND_SEED; 3];
        let mut pos_arr = [[Vec3::ZERO; VERT_COUNT]; 2];
        let mut nrm_arr = [[Vec3::ZERO; VERT_COUNT]; 2];
        let mut nrm_arr_back = [[Vec3::ZERO; VERT_COUNT]; 2];
        let mut speed = [Vec3::ZERO; VERT_COUNT];
        for i in 0..VERT_COUNT {
            pos_arr[0][i] = MAJUU_FLAG_POS[i];
            nrm_arr[0][i] = Vec3::new(CLOTH_NRM_X, CLOTH_NRM_Y, CLOTH_NRM_Z);
            nrm_arr_back[0][i] = Vec3::ZERO - nrm_arr[0][i];
            speed[i] = Vec3::ZERO;
            if i != 15 && i != 20 {
                pos_arr[0][i].x += rnd_fx(&mut seed, CLOTH_RND_JITTER);
                pos_arr[0][i].y += rnd_fx(&mut seed, CLOTH_RND_JITTER);
                pos_arr[0][i].z += rnd_fx(&mut seed, CLOTH_RND_JITTER);
            }
        }
        let mut flag = MajuuFlag {
            flag_type,
            tex_type,
            use_player_tev_str: 0,
            flag_scale: 1.0,
            wave: 0,
            rotate_y: 0,
            m796: 0,
            m798: 0,
            cur_arr: 0,
            m79b: 1,
            pos_arr,
            nrm_arr,
            nrm_arr_back,
            speed,
            mtx: Mat4::IDENTITY,
            parent_mtx: None,
            parent_pos: None,
            seed,
            created: false,
        };
        if flag.tex_type == 0 {
            if flag.flag_type == FLAG_TYPE_2 {
                flag.flag_scale = SCALE_TEX_TYPE_2;
            } else if flag.flag_type == FLAG_TYPE_3 {
                flag.flag_scale = SCALE_TEX_TYPE_3;
                flag.use_player_tev_str = 1;
            } else if flag.flag_type == FLAG_TYPE_4 {
                flag.flag_scale = SCALE_TEX_TYPE_4;
            } else {
                flag.flag_scale = 1.0;
            }
        } else if flag.flag_type == FLAG_TYPE_0xFF {
            flag.flag_scale = SCALE_TEX_TYPE_0xFF;
        } else {
            flag.flag_scale = flag.flag_type as f32 * SCALE_STEP + SCALE_BASE;
        }
        flag
    }

    pub fn set_base_scale(&mut self, scale: f32) {
        self.flag_scale = scale;
    }

    pub fn set_other_matrix(&mut self, mtx: Mat4) {
        self.parent_mtx = Some(mtx);
    }

    pub fn set_other_offset(&mut self, pos: Vec3) {
        self.parent_pos = Some(pos);
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn change_current_pos(&mut self) {
        self.cur_arr ^= 1;
    }

    fn create_post(&mut self, pos: Vec3, angle: [i16; 3], hio: &Statics) -> i16 {
        self.set_mtx(pos, angle, hio);
        let yaw = atan2s(hio.wind_vec.x, hio.wind_vec.z);
        if self.tex_type == 3 {
            for _ in 0..TEX_TYPE_3_SETTLE {
                self.majuu_flag_move(angle, hio);
            }
        }
        yaw
    }

    fn set_mtx(&mut self, pos: Vec3, angle: [i16; 3], hio: &Statics) -> Option<Vec3> {
        let scale = self.flag_scale * hio.flag_scale;
        if let (Some(parent), Some(parent_pos)) = (self.parent_mtx, self.parent_pos) {
            let mut stack = parent;
            stack *= Mat4::from_translation(parent_pos);
            stack *= rot_y(angle[1]);
            stack *= Mat4::from_translation(Vec3::new(0.0, 0.0, PARENT_TRANS_Z));
            stack *= Mat4::from_scale(Vec3::splat(scale));
            self.mtx = stack;
            return Some(stack.transform_point3(Vec3::ZERO));
        }
        let mut calc = Mat4::from_translation(pos);
        if hio.has_custom_rotation {
            calc *= rot_z(deg2s(hio.custom_rotation.z));
            calc *= rot_x(deg2s(hio.custom_rotation.x));
            calc *= rot_y(deg2s(hio.custom_rotation.y));
        } else {
            calc *= rot_z(angle[2]);
            calc *= rot_x(angle[0]);
            calc *= rot_y(angle[1]);
            if self.flag_type == FLAG_TYPE_4 || self.tex_type != 0 {
                calc *= Mat4::from_translation(Vec3::new(0.0, 0.0, TEX_FLAG_TRANS_Z));
            } else {
                calc *= rot_x(hio.rot_x as i16);
                calc *= Mat4::from_translation(Vec3::new(0.0, 0.0, hio.trans_z));
            }
        }
        calc *= Mat4::from_scale(Vec3::splat(scale));
        self.mtx = calc;
        None
    }

    fn set_nrm_mtx(&self) -> Mat4 {
        rot_y(self.rotate_y)
    }

    fn set_back_nrm(&mut self) {
        let cur = self.cur_arr as usize;
        for i in 0..VERT_COUNT {
            self.nrm_arr_back[cur][i] = Vec3::ZERO - self.nrm_arr[cur][i];
        }
    }

    fn set_nrm_vtx(&mut self, calc_mtx: Mat4, index: usize) {
        let cur = self.cur_arr as usize;
        let pos = self.pos_arr[cur];
        let sp50 = pos[index];
        let mut sp5c = Vec3::ZERO;
        let row = &REL_POS_IDX[index];
        for i in 0..BAND_COUNT {
            if row[i + 1] == -1 {
                break;
            }
            let sp80 = pos[row[i] as usize] - sp50;
            let sp74 = pos[row[i + 1] as usize] - sp50;
            let sp68 = norm_zc(sp74.cross(sp80));
            sp5c += sp68;
        }
        sp5c = norm_zc(sp5c);
        let (i_var4, i_var2) = band(index);
        let deg = ssin((i_var2 + i_var4) * CLOTH_SET_NRM_VTX_DEG as i32) * CLOTH_SET_NRM_VTX_GAIN;
        let calc = calc_mtx * rot_y(deg as i16);
        let sp68 = calc.transform_point3(sp5c);
        self.nrm_arr[cur][index] = norm_zc(sp68);
    }

    fn majuu_flag_move(&mut self, angle: [i16; 3], hio: &Statics) {
        let from = self.cur_arr as usize;
        let pos1 = self.pos_arr[from];
        let nrm_old = self.nrm_arr[from];
        self.change_current_pos();
        self.wave = self.wave.wrapping_add(hio.wave_speed);
        let to = self.cur_arr as usize;

        let f_var5 = ssin(self.wave as i32) * CLOTH_WAVE_BIAS + CLOTH_WAVE_BIAS;
        let mut sp20 = Vec3::new(
            0.0,
            0.0,
            hio.wind_speed1 * f_var5 + hio.wind_speed2 * (1.0 - f_var5),
        );
        sp20 *= hio.wind_pow * CLOTH_WIND_POW;
        let mut wind_calc = rot_z(angle[2].wrapping_neg());
        wind_calc *= rot_x(angle[0].wrapping_neg());
        let wind = wind_calc.transform_point3(sp20);

        for i in 0..VERT_COUNT {
            let tmp = CLOTH_SPEED_DAMP;
            self.pos_arr[to][i] = pos1[i];
            let sp2c = get_cloth_anim_factor(&pos1, &nrm_old, wind, i, hio);
            self.speed[i] += sp2c;
            self.speed[i] *= tmp;
            self.pos_arr[to][i] += self.speed[i];
        }

        let calc_mtx = self.set_nrm_mtx();
        for i in 0..VERT_COUNT {
            self.set_nrm_vtx(calc_mtx, i);
        }
        self.set_back_nrm();
    }
}

fn deg2s(deg: f32) -> i16 {
    (deg * DEG2S_CONSTANT) as i16
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

fn ssin(angle: i32) -> f32 {
    walk::from_s16(angle as i16).sin()
}

fn atan2s(y: f32, x: f32) -> i16 {
    walk::to_s16(y.atan2(x))
}

fn norm_zc(v: Vec3) -> Vec3 {
    if v.length_squared() < NEAR_ZERO_SQUARE {
        if v.length_squared() == 0.0 {
            return Vec3::Z;
        }
        return v.normalize();
    }
    v.normalize()
}

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn rnd_fx(seed: &mut [i32; 3], range: f32) -> f32 {
    range * (rnd(seed) - 0.5) * 2.0
}

fn band(index: usize) -> (i32, i32) {
    let index = index as i32;
    if index == 0 {
        (0, 0)
    } else if index < BAND_HIGH_INDEX {
        (1, index - 1)
    } else if index < BAND_INDEX_6 {
        (2, index - 3)
    } else if index < BAND_INDEX_10 {
        (3, index - 6)
    } else if index < BAND_INDEX_15 {
        (4, index - 10)
    } else {
        (5, index - 0xF)
    }
}

fn get_cloth_anim_sub_factor(
    p_pos: Vec3,
    p_other: Vec3,
    p_dst: &mut Vec3,
    dist_ideal: f32,
    spring: f32,
) {
    let sp30 = p_other - p_pos;
    let mut sp24 = norm_zc(sp30);
    let mut tmp = sp30.length() - dist_ideal;
    tmp *= spring;
    sp24 *= tmp;
    *p_dst += sp24;
}

fn get_cloth_anim_factor(
    pos: &[Vec3],
    nrm: &[Vec3],
    wind: Vec3,
    index: usize,
    hio: &Statics,
) -> Vec3 {
    let sp20 = pos[index];
    let row = &REL_POS_IDX[index];
    let f_var4 = wind.dot(nrm[index]);

    if index == 15 || index == 20 {
        return Vec3::ZERO;
    }

    let mut sp14 = nrm[index] * f_var4;
    sp14.y += hio.gravity;

    for i in 0..5 {
        let rel = row[i];
        if rel == -1 {
            break;
        }
        let dist = if rel == 0 || index == 0 || (rel - index as i32).unsigned_abs() > 1 {
            CLOTH_IDEAL_LONG
        } else {
            CLOTH_IDEAL_SHORT
        };
        get_cloth_anim_sub_factor(sp20, pos[rel as usize], &mut sp14, dist, hio.spring);
    }

    sp14
}

pub fn control(mut flags: Query<(&mut Actor, &mut MajuuFlag)>, statics: Res<Statics>) {
    let hio = *statics;
    for (mut actor, mut flag) in &mut flags {
        if !flag.created {
            let yaw = flag.create_post(actor.pos, actor.angle, &hio);
            actor.angle[1] = yaw;
            flag.created = true;
        }
        if hio.monotone {
            continue;
        }
        let mut target = atan2s(hio.wind_vec.x, hio.wind_vec.z);
        if let (Some(parent), Some(_parent_pos)) = (flag.parent_mtx, flag.parent_pos) {
            let sp08 = Mat3::from_mat4(parent) * Vec3::Z;
            target = target.wrapping_sub(atan2s(sp08.x, sp08.z));
        }
        add_calc_angle_s2(&mut actor.angle[1], target, YAW_SCALE, YAW_MAX_STEP);
        flag.majuu_flag_move(actor.angle, &hio);
        if let Some(pos) = flag.set_mtx(actor.pos, actor.angle, &hio) {
            actor.pos = pos;
        }
    }
}
