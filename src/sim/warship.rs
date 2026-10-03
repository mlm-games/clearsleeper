use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::flags::Switches;
use super::health::Health;
use super::player::Player;
use super::walk::{add_calc_angle_s, from_s16, to_s16};

pub const SHIP_ACTOR: &str = "Oship";

const SWITCH_SHIFT: u32 = 24;
const HOME_SWITCH_MASK: u16 = 0xFF;
const NEAR_RANGE: f32 = 2500.0;
const FAR_RANGE: f32 = 12000.0;
const FIRE_DELAY: u32 = 30;
const RELOAD_DELAY: u32 = 200;
const CANNON_OFFSET: f32 = 450.0;
const CANNON_SPEED: f32 = 85.0;
const CANNON_GRAVITY: f32 = -2.0;
const CANNON_FREE: u32 = 35;
const CANNON_LIFE: u32 = 240;
const CANNON_REACH: f32 = 40.0;
const CANNON_POWER: u8 = 4;
const SAIL_SPEED: f32 = 20.0;
const SAIL_SCALE: f32 = 0.25;
const TURN_SCALE: i16 = 2;
const TURN_MAX: i16 = 0x100;
const AIM_SCALE: i16 = 6;
const AIM_MAX: i16 = 0x300;
const BODY_HEIGHT: f32 = 300.0;
const BODY_HALF: f32 = 300.0;

pub const BODY: (f32, f32) = (BODY_HEIGHT, BODY_HALF);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Warship {
    pub home_yaw: i16,
    pub aim: i16,
    pub pitch: f32,
    pub timer: u32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Cannon {
    pub pos: Vec3,
    pub vel: Vec3,
    pub free: u32,
    pub life: u32,
}

pub fn is_switched(parameters: u32, home_x: i16, switches: &Switches) -> bool {
    let away = ((parameters >> SWITCH_SHIFT) & 0xFF) as u8;
    let home = (home_x as u16 & HOME_SWITCH_MASK) as u8;
    if away != 0xFF && switches.0.test(away) {
        return false;
    }
    home != 0xFF && !switches.0.test(home)
}

pub fn spawn(home_yaw: i16) -> Warship {
    Warship {
        home_yaw,
        aim: home_yaw,
        pitch: 0.0,
        timer: 0,
    }
}

fn fire(ship: &Actor, warship: &Warship, target: Vec3, commands: &mut Commands) {
    let yaw = from_s16(warship.aim);
    let flat = Vec3::new(target.x - ship.pos.x, 0.0, target.z - ship.pos.z);
    let range = flat.length().max(1.0);
    let pitch = (target.y + 60.0 - ship.pos.y).atan2(range);
    let (sin, cos) = pitch.sin_cos();
    let pos = ship.pos + Vec3::new(CANNON_OFFSET * yaw.sin(), 0.0, CANNON_OFFSET * yaw.cos());
    commands.spawn(Cannon {
        pos,
        vel: Vec3::new(
            CANNON_SPEED * cos * yaw.sin(),
            CANNON_SPEED * sin,
            CANNON_SPEED * cos * yaw.cos(),
        ),
        free: CANNON_FREE,
        life: CANNON_LIFE,
    });
}

pub fn control(
    mut health: ResMut<Health>,
    players: Query<&Player>,
    mut ships: Query<(&mut Actor, &mut Warship)>,
    mut shots: Query<(Entity, &mut Cannon)>,
    mut commands: Commands,
) {
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    for (mut ship, mut warship) in &mut ships {
        let dx = player.pos.x - ship.pos.x;
        let dz = player.pos.z - ship.pos.z;
        let dist = (dx * dx + dz * dz).sqrt();
        ship.angle[1] = add_calc_angle_s(ship.angle[1], warship.home_yaw, TURN_SCALE, TURN_MAX, 0);
        if dist < FAR_RANGE {
            warship.aim =
                add_calc_angle_s(warship.aim, to_s16(dx.atan2(dz)), AIM_SCALE, AIM_MAX, 0);
            let range = dist.max(1.0);
            warship.pitch = (player.pos.y + 60.0 - ship.pos.y).atan2(range);
        }
        let yaw = from_s16(ship.angle[1]);
        ship.pos.x += SAIL_SPEED * SAIL_SCALE * yaw.sin();
        ship.pos.z += SAIL_SPEED * SAIL_SCALE * yaw.cos();
        if warship.timer > 0 {
            warship.timer -= 1;
            continue;
        }
        if dist < NEAR_RANGE {
            fire(&ship, &warship, player.pos, &mut commands);
            warship.timer = FIRE_DELAY;
        } else {
            warship.timer = RELOAD_DELAY;
        }
    }
    for (entity, mut shot) in &mut shots {
        if shot.free > 0 {
            shot.free -= 1;
        } else {
            shot.vel.y += CANNON_GRAVITY;
        }
        let step = shot.vel;
        shot.pos += step;
        shot.life = shot.life.saturating_sub(1);
        if shot.pos.distance(player.pos) <= CANNON_REACH {
            health.damage(CANNON_POWER);
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if shot.life == 0 || shot.pos.y < player.pos.y - 400.0 {
            let _ = commands.entity(entity).despawn();
        }
    }
}
