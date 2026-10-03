use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::collision::Collision;
use super::ground::grounded;
use super::health::Health;
use super::hop::Hop;
use super::intent::{CameraYaw, ItemIntent, MoveIntent};
use super::jump::{self, Jump};
use super::lock::Lock;
use super::player::Player;
use super::roll::Roll;
use super::ship::Ship;
use super::swim::Swim;
use super::walk::{add_calc, from_s16, target_yaw, to_s16, turn_toward};

const STICK_GATE: f32 = 0.05;
const DAMAGE_START: u32 = 1;
const DAMAGE_END: u32 = 6;
const LUNGE_FRAME: u32 = 2;
const CHAIN_FRAME: u32 = 11;
const LUNGE_FACTOR: f32 = 0.2;
const DECEL_SCALE: f32 = 0.7;
const DECEL_MIN: f32 = 0.5;
const SWORD_OFFSET: f32 = 20.0;
const SWORD_RADIUS: f32 = 20.0;
const NORMAL_POWER: i32 = 1;
const FINISH_POWER: i32 = 2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CutKind {
    #[default]
    A,
    F,
    R,
    L,
    EA,
    EB,
}

fn cut(kind: CutKind) -> (u32, f32, f32) {
    match kind {
        CutKind::A => (13, 10.0, 2.6),
        CutKind::F => (13, 8.0, 0.95),
        CutKind::R => (13, 1.0, 0.95),
        CutKind::L => (12, 1.0, 0.95),
        CutKind::EA => (15, 15.0, 0.95),
        CutKind::EB => (17, 7.0, 0.95),
    }
}

fn is_finisher(kind: CutKind) -> bool {
    kind == CutKind::EA || kind == CutKind::EB
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Combat {
    pub active: bool,
    pub frame: u32,
    pub combo: u32,
    pub kind: CutKind,
    heading: i16,
    queued: bool,
    held: bool,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct HitPoints(pub i32);

impl Default for HitPoints {
    fn default() -> Self {
        Self(1)
    }
}

fn start(
    combat: &mut Combat,
    player: &mut Player,
    intent: &MoveIntent,
    camera: &CameraYaw,
    lock: &Lock,
) {
    combat.combo += 1;
    let stick = intent.axis.length().min(1.0);
    let desired = target_yaw(intent, camera);
    let dir = if stick <= STICK_GATE {
        None
    } else {
        Some(jump::direction(desired, to_s16(player.face)))
    };
    combat.kind = if combat.combo >= 4 {
        combat.combo = 0;
        match dir {
            Some(jump::Direction::Forward) | Some(jump::Direction::Right) => CutKind::EA,
            _ => CutKind::EB,
        }
    } else {
        match dir {
            Some(jump::Direction::Forward) => CutKind::F,
            Some(jump::Direction::Right) => CutKind::R,
            None if lock.target.is_some() => CutKind::A,
            _ => CutKind::L,
        }
    };
    combat.heading = if lock.target.is_some() || stick <= STICK_GATE {
        to_s16(player.face)
    } else {
        desired
    };
    combat.active = true;
    combat.frame = 1;
    combat.queued = false;
    player.yaw = player.face;
}

pub fn control(
    collision: Option<Res<Collision>>,
    item: Res<ItemIntent>,
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    health: Res<Health>,
    swim: Res<Swim>,
    roll: Res<Roll>,
    hop: Res<Hop>,
    jump: Res<Jump>,
    lock: Res<Lock>,
    ships: Query<&Ship>,
    mut actors: Query<(Entity, &Actor, Option<&mut HitPoints>), Without<Ship>>,
    mut combat: ResMut<Combat>,
    mut players: Query<&mut Player>,
    mut commands: Commands,
) {
    let pressed = item.0 && !combat.held;
    combat.held = item.0;
    let riding = ships.iter().any(|ship| ship.riding);
    for mut player in &mut players {
        let player = &mut *player;
        if swim.active || health.units == 0 {
            combat.active = false;
            combat.combo = 0;
            combat.queued = false;
            continue;
        }
        if !combat.active {
            if !pressed
                || riding
                || roll.active
                || hop.active
                || jump.active
                || !grounded(collision.as_deref(), player.pos)
            {
                continue;
            }
            start(&mut combat, player, &intent, &camera, &lock);
            continue;
        }
        let (duration, lunge_base, decel_max) = cut(combat.kind);
        if combat.frame >= duration {
            combat.active = false;
            combat.combo = 0;
            combat.queued = false;
            player.speed = 0.0;
            continue;
        }
        if combat.frame >= CHAIN_FRAME
            && combat.queued
            && grounded(collision.as_deref(), player.pos)
        {
            player.speed = 0.0;
            start(&mut combat, player, &intent, &camera, &lock);
            continue;
        }
        if pressed && !is_finisher(combat.kind) {
            combat.queued = true;
        }
        player.face = from_s16(turn_toward(to_s16(player.face), combat.heading));
        player.yaw = player.face;
        if combat.frame == LUNGE_FRAME {
            player.speed = LUNGE_FACTOR * player.speed.abs() + lunge_base;
        } else {
            add_calc(&mut player.speed, 0.0, DECEL_SCALE, decel_max, DECEL_MIN);
        }
        if combat.frame >= DAMAGE_START && combat.frame < DAMAGE_END {
            let center_x = player.pos.x + SWORD_OFFSET * player.yaw.sin();
            let center_z = player.pos.z + SWORD_OFFSET * player.yaw.cos();
            let power = if is_finisher(combat.kind) {
                FINISH_POWER
            } else {
                NORMAL_POWER
            };
            let mut dead: Vec<Entity> = Vec::new();
            for (entity, actor, hp) in actors.iter_mut() {
                let Some(mut hp) = hp else {
                    continue;
                };
                let dx = actor.pos.x - center_x;
                let dz = actor.pos.z - center_z;
                if dx * dx + dz * dz <= SWORD_RADIUS * SWORD_RADIUS {
                    hp.0 -= power;
                    if hp.0 <= 0 {
                        dead.push(entity);
                    }
                }
            }
            for entity in dead {
                let _ = commands.entity(entity).despawn();
            }
        }
        combat.frame += 1;
    }
}
