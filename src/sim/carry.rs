use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3, Vec3Swizzles};

use super::actor::Actor;
use super::intent::{DoIntent, MoveIntent};
use super::player::Player;

pub const PIG_ACTOR: &str = "Pig";
pub const GRAB_RANGE: f32 = 120.0;
pub const CARRY_HEIGHT: f32 = 70.0;
pub const CARRY_TURN_RATE: f32 = 0.4;
pub const RELEASE_SPEED: f32 = 12.0;
pub const CARRY_TRAIL: f32 = 47.0;
const TURN_SCALE: i16 = 4;
const TURN_MAX_STEP: i16 = 3000;
const TURN_MIN_STEP: i16 = 10;
pub const MAX_STACK: usize = 3;
pub const STACK_SPACING: f32 = 40.0;

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Carry {
    pub held: Option<Entity>,
    held_press: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Carryable {
    Tsubo,
    Stone,
    Pig,
    Barrel,
}

pub fn is_carryable(name: &str) -> bool {
    super::tsubo::is_tsubo(name)
        || super::koisi::is_koisi(name)
        || super::barrel::is_barrel(name)
        || name == PIG_ACTOR
}

fn chain_from<D: bevy_ecs::query::QueryFilter>(
    carryables: &Query<(Entity, &mut Actor, &Carryable), D>,
    base: Entity,
) -> Vec<Entity> {
    let mut out = vec![base];
    for (entity, _, _) in carryables.iter() {
        if out.len() >= MAX_STACK {
            break;
        }
        if entity != base {
            out.push(entity);
        }
    }
    out
}

fn reach(a: Vec3, b: Vec3) -> f32 {
    let dx = a.x - b.x;
    let dz = a.z - b.z;
    dx * dx + dz * dz
}

fn angle_of(yaw: f32) -> i16 {
    super::walk::to_s16(yaw)
}

fn turn_toward(from: i16, to: i16) -> i16 {
    super::walk::add_calc_angle_s(from, to, TURN_SCALE, TURN_MAX_STEP, TURN_MIN_STEP)
}

pub fn control(
    do_intent: Res<DoIntent>,
    intent: Res<MoveIntent>,
    mut carry: ResMut<Carry>,
    mut carryables: Query<(Entity, &mut Actor, &Carryable), Without<Player>>,
    mut pots: Query<&mut super::tsubo::Tsubo>,
    mut rocks: Query<&mut super::koisi::Koisi>,
    mut casks: Query<&mut super::barrel::Barrel>,
    players: Query<&Player>,
) {
    let pressed = do_intent.0 && !carry.held_press;
    carry.held_press = do_intent.0;
    let Some(player) = players.iter().next() else {
        carry.held = None;
        return;
    };
    let yaw = player.yaw + axis_yaw(intent.axis);
    let forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    match carry.held {
        Some(held) => {
            let releasing = pressed;
            if !releasing
                && let Ok(mut cask) = casks.get_mut(held)
            {
                cask.carry_step(true);
            }
            let chain = chain_from(&carryables, held);
            for (slot, entity) in chain.iter().enumerate() {
                let Ok((_, mut actor, _)) = carryables.get_mut(*entity) else {
                    continue;
                };
                let lift = CARRY_HEIGHT - STACK_SPACING * slot as f32;
                let anchor = Vec3::new(player.pos.x, player.pos.y + lift, player.pos.z);
                let trail = ((actor.pos - player.pos).xz().length() - CARRY_TRAIL).max(0.0);
                let step = (CARRY_TURN_RATE + trail.min(1.0)).min(1.0);
                actor.pos = actor.pos.lerp(anchor, step);
                actor.angle[1] = turn_toward(actor.angle[1], angle_of(yaw));
            }
            if releasing {
                carry.held = None;
                for entity in chain {
                    if let Ok((_, mut actor, _)) = carryables.get_mut(entity) {
                        actor.pos += forward * RELEASE_SPEED;
                        if let Ok(mut pot) = pots.get_mut(entity) {
                            pot.toss(forward, player.speed);
                        }
                        if let Ok(mut rock) = rocks.get_mut(entity) {
                            rock.toss(forward, player.speed);
                        }
                        if let Ok(mut cask) = casks.get_mut(entity) {
                            cask.carry_step(false);
                        }
                    }
                }
            }
        }
        None => {
            if !pressed {
                return;
            }
            let nearest = carryables
                .iter()
                .min_by(|(_, a, _), (_, b, _)| {
                    reach(a.pos, player.pos).total_cmp(&reach(b.pos, player.pos))
                })
                .map(|(entity, actor, _)| (entity, actor.pos));
            if let Some((entity, pos)) = nearest {
                if reach(pos, player.pos) > GRAB_RANGE * GRAB_RANGE {
                    return;
                }
                let liftable = match casks.get_mut(entity) {
                    Ok(mut cask) => cask.liftable(true),
                    Err(_) => true,
                };
                if !liftable {
                    return;
                }
                if let Ok(mut cask) = casks.get_mut(entity) {
                    cask.pick_up(angle_of(yaw));
                }
                carry.held = Some(entity);
            }
        }
    }
}

fn axis_yaw(axis: Vec2) -> f32 {
    if axis == Vec2::ZERO {
        0.0
    } else {
        axis.x.atan2(axis.y)
    }
}