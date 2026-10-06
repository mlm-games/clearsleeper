use bevy_ecs::prelude::*;
use glam::Vec3;

use crate::sim::damage_fx::DamageParticle;
use crate::sim::damage_resolve::{PARTICLE_CRITICALHITFLASH, PARTICLE_OK};
use crate::sim::walk;

/// Live emitters at once. A spawn past this recycles the oldest slot, oldest
/// meaning created longest ago, so a burst never drops its first particle.
pub const CAPACITY: usize = 48;

/// One live emitter: the position, velocity, acceleration, lifetime and size a
/// `JPABaseParticle` carries, minus everything this port cannot source.
#[derive(Debug, Clone, Copy)]
pub struct Emitter {
    pub id: u16,
    pub pos: Vec3,
    /// `mBaseVel`: velocity accumulates acceleration and decays by drag.
    pub vel: Vec3,
    pub accel: Vec3,
    pub drag: f32,
    pub age: f32,
    pub life: f32,
    /// The spawn's half-extents, already carrying the `cXyz` scale the overload was given.
    pub size: Vec3,
    /// Half-extent growth across the life, positive for a puff, negative for a flash.
    pub grow: f32,
    pub color: [f32; 3],
    pub roll: f32,
    pub roll_rate: f32,
    pub alive: bool,
}

impl Emitter {
    /// `mCurNormTime`: age over lifetime, the fraction the size and alpha curves read.
    pub fn norm(&self) -> f32 {
        if self.life > 0.0 {
            (self.age / self.life).min(1.0)
        } else {
            1.0
        }
    }

    /// Fades across the whole life.
    pub fn alpha(&self) -> f32 {
        1.0 - self.norm()
    }

    /// Half-extents now.
    pub fn half(&self) -> Vec3 {
        self.size * (1.0 + self.grow * self.norm())
    }
}

/// Every live emitter, pooled at a fixed capacity so a long fight never
/// allocates. Retail's `JPAEmitterManager` takes its 3000-particle cap the same
/// way; the number here is chosen, since nothing in this port counts emitters.
#[derive(Resource, Debug)]
pub struct Particles {
    pub items: Vec<Emitter>,
    cursor: usize,
    rand: u32,
}

impl Default for Particles {
    fn default() -> Self {
        Self {
            items: (0..CAPACITY)
                .map(|_| Emitter {
                    id: 0,
                    pos: Vec3::ZERO,
                    vel: Vec3::ZERO,
                    accel: Vec3::ZERO,
                    drag: 0.0,
                    age: 0.0,
                    life: 0.0,
                    size: Vec3::ZERO,
                    grow: 0.0,
                    color: [1.0; 3],
                    roll: 0.0,
                    roll_rate: 0.0,
                    alive: false,
                })
                .collect(),
            cursor: 0,
            rand: 0x02f6_e2b1,
        }
    }
}

/// What one particle id looks like: how many emitters a spawn makes and how each
/// one moves.
///
/// Lifetimes, speeds, sizes, gravity and colours here are **chosen**, not
/// sourced. The retail tables that carry them live in the packed particle data
/// this port does not read, and the sprite sheets they index live with it, so
/// there is nothing to lift a value out of. What is sourced is the shape of the
/// integration, from `JPABaseParticle`: `calcVelocity` accumulates acceleration
/// into the base velocity and scales it by an air-resistance factor,
/// `calcPosition` integrates that velocity, and `incFrame` advances the age,
/// raises the delete flag once the age passes the lifetime, and keeps the
/// normalized age the draw curves read. Retail steps those once per frame at
/// its own rate; this port steps them by `dt`, so the drag factor here is a
/// per-second rate rather than retail's per-frame factor, and the particle is
/// one billboard instead of a textured quad.
struct Spec {
    count: u32,
    life: f32,
    size: Vec3,
    grow: f32,
    speed: f32,
    spread: f32,
    gravity: f32,
    drag: f32,
    color: [f32; 3],
    roll_rate: f32,
}

const PUFF: Spec = Spec {
    count: 4,
    life: 0.5,
    size: Vec3::splat(9.0),
    grow: 1.2,
    speed: 55.0,
    spread: 0.6,
    gravity: 140.0,
    drag: 2.4,
    color: [1.0, 0.96, 0.86],
    roll_rate: 1.5,
};

const FLASH: Spec = Spec {
    count: 1,
    life: 0.22,
    size: Vec3::splat(16.0),
    grow: -0.65,
    speed: 0.0,
    spread: 0.0,
    gravity: 0.0,
    drag: 0.0,
    color: [1.0, 0.98, 0.86],
    roll_rate: 0.0,
};

/// The table: the flash gets the short collapsing one, the hit mark the puff,
/// and any id this port has no entry for falls back to the puff so a queued
/// spawn is never dropped.
fn spec_for(id: u16) -> &'static Spec {
    match id {
        PARTICLE_CRITICALHITFLASH => &FLASH,
        PARTICLE_OK => &PUFF,
        _ => &PUFF,
    }
}

impl Particles {
    /// Every emitter still inside its lifetime.
    pub fn live(&self) -> impl Iterator<Item = &Emitter> {
        self.items.iter().filter(|emitter| emitter.alive)
    }

    /// Turn one queued `dComIfGp_particle_set` into its emitters. The point is the
    /// spawn's own, the yaw is the `csXyz` angle the overload was handed, and the
    /// size carries the `cXyz` scale, defaulting to one for the overload that
    /// takes neither.
    pub fn spawn(&mut self, particle: &DamageParticle) {
        let spec = spec_for(particle.id);
        let yaw = particle
            .angle
            .map_or(0.0, |angle| walk::from_s16(angle.y as i16));
        let scale = particle.scale.unwrap_or(Vec3::ONE);
        for _ in 0..spec.count {
            let slot = self.cursor;
            self.cursor = (slot + 1) % CAPACITY;
            let (sin, cos) = (yaw + self.jitter(spec.spread)).sin_cos();
            let roll = self.jitter(std::f32::consts::PI);
            self.items[slot] = Emitter {
                id: particle.id,
                pos: particle.pos,
                vel: Vec3::new(sin, 0.0, cos) * spec.speed,
                accel: Vec3::new(0.0, -spec.gravity, 0.0),
                drag: spec.drag,
                age: 0.0,
                life: spec.life,
                size: spec.size * scale,
                grow: spec.grow,
                color: spec.color,
                roll,
                roll_rate: spec.roll_rate,
                alive: true,
            };
        }
    }

    /// Advance every live emitter by `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        for emitter in &mut self.items {
            if !emitter.alive {
                continue;
            }
            let decay = (-emitter.drag * dt).exp();
            emitter.vel = (emitter.vel + emitter.accel * dt) * decay;
            emitter.pos += emitter.vel * dt;
            emitter.roll += emitter.roll_rate * dt;
            emitter.age += dt;
            if emitter.age >= emitter.life {
                emitter.alive = false;
            }
        }
    }

    fn jitter(&mut self, amount: f32) -> f32 {
        self.rand = self
            .rand
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let unit = (self.rand >> 8) as f32 / (1 << 24) as f32;
        (unit * 2.0 - 1.0) * amount
    }
}
