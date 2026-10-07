use bevy_ecs::prelude::*;

use super::collision::Collision;
use super::link_proc::{SWIM_ALLOWED, allows};
use super::player::Player;
use super::sea::Sea;
use super::water::{probe, Water};

pub const MIN_DEPTH: f32 = 90.0;
const STAND_DEPTH: f32 = 85.0;
const ENTRY_SPEED_SCALE: f32 = 0.75;
const PLUNGE_MIN_SPEED: f32 = -50.0;
const BUOY_ACCEL: f32 = 6.0;
const BUOY_TERM: f32 = 9.5;
const SURFACE_ARRIVE_BASE: f32 = 40.1;
const SURFACE_ARRIVE_HIO_28: f32 = 15.0;
const SURFACE_ARRIVE: f32 = SURFACE_ARRIVE_BASE + SURFACE_ARRIVE_HIO_28;
pub const AIR_MAX: i32 = 900;
const AIR_DEEP: f32 = 175.0;

pub const SWIM_MAX_SPEED: f32 = 18.0;
pub const SWIM_DRIVE: f32 = 3.0;
pub const SWIM_TURN_SCALE: i16 = 17;
pub const SWIM_TURN_MAX: i16 = 5000;
pub const SWIM_TURN_MIN: i16 = 1200;
pub const SWIM_DECEL_SCALE: f32 = 0.02;
pub const SWIM_DECEL_MAX: f32 = 2.0;
pub const SWIM_DECEL_MIN: f32 = 0.5;
pub const BACKWARD_LIMIT: u16 = 0x6000;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Swim {
    pub active: bool,
    pub surfaced: bool,
    pub air: i32,
}

pub fn control(
    water: Res<Water>,
    collision: Option<Res<Collision>>,
    proc: Option<Res<super::link_proc::LinkProc>>,
    mut swim: ResMut<Swim>,
    mut players: Query<&mut Player>,
) {
    for mut player in &mut players {
        let player = &mut *player;
        if !allows(proc.as_deref(), SWIM_ALLOWED) {
            continue;
        }
        if !swim.active && water.in_area && water.surface - player.pos.y > MIN_DEPTH {
            enter(player, &mut swim, &water);
        }
        if swim.active {
            keep(player, &mut swim, &water, collision.as_deref());
        }
    }
}

pub fn surface(
    sea: Option<Res<Sea>>,
    mut water: ResMut<Water>,
    swim: Res<Swim>,
    mut players: Query<&mut Player>,
) {
    for mut player in &mut players {
        let player = &mut *player;
        let (in_area, height) = probe(sea.as_deref(), player.pos.x, player.pos.z);
        water.in_area = in_area;
        water.surface = height;
        if swim.active && swim.surfaced && in_area {
            player.pos.y = height;
            player.speed_y = 0.0;
        }
    }
}

fn enter(player: &mut Player, swim: &mut Swim, water: &Water) {
    swim.active = true;
    swim.surfaced = false;
    swim.air = AIR_MAX;
    player.speed = (player.speed * ENTRY_SPEED_SCALE).min(SWIM_MAX_SPEED);
    if player.speed_y < 0.0 {
        player.pos.y += MIN_DEPTH;
        player.speed_y = player.speed_y.clamp(PLUNGE_MIN_SPEED, 0.0);
    } else {
        player.pos.y = water.surface;
        player.speed_y = 0.0;
        swim.surfaced = true;
    }
}

fn keep(
    player: &mut Player,
    swim: &mut Swim,
    water: &Water,
    collision: Option<&Collision>,
) {
    if !water.in_area {
        fall_out(player, swim);
        return;
    }
    let ground =
        collision.and_then(|collision| collision.ground_at(player.pos.x, player.pos.z));
    if let Some(ground) = ground {
        if water.surface - ground < STAND_DEPTH {
            stand_out(player, swim, water);
            return;
        }
    }
    let deep = match ground {
        Some(ground) => ground <= water.surface - AIR_DEEP,
        None => true,
    };
    if deep {
        swim.air = (swim.air - 1).max(0);
    } else {
        swim.air = AIR_MAX;
    }
    if swim.surfaced {
        player.pos.y = water.surface;
        player.speed_y = 0.0;
        return;
    }
    if player.speed_y < BUOY_TERM {
        player.speed_y = (player.speed_y + BUOY_ACCEL).min(BUOY_TERM);
    }
    player.pos.y += player.speed_y;
    if water.surface - player.pos.y < SURFACE_ARRIVE && player.speed_y >= 0.0 {
        swim.surfaced = true;
        player.pos.y = water.surface;
        player.speed_y = 0.0;
    }
}

fn fall_out(player: &mut Player, swim: &mut Swim) {
    swim.active = false;
    swim.surfaced = false;
    swim.air = 0;
    player.speed = 0.0;
    player.speed_y = 0.0;
}

fn stand_out(player: &mut Player, swim: &mut Swim, water: &Water) {
    swim.active = false;
    swim.surfaced = false;
    swim.air = 0;
    player.pos.y = water.surface;
    player.speed_y = 0.0;
}
