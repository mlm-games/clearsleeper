use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};

use super::actor::Actor;
use super::walk::RAD_PER_UNIT;

pub const GOAL_FLAG_ACTORS: [&str; 1] = ["Gflag"];

pub fn is_goal_flag(name: &str) -> bool {
    GOAL_FLAG_ACTORS.contains(&name)
}

pub const BUOY_RACE_ACTOR: &str = "Buoyrace";

pub const PRM_FLAG_SHIFT: u32 = 0;
pub const PRM_FLAG_MASK: u32 = 0xFF;
pub const PRM_PATH_SHIFT: u32 = 16;
pub const PRM_PATH_MASK: u32 = 0xFF;

pub const PATH_ID_NONE: u8 = 0xFF;

pub const CLOTH_COLS: usize = 9;
pub const CLOTH_ROWS: usize = 5;
pub const CLOTH_VERTS: usize = CLOTH_COLS * CLOTH_ROWS;
pub const CLOTH_LAST_COL: i32 = 8;
pub const CLOTH_LAST_ROW: i32 = 4;

pub const ROPE_COUNT: usize = 4;
pub const ROPE_MAX_BUOYS: i32 = 68;
pub const ROPE_MAX_SEGMENTS: usize = (ROPE_MAX_BUOYS as usize) * 4 + 1;
pub const ROPE_HEAP_BYTES: u32 = 0x10000;
pub const ROPE_BYTES_PER_SEGMENT: u32 = 60;
pub const ROPE_INTERP: i32 = 4;

pub const CREATED_WARMUP_FRAMES: i32 = 20;

pub const TIME_LIMIT_DEFAULT: u32 = 240;
pub const TIME_LIMIT_EVENT_MULT: u32 = 10;
pub const EVENT_REG_TIME_LIMIT: u32 = 0xAAFF;

pub const MINI_GAME_TYPE_BOAT_RACE: i32 = 1;
pub const MINI_GAME_WIN: i32 = 1;
pub const MINI_GAME_LOSE: i32 = 2;

pub const FINISH_NONE: i32 = 0;
pub const FINISH_STANDARD: i32 = 1;
pub const FINISH_PERFECT: i32 = 2;

pub const RACE_STAGE_NAME: &str = "Ocean";
pub const RACE_STAGE_POINT: i16 = 1;
pub const RACE_NEXT_STAGE: &str = "sea";
pub const RACE_NEXT_POINT: i16 = 1;
pub const RACE_NEXT_ROOM: i32 = 34;

pub const EVENT_RACE_START_CAM: &str = "race_start_cam";
pub const EVENT_RACE_GOAL_CAM: &str = "race_goal_cam";
pub const EVENT_RACE_FAIL_CAM: &str = "race_fail_cam";

pub const TIMER_START_COUNT: i32 = 7;
pub const TIMER_HIDE_FRAME: i32 = 3;
pub const TIMER_REST_SCALE: i32 = 10;
pub const TIMER_END_ARG: i32 = -1;

pub const HIO_M04: i8 = -1;
pub const HIO_M06: i8 = 0;
pub const HIO_M07: i8 = 0;
pub const HIO_M08: i8 = 0;
pub const HIO_M09: i8 = 0;
pub const HIO_MWIND_SCALE_PHASE_INC: i16 = 0x2EE;
pub const HIO_MNORMAL_FLUTTER_PHASE_INC: i16 = 0x800;
pub const HIO_MFLAG_WAVE_PHASE_INC: i16 = 0xBD1;
pub const HIO_MTIME_LIMIT: u32 = 240;
pub const HIO_MPERFECT_RUPEE_SCORE: i32 = 150;
pub const HIO_MEND_CAM_EARLY_FRAME: i32 = 150;
pub const HIO_MWIND_SCALE1: f32 = 12.0;
pub const HIO_MWIND_SCALE2: f32 = 4.0;
pub const HIO_MFLAG_SAG_FACTOR: f32 = -1.0;
pub const HIO_MCLOTH_STIFFNESS: f32 = 0.4;
pub const HIO_MROPE_COLOR_R: u32 = 0xFF;
pub const HIO_MROPE_COLOR_G: u32 = 0x80;
pub const HIO_MROPE_COLOR_B: u32 = 0x80;

