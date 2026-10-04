use bevy_ecs::prelude::*;

pub const TINGLE_ACTOR: &str = "Tn";

pub const ACTION_JYUNKAI: u8 = 0;
pub const ACTION_STAND: u8 = 1;
pub const ACTION_FIGHT_RUN: u8 = 4;
pub const ACTION_FIGHT: u8 = 5;
pub const ACTION_P_LOST: u8 = 8;
pub const ACTION_B_NIGE: u8 = 9;
pub const ACTION_DEFENCE: u8 = 10;
pub const ACTION_HUKKI: u8 = 11;
pub const ACTION_WEPON_SEARCH: u8 = 12;
pub const ACTION_AITE_MIRU: u8 = 14;
pub const ACTION_D_SIT: u8 = 17;
pub const ACTION_D_MAHI: u8 = 18;
pub const ACTION_FAIL: u8 = 20;
pub const ACTION_YOGAN_FAIL: u8 = 21;

pub const MODE_IDLE: u8 = 0;
pub const MODE_HOMING: u8 = 1;
pub const MODE_FALLEN: u8 = 10;
pub const MODE_RISE: u8 = 11;
pub const MODE_CHARGE: u8 = 20;
pub const MODE_RECOVER: u8 = 50;

pub const HOME_STEP: f32 = 0.5;
pub const HOME_RATE: f32 = 0.25;
pub const TURN_STEP: i16 = 0x800;
pub const HOME_RATE_MAX: i16 = 0x800;
pub const SHIELD_RADIUS: f32 = 80.0;
pub const WEAPON_RADIUS: f32 = 60.0;
pub const WEAPON_MASS: f32 = 3.0;
pub const WEAPON_BOUNCE: f32 = 8000.0;
pub const FALL_LIMIT: f32 = -50.0;
pub const FLEE_SPEED: f32 = -30.0;
pub const RISE_TICKS: u32 = 50;
pub const CHARGE_TICKS: u32 = 30;
pub const NOTALK_RANGE: f32 = 1000.0;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Tingle {
    pub action: u8,
    pub mode: u8,
    pub timer: u32,
    pub distance: f32,
    pub facing: i16,
    pub seeking_weapon: bool,
    pub shield: bool,
}

impl Tingle {
    pub fn spawn() -> Self {
        Self {
            action: ACTION_STAND,
            mode: MODE_IDLE,
            timer: 0,
            distance: 0.0,
            facing: 0,
            seeking_weapon: false,
            shield: false,
        }
    }

    pub fn home_rate(&self) -> f32 {
        match self.mode {
            MODE_HOMING => HOME_STEP,
            _ => HOME_RATE,
        }
    }

    pub fn strike(&mut self) {
        self.action = ACTION_FIGHT;
        self.mode = MODE_IDLE;
    }

    pub fn flinch(&mut self) {
        self.action = ACTION_DEFENCE;
        self.mode = MODE_IDLE;
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