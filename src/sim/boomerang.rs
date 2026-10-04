use bevy_ecs::prelude::*;

use glam::Vec3;

use super::combat::HitPoints;
use super::damage;
use super::enemy::Hurt;
use super::player::Player;
use super::ship::Ship;
use super::walk::within;

pub const BOOMERANG_ACTOR: &str = "Boo";

pub const MODE_WAIT: u8 = 0;
pub const MODE_THROW: u8 = 1;

pub const LOCK_MAX: usize = 5;
pub const SPEED: f32 = 60.0;
pub const FLY_MAX: f32 = 2500.0;
pub const FLY_MAX_RIDING: f32 = 5000.0;
pub const RETURN_REACH: f32 = SPEED * 2.0;
pub const HIT_REACH: f32 = SPEED;
pub const CAPSULE_RADIUS: f32 = 30.0;
pub const SPIN: i16 = 0x1F00;
pub const SPIN_ANGLE: i16 = 0x2000;
pub const NOMINAL_TURN: f32 = 0.75;
pub const TURN_BASE: f32 = 20.0;
pub const TURN_SOFT: f32 = 18.0;
pub const TURN_KICK: f32 = 40.0;
pub const TURN_BIAS: f32 = 1024.0;
pub const TURN_SCALE: f32 = 256.0;
pub const TURN_CAP: f32 = 16384.0;
pub const NOMINAL_RANGE: f32 = 0.1;
pub const FLINCH: u32 = 6;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Boomerang {
    pub pos: Vec3,
    pub parameter: u8,
    pub target: Vec3,
    pub heading: f32,
    pub pitch: f32,
    pub speed: f32,
    pub lock: usize,
    pub locks: usize,
    pub returning: bool,
    pub free: bool,
    pub struck: Vec<Entity>,
    pub model_spin: i16,
    pub model_yaw: i16,
}

impl Boomerang {
    pub fn nocked() -> Self {
        Self {
            pos: Vec3::ZERO,
            parameter: MODE_WAIT,
            target: Vec3::ZERO,
            heading: 0.0,
            pitch: 0.0,
            speed: 0.0,
            lock: 0,
            locks: 0,
            returning: false,
            free: false,
            struck: Vec::new(),
            model_spin: SPIN_ANGLE,
            model_yaw: 0,
        }
    }

    pub fn toss(&mut self, pos: Vec3, heading: f32, pitch: f32, locks: usize) {
        self.pos = pos;
        self.parameter = MODE_THROW;
        self.speed = SPEED;
        self.struck.clear();
        self.returning = false;
        self.lock = 0;
        self.model_spin = SPIN_ANGLE;
        self.model_yaw = 0;
        self.aim_at(heading, pitch, locks);
    }

    pub fn reach(&self, riding: bool) -> f32 {
        if riding { FLY_MAX_RIDING } else { FLY_MAX }
    }

    pub fn aim(&mut self) {
        let delta = self.target - self.pos;
        let flat = delta.x.hypot(delta.z);
        self.pitch = -(delta.y).atan2(flat);
        self.heading = delta.x.atan2(delta.z);
        if self.locks == 0 {
            self.heading += NOMINAL_TURN * core::f32::consts::TAU;
            self.free = true;
        } else {
            self.free = false;
        }
    }

    pub fn aim_at(&mut self, heading: f32, pitch: f32, locks: usize) {
        let (sin, cos) = heading.sin_cos();
        let reach = self.reach(false);
        self.target = self.pos
            + Vec3::new(reach * sin * pitch.cos(), -reach * pitch.sin(), reach * cos * pitch.cos());
        self.locks = locks;
        self.aim();
    }

    pub fn extend(&mut self, riding: bool) {
        let reach = self.reach(riding);
        let (sin, cos) = self.heading.sin_cos();
        self.target = self.pos
            + Vec3::new(reach * sin * self.pitch.cos(), -reach * self.pitch.sin(), reach * cos * self.pitch.cos());
    }

