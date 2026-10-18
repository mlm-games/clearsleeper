use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};

use super::actor::Actor;
use super::collision::Collision;
use super::combat::HitPoints;
use super::enemy::Hurt;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::ground;
use super::health::Health;
use super::intent::MoveIntent;
use super::player::Player;
use super::stream;
use super::walk;

pub const RDEAD_ACTORS: [&str; 2] = ["Rdead1", "Rdead2"];

pub const MODE_WAIT: u8 = 0x0;
pub const MODE_DAMAGE: u8 = 0x1;
pub const MODE_DEATH: u8 = 0x3;
pub const MODE_MOVE: u8 = 0x4;
pub const MODE_CRY_WAIT: u8 = 0x6;
pub const MODE_ATTACK: u8 = 0x7;
pub const MODE_RETURN: u8 = 0x8;
pub const MODE_SW_WAIT: u8 = 0xA;
pub const MODE_KANOKE: u8 = 0xB;
pub const MODE_NULL: u8 = 0xC;

pub const AREA_SHIFT: u32 = 0x01;
pub const AREA_BITS: u32 = 0x07;
pub const AREA_MAX: u32 = 0x7F;
pub const CHECK_SHIFT: u32 = 0x08;
pub const CHECK_BITS: u32 = 0x08;
pub const SW_NO_SHIFT: u32 = 0x18;
pub const SW_NO_BITS: u32 = 0x08;

pub const AREA_RADIUS_BASE: f32 = 650.0;
pub const SIGHT_RADIUS: f32 = 650.0;
pub const SIGHT_SPREAD: u16 = 0x2000;
pub const ATTACK_RADIUS: f32 = 125.0;
pub const ATTACK_SPREAD: u16 = 0x6000;
pub const RETURN_SPEED: f32 = 2.0;
pub const RETURN_SCALE: f32 = 0.1;
pub const RETURN_MAX: f32 = 0.1;
pub const GRAVITY: f32 = -4.5;
pub const MAX_FALL: f32 = -100.0;
pub const IDLE_SPEED: f32 = 0.0;

pub const RETURN_HEIGHT: f32 = 1000.0;
pub const WAKE_HEIGHT: f32 = 100.0;
pub const FAN_HEIGHT: f32 = 100.0;
pub const WAKE_SPEED: f32 = 10.0;
pub const HOME_RADIUS: f32 = 100.0;
pub const HOME_TURN_SCALE: i16 = 0x4;
pub const HOME_TURN_STEP: i16 = 0x200;
pub const HOME_TOLERANCE: u16 = 0x200;
pub const RETURN_TURN_SCALE: i16 = 0xA;
pub const RETURN_TURN_STEP: i16 = 0x200;
pub const FACE_TURN_SCALE: i16 = 0x4;
pub const FACE_TURN_STEP: i16 = 0x2000;

pub const ATTACK_TIMER: i32 = 0x96;
pub const ATTACK_BUDGET: i32 = 0x28;
pub const GRAB_FRAMES: i32 = 30;
pub const GRAB_POWER: u8 = 1;
pub const GRAB_RANGE: f32 = 20.0;
pub const PULL_SCALE: f32 = 0.3;
pub const PULL_NEAR_STEP: f32 = 1.0;
pub const PULL_FAR_STEP: f32 = 10.0;

pub const CRY_WAIT_HOLD: i32 = 60;
pub const CRY_WAIT_STRIKE: i32 = 45;
pub const DEATH_FRAMES: i32 = 10 * 30;
pub const KANOKE_FRAMES: i32 = 90;
pub const KANOKE_FORWARD: f32 = 150.0;

pub const START_STICK_X: i32 = 1;
pub const START_STICK_Y: i32 = 1;

pub const SE_RD_DAMAGE: u32 = 0x48D4;
pub const SE_RD_DIE: u32 = 0x48D5;

const fn prm(width: u32, shift: u32, param: u32) -> u32 {
    ((1 << width) - 1) & (param >> shift)
}

fn timer(value: &mut i32) -> i32 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn angle_distance(a: i16, b: i16) -> u16 {
    a.wrapping_sub(b).unsigned_abs()
}