pub const CLOTH_WIND_X_SCALE: f32 = 0.2;
pub const CLOTH_VELOCITY_DAMP: f32 = 0.85;
pub const CLOTH_WAVE_AMPLITUDE: f32 = 40.0;
pub const CLOTH_FALLOFF_EDGE: f32 = 20.25;
pub const CLOTH_FALLOFF_CENTER: f32 = 4.5;
pub const CLOTH_WAVE_COL_STEP: f32 = 16384.0;
pub const CLOTH_WAVE_ROW_STEP: f32 = 8192.0;
pub const CLOTH_SAG_ROW_STEP: f32 = 0.25;

pub const NRM_FLUTTER_AMP: f32 = 512.0;
pub const NRM_FLUTTER_COL_STEP: i32 = -0x400;
pub const NRM_FLUTTER_ROW_STEP: i32 = 0x100;

pub const CLOTH_IDEAL_COL_STEP: f32 = 250.0;
pub const CLOTH_IDEAL_ROW_STEP: f32 = 120.0;
pub const CLOTH_IDEAL_DIAG: f32 = 277.3085;

pub const ROPE_DOWN_OFFSET: [f32; 4] = [0.0, 150.0, 225.0, 150.0];
pub const ROPE_POINT_LIFT: f32 = 250.0;
pub const ROPE_SUB_INDEX_SHIFT: u32 = 2;

pub const CULL_MIN: [f32; 3] = [-400.0, -300.0, -400.0];
pub const CULL_MAX: [f32; 3] = [400.0, 600.0, 400.0];

const L_POS: [Vec3; CLOTH_VERTS] = [Vec3::ZERO; CLOTH_VERTS];

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Statics {
    pub m04: i8,
    pub draw_cel_shaded: bool,
    pub m06: i8,
    pub m07: i8,
    pub m08: i8,
    pub m09: i8,
    pub wind_scale_phase_inc: i16,
    pub normal_flutter_phase_inc: i16,
    pub flag_wave_phase_inc: i16,
    pub time_limit: u32,
    pub perfect_rupee_score: i32,
    pub end_cam_early_frame: i32,
    pub rope_color_r: u32,
    pub rope_color_g: u32,
    pub rope_color_b: u32,
    pub wind_scale1: f32,
    pub wind_scale2: f32,
    pub flag_sag_factor: f32,
    pub cloth_stiffness: f32,
}