    pub fn returning(&mut self) {
        self.returning = true;
        self.free = false;
        self.locks = 0;
        self.lock = 0;
    }

    fn steer(&mut self, bearing: f32) -> f32 {
        let mut turn = shortest(bearing, self.heading);
        if self.free || self.locks == 0 {
            let distance = self.distance();
            let mut arc = TURN_BASE - distance * 2.0 / self.speed;
            if arc < 0.0 {
                arc = 0.0;
            } else if arc > TURN_SOFT {
                arc = TURN_BASE + TURN_KICK * (arc - TURN_SOFT);
            }
            turn = turn.clamp(-(arc * TURN_SCALE + TURN_BIAS), arc * TURN_SCALE + TURN_BIAS);
        } else {
            turn = turn.clamp(-TURN_CAP, TURN_CAP);
        }
        self.heading += turn;
        turn
    }

    pub fn distance(&self) -> f32 {
        (self.target - self.pos).length()
    }

    pub fn home(&mut self, player: &Player) -> f32 {
        let delta = player.pos - self.pos;
        let distance = delta.length();
        if distance > NOMINAL_RANGE {
            let bearing = delta.x.atan2(delta.z);
            self.target = player.pos;
            self.aim_at(bearing, self.pitch, self.locks);
            self.free = false;
        }
        distance
    }

    pub fn step(&mut self) -> Vec3 {
        self.model_yaw = self.model_yaw.wrapping_sub(SPIN);
        let delta = self.target - self.pos;
        let distance = delta.length();
        let bearing = if distance > NOMINAL_RANGE {
            delta.x.atan2(delta.z)
        } else {
            self.heading
        };
        self.steer(bearing);
        let step = Vec3::new(
            self.speed * self.pitch.cos() * self.heading.sin(),
            -self.speed * self.pitch.sin(),
            self.speed * self.pitch.cos() * self.heading.cos(),
        );
        self.pos += step;
        step
    }

    pub fn catches(&self, home: Vec3) -> bool {
        self.returning && within(self.pos, home, RETURN_REACH)
    }
}

fn shortest(from: f32, to: f32) -> f32 {
    (to - from + core::f32::consts::PI).rem_euclid(core::f32::consts::TAU) - core::f32::consts::PI
}

pub fn control(
    mut zones: ResMut<super::flags::ZoneActors>,
    mut booms: Query<(Entity, &mut Boomerang)>,
    players: Query<&Player>,
    riding: Query<&Ship>,
    mut foes: Query<(Entity, &super::actor::Actor, &mut HitPoints, Option<&mut Hurt>)>,
    mut commands: Commands,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    let on_ship = riding.iter().any(|ship| ship.riding);
    let mut spent: Vec<Entity> = Vec::new();
    let mut slain: Vec<Entity> = Vec::new();
    for (entity, mut boom) in &mut booms {
        if boom.parameter == MODE_WAIT {
            continue;
        }
        if on_ship {
            boom.extend(true);
        }
        boom.step();
        if boom.returning {
            boom.home(player);
        } else {
            for (foe, actor, mut hp, mut hurt) in &mut foes {
                if boom.struck.contains(&foe) || !within(boom.pos, actor.pos, CAPSULE_RADIUS + HIT_REACH) {
                    continue;
                }
                hp.0 -= damage::BOMERANG;
                if let Some(hurt) = hurt.as_deref_mut() {
                    hurt.0 = FLINCH;
                }
                if hp.0 <= 0 {
                    slain.push(foe);
                }
                boom.struck.push(foe);
                boom.lock += 1;
                if boom.lock >= LOCK_MAX {
                    boom.returning();
                }
            }
        }
        if boom.catches(player.pos) {
            spent.push(entity);
        }
    }
    for foe in slain {
        let Ok((_, actor, _, _)) = foes.get(foe) else {
            continue;
        };
        zones.mark_done(actor.room, actor.set_id);
        let _ = commands.entity(foe).despawn();
    }
    for entity in spent {
        let _ = commands.entity(entity).despawn();
    }
}