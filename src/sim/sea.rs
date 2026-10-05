use bevy_ecs::prelude::*;
use glam::Vec2;

use crate::dzs::DEFAULT_WAVE_MAX;

use super::player::Player;

pub const GRID_SIZE: f32 = 800.0;
pub const GRID_CELLS: usize = 65;

const DRAW_HALF: f32 = 25_600.0;
const BASE_HEIGHT: f32 = 1.0;
const FLAT_MARGIN: f32 = 12_800.0;
const CULL_MARGIN: f32 = 25_600.0;
const CHART_SPAN: f32 = 100_000.0;
const CHART_HALF: f32 = 450_000.0;
const CHART_CELLS: i32 = 9;
const SCALE_STEPS: f32 = 100.0;
const EDGE_FADE: f32 = 1.0 / 6.0;
const EDGE_FADE_SPAN: usize = 5;
const KILOMETER_TAU: f32 = 6.28;
const RAD_TO_S16: f64 = 10430.3779296875;
const COS_TABLE: u32 = 4096;
const COS_QUARTER: u32 = COS_TABLE / 4;
const COS_SHIFT: u16 = 4;
const FULL_TURN: f32 = 65_536.0;
const PLANE_EPSILON: f32 = 0.02;

#[derive(Clone, Copy)]
struct WaveParam {
    height: f32,
    km: f32,
    phase: i16,
    scale_x: f32,
    scale_z: f32,
    counter_max: i32,
}

const OCEAN: [WaveParam; 4] = [
    WaveParam {
        height: 2.5,
        km: 7.352941E-5,
        phase: 0,
        scale_x: 0.98,
        scale_z: 0.2,
        counter_max: 200,
    },
    WaveParam {
        height: 2.5,
        km: 8.928571E-5,
        phase: 4000,
        scale_x: 0.2,
        scale_z: 0.98,
        counter_max: 190,
    },
    WaveParam {
        height: 2.5,
        km: 1.13636364E-4,
        phase: 8000,
        scale_x: -0.98,
        scale_z: 0.2,
        counter_max: 210,
    },
    WaveParam {
        height: 2.5,
        km: 1.5625E-4,
        phase: 12000,
        scale_x: 0.2,
        scale_z: -0.98,
        counter_max: 180,
    },
];

const NEIGHBORS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
];

#[derive(Resource, Debug, Clone)]
pub struct SeaTables {
    wave_max: [[u8; CHART_CELLS as usize]; CHART_CELLS as usize],
}

impl SeaTables {
    pub fn new(mut cell: impl FnMut(u8) -> u8) -> Self {
        let mut wave_max = [[DEFAULT_WAVE_MAX; CHART_CELLS as usize]; CHART_CELLS as usize];
        let mut room = 1_u8;
        for z in 1..CHART_CELLS as usize - 1 {
            for x in 1..CHART_CELLS as usize - 1 {
                wave_max[z][x] = cell(room);
                room += 1;
            }
        }
        Self { wave_max }
    }
}

#[derive(Resource, Debug)]
pub struct Sea {
    tables: SeaTables,
    heights: Vec<f32>,
    counters: [i32; 4],
    cur_scale: f32,
    flat: f32,
    origin: Vec2,
    culled: bool,
}

impl Sea {
    pub fn new(tables: SeaTables) -> Self {
        Self {
            tables,
            heights: vec![BASE_HEIGHT; GRID_CELLS * GRID_CELLS],
            counters: [0; 4],
            cur_scale: 1.0,
            flat: 0.0,
            origin: Vec2::ZERO,
            culled: true,
        }
    }

