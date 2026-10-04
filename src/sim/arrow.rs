use bevy_ecs::prelude::*;

use glam::Vec3;

use super::actor::Actor;
use super::combat::HitPoints;
use super::damage;
use super::enemy::Hurt;
use super::ground;
use super::ship::Ship;
use super::walk::swept;

pub const ARROW_ACTOR: &str = "Arrow";

pub const PARAM_NOCKED: u32 = 0;
pub const PARAM_LOOSED: u32 = 1;
pub const PARAM_WATER: u32 = 4;

pub const TYPE_NORMAL: u8 = 0;
pub const TYPE_FIRE: u8 = 1;
pub const TYPE_ICE: u8 = 2;
pub const TYPE_LIGHT: u8 = 3;

pub const MUZZLE_SPEED: f32 = 200.0;
pub const CAPSULE_RADIUS: f32 = 5.0;
pub const CAPSULE_STRETCH: f32 = 1.25;
pub const BODY_HALF: f32 = 30.0;
pub const POWER: i32 = damage::ARROW;
pub const FLINCH: u32 = 10;
pub const STUCK_FRAMES: i32 = 300;
pub const BLINK_FRAMES: i32 = 60;
pub const FIRE_WATER_FRAMES: i32 = 1;
pub const ICE_WATER_FRAMES: i32 = 10 * 30;
pub const MAGIC_COST: [i16; 4] = [0, 1, 1, 2];
pub const ROUNDS: u8 = 5;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Nocked,
    Flying,
    StuckActor,
    StuckGround,
    Drowning,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Arrow {
    pub pos: Vec3,
    pub parameter: u32,
    pub kind: u8,
    pub phase: Phase,
    pub speed: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub round: u8,
    pub frames: i32,
    pub reflected: bool,
    pub visible: bool,
}

impl Arrow {
    pub fn nocked(kind: u8, round: u8) -> Self {
        Self {
            pos: Vec3::ZERO,
            parameter: PARAM_NOCKED,
            kind,
            phase: Phase::Nocked,
            speed: Vec3::ZERO,
            pitch: 0.0,
            yaw: 0.0,
            round,
            frames: 0,
            reflected: false,
            visible: true,
        }
    }

    pub fn shoot(&mut self, pitch: f32, yaw: f32) {
        self.parameter = PARAM_LOOSED;
        self.phase = Phase::Flying;
        self.pitch = pitch;
        self.yaw = yaw;
        let x_cos = pitch.cos();
        self.speed = Vec3::new(
            MUZZLE_SPEED * yaw.sin() * x_cos,
            MUZZLE_SPEED * pitch.sin(),
            MUZZLE_SPEED * yaw.cos() * x_cos,
        );
    }

    pub fn water_frames(&self) -> i32 {
        match self.kind {
            TYPE_FIRE => FIRE_WATER_FRAMES,
            TYPE_ICE => ICE_WATER_FRAMES,
            _ => 0,
        }
    }

    pub fn drown(&mut self) {
        self.parameter = PARAM_WATER;
        self.phase = Phase::Drowning;
        self.frames = self.water_frames();
        self.visible = self.frames > 0;
    }

    pub fn stick(&mut self, phase: Phase) {
        self.phase = phase;
        self.frames = STUCK_FRAMES;
        self.speed = Vec3::ZERO;
    }

    pub fn blink(&mut self) -> bool {
        self.visible = self.frames > BLINK_FRAMES || self.frames % 2 == 0;
        self.visible
    }

}

pub fn is_arrow(name: &str) -> bool {
    name == ARROW_ACTOR
}

pub fn control(
    collision: Option<Res<super::collision::Collision>>,
    wet: Option<Res<super::water::Water>>,
    mut arrows: Query<(Entity, &mut Arrow)>,
    mut foes: Query<(&Actor, &mut HitPoints, Option<&mut Hurt>), Without<Ship>>,
    mut flags: ResMut<super::flags::ZoneActors>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let soaked = wet.as_deref().is_some_and(|water| water.in_area);
    let mut spent: Vec<Entity> = Vec::new();
    let mut slain: Vec<(i8, u16)> = Vec::new();
    for (entity, mut arrow) in &mut arrows {
        match arrow.phase {
            Phase::Nocked => continue,
            Phase::Flying => {
                let from = arrow.pos;
                let step = arrow.speed;
                arrow.pos += step;
                let tip = arrow.pos + step * CAPSULE_STRETCH;
                let mut struck = false;
                for (foe, mut hp, mut hurt) in &mut foes {
                    if !swept(from, tip, foe.pos, CAPSULE_RADIUS + BODY_HALF) {
                        continue;
                    }
                    hp.0 -= POWER;
                    if let Some(hurt) = hurt.as_deref_mut() {
                        hurt.0 = FLINCH;
                    }
                    if hp.0 <= 0 {
                        slain.push((foe.room, foe.set_id));
                    }
                    struck = true;
                    break;
                }
                if struck {
                    arrow.stick(Phase::StuckActor);
                    continue;
                }
                if soaked && arrow.kind != TYPE_LIGHT {
                    arrow.drown();
                    continue;
                }
                if ground::surface(solid, arrow.pos).is_some_and(|floor| arrow.pos.y <= floor) {
                    arrow.stick(Phase::StuckGround);
                }
            }
            Phase::StuckActor | Phase::StuckGround => {
                arrow.frames -= 1;
                arrow.blink();
                if arrow.frames <= 0 {
                    spent.push(entity);
                }
            }
            Phase::Drowning => {
                arrow.frames -= 1;
                arrow.visible = arrow.frames > 0;
                if arrow.frames <= 0 {
                    spent.push(entity);
                }
            }
        }
    }
    for (room, set_id) in slain {
        flags.mark_done(room, set_id);
    }
    for entity in spent {
        let _ = commands.entity(entity).despawn();
    }
}