impl Default for Statics {
    fn default() -> Self {
        Statics {
            m04: HIO_M04,
            draw_cel_shaded: true,
            m06: HIO_M06,
            m07: HIO_M07,
            m08: HIO_M08,
            m09: HIO_M09,
            wind_scale_phase_inc: HIO_MWIND_SCALE_PHASE_INC,
            normal_flutter_phase_inc: HIO_MNORMAL_FLUTTER_PHASE_INC,
            flag_wave_phase_inc: HIO_MFLAG_WAVE_PHASE_INC,
            time_limit: HIO_MTIME_LIMIT,
            perfect_rupee_score: HIO_MPERFECT_RUPEE_SCORE,
            end_cam_early_frame: HIO_MEND_CAM_EARLY_FRAME,
            rope_color_r: HIO_MROPE_COLOR_R,
            rope_color_g: HIO_MROPE_COLOR_G,
            rope_color_b: HIO_MROPE_COLOR_B,
            wind_scale1: HIO_MWIND_SCALE1,
            wind_scale2: HIO_MWIND_SCALE2,
            flag_sag_factor: HIO_MFLAG_SAG_FACTOR,
            cloth_stiffness: HIO_MCLOTH_STIFFNESS,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RacePath {
    pub id: u8,
    pub room: i8,
    pub next_id: u8,
    pub points: Vec<Vec3>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct RacePaths {
    pub paths: Vec<RacePath>,
}

impl RacePaths {
    pub fn get(&self, id: u8, room: i8) -> Option<&RacePath> {
        self.paths.iter().find(|path| path.id == id && path.room == room)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalFlagProc {
    None,
    RaceStart,
    TimerExecute,
    RaceEnd,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GoalFlag {
    pub arc_index: u8,
    pub prm: u8,
    pub path_id: u8,
    pub shape_angle: i16,
    pub scale: Vec3,
    pub mtx: Mat4,
    pub goal_flag_pole_pos: [Vec3; 2],
    pub wind_scale_phase: i16,
    pub normal_flutter_phase: i16,
    pub flag_wave_phase: i16,
    pub curr_arr: u8,
    pub pos_arr: [[Vec3; CLOTH_VERTS]; 2],
    pub dpos_arr: [[Vec3; CLOTH_VERTS]; 2],
    pub nrm_arr: [[Vec3; CLOTH_VERTS]; 2],
    pub back_nrm_arr: [[Vec3; CLOTH_VERTS]; 2],
    pub velocity: [Vec3; CLOTH_VERTS],
    pub timer_proc_id: i32,
    pub mgame_start_proc_id: i32,
    pub mgame_term_proc_id: i32,
    pub prev_player_line_side: f32,
    pub race_end_state: i16,
    pub cam_frames_passed: i16,
    pub race_start_state: u8,
    pub rope_path_ids: [u8; ROPE_COUNT],
    pub rope_path_ok: [bool; ROPE_COUNT],
    pub num_rope_buoys: [i32; ROPE_COUNT],
    pub num_ropes: i32,
    pub rope_points: [Vec3; ROPE_COUNT * ROPE_MAX_SEGMENTS],
    pub heap_ok: bool,
    pub curr_proc: GoalFlagProc,
    pub created: bool,
}

impl GoalFlag {
    pub fn spawn(parameters: u32) -> Self {
        let prm = ((parameters >> PRM_FLAG_SHIFT) & PRM_FLAG_MASK) as u8;
        let path_id = ((parameters >> PRM_PATH_SHIFT) & PRM_PATH_MASK) as u8;
        let (arc_index, scale) = if prm == 0 || prm == 0xFF {
            (0u8, Vec3::new(1.05, 1.0, 1.0))
        } else {
            (1u8, Vec3::new(0.98, 1.0, 1.0))
        };
        let mut pos_arr = [[Vec3::ZERO; CLOTH_VERTS]; 2];
        for i in 0..CLOTH_VERTS {
            pos_arr[0][i] = L_POS[i];
        }
        GoalFlag {
            arc_index,
            prm,
            path_id,
            shape_angle: 0,
            scale,
            mtx: Mat4::IDENTITY,
            goal_flag_pole_pos: [Vec3::ZERO; 2],
            wind_scale_phase: 0,
            normal_flutter_phase: 0,
            flag_wave_phase: 0,
            curr_arr: 0,
            pos_arr,
            dpos_arr: [[Vec3::ZERO; CLOTH_VERTS]; 2],
            nrm_arr: [[Vec3::ZERO; CLOTH_VERTS]; 2],
            back_nrm_arr: [[Vec3::ZERO; CLOTH_VERTS]; 2],
            velocity: [Vec3::ZERO; CLOTH_VERTS],
            timer_proc_id: -1,
            mgame_start_proc_id: -1,
            mgame_term_proc_id: -1,
            prev_player_line_side: 0.0,
            race_end_state: 0,
            cam_frames_passed: 0,
            race_start_state: 0,
            rope_path_ids: [0; ROPE_COUNT],
            rope_path_ok: [false; ROPE_COUNT],
            num_rope_buoys: [0; ROPE_COUNT],
            num_ropes: 0,
            rope_points: [Vec3::ZERO; ROPE_COUNT * ROPE_MAX_SEGMENTS],
            heap_ok: false,
            curr_proc: GoalFlagProc::None,
            created: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn rope_base(&self, rope: usize) -> usize {
        rope * ROPE_MAX_SEGMENTS
    }

    pub fn rope_pt(&self, rope: usize, index: usize) -> Vec3 {
        self.rope_points
            .get(self.rope_base(rope).wrapping_add(index))
            .copied()
            .unwrap_or(Vec3::ZERO)
    }

    pub fn set_rope_pt(&mut self, rope: usize, index: usize, value: Vec3) {
        let slot = self.rope_base(rope).wrapping_add(index);
        if let Some(point) = self.rope_points.get_mut(slot) {
            *point = value;
        }
    }

    pub fn rope_pos(&self, rope: usize, segment: usize) -> Vec3 {
        self.rope_pt(rope, segment.wrapping_mul(ROPE_INTERP as usize))
    }

    pub fn set_rope_pos(&mut self, rope: usize, segment: usize, value: Vec3) {
        self.set_rope_pt(rope, segment.wrapping_mul(ROPE_INTERP as usize), value);
    }

    pub fn set_action(&mut self, proc: GoalFlagProc) {
        self.curr_proc = proc;
    }

    pub fn get_race_path(&mut self, path_id: u8, room: i8, paths: &RacePaths) -> bool {
        let mut i = 0usize;
        let first = match paths.get(path_id, room) {
            Some(path) => path,
            None => return false,
        };
        self.rope_path_ids[0] = path_id;
        self.rope_path_ok[0] = true;
        self.num_rope_buoys[0] = i32::from(first.points.len().min(u16::MAX as usize) as u16);
        let mut next_path_id = first.next_id;
        i += 1;
        while i < ROPE_COUNT && next_path_id != PATH_ID_NONE {
            let chained = paths.get(next_path_id, room);
            self.rope_path_ids[i] = next_path_id;
            self.rope_path_ok[i] = chained.is_some();
            match chained {
                Some(path) => {
                    self.num_rope_buoys[i] =
                        i32::from(path.points.len().min(u16::MAX as usize) as u16);
                    next_path_id = path.next_id;
                }
                None => {
                    self.num_rope_buoys[i] = 0;
                    next_path_id = PATH_ID_NONE;
                }
            }
            i += 1;
        }
        self.num_ropes = i32::try_from(i).unwrap_or(i32::MAX);
        true
    }

    pub fn create_heap(&mut self) -> bool {
        let mut needed = 0u32;
        for i in 0..self.num_ropes.clamp(0, ROPE_COUNT as i32) as usize {
            let segments = (self.num_rope_buoys[i].max(0) << 2) + 1;
            let bytes = u32::try_from(segments).unwrap_or(u32::MAX);
            needed = needed.saturating_add(bytes.saturating_mul(ROPE_BYTES_PER_SEGMENT));
        }
        self.heap_ok = needed <= ROPE_HEAP_BYTES;
        self.heap_ok
    }

    pub fn rope_move(&mut self) {
        let down_offset = ROPE_DOWN_OFFSET;
        for i in 0..self.num_ropes.clamp(0, ROPE_COUNT as i32) as usize {
            let start = self.rope_pos(i, 0);
            let buoys = self.num_rope_buoys[i].max(0);
            self.set_rope_pos(i, buoys as usize, start);
            for j in 0..(buoys << 2) {
                let interpolated_point = j & 3;
                let anchor_point = (j >> 2) as usize;
                if interpolated_point != 0 {
                    let a = self.rope_pos(i, anchor_point);
                    let b = self.rope_pos(i, anchor_point + 1);
                    let t = interpolated_point as f32 * 0.25;
                    let one_minus_t = 1.0 - t;
                    let mut c = a * one_minus_t + b * t;
                    c.y -= down_offset
                        .get(interpolated_point as usize)
                        .copied()
                        .unwrap_or(0.0);
                    self.set_rope_pt(
                        i,
                        anchor_point * ROPE_INTERP as usize + interpolated_point as usize,
                        c,
                    );
                }
            }
        }
    }

    pub fn create_buoy_races(&mut self, room: i8, parent: u32, out: &mut Vec<(u32, Vec3)>) {
        for i in 0..self.num_ropes.clamp(0, ROPE_COUNT as i32) as usize {
            let buoys = self.num_rope_buoys[i].max(0);
            for j in 0..buoys {
                let (x, y, z) = self.rope_pt(i, j as usize);
                out.push((
                    j as u32 | ((i as u32) << 8),
                    Vec3::new(x, y + ROPE_POINT_LIFT, z),
                ));
                self.set_rope_pt(i, j as usize, Vec3::new(x, y + ROPE_POINT_LIFT, z));
            }
            let _ = (room, parent);
            let first = self.rope_pos(i, 0);
            self.set_rope_pt(i, buoys as usize, first);
        }
    }

    pub fn goal_check(&mut self, player_pos: Vec3) -> i32 {
        let player_from_line_start = player_pos - self.goal_flag_pole_pos[0];
        let mut finish_line_dir = self.goal_flag_pole_pos[1] - self.goal_flag_pole_pos[0];
        finish_line_dir.y = 0.0;
        finish_line_dir = norm_zp(finish_line_dir);
        let mut player_along = player_from_line_start;
        player_along.y = 0.0;

        let finish_line_len = (self.goal_flag_pole_pos[1] - self.goal_flag_pole_pos[0]).length();
        let player_along_line = finish_line_dir.dot(player_along);
        let finish_line_normal =
            Vec3::new(finish_line_dir.z, 0.0, -finish_line_dir.x);
        let player_finish_side = finish_line_normal.dot(player_from_line_start);

        let mut o_result = FINISH_NONE;
        if player_along_line > 0.0 && player_along_line < finish_line_len {
            if self.prev_player_line_side > 0.0 && player_finish_side <= 0.0 {
                o_result = 1;
            } else if self.prev_player_line_side <= 0.0 && player_finish_side > 0.0 {
                o_result = -1;
            }
        }
        self.prev_player_line_side = player_finish_side;
        o_result
    }

    pub fn finish_side(&self, player_pos: Vec3) -> f32 {
        let player_from_line_start = player_pos - self.goal_flag_pole_pos[0];
        let mut finish_line_dir = self.goal_flag_pole_pos[1] - self.goal_flag_pole_pos[0];
        finish_line_dir.y = 0.0;
        finish_line_dir = norm_zp(finish_line_dir);
        let normal = Vec3::new(finish_line_dir.z, 0.0, -finish_line_dir.x);
        normal.dot(player_from_line_start)
    }

    pub fn change_current_pos(&mut self) {
        self.curr_arr ^= 1;
    }

    pub fn set_back_nrm(&mut self) {
        let cur = self.curr_arr as usize;
        for i in 0..CLOTH_VERTS {
            self.back_nrm_arr[cur][i] = Vec3::ZERO - self.nrm_arr[cur][i];
        }
    }

    pub fn set_nrm_vtx(&mut self, calc_mtx: Mat4, i_col: i32, i_row: i32) {
        let cur = self.curr_arr as usize;
        let dpos = self.dpos_arr[cur];
        let center_vertex = cloth_at(&dpos, i_col, i_row);
        let mut averaged_normal = Vec3::ZERO;

        if i_col != 0 {
            let horizontal_edge = cloth_at(&dpos, i_col - 1, i_row) - center_vertex;
            if i_row != 0 {
                let vertical_edge = cloth_at(&dpos, i_col, i_row - 1) - center_vertex;
                let tri_normal = norm_zp(vertical_edge.cross(horizontal_edge));
                averaged_normal += tri_normal;
            }
            if i_row != CLOTH_LAST_ROW {
                let vertical_edge = cloth_at(&dpos, i_col, i_row + 1) - center_vertex;
                let tri_normal = norm_zp(horizontal_edge.cross(vertical_edge));
                averaged_normal += tri_normal;
            }
        }
        if i_col != CLOTH_LAST_COL {
            let horizontal_edge = cloth_at(&dpos, i_col + 1, i_row) - center_vertex;
            if i_row != 0 {
                let vertical_edge = cloth_at(&dpos, i_col, i_row - 1) - center_vertex;
                let tri_normal = norm_zp(horizontal_edge.cross(vertical_edge));
                averaged_normal += tri_normal;
            }
            if i_row != CLOTH_LAST_ROW {
                let vertical_edge = cloth_at(&dpos, i_col, i_row + 1) - center_vertex;
                let tri_normal = norm_zp(vertical_edge.cross(horizontal_edge));
                averaged_normal += tri_normal;
            }
        }
        averaged_normal = norm_zc(avg_norm_helper(averaged_normal));

        let flutter_arg = i32::from(self.normal_flutter_phase)
            + i_col * (0 + NRM_FLUTTER_COL_STEP)
            + i_row * (0 + NRM_FLUTTER_ROW_STEP);
        let y_deg = NRM_FLUTTER_AMP * ssin(flutter_arg);
        let x_deg = NRM_FLUTTER_AMP * scos(flutter_arg);
        let mut calc = calc_mtx;
        calc *= Mat4::from_rotation_y(walk::from_s16(walk::to_s16(y_deg)));
        calc *= Mat4::from_rotation_x(walk::from_s16(walk::to_s16(x_deg)));
        let tri_normal = calc.transform_point3(averaged_normal);
        self.nrm_arr[cur][cloth_index(i_col, i_row)] = norm_zp(tri_normal);
    }

    pub fn flag_move(&mut self, angle_y: i16, wind_vec: Vec3, hio: &Statics) {
        let from = self.curr_arr as usize;
        let prev_pos_arr = self.pos_arr[from];
        let prev_nrm_arr = self.nrm_arr[from];
        self.change_current_pos();
        let to = self.curr_arr as usize;

        self.wind_scale_phase = self.wind_scale_phase.wrapping_add(hio.wind_scale_phase_inc);
        self.normal_flutter_phase =
            self.normal_flutter_phase.wrapping_add(hio.normal_flutter_phase_inc);
        self.flag_wave_phase = self.flag_wave_phase.wrapping_add(hio.flag_wave_phase_inc);

        let scale_blend_factor = 0.5 + ssin(i32::from(self.wind_scale_phase)) * 0.5;
        let wind_vector_scale =
            hio.wind_scale1 * scale_blend_factor + hio.wind_scale2 * (1.0 - scale_blend_factor);

        let calc_mtx = Mat4::from_rotation_y(walk::from_s16(angle_y.wrapping_neg()));
        let mut wind_vector = calc_mtx.transform_point3(wind_vec);
        let mag = wind_vector.length_squared();
        if mag == 0.0 {
            wind_vector = Vec3::ZERO;
        } else {
            wind_vector = wind_vector.normalize();
            wind_vector.x *= CLOTH_WIND_X_SCALE;
            wind_vector *= wind_vector_scale;
        }

        for i in 0..CLOTH_COLS as i32 {
            for j in 0..CLOTH_ROWS as i32 {
                let index = cloth_index(i, j);
                self.pos_arr[to][index] = prev_pos_arr[index];
                let factor = self.get_cloth_anim_factor(&prev_pos_arr, &prev_nrm_arr, &wind_vector, i, j, hio);
                self.velocity[index] += factor;
                self.velocity[index] *= CLOTH_VELOCITY_DAMP;
                self.pos_arr[to][index] += self.velocity[index];
            }
        }

        let curr_pos_arr = self.pos_arr[to];
        for k in 0..CLOTH_COLS as i32 {
            for l in 0..CLOTH_ROWS as i32 {
                let index = cloth_index(k, l);
                let mut dpos = curr_pos_arr[index];
                let dk = (CLOTH_FALLOFF_CENTER - k as f32).abs();
                let center_to_edge_falloff =
                    (CLOTH_FALLOFF_EDGE - dk * dk) / CLOTH_FALLOFF_EDGE;
                dpos.z += CLOTH_WAVE_AMPLITUDE
                    * center_to_edge_falloff
                    * ssin(
                        (k as f32 * CLOTH_WAVE_COL_STEP
                            + l as f32 * CLOTH_WAVE_ROW_STEP
                            + f32::from(self.flag_wave_phase)) as i32,
                    );
                self.dpos_arr[to][index] = dpos;
            }
        }

        let nrm_mtx = Mat4::from_rotation_y(walk::from_s16(angle_y.wrapping_neg()));
        for m in 0..CLOTH_ROWS as i32 {
            for n in 0..CLOTH_COLS as i32 {
                self.set_nrm_vtx(nrm_mtx, n, m);
            }
        }

        self.set_back_nrm();
    }

    fn get_cloth_anim_factor(
        &self,
        i_pos_arr: &[Vec3; CLOTH_VERTS],
        i_nrm_arr: &[Vec3; CLOTH_VERTS],
        i_wind_vec: &Vec3,
        i_col: i32,
        i_row: i32,
        hio: &Statics,
    ) -> Vec3 {
        let index = cloth_index(i_col, i_row);
        let pos = i_pos_arr[index];
        let dot = i_wind_vec.dot(i_nrm_arr[index]);
        if (i_row == 0 || i_row == CLOTH_LAST_ROW)
            && (i_col == 0 || i_col == CLOTH_LAST_COL)
        {
            return Vec3::ZERO;
        }
        let mut anim_factor = i_nrm_arr[index] * dot;
        anim_factor.y += hio.flag_sag_factor * (i_row as f32 * CLOTH_SAG_ROW_STEP);
        if i_col != 0 {
            sub_factor(
                &mut anim_factor,
                pos,
                i_pos_arr[cloth_index(i_col - 1, i_row)],
                CLOTH_IDEAL_COL_STEP,
                hio.cloth_stiffness,
            );
            if i_row != 0 {
                sub_factor(
                    &mut anim_factor,
                    pos,
                    i_pos_arr[cloth_index(i_col, i_row - 1)],
                    CLOTH_IDEAL_ROW_STEP,
                    hio.cloth_stiffness,
                );
                sub_factor(
                    &mut anim_factor,
                    pos,
                    i_pos_arr[cloth_index(i_col - 1, i_row - 1)],
                    CLOTH_IDEAL_DIAG,
                    hio.cloth_stiffness,
                );
            }
            if i_row != CLOTH_LAST_ROW {
                sub_factor(
                    &mut anim_factor,
                    pos,
                    i_pos_arr[cloth_index(i_col, i_row + 1)],
                    CLOTH_IDEAL_ROW_STEP,
                    hio.cloth_stiffness,
                );
                sub_factor(
                    &mut anim_factor,
                    pos,
                    i_pos_arr[cloth_index(i_col - 1, i_row + 1)],
                    CLOTH_IDEAL_DIAG,
                    hio.cloth_stiffness,
                );
            }
            if i_col != CLOTH_LAST_COL {
                sub_factor(
                    &mut anim_factor,
                    pos,
                    i_pos_arr[cloth_index(i_col + 1, i_row)],
                    CLOTH_IDEAL_COL_STEP,
                    hio.cloth_stiffness,
                );
                if i_row != 0 {
                    sub_factor(
                        &mut anim_factor,
                        pos,
                        i_pos_arr[cloth_index(i_col + 1, i_row - 1)],
                        CLOTH_IDEAL_DIAG,
                        hio.cloth_stiffness,
                    );
                }
                if i_row != CLOTH_LAST_ROW {
                    sub_factor(
                        &mut anim_factor,
                        pos,
                        i_pos_arr[cloth_index(i_col + 1, i_row + 1)],
                        CLOTH_IDEAL_DIAG,
                        hio.cloth_stiffness,
                    );
                }
            }
            return anim_factor;
        }
        sub_factor(
            &mut anim_factor,
            pos,
            i_pos_arr[cloth_index(i_col + 1, i_row)],
            CLOTH_IDEAL_COL_STEP,
            hio.cloth_stiffness,
        );
        if i_row != 0 {
            sub_factor(
                &mut anim_factor,
                pos,
                i_pos_arr[cloth_index(i_col, i_row - 1)],
                CLOTH_IDEAL_ROW_STEP,
                hio.cloth_stiffness,
            );
            sub_factor(
                &mut anim_factor,
                pos,
                i_pos_arr[cloth_index(i_col + 1, i_row - 1)],
                CLOTH_IDEAL_DIAG,
                hio.cloth_stiffness,
            );
        }
        if i_row != CLOTH_LAST_ROW {
            sub_factor(
                &mut anim_factor,
                pos,
                i_pos_arr[cloth_index(i_col, i_row + 1)],
                CLOTH_IDEAL_ROW_STEP,
                hio.cloth_stiffness,
            );
            sub_factor(
                &mut anim_factor,
                pos,
                i_pos_arr[cloth_index(i_col + 1, i_row + 1)],
                CLOTH_IDEAL_DIAG,
                hio.cloth_stiffness,
            );
        }
        anim_factor
    }
}

fn cloth_index(i_col: i32, i_row: i32) -> usize {
    let index = i_row * CLOTH_COLS as i32 + i_col;
    if index < 0 {
        return 0;
    }
    (index as usize).min(CLOTH_VERTS - 1)
}

fn cloth_at(dpos: &[Vec3; CLOTH_VERTS], i_col: i32, i_row: i32) -> Vec3 {
    dpos[cloth_index(i_col, i_row)]
}

fn sub_factor(dst: &mut Vec3, pos: Vec3, other: Vec3, ideal: f32, stiffness: f32) {
    let vec_to_neighbor = other - pos;
    let mut correction_dir = norm_zp(vec_to_neighbor);
    let mut diff_from_ideal = vec_to_neighbor.abs() - ideal;
    diff_from_ideal *= stiffness;
    correction_dir *= diff_from_ideal;
    *dst += correction_dir;
}

fn ssin(angle: i32) -> f32 {
    (angle as f32 * RAD_PER_UNIT).sin()
}

fn scos(angle: i32) -> f32 {
    (angle as f32 * RAD_PER_UNIT).cos()
}

fn norm_zp(v: Vec3) -> Vec3 {
    v.normalize_or_zero()
}

fn norm_zc(v: Vec3) -> Vec3 {
    v.normalize_or_zero()
}

fn avg_norm_helper(v: Vec3) -> Vec3 {
    v
}