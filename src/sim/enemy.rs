use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::combat::HitPoints;
use super::health::Health;
use super::player::Player;
use super::walk::{add_calc, add_calc_angle_s, from_s16, to_s16};

pub const ENEMY_ACTOR: &str = "Bk";

const ALERT_TICKS: u32 = 30;
const SIGHT_WANDER: f32 = 1000.0;
const SIGHT_STAND: f32 = 500.0;
const HEIGHT_EYE: f32 = 60.0;
const HEIGHT_CENTER: f32 = 50.0;
const HEIGHT_LIMIT: f32 = 300.0;
const CONE_LIMIT: u16 = 0x6590;
const BOX_SIDE: f32 = 500.0;
const BOX_UP: f32 = 300.0;
const BOX_NEAR: f32 = -125.0;
const BOX_FAR: f32 = 500.0;
const TURN_SCALE: i16 = 4;
const TURN_MAX: i16 = 0x800;
const HOME_TURN_SCALE: i16 = 2;
const HOME_TURN_MAX: i16 = 0x800;
const CHASE_SPEED: f32 = 90.0;
const RAMP_SPEED: f32 = 45.0;
const RAMP_STEP: f32 = 5.0;
const MOVE_SCALE: f32 = 0.25;
const ATTACK_RANGE: f32 = 240.0;
const REATTACK_RANGE: f32 = 400.0;
const GIVE_UP_RANGE: f32 = 475.0;
const COOLDOWN_TICKS: u32 = 35;
const KICK_SPEED: f32 = 70.0;
const DECEL_STEP: f32 = 20.0;
const HIT_OFFSET: f32 = 20.0;
const SWORD_RADIUS: f32 = 60.0;
const KICK_RADIUS: f32 = 25.0;
const ATTACK_POWER: u8 = 1;
const SWORD_WINDOW: (f32, f32) = (23.0, 26.0);
const KICK_WINDOW: (f32, f32) = (0.0, 20.0);
const SWORD_TRACK: f32 = 10.0;
const KICK_TRACK: f32 = 0.0;
const SWORD_STEPS: [u32; 3] = [16, 6, 22];
const KICK_STEPS: [u32; 2] = [23, 8];
const SWORD_SPEEDS: [f32; 3] = [1.2, 1.0, 1.0];
const KICK_SPEEDS: [f32; 2] = [1.0, 1.0];
const STAND_KINDS: [u32; 3] = [4, 0xA, 0xB];
const PLANTED_KIND: u32 = 0xB;
const STRONG_BIT: u32 = 0x20;
const STRONG_HP: i32 = 7;
const BASE_HP: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyState {
    Wait,
    Alert,
    Chase,
    Attack,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Enemy {
    pub armed: bool,
    pub always_sees: bool,
    pub sight: f32,
    pub home_yaw: i16,
    pub state: EnemyState,
    pub alert: u32,
    pub cooldown: u32,
    pub closing: bool,
    pub speed: f32,
    pub heading: i16,
    pub track: f32,
    pub clock: f32,
    pub step: usize,
    pub step_tick: u32,
    pub struck: bool,
}

pub fn spawn(parameters: u32, home_yaw: i16) -> (Enemy, HitPoints) {
    let kind = parameters & 0xF;
    let armed = kind != PLANTED_KIND;
    let strong = parameters & STRONG_BIT != 0 && armed;
    let sight = if STAND_KINDS.contains(&kind) {
        let band = (parameters >> 8) & 0xFF;
        if band == 0xFF {
            SIGHT_STAND
        } else {
            band as f32 * 10.0
        }
    } else {
        SIGHT_WANDER
    };
    (
        Enemy {
            armed,
            always_sees: kind == 0xA,
            sight,
            home_yaw,
            state: EnemyState::Wait,
            alert: 0,
            cooldown: 0,
            closing: false,
            speed: 0.0,
            heading: home_yaw,
            track: 0.0,
            clock: 0.0,
            step: 0,
            step_tick: 0,
            struck: false,
        },
        HitPoints(if strong { STRONG_HP } else { BASE_HP }),
    )
}

fn visible(enemy: &Enemy, actor: &Actor, yaw: i16, player_pos: Vec3) -> bool {
    if enemy.always_sees {
        return true;
    }
    let dx = player_pos.x - actor.pos.x;
    let dy = player_pos.y - actor.pos.y;
    let dz = player_pos.z - actor.pos.z;
    let eye = actor.pos.y + HEIGHT_EYE;
    if (player_pos.y + HEIGHT_CENTER - eye).abs() > HEIGHT_LIMIT {
        return false;
    }
    let angle = to_s16(dx.atan2(dz));
    if yaw.wrapping_sub(angle).unsigned_abs() < CONE_LIMIT {
        return true;
    }
    let (sin, cos) = from_s16(yaw).sin_cos();
    let local_x = dx * cos - dz * sin;
    let local_z = dx * sin + dz * cos;
    local_x.abs() < BOX_SIDE && dy.abs() < BOX_UP && local_z > BOX_NEAR && local_z < BOX_FAR
}

fn step_ticks(armed: bool, step: usize) -> u32 {
    if armed {
        SWORD_STEPS[step]
    } else {
        KICK_STEPS[step]
    }
}

fn step_speed(armed: bool, step: usize) -> f32 {
    if armed {
        SWORD_SPEEDS[step]
    } else {
        KICK_SPEEDS[step]
    }
}

fn step_total(armed: bool) -> usize {
    if armed {
        SWORD_STEPS.len()
    } else {
        KICK_STEPS.len()
    }
}

fn advance(actor: &mut Actor, enemy: &Enemy) {
    let yaw = from_s16(actor.angle[1]);
    actor.pos.x += enemy.speed * MOVE_SCALE * yaw.sin();
    actor.pos.z += enemy.speed * MOVE_SCALE * yaw.cos();
}

fn begin(enemy: &mut Enemy, angle: i16) {
    enemy.state = EnemyState::Attack;
    enemy.heading = angle;
    enemy.track = if enemy.armed { SWORD_TRACK } else { KICK_TRACK };
    enemy.clock = 0.0;
    enemy.step = 0;
    enemy.step_tick = 0;
    enemy.struck = false;
}

fn finish(enemy: &mut Enemy, actor: &Actor, player: &Player, dist: f32) {
    enemy.cooldown = COOLDOWN_TICKS;
    enemy.clock = 0.0;
    enemy.step = 0;
    enemy.step_tick = 0;
    if dist < REATTACK_RANGE && visible(enemy, actor, actor.angle[1], player.pos) {
        enemy.state = EnemyState::Chase;
    } else {
        enemy.state = EnemyState::Wait;
        enemy.speed = 0.0;
    }
}

fn tick(enemy: &mut Enemy, actor: &mut Actor, player: &Player, health: &mut Health) {
    if enemy.cooldown > 0 {
        enemy.cooldown -= 1;
    }
    let dx = player.pos.x - actor.pos.x;
    let dz = player.pos.z - actor.pos.z;
    let dist = (dx * dx + dz * dz).sqrt();
    let bearing = to_s16(dx.atan2(dz));
    match enemy.state {
        EnemyState::Wait => {
            enemy.speed = 0.0;
            enemy.closing = false;
            actor.angle[1] = add_calc_angle_s(
                actor.angle[1],
                enemy.home_yaw,
                HOME_TURN_SCALE,
                HOME_TURN_MAX,
                0,
            );
            if dist < enemy.sight && visible(enemy, actor, actor.angle[1], player.pos) {
                enemy.state = EnemyState::Alert;
                enemy.alert = ALERT_TICKS;
            }
        }
        EnemyState::Alert => {
            enemy.speed = 0.0;
            enemy.alert -= 1;
            if enemy.alert == 0 {
                enemy.state = EnemyState::Chase;
            }
        }
        EnemyState::Chase => {
            actor.angle[1] = add_calc_angle_s(actor.angle[1], bearing, TURN_SCALE, TURN_MAX, 0);
            if enemy.armed && enemy.cooldown == 0 {
                enemy.speed = CHASE_SPEED;
            } else {
                add_calc(&mut enemy.speed, RAMP_SPEED, 1.0, RAMP_STEP, 0.0);
            }
            if dist <= GIVE_UP_RANGE {
                enemy.closing = true;
            }
            if dist > enemy.sight || (enemy.closing && dist > GIVE_UP_RANGE) {
                enemy.state = EnemyState::Wait;
                enemy.speed = 0.0;
                enemy.closing = false;
            } else if dist < ATTACK_RANGE && enemy.cooldown == 0 {
                begin(enemy, bearing);
            }
            advance(actor, enemy);
        }
        EnemyState::Attack => {
            let armed = enemy.armed;
            if enemy.clock == 0.0 && !armed {
                enemy.speed = KICK_SPEED;
            }
            add_calc(&mut enemy.speed, 0.0, 1.0, DECEL_STEP, 0.0);
            enemy.clock += step_speed(armed, enemy.step);
            if enemy.clock < enemy.track {
                enemy.heading = bearing;
            }
            actor.angle[1] =
                add_calc_angle_s(actor.angle[1], enemy.heading, TURN_SCALE, TURN_MAX, 0);
            if !enemy.struck {
                let (low, high) = if armed { SWORD_WINDOW } else { KICK_WINDOW };
                if enemy.clock >= low && enemy.clock <= high {
                    let yaw = from_s16(actor.angle[1]);
                    let center_x = actor.pos.x + HIT_OFFSET * yaw.sin();
                    let center_z = actor.pos.z + HIT_OFFSET * yaw.cos();
                    let reach = if armed { SWORD_RADIUS } else { KICK_RADIUS };
                    let hit_x = player.pos.x - center_x;
                    let hit_z = player.pos.z - center_z;
                    if hit_x * hit_x + hit_z * hit_z <= reach * reach {
                        enemy.struck = true;
                        health.damage(ATTACK_POWER);
                    }
                }
            }
            enemy.step_tick += 1;
            if enemy.step_tick == step_ticks(armed, enemy.step) {
                enemy.step += 1;
                enemy.step_tick = 0;
                if enemy.step == step_total(armed) {
                    finish(enemy, actor, player, dist);
                }
            }
            advance(actor, enemy);
        }
    }
}

pub fn control(
    mut health: ResMut<Health>,
    players: Query<&Player>,
    mut enemies: Query<(&mut Enemy, &mut Actor)>,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    for (mut enemy, mut actor) in &mut enemies {
        tick(&mut enemy, &mut actor, player, &mut health);
    }
}
