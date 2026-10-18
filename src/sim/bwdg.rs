use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::molgera::Molgera;
use super::walk;

pub const BWDG_ACTORS: [&str; 0] = [];

pub const BWDG_ACTOR: &str = "Bwdg";

pub const BWD_ACTOR: &str = super::molgera::MOLGERA_ACTOR;

pub fn is_bwdg(name: &str) -> bool {
    BWDG_ACTORS.contains(&name)
}

pub fn boss_a_d_sub(name: &str) -> bool {
    name == BWD_ACTOR
}

pub const PARAMETERS: u32 = 0;
pub const LIST_ID: u32 = 0x0007;
pub const HEAP_SIZE: u32 = 0x96000;

pub const GRID_DIM: usize = 0x41;
pub const GRID_VERTS: usize = 0x1081;
pub const CELL: f32 = 130.0;
pub const GRID_ORIGIN: f32 = -4160.0;

pub const WAVE_REACH: f32 = 3000.0;
pub const WAVE_FALLOFF: f32 = 0.01;
pub const WAVE_RISE_BASE: f32 = 10.0;
pub const WAVE_RISE_GAIN: f32 = 0.00667;
pub const WAVE_RISE_CAP: f32 = 30.0;
pub const WAVE_FREQ: f32 = 200.0;
pub const WAVE_FOLLOW_GAIN: f32 = 0.3;

pub const NORMAL_VERTS: usize = 0x1040;
pub const NORMAL_DENOM: f32 = 400.0;
pub const NORMAL_ANGLE: i16 = -0x4A38;

pub const INIT_SCALE: f32 = 1.0;
pub const INIT_MAX_STEP: f32 = 2000.0;
pub const INIT_ORIGIN: Vec3 = Vec3::ZERO;
pub const INIT_BIAS: f32 = -0.75;
pub const INIT_FOLLOW: f32 = 1.0;
pub const INIT_LIFT: f32 = 600.0;

pub const RUN_SCALE: f32 = 0.5;
pub const RUN_MAX_STEP: f32 = 0.0;

pub const DEFAULT_BIAS: f32 = -0.75;
pub const DEFAULT_PHASE_STEP: i16 = 0;
pub const DEFAULT_FOLLOW: f32 = 1.0;
pub const DEFAULT_LIFT: f32 = 900.0;

pub const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0];
pub const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0];

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct BossWave {
    pub origin: Vec3,
    pub bias: f32,
    pub phase_step: i16,
    pub follow: f32,
    pub lift: f32,
}

impl Default for BossWave {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            bias: DEFAULT_BIAS,
            phase_step: DEFAULT_PHASE_STEP,
            follow: DEFAULT_FOLLOW,
            lift: DEFAULT_LIFT,
        }
    }
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct BwdgStatics {
    pub boss: Option<Entity>,
    pub wave: BossWave,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bwdg {
    pub bank: u8,
    pub frame: i16,
    pub phase: i16,
    pub lift: f32,
    pub heights: [f32; GRID_VERTS],
}

impl Bwdg {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        let mut mesh = Self {
            bank: 0,
            frame: 0,
            phase: 0,
            lift: 0.0,
            heights: [0.0; GRID_VERTS],
        };
        mesh.bank = 1;
        mesh.wave_cont(1, &BossWave::default());
        mesh
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn is_delete(&self) -> bool {
        true
    }

    pub fn base_x(&self, index: usize) -> f32 {
        let j = index % GRID_DIM;
        j as f32 * CELL + GRID_ORIGIN
    }

    pub fn base_z(&self, index: usize) -> f32 {
        let i = index / GRID_DIM;
        i as f32 * CELL + GRID_ORIGIN
    }

    pub fn vertex(&self, index: usize) -> Vec3 {
        Vec3::new(self.base_x(index), self.vertex_y(index), self.base_z(index))
    }

    pub fn vertex_y(&self, index: usize) -> f32 {
        self.heights[index] + self.lift
    }

    pub fn wave_cont(&mut self, init: u8, boss: &BossWave) {
        let (scale, max_step, origin, bias, follow, lift) = if init != 0 {
            (
                INIT_SCALE,
                INIT_MAX_STEP,
                INIT_ORIGIN,
                INIT_BIAS,
                INIT_FOLLOW,
                INIT_LIFT,
            )
        } else {
            self.phase = self.phase.wrapping_add(boss.phase_step);
            (
                RUN_SCALE,
                RUN_MAX_STEP,
                boss.origin,
                boss.bias,
                boss.follow,
                boss.lift,
            )
        };
        self.lift = lift;
        for index in 0..GRID_VERTS {
            let dx = self.base_x(index) - origin.x;
            let dz = self.base_z(index) - origin.z;
            let reach = (dx * dx + dz * dz).sqrt();
            let mut fall = WAVE_REACH - reach;
            let mut flat = fall;
            if flat < 0.0 {
                flat = 0.0;
            }
            flat *= WAVE_FALLOFF;
            if fall < 0.0 {
                fall = 0.0;
            }
            let mut rise = WAVE_RISE_BASE + fall * WAVE_RISE_GAIN;
            if rise > WAVE_RISE_CAP {
                rise = WAVE_RISE_CAP;
            }
            let max = follow * (WAVE_FOLLOW_GAIN * rise + max_step);
            let arg = (i32::from(self.phase).wrapping_add((reach * WAVE_FREQ) as i32)) as i16;
            let wave = walk::from_s16(arg).sin();
            let target = bias * (flat * flat) + rise * wave;
            let slot = &mut self.heights[index];
            if *slot != target {
                let mut step = scale * (target - *slot);
                if step > max {
                    step = max;
                } else if step < -max {
                    step = -max;
                }
                *slot += step;
            }
        }
    }
}

pub fn control(
    mut stat: ResMut<BwdgStatics>,
    mut meshes: Query<(Entity, &mut Actor, &mut Bwdg), (With<Bwdg>, Without<Molgera>)>,
    bosses: Query<(Entity, &Actor), (With<Molgera>, Without<Bwdg>)>,
) {
    let mut found: Option<Entity> = None;
    for (entity, actor) in bosses.iter() {
        if found.is_none() && boss_a_d_sub(&actor.name) {
            found = Some(entity);
        }
    }
    let wave = stat.wave;
    for (_, mut actor, mut mesh) in meshes.iter_mut() {
        let actor = &mut *actor;
        let mesh = &mut *mesh;
        actor.pos = Vec3::ZERO;
        if stat.boss.is_none() {
            stat.boss = found;
            if stat.boss.is_none() {
                continue;
            }
        }
        mesh.frame = mesh.frame.wrapping_add(1);
        mesh.wave_cont(0, &wave);
    }
}
