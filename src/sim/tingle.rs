use bevy_ecs::prelude::*;
use glam::Vec3;

pub const TINGLE_ACTOR: &str = "Tn";

pub const ACTION_STAND: u8 = 1;
pub const ACTION_FIGHT_RUN: u8 = 4;
pub const ACTION_FIGHT: u8 = 5;
pub const ACTION_DEFENCE: u8 = 10;
pub const ACTION_HUKKI: u8 = 11;
pub const ACTION_FAIL: u8 = 20;

pub const MODE_IDLE: u8 = 0;
pub const MODE_HOMING: u8 = 1;
pub const MODE_FALLEN: u8 = 10;
pub const MODE_RISE: u8 = 11;
pub const MODE_CHARGE: u8 = 20;

pub const FLEE_SPEED: f32 = -30.0;
pub const RISE_TICKS: u32 = 50;
pub const HOME_SCALE: f32 = 0.5;
pub const HOME_RATE: f32 = 0.25;
pub const HOME_MIN: f32 = 0.0;
pub const CHARGE_TICKS: u32 = 30;
pub const NOTALK_RANGE: f32 = 1000.0;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Tingle {
    pub action: u8,
    pub mode: u8,
    pub timer: u32,
    pub distance: f32,
    pub facing: i16,
    pub shield: bool,
    pub home: Vec3,
    pub speed: Vec3,
}

impl Tingle {
    pub fn spawn(home: Vec3) -> Self {
        Self {
            action: ACTION_STAND,
            mode: MODE_IDLE,
            timer: 0,
            distance: 0.0,
            facing: 0,
            shield: false,
            home,
            speed: Vec3::ZERO,
        }
    }

    pub fn homing(&mut self, pos: &mut Vec3) {
        super::walk::add_calc(
            &mut pos.x,
            self.home.x,
            HOME_SCALE,
            self.speed.x * HOME_RATE,
            HOME_MIN,
        );
        super::walk::add_calc(
            &mut pos.z,
            self.home.z,
            HOME_SCALE,
            self.speed.z * HOME_RATE,
            HOME_MIN,
        );
        self.speed = Vec3::ZERO;
    }



    pub fn strike(&mut self) {
        self.action = ACTION_FIGHT;
        self.mode = MODE_IDLE;
    }

    pub fn flinch(&mut self) {
        self.action = ACTION_DEFENCE;
        self.mode = MODE_HOMING;
        self.timer = CHARGE_TICKS;
    }

    pub fn recover(&mut self) {
        self.action = ACTION_STAND;
        self.mode = MODE_IDLE;
        self.timer = 0;
        self.shield = false;
    }
}

pub fn control(
    players: Query<&super::player::Player>,
    mut actors: Query<(&mut super::actor::Actor, &mut Tingle)>,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    for (actor, mut tingle) in &mut actors {
        let dx = player.pos.x - actor.pos.x;
        let dz = player.pos.z - actor.pos.z;
        tingle.distance = (dx * dx + dz * dz).sqrt();
        if tingle.mode == MODE_HOMING {
            tingle.homing(&mut actor.pos);
        }
        tingle.facing = super::walk::to_s16(dx.atan2(dz));
        match tingle.mode {
            MODE_FALLEN => {
                tingle.timer += 1;
                if tingle.timer == CHARGE_TICKS {
                    tingle.action = ACTION_FAIL;
                    tingle.mode = MODE_IDLE;
                    tingle.timer = 0;
                }
            }
            MODE_RISE => {
                tingle.timer += 1;
                if tingle.timer >= RISE_TICKS {
                    tingle.recover();
                }
            }
            MODE_CHARGE => {
                tingle.timer += 1;
                if tingle.timer >= CHARGE_TICKS {
                    tingle.action = ACTION_FIGHT_RUN;
                    tingle.mode = MODE_IDLE;
                    tingle.timer = 0;
                }
            }
            _ => {
                if tingle.action == ACTION_STAND && tingle.distance > NOTALK_RANGE {
                    tingle.action = ACTION_HUKKI;
                }
            }
        }
    }
}