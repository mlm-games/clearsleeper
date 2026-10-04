use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::combat::HitPoints;
use super::health::Health;
use super::intent::MoveIntent;
use super::player::Player;

pub const BIRD_ACTOR: &str = "Bm1";
pub const MAX_ALTITUDE: f32 = 600.0;
pub const HOVER_STEP: f32 = 0.25;
pub const CHASE_STEP: f32 = 0.125;
pub const DIVE_ACCEL: f32 = 4.0;
pub const CLIMB_ACCEL: f32 = 3.0;
pub const DIVE_SPEED: f32 = 40.0;
pub const CRUISE_SPEED: f32 = 18.0;
pub const PLUNGE_RANGE: f32 = 90.0;
pub const PLUNGE_SPEED: f32 = 55.0;
pub const SIGHT: f32 = 260.0;
pub const HOVER_HEIGHT: f32 = 120.0;
pub const TURN_RATE: i16 = 0x400;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flight {
    Perch,
    Launch,
    Cruise,
    Plunge,
    Recover,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bird {
    pub flight: Flight,
    pub altitude: f32,
    pub speed_y: f32,
    pub timer: u32,
}

impl Bird {
    pub fn spawn() -> Self {
        Self {
            flight: Flight::Perch,
            altitude: 0.0,
            speed_y: 0.0,
            timer: 0,
        }
    }
}

pub fn spawn() -> (Bird, HitPoints) {
    (Bird::spawn(), HitPoints(1))
}

fn chase(current: f32, target: f32, step: f32) -> f32 {
    let diff = target - current;
    if diff.abs() <= step {
        target
    } else {
        current + diff.signum() * step
    }
}

pub fn control(
    intent: Res<MoveIntent>,
    mut health: ResMut<Health>,
    birds: Query<(Entity, &Actor, &mut Bird)>,
    players: Query<&Player>,
    mut commands: Commands,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    let swing = intent.axis.x != 0.0 || intent.axis.y != 0.0;
    for (entity, actor, mut bird) in &mut birds {
        let dx = player.pos.x - actor.pos.x;
        let dz = player.pos.z - actor.pos.z;
        let flat = (dx * dx + dz * dz).sqrt();
        match bird.flight {
            Flight::Perch => {
                if flat < SIGHT && !swing {
                    bird.flight = Flight::Launch;
                    bird.timer = 0;
                }
            }
            Flight::Launch => {
                bird.speed_y = chase(bird.speed_y, CRUISE_SPEED, CLIMB_ACCEL);
                bird.altitude += bird.speed_y;
                bird.timer += 1;
                if bird.altitude >= HOVER_HEIGHT {
                    bird.flight = Flight::Cruise;
                    bird.speed_y = 0.0;
                }
            }
            Flight::Cruise => {
                if flat < PLUNGE_RANGE {
                    bird.flight = Flight::Plunge;
                    bird.speed_y = -PLUNGE_SPEED;
                    continue;
                }
                if flat > SIGHT * 2.0 {
                    bird.flight = Flight::Recover;
                    continue;
                }
                bird.speed_y = chase(bird.speed_y, 0.0, HOVER_STEP);
                bird.altitude += bird.speed_y;
            }
            Flight::Plunge => {
                if flat <= PLUNGE_RANGE {
                    health.damage(1);
                    bird.flight = Flight::Recover;
                    bird.timer = 0;
                    let _ = commands.entity(entity).despawn();
                    continue;
                }
                bird.altitude += bird.speed_y;
                if bird.altitude <= 0.0 {
                    bird.flight = Flight::Recover;
                }
            }
            Flight::Recover => {
                bird.timer += 1;
                if bird.timer > 60 {
                    bird.flight = Flight::Perch;
                    bird.altitude = 0.0;
                    bird.timer = 0;
                }
            }
        }
        bird.altitude = bird.altitude.clamp(0.0, MAX_ALTITUDE);
    }
}

pub fn face(actor: &mut Actor, target: Vec3, home_yaw: i16) {
    let dx = target.x - actor.pos.x;
    let dz = target.z - actor.pos.z;
    if dx == 0.0 && dz == 0.0 {
        return;
    }
    let yaw = (dx.atan2(dz) / core::f32::consts::TAU * i16::MAX as f32) as i16;
    actor.angle[1] = home_yaw.wrapping_add((yaw.wrapping_sub(home_yaw)) / 4);
}

