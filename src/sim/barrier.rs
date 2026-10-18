use bevy_ecs::prelude::*;

use glam::Vec3;

use super::actor::Actor;
use super::flags::EventFlags;

pub const BARRIER_ACTOR: &str = "Ycage00";

pub const BASE_RADIUS: f32 = 1000.0;
pub const BASE_HEIGHT: f32 = 10000.0;
pub const DROP: f32 = 300.0;
pub const AT_INSET: f32 = 60.0;
pub const TG_INSET: f32 = 20.0;
pub const FRAME_REACH: f32 = 150.0;
pub const FRAME_FAR: f32 = 700.0;
pub const FRAME_SPAN: f32 = 59.0;
pub const SWORD_REACH: f32 = 8800.0;

pub const BREAK_TRIGGER: u32 = 0x3980;
pub const BROKEN: u32 = 0x2C02;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Sealed,
    Cut,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Barrier {
    pub phase: Phase,
    pub scale: Vec3,
    pub frame: f32,
}

impl Barrier {
    pub fn spawn(scale: Vec3) -> Self {
        Self {
            phase: Phase::Sealed,
            scale,
            frame: 1.0,
        }
    }

    pub fn centre(&self, pos: Vec3) -> Vec3 {
        Vec3::new(pos.x, pos.y - DROP, pos.z)
    }

    pub fn radius(&self, at: bool) -> f32 {
        let base = self.scale.x * BASE_RADIUS;
        base - if at { AT_INSET } else { TG_INSET }
    }

    pub fn height(&self) -> f32 {
        self.scale.y * BASE_HEIGHT + DROP
    }

    pub fn held(&self, player: Vec3, pos: Vec3) -> bool {
        let dx = player.x - pos.x;
        let dz = player.z - pos.z;
        dx * dx + dz * dz <= self.radius(false) * self.radius(false)
    }

    pub fn shade(&mut self, player: Vec3, pos: Vec3) -> f32 {
        let dx = player.x - pos.x;
        let dz = player.z - pos.z;
        let reach = self.scale.x * BASE_RADIUS - FRAME_REACH;
        let away = (dx * dx + dz * dz).sqrt();
        let near = if away > reach { 0.0 } else { reach - away };
        if near > FRAME_FAR {
            self.frame = 1.0;
        } else {
            self.frame = (1.0 - near / FRAME_FAR) * FRAME_SPAN + 1.0;
        }
        self.frame
    }

    pub fn cut(&mut self) -> bool {
        if self.phase == Phase::Cut {
            return false;
        }
        self.phase = Phase::Cut;
        true
    }
}

pub fn is_barrier(name: &str) -> bool {
    name == BARRIER_ACTOR
}

pub fn armed(flags: &EventFlags) -> bool {
    flags.test(BREAK_TRIGGER) && !flags.test(BROKEN)
}

pub fn control(
    flags: Res<EventFlags>,
    weapon: Res<super::item::Held>,
    mut barriers: Query<(&Actor, &mut Barrier)>,
    players: Query<&super::player::Player>,
) {
    if !armed(&flags) {
        return;
    }
    let blade = super::damage::is_master_sword(weapon.item);
    let Some(player) = players.iter().next() else {
        return;
    };
    for (actor, mut barrier) in &mut barriers {
        barrier.shade(player.pos, actor.pos);
        if !blade {
            continue;
        }
        let dx = player.pos.x - actor.pos.x;
        let dz = player.pos.z - actor.pos.z;
        if dx * dx + dz * dz >= SWORD_REACH * SWORD_REACH {
            continue;
        }
        barrier.cut();
    }
}
