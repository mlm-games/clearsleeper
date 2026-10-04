use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};

use super::actor::Actor;
use super::intent::{CameraYaw, DoIntent, MoveIntent};
use super::player::Player;
use super::sea::Sea;
use super::talk::Dialogue;
use super::walk::{add_calc, add_calc_angle_s, from_s16};
use super::water;

pub const SHIP_ACTOR: &str = "Ship";

const BOARD_RADIUS: f32 = 75.0;
const RIDE_OFFSET: [f32; 3] = [0.0, 15.0, -35.0];
pub(crate) const LEDGE_OFFSET_X: f32 = 57.0;
pub(crate) const DECK_RISE: f32 = 35.0;
const ROW_MAX_SPEED: f32 = 10.0;
const ROW_SCALE: f32 = 0.05;
const ROW_MAX_STEP: f32 = 0.1;
const ROW_MIN_STEP: f32 = 0.015;
const BRAKE_SCALE: f32 = 0.1;
const BRAKE_MAX_STEP: f32 = 1.0;
const BRAKE_MIN_STEP: f32 = 0.1;
const IDLE_SCALE: f32 = 0.1;
const IDLE_MAX_STEP: f32 = 1.0;
const IDLE_MIN_STEP: f32 = 0.05;
const STOP_SPEED: f32 = 3.0;
pub(crate) const GET_OFF_SPEED: f32 = 8.0;
const TILLER_LIMIT: i16 = 0x2000;
const TILLER_SCALE: i16 = 4;
const TILLER_MAX_STEP: i16 = 700;
const TILLER_MIN_STEP: i16 = 0x100;
const IDLE_TILLER_SCALE: i16 = 2;
const IDLE_TILLER_MAX_STEP: i16 = 0x1000;
const IDLE_TILLER_MIN_STEP: i16 = 0x400;
const TURN_BASE: f32 = 4.0;
const TURN_SPEED_DIVISOR: f32 = 55.0;
const TURN_SPEED_DROP: f32 = 3.0;
const TURN_RATE_MAX: f32 = 3.6;
const TURN_RATE_MIN: f32 = 0.1;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Ship {
    pub speed: f32,
    pub tiller: i16,
    pub riding: bool,
    pub warp_y: f32,
    held: bool,
}

impl Ship {
    pub fn velocity(&self, heading: i16) -> Vec3 {
        let yaw = from_s16(heading);
        Vec3::new(self.speed * yaw.sin(), 0.0, self.speed * yaw.cos())
    }
}

pub fn control(
    sea: Option<Res<Sea>>,
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    do_intent: Res<DoIntent>,
    dialogue: Res<Dialogue>,
    mut ships: Query<(&mut Actor, &mut Ship)>,
    mut players: Query<&mut Player>,
) {
    for (mut actor, mut ship) in &mut ships {
        let mut edge = do_intent.0 && !ship.held && !dialogue.open;
        ship.held = do_intent.0;
        if let Some(mut player) = players.iter_mut().next() {
            let player = &mut *player;
            let dx = player.pos.x - actor.pos.x;
            let dz = player.pos.z - actor.pos.z;
            let distance_sq = dx * dx + dz * dz;
            let radius_sq = BOARD_RADIUS * BOARD_RADIUS;
            if ship.riding {
                if distance_sq > radius_sq {
                    ship.riding = false;
                }
            } else if edge && distance_sq <= radius_sq {
                ship.riding = true;
                edge = false;
            }
            if ship.riding {
                ride(
                    &mut actor,
                    &mut ship,
                    player,
                    &intent,
                    &camera,
                    do_intent.0,
                    edge,
                );
            } else {
                idle(&mut ship);
            }
        } else {
            ship.riding = false;
            idle(&mut ship);
        }
        advance(&mut actor, &ship);
        settle(sea.as_deref(), &mut actor);
        if ship.riding {
            if let Some(mut player) = players.iter_mut().next() {
                carry(&actor, &mut *player);
            }
        }
    }
}

