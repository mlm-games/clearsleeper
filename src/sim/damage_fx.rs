use bevy_ecs::prelude::*;
use glam::Vec3;

use super::damage_resolve::DamageFx;

/// The `DamageFx` values `cc_at_check` produced this frame. `items` is the `&mut Vec<DamageFx>`
/// the call sites hand the function, so the death branch's two hit marks and the steal-item
/// create outlive the sim step until the shell drains the queue.
#[derive(Resource, Debug, Default)]
pub struct DamageFxQueue {
    pub items: Vec<DamageFx>,
}

impl DamageFxQueue {
    /// One effect.
    pub fn push(&mut self, fx: DamageFx) {
        self.items.push(fx);
    }

    /// Every effect an iteration produced.
    pub fn extend(&mut self, fx: impl IntoIterator<Item = DamageFx>) {
        self.items.extend(fx);
    }

    /// The frame's effects, leaving the vector's allocation behind.
    pub fn take(&mut self) -> Vec<DamageFx> {
        std::mem::take(&mut self.items)
    }

    /// Splits the queue: the particles replace the presentation list and the steal-item creates
    /// come back for the caller to hand on. Neither list allocates again once both are warm.
    pub fn drain(&mut self, particles: &mut DamageParticles) -> Vec<StealItemCreate> {
        let mut steals = Vec::new();
        particles.items.clear();
        for fx in self.items.drain(..) {
            if let Some(particle) = DamageParticle::of(fx) {
                particles.items.push(particle);
            } else if let Some(steal) = StealItemCreate::of(fx) {
                steals.push(steal);
            }
        }
        steals
    }
}

/// One `dComIfGp_particle_set` the shell still has to draw: the particle name, the point it goes
/// at, and the angle and scale its overload was given. Both are `None` for the overload that
/// takes neither.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageParticle {
    pub id: u16,
    pub pos: Vec3,
    pub angle: Option<Vec3>,
    pub scale: Option<Vec3>,
}

impl DamageParticle {
    /// The particle a [`DamageFx`] holds, or `None` for the steal-item create.
    pub fn of(fx: DamageFx) -> Option<Self> {
        match fx {
            DamageFx::Particle {
                id,
                pos,
                angle,
                scale,
            } => Some(Self {
                id,
                pos,
                angle,
                scale,
            }),
            DamageFx::StealItem { .. } => None,
        }
    }
}

/// The steal-item create call's arguments: the point, the life-ball table and room of the
/// struck `fpcNm_HIMO2_e`, and the bit number its own counter held before the increment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StealItemCreate {
    pub pos: Vec3,
    pub table: i32,
    pub room: i8,
    pub item_bit_no: u8,
}

impl StealItemCreate {
    /// The steal-item create a [`DamageFx`] holds, or `None` for a particle.
    pub fn of(fx: DamageFx) -> Option<Self> {
        match fx {
            DamageFx::StealItem {
                pos,
                table,
                room,
                item_bit_no,
            } => Some(Self {
                pos,
                table,
                room,
                item_bit_no,
            }),
            DamageFx::Particle { .. } => None,
        }
    }
}

/// The frame's particles, for the view layer. The port has no particle renderer, so nothing
/// reads this yet; it is the one-frame list a renderer would take its spawns from.
#[derive(Resource, Debug, Default)]
pub struct DamageParticles {
    pub items: Vec<DamageParticle>,
}