fn in_circle(center: Vec3, pos: Vec3, radius: f32, half_height: f32) -> bool {
    let dx = pos.x - center.x;
    let dz = pos.z - center.z;
    (dx * dx + dz * dz).sqrt() < radius && (pos.y - center.y).abs() < half_height
}

fn in_fan(center: Vec3, pos: Vec3, yaw: i16, spread: u16, radius: f32, half_height: f32) -> bool {
    in_circle(center, pos, radius, half_height)
        && angle_distance(yaw, bearing(center, pos)) < spread
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let mut step = scale * (target - *value);
        if step > max_step {
            step = max_step;
        } else if step < -max_step {
            step = -max_step;
        }
        *value += step;
    }
}

fn add_calc_pos_xz2(pos: &mut Vec3, target: Vec3, scale: f32, max_step: f32) {
    if pos.x != target.x || pos.z != target.z {
        let mut diff = (*pos - target) * scale;
        let step = (diff.x * diff.x + diff.z * diff.z).sqrt();
        if step > 0.0 {
            if step > max_step {
                diff *= max_step / step;
            }
            pos.x -= diff.x;
            pos.z -= diff.z;
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ReDead {
    pub area_radius: f32,
    pub sw_no: u8,
    pub spawn_pos: Vec3,
    pub spawn_angle: i16,
    pub mode: u8,
    pub speed: f32,
    pub speed_y: f32,
    pub timer1: i32,
    pub timer2: i32,
    pub budget: i32,
    pub stick_x: i32,
    pub stick_y: i32,
}

impl ReDead {
    pub fn spawn(parameters: u32, spawn_pos: Vec3, spawn_angle: i16) -> Option<(Self, HitPoints)> {
        let mut area = prm(AREA_BITS, AREA_SHIFT, parameters);
        let checks_switch = prm(CHECK_BITS, CHECK_SHIFT, parameters) as u8;
        let sw_no = prm(SW_NO_BITS, SW_NO_SHIFT, parameters) as u8;
        if area == AREA_MAX {
            area = 0;
        }
        Some((
            Self {
                area_radius: AREA_RADIUS_BASE + area as f32,
                sw_no,
                spawn_pos,
                spawn_angle,
                mode: if checks_switch == 0 {
                    MODE_SW_WAIT
                } else {
                    MODE_WAIT
                },
                speed: IDLE_SPEED,
                speed_y: 0.0,
                timer1: 0,
                timer2: 0,
                budget: 0,
                stick_x: START_STICK_X,
                stick_y: START_STICK_Y,
            },
            HitPoints(super::damage::KILL_POWER),
        ))
    }

    fn mode_proc(&mut self, actor: &mut Actor, next: u8, events: &mut SimEvents) {
        self.mode = next;
        match next {
            MODE_WAIT => self.speed = IDLE_SPEED,
            MODE_DAMAGE => {
                events.list.push(SimEvent::Se(SE_RD_DAMAGE));
                self.speed = IDLE_SPEED;
            }
            MODE_DEATH => {
                events.list.push(SimEvent::Se(SE_RD_DIE));
                self.speed = IDLE_SPEED;
                self.timer1 = DEATH_FRAMES;
            }
            MODE_CRY_WAIT => {
                self.timer1 = CRY_WAIT_HOLD;
                self.timer2 = CRY_WAIT_STRIKE;
            }
            MODE_ATTACK => {
                self.timer1 = ATTACK_TIMER;
                self.budget = ATTACK_BUDGET;
                self.timer2 = GRAB_FRAMES;
                self.speed = IDLE_SPEED;
            }
            MODE_KANOKE => {
                self.timer1 = KANOKE_FRAMES;
                let yaw = walk::from_s16(actor.angle[1]);
                self.spawn_pos = actor.pos
                    + Vec3::new(KANOKE_FORWARD * yaw.sin(), 0.0, KANOKE_FORWARD * yaw.cos());
            }
            _ => {}
        }
    }

    fn in_attack(&self, actor: &Actor, player: &Player) -> bool {
        in_fan(
            actor.pos,
            player.pos,
            actor.angle[1],
            ATTACK_SPREAD,
            ATTACK_RADIUS,
            FAN_HEIGHT,
        )
    }

    fn wake(&self, actor: &Actor, player: &Player) -> bool {
        if !in_circle(self.spawn_pos, actor.pos, self.area_radius, RETURN_HEIGHT) {
            return false;
        }
        if !in_circle(self.spawn_pos, player.pos, self.area_radius, WAKE_HEIGHT) {
            return false;
        }
        player.speed > WAKE_SPEED
            || in_fan(
                actor.pos,
                player.pos,
                actor.angle[1],
                SIGHT_SPREAD,
                SIGHT_RADIUS,
                FAN_HEIGHT,
            )
    }

    fn mode_wait(&mut self, actor: &mut Actor, player: &Player) -> u8 {
        self.speed = IDLE_SPEED;
        if self.wake(actor, player) {
            return MODE_MOVE;
        }
        if in_circle(self.spawn_pos, actor.pos, HOME_RADIUS, RETURN_HEIGHT) {
            stream::add_calc_angle_s2(
                &mut actor.angle[1],
                self.spawn_angle,
                HOME_TURN_SCALE,
                HOME_TURN_STEP,
            );
            if angle_distance(actor.angle[1], self.spawn_angle) <= HOME_TOLERANCE {
                actor.angle[1] = self.spawn_angle;
            }
        } else if !in_circle(self.spawn_pos, actor.pos, self.area_radius, RETURN_HEIGHT)
            || !in_circle(self.spawn_pos, player.pos, self.area_radius, WAKE_HEIGHT)
        {
            return MODE_RETURN;
        }
        if self.in_attack(actor, player) {
            return MODE_ATTACK;
        }
        MODE_NULL
    }

    fn mode_damage(&mut self, player: &Player, stunned: bool) -> u8 {
        if stunned {
            return MODE_NULL;
        }
        if in_circle(self.spawn_pos, player.pos, self.area_radius, WAKE_HEIGHT) {
            MODE_MOVE
        } else {
            MODE_WAIT
        }
    }

    fn mode_move(&mut self, actor: &Actor, player: &Player) -> u8 {
        if !in_circle(self.spawn_pos, actor.pos, self.area_radius, RETURN_HEIGHT)
            || !in_circle(self.spawn_pos, player.pos, self.area_radius, WAKE_HEIGHT)
        {
            return MODE_RETURN;
        }
        if self.in_attack(actor, player) {
            return MODE_ATTACK;
        }
        MODE_NULL
    }

    fn mode_cry_wait(&mut self, actor: &Actor, player: &Player) -> u8 {
        if timer(&mut self.timer1) == 0 {
            return MODE_MOVE;
        }
        if timer(&mut self.timer2) == 0 && self.in_attack(actor, player) {
            return MODE_ATTACK;
        }
        MODE_NULL
    }

    fn mode_attack(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        axis: Vec2,
        health: &mut Health,
    ) -> u8 {
        stream::add_calc_angle_s2(
            &mut actor.angle[1],
            bearing(actor.pos, player.pos),
            FACE_TURN_SCALE,
            FACE_TURN_STEP,
        );
        let dx = actor.pos.x - player.pos.x;
        let dz = actor.pos.z - player.pos.z;
        let dist = (dx * dx + dz * dz).sqrt();
        if dist <= GRAB_RANGE {
            add_calc_pos_xz2(&mut actor.pos, player.pos, PULL_SCALE, PULL_NEAR_STEP);
            if timer(&mut self.timer2) == 0 {
                health.damage(GRAB_POWER);
                self.timer2 = GRAB_FRAMES;
            }
        } else {
            add_calc_pos_xz2(&mut actor.pos, player.pos, PULL_SCALE, PULL_FAR_STEP);
        }
        if health.units == 0 {
            return MODE_WAIT;
        }
        if self.struggle(axis.x, axis.y) {
            self.budget -= 1;
        }
        if timer(&mut self.timer1) == 0 || self.budget < 0 {
            return MODE_CRY_WAIT;
        }
        MODE_NULL
    }

    fn struggle(&mut self, stick_x: f32, stick_y: f32) -> bool {
        let mut moved = false;
        if self.stick_x > 0 && stick_x < 0.0 {
            self.stick_x = -1;
            moved = true;
        } else if self.stick_x < 0 && stick_x > 0.0 {
            self.stick_x = 1;
            moved = true;
        }
        if self.stick_y > 0 && stick_y < 0.0 {
            self.stick_y = -1;
            moved = true;
        } else if self.stick_y < 0 && stick_y > 0.0 {
            self.stick_y = 1;
            moved = true;
        }
        moved
    }

    fn mode_return(&mut self, actor: &mut Actor, player: &Player) -> u8 {
        if in_circle(self.spawn_pos, actor.pos, HOME_RADIUS, RETURN_HEIGHT) {
            self.speed = IDLE_SPEED;
            stream::add_calc_angle_s2(
                &mut actor.angle[1],
                self.spawn_angle,
                RETURN_TURN_SCALE,
                RETURN_TURN_STEP,
            );
            if angle_distance(actor.angle[1], self.spawn_angle) <= HOME_TOLERANCE {
                actor.angle[1] = self.spawn_angle;
                return MODE_WAIT;
            }
        } else {
            add_calc2(&mut self.speed, RETURN_SPEED, RETURN_SCALE, RETURN_MAX);
        }
        if self.wake(actor, player) {
            return MODE_MOVE;
        }
        MODE_NULL
    }

    fn mode_sw_wait(&mut self, switches: &Switches) -> u8 {
        if switches.test(self.sw_no) {
            MODE_KANOKE
        } else {
            MODE_NULL
        }
    }

    fn mode_kanoke(&mut self) -> u8 {
        if timer(&mut self.timer1) == 0 {
            MODE_RETURN
        } else {
            MODE_NULL
        }
    }

    fn mode_death(&mut self) -> bool {
        timer(&mut self.timer1) == 0
    }

    fn mode_exec(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        stunned: bool,
        switches: &Switches,
        axis: Vec2,
        health: &mut Health,
        events: &mut SimEvents,
    ) {
        let next = match self.mode {
            MODE_WAIT => self.mode_wait(actor, player),
            MODE_DAMAGE => self.mode_damage(player, stunned),
            MODE_MOVE => self.mode_move(actor, player),
            MODE_CRY_WAIT => self.mode_cry_wait(actor, player),
            MODE_ATTACK => self.mode_attack(actor, player, axis, health),
            MODE_RETURN => self.mode_return(actor, player),
            MODE_SW_WAIT => self.mode_sw_wait(switches),
            MODE_KANOKE => self.mode_kanoke(),
            _ => MODE_NULL,
        };
        if next != MODE_NULL {
            self.mode_proc(actor, next, events);
        }
    }

    fn integrate(&mut self, actor: &mut Actor) {
        let (speed, _) =
            stream::calc_speed(self.speed, actor.angle[1], self.speed_y, GRAVITY, MAX_FALL);
        self.speed_y = speed.y;
        actor.pos += speed;
    }
}

pub fn is_redead(name: &str) -> bool {
    RDEAD_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    switches: Res<Switches>,
    intent: Res<MoveIntent>,
    players: Query<&Player>,
    mut health: ResMut<Health>,
    mut events: ResMut<SimEvents>,
    mut redeads: Query<(Entity, &mut Actor, &mut ReDead, Option<&mut Hurt>)>,
    mut commands: Commands,
) {
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    let solid = collision.as_deref();
    let axis = intent.axis;
    for (entity, mut actor, mut state, hurt) in &mut redeads {
        let actor = &mut *actor;
        let state = &mut *state;
        let mut stunned = false;
        if let Some(mut hurt) = hurt {
            if hurt.0 > 0 && state.mode != MODE_DEATH && state.mode != MODE_SW_WAIT {
                hurt.0 -= 1;
                stunned = true;
                if state.mode != MODE_DAMAGE {
                    state.mode_proc(actor, MODE_DAMAGE, &mut events);
                }
            }
        }
        if state.mode == MODE_DEATH {
            if state.mode_death() {
                let _ = commands.entity(entity).despawn();
            }
            continue;
        }
        state.integrate(actor);
        if let Some(floor) = ground::surface(solid, actor.pos)
            && actor.pos.y < floor
        {
            actor.pos.y = floor;
        }
        state.mode_exec(
            actor,
            &player,
            stunned,
            &switches,
            axis,
            &mut health,
            &mut events,
        );
    }
}