fn ride(
    actor: &mut Actor,
    ship: &mut Ship,
    player: &mut Player,
    intent: &MoveIntent,
    camera: &CameraYaw,
    pressed: bool,
    edge: bool,
) {
    if edge && ship.speed < STOP_SPEED {
        let yaw = from_s16(actor.angle[1].wrapping_sub(0x4000));
        player.pos.x += yaw.sin() * LEDGE_OFFSET_X;
        player.pos.y = actor.pos.y + DECK_RISE;
        player.pos.z += yaw.cos() * LEDGE_OFFSET_X;
        player.yaw = yaw;
        player.speed = GET_OFF_SPEED;
        player.speed_y = 0.0;
        ship.riding = false;
        return;
    }
    let axis = clamp_axis(intent.axis);
    let forward = Vec2::new(camera.0.sin(), camera.0.cos());
    let right = Vec2::new(-camera.0.cos(), camera.0.sin());
    let direction = forward * axis.y + right * axis.x;
    let yaw = from_s16(actor.angle[1]);
    let longitudinal = direction.x * yaw.sin() + direction.y * yaw.cos();
    let lateral = -direction.x * yaw.cos() + direction.y * yaw.sin();
    let target_tiller = (lateral * TILLER_LIMIT as f32) as i16;
    ship.tiller = add_calc_angle_s(
        ship.tiller,
        target_tiller,
        TILLER_SCALE,
        TILLER_MAX_STEP,
        TILLER_MIN_STEP,
    );
    let mut turn_rate = TURN_BASE - ship.speed.abs() / TURN_SPEED_DIVISOR * TURN_SPEED_DROP;
    if turn_rate > TURN_RATE_MAX {
        turn_rate = TURN_RATE_MAX;
    } else if turn_rate < TURN_RATE_MIN {
        turn_rate = TURN_RATE_MIN;
    }
    let step = (-turn_rate * f32::from(ship.tiller >> 6)) as i16;
    actor.angle[1] = actor.angle[1].wrapping_add(step);
    if pressed && ship.speed >= STOP_SPEED {
        add_calc(
            &mut ship.speed,
            0.0,
            BRAKE_SCALE,
            BRAKE_MAX_STEP,
            BRAKE_MIN_STEP,
        );
    } else {
        add_calc(
            &mut ship.speed,
            longitudinal * ROW_MAX_SPEED,
            ROW_SCALE,
            ROW_MAX_STEP,
            ROW_MIN_STEP,
        );
    }
}

fn idle(ship: &mut Ship) {
    add_calc(
        &mut ship.speed,
        0.0,
        IDLE_SCALE,
        IDLE_MAX_STEP,
        IDLE_MIN_STEP,
    );
    ship.tiller = add_calc_angle_s(
        ship.tiller,
        0,
        IDLE_TILLER_SCALE,
        IDLE_TILLER_MAX_STEP,
        IDLE_TILLER_MIN_STEP,
    );
}

fn advance(actor: &mut Actor, ship: &Ship) {
    let yaw = from_s16(actor.angle[1]);
    actor.pos.x += ship.speed * yaw.sin();
    actor.pos.z += ship.speed * yaw.cos();
}

fn settle(sea: Option<&Sea>, actor: &mut Actor) {
    let (_, surface) = water::probe(sea, actor.pos.x, actor.pos.z);
    actor.pos.y = surface;
}

fn carry(actor: &Actor, player: &mut Player) {
    let yaw = from_s16(actor.angle[1]);
    player.pos.x = actor.pos.x + RIDE_OFFSET[0] * yaw.cos() + RIDE_OFFSET[2] * yaw.sin();
    player.pos.y = actor.pos.y + RIDE_OFFSET[1];
    player.pos.z = actor.pos.z - RIDE_OFFSET[0] * yaw.sin() + RIDE_OFFSET[2] * yaw.cos();
    player.yaw = from_s16(actor.angle[1]);
    player.speed_y = 0.0;
}

fn clamp_axis(axis: Vec2) -> Vec2 {
    let length = axis.length();
    if length > 1.0 { axis / length } else { axis }
}