    pub fn execute(&mut self, x: f32, z: f32) {
        let cell = (chart_index(x), chart_index(z));
        self.flat = flat_target(x, z, cell, &self.tables);
        self.origin = Vec2::new(x - DRAW_HALF, z - DRAW_HALF);
        self.culled = cull_flag(x, z, cell, &self.tables);
        if self.culled {
            return;
        }
        let scale = self.flat * self.next_scale((wave_at(&self.tables, cell)) as f32);
        self.fill(scale);
        self.advance_counters();
    }

    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        if !self.draw_contains(x, z) {
            return BASE_HEIGHT;
        }
        let frac = 1.0 / GRID_SIZE;
        let cell_x = ((x - self.origin.x) * frac) as i32;
        let cell_z = ((z - self.origin.y) * frac) as i32;
        let base_x = cell_x as f32 * GRID_SIZE + self.origin.x;
        let base_z = cell_z as f32 * GRID_SIZE + self.origin.y;
        let height = |x: i32, z: i32| self.heights[(z * GRID_CELLS as i32 + x) as usize];
        let v00 = [base_x, height(cell_x, cell_z), base_z];
        let v01 = [base_x, height(cell_x, cell_z + 1), base_z + GRID_SIZE];
        let v10 = [base_x + GRID_SIZE, height(cell_x + 1, cell_z), base_z];
        let v11 = [
            base_x + GRID_SIZE,
            height(cell_x + 1, cell_z + 1),
            base_z + GRID_SIZE,
        ];
        let frac_x = (x - v01[0]) * frac;
        let frac_z = (z - v10[2]) * frac;
        let (normal, base) = if frac_x + frac_z >= 1.0 {
            plane_through(v01, v10, v11)
        } else {
            plane_through(v00, v01, v10)
        };
        -(normal[0] * x + normal[2] * z + base) / normal[1]
    }

    pub fn culled(&self) -> bool {
        self.culled
    }

    pub fn flat(&self) -> f32 {
        self.flat
    }

    pub fn origin(&self) -> Vec2 {
        self.origin
    }

    pub fn heights(&self) -> &[f32] {
        &self.heights
    }

    pub fn counter_ratio(&self, index: usize) -> f32 {
        self.counters[index] as f32 / OCEAN[index].counter_max as f32
    }

    pub fn draw_contains(&self, x: f32, z: f32) -> bool {
        if wave_at(&self.tables, (chart_index(x), chart_index(z))) == 0 && self.culled {
            return false;
        }
        let max_x = self.origin.x + DRAW_HALF * 2.0;
        let max_z = self.origin.y + DRAW_HALF * 2.0;
        self.origin.x < x && x < max_x && self.origin.y < z && z < max_z
    }

    fn next_scale(&mut self, target: f32) -> f32 {
        self.cur_scale += (target - self.cur_scale) / SCALE_STEPS;
        self.cur_scale
    }

    fn advance_counters(&mut self) {
        for index in 0..self.counters.len() {
            self.counters[index] += 1;
            if self.counters[index] >= OCEAN[index].counter_max {
                self.counters[index] = 0;
            }
        }
    }

    fn fill(&mut self, scale: f32) {
        let mut theta_x = [0.0_f32; 4];
        let mut theta_z = [0.0_f32; 4];
        let mut offsets = [0_i16; 4];
        let mut amplitudes = [0.0_f32; 4];
        for index in 0..4 {
            let wave = &OCEAN[index];
            theta_x[index] = wave.km * KILOMETER_TAU * wave.scale_x;
            theta_z[index] = wave.km * KILOMETER_TAU * wave.scale_z;
            offsets[index] = (FULL_TURN * (self.counter_ratio(index) - 0.5)) as i16;
            amplitudes[index] = wave.height * scale;
        }

        let mut theta = [0_i16; 4];
        for index in 0..4 {
            theta[index] = rad_to_s16(theta_z[index] * (self.origin.y + GRID_SIZE));
            theta[index] = theta[index].wrapping_sub(offsets[index]);
            theta[index] = theta[index].wrapping_add(OCEAN[index].phase);
        }

        let mut delta_z = [0_i16; 4];
        let mut delta_x = [0_i16; 4];
        let mut phases = [0_i16; 4];
        for index in 0..4 {
            delta_z[index] = rad_to_s16(theta_z[index] * GRID_SIZE);
            delta_x[index] = rad_to_s16(theta_x[index] * GRID_SIZE);
            phases[index] = rad_to_s16(theta_x[index] * (self.origin.x + GRID_SIZE));
        }

        let mut fade = [1.0_f32; GRID_CELLS];
        fade[0] = 0.0;
        fade[GRID_CELLS - 1] = 0.0;
        for step in 1..=EDGE_FADE_SPAN {
            let weight = EDGE_FADE * step as f32;
            fade[step] = weight;
            fade[GRID_CELLS - 1 - step] = weight;
        }

        for z in 1..GRID_CELLS - 1 {
            let mut wave = [0_i16; 4];
            for index in 0..4 {
                wave[index] = phases[index].wrapping_add(theta[index]);
            }
            for x in 1..GRID_CELLS - 1 {
                let mut height = amplitudes[0] * scos(wave[0])
                    + amplitudes[1] * scos(wave[1])
                    + amplitudes[2] * scos(wave[2])
                    + amplitudes[3] * scos(wave[3])
                    + BASE_HEIGHT;
                height *= fade[z] * fade[x];
                self.heights[z * GRID_CELLS + x] = height;
                for index in 0..4 {
                    wave[index] = wave[index].wrapping_add(delta_x[index]);
                }
            }
            for index in 0..4 {
                theta[index] = theta[index].wrapping_add(delta_z[index]);
            }
        }
    }
}

pub fn sea_tick(mut sea: Option<ResMut<Sea>>, players: Query<&Player>) {
    let Some(sea) = sea.as_deref_mut() else {
        return;
    };
    for player in &players {
        sea.execute(player.pos.x, player.pos.z);
    }
}

fn chart_index(value: f32) -> i32 {
    ((value + CHART_HALF) / CHART_SPAN) as i32
}

fn cell_box(cell: (i32, i32)) -> (f32, f32, f32, f32) {
    let min_x = cell.0 as f32 * CHART_SPAN - CHART_HALF;
    let min_z = cell.1 as f32 * CHART_SPAN - CHART_HALF;
    (min_x, min_x + CHART_SPAN, min_z, min_z + CHART_SPAN)
}

fn wave_at(tables: &SeaTables, cell: (i32, i32)) -> u8 {
    let (x, z) = cell;
    if x < 0 || z < 0 || x >= CHART_CELLS || z >= CHART_CELLS {
        return DEFAULT_WAVE_MAX;
    }
    tables.wave_max[z as usize][x as usize]
}

fn flat_target(x: f32, z: f32, cell: (i32, i32), tables: &SeaTables) -> f32 {
    if wave_at(tables, cell) == 0 {
        return 0.0;
    }
    let mut result: f32 = 1.0;
    for (dx, dz) in NEIGHBORS {
        let neighbor = (cell.0 + dx, cell.1 + dz);
        if wave_at(tables, neighbor) != 0 {
            continue;
        }
        let (min_x, max_x, min_z, max_z) = cell_box(neighbor);
        let distance = box_len(
            x,
            z,
            min_x - FLAT_MARGIN,
            min_z - FLAT_MARGIN,
            max_x + FLAT_MARGIN,
            max_z + FLAT_MARGIN,
        );
        result = result.min(distance.min(FLAT_MARGIN) / FLAT_MARGIN);
    }
    result
}

fn cull_flag(x: f32, z: f32, cell: (i32, i32), tables: &SeaTables) -> bool {
    if wave_at(tables, cell) != 0 {
        return false;
    }
    let (min_x, max_x, min_z, max_z) = cell_box(cell);
    min_x + CULL_MARGIN < x
        && x < max_x - CULL_MARGIN
        && min_z + CULL_MARGIN < z
        && z < max_z - CULL_MARGIN
}

fn box_len(x: f32, z: f32, min_x: f32, min_z: f32, max_x: f32, max_z: f32) -> f32 {
    if min_x < x && x < max_x && min_z < z && z < max_z {
        return 0.0;
    }
    if min_x < x && x < max_x {
        return (min_z - z).abs().min((max_z - z).abs());
    }
    if min_z < z && z < max_z {
        return (min_x - x).abs().min((max_x - x).abs());
    }
    let edge_x = if x < min_x { min_x } else { max_x };
    let edge_z = if z < min_z { min_z } else { max_z };
    ((x - edge_x) * (x - edge_x) + (z - edge_z) * (z - edge_z)).sqrt()
}

fn rad_to_s16(radians: f32) -> i16 {
    let value = (f64::from(radians) % std::f64::consts::TAU) * RAD_TO_S16;
    let mut wrapped = value as i32;
    if wrapped < i16::MIN as i32 {
        wrapped += 0x1_0000;
    } else if wrapped > i16::MAX as i32 {
        wrapped -= 0x1_0000;
    }
    wrapped as i16
}

fn scos(angle: i16) -> f32 {
    let index = u32::from(angle as u16 >> COS_SHIFT) + COS_QUARTER;
    ((std::f64::consts::TAU / f64::from(COS_TABLE)) * f64::from(index)).sin() as f32
}

fn plane_through(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> ([f32; 3], f32) {
    let edge_a = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let edge_b = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let mut cross = [
        edge_a[1] * edge_b[2] - edge_a[2] * edge_b[1],
        edge_a[2] * edge_b[0] - edge_a[0] * edge_b[2],
        edge_a[0] * edge_b[1] - edge_a[1] * edge_b[0],
    ];
    let length = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    if length < PLANE_EPSILON {
        return ([0.0; 3], 0.0);
    }
    let inverse = 1.0 / length;
    for axis in &mut cross {
        *axis *= inverse;
    }
    let base = -(cross[0] * p0[0] + cross[1] * p0[1] + cross[2] * p0[2]);
    (cross, base)
}
