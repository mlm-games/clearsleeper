use bevy_ecs::prelude::*;

use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::combat::HitPoints;
use super::damage;
use super::enemy::Hurt;
use super::flags::{Switches, ZoneActors};
use super::ground;
use super::player::Player;
use super::walk::within;

pub const FLOWER_ACTORS: [&str; 2] = ["BFlower", "VbakH"];
pub const BOMB_ACTOR: &str = "Bomb2";

pub const LIVE: u8 = 0;
pub const DEAD: u8 = 1;

pub const KIND_SHIFT: u32 = 4;
pub const KIND_MASK: u32 = 0xF;
pub const SWITCH_SHIFT: u32 = 8;
pub const SWITCH_MASK: u32 = 0xFF;

pub const FLOWER_RADIUS: f32 = 30.0;
pub const SPENT_RADIUS: f32 = 15.0;
pub const WATER_RADIUS: f32 = 50.0;
pub const WATER_HEIGHT: f32 = 10.0;
pub const FLOWER_REACH: f32 = 50.0;
pub const FLOWER_NUDGE: f32 = 2.0;
pub const FLOWER_HEIGHT: f32 = 10.0;
pub const DROP_REACH: f32 = 200.0;
pub const DROP_RADIUS: f32 = 70.0;
pub const REVIVE_FRAMES: u8 = 0x46;
pub const IDLE_MAX: u32 = 150;
pub const IDLE_BASE: u32 = 60;
pub const IDLE_SHORT: u32 = 100;
pub const IDLE_SHORT_BASE: u32 = 100;

pub const START_INSTANT: u8 = 0;
pub const START_FUSED: u8 = 1;
pub const START_CARRIED: u8 = 2;
pub const START_MASK: u32 = 0x3;
pub const STICK_SHIFT: u32 = 8;

pub const FUSE: i16 = 0x96;
pub const FUSE_CAP: i16 = 0x1E;
pub const FUSE_FRAME: i16 = 0x87;
pub const GRAVITY: f32 = -2.9;
pub const MAX_FALL: f32 = -100.0;
pub const STREAM_DAMP: f32 = 0.002;
pub const STREAM_DRAG: f32 = 0.0005;
pub const BOUNCE_DAMP: f32 = -0.6;
pub const ROLL_DAMP: f32 = 0.8;
pub const LAND_DAMP: f32 = 0.9;
pub const SETTLE_SPEED: f32 = 19.5;
pub const HOP_SPEED: f32 = 13.0;
pub const WATER_SPEED: f32 = 20.0;
pub const WALL_SPEED: f32 = 25.0;
pub const BLAST_RADIUS: f32 = 200.0;
pub const BLAST_MASS: u8 = 3;
pub const BOMB_CULL_XZ: f32 = 36.0;
pub const BOMB_CULL_Y: f32 = 66.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flower {
    Live,
    Spent,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bombflower {
    pub kind: u8,
    pub switch_no: u8,
    pub state: Flower,
    pub idle: u32,
    pub revive: u8,
    pub previous_reach: f32,
    pub carrying: bool,
}

impl Bombflower {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: ((parameters >> KIND_SHIFT) & KIND_MASK) as u8,
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
            state: Flower::Live,
            idle: IDLE_BASE,
            revive: 0,
            previous_reach: f32::INFINITY,
            carrying: false,
        }
    }

    pub fn arm(&mut self, switches: &Switches) {
        if switches.test(self.switch_no) {
            self.state = Flower::Live;
        }
    }

    pub fn radius(&self) -> f32 {
        match self.state {
            Flower::Live => FLOWER_RADIUS,
            Flower::Spent => SPENT_RADIUS,
        }
    }

    pub fn centre(&self) -> Vec3 {
        Vec3::new(0.0, FLOWER_HEIGHT * 3.0, 0.0)
    }

    pub fn revived(&mut self) {
        self.state = Flower::Live;
        self.revive = REVIVE_FRAMES;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Crawler {
    pub parameter: u8,
    pub start: u8,
    pub fuse: i16,
    pub speed: f32,
    pub speed_y: f32,
    pub heading: Vec3,
    pub settled: bool,
    pub spent: bool,
    pub exploded: bool,
}

impl Crawler {
    pub fn spawn(parameters: u32) -> Self {
        let start = (parameters & START_MASK) as u8;
        Self {
            parameter: ((parameters >> STICK_SHIFT) & 1) as u8,
            start,
            fuse: if start == START_INSTANT { 0 } else { FUSE },
            speed: 0.0,
            speed_y: 0.0,
            heading: Vec3::ZERO,
            settled: false,
            spent: false,
            exploded: false,
        }
    }

    pub fn make(start: u8, stick: bool) -> u32 {
        u32::from(start) | (u32::from(stick) << STICK_SHIFT)
    }

    pub fn carryable(&self) -> bool {
        self.parameter == 0
    }

    pub fn lit(&self) -> bool {
        self.fuse == 0
    }

    pub fn rush(&mut self) {
        self.fuse = self.fuse.min(FUSE_CAP);
    }

    pub fn detonate(&mut self) {
        self.exploded = true;
        self.spent = true;
    }
}

pub fn is_flower(name: &str) -> bool {
    FLOWER_ACTORS.contains(&name)
}

pub fn flower_control(
    wet: Option<Res<super::water::Water>>,
    mut switches: ResMut<Switches>,
    mut flowers: Query<(Entity, &Actor, &mut Bombflower)>,
    players: Query<&Player>,
    mut commands: Commands,
) {
    let soaked = wet.as_deref().is_some_and(|water| water.in_area);
    let home = players.iter().next().map(|player| player.pos);
    for (entity, actor, mut flower) in &mut flowers {
        flower.arm(&switches);
        if flower.idle > 0 {
            flower.idle -= 1;
        } else {
            flower.idle = IDLE_BASE + IDLE_MAX;
        }
        match flower.state {
            Flower::Spent => {
                flower.revive = flower.revive.saturating_sub(1);
                if soaked {
                    flower.revived();
                    switches.set(flower.switch_no);
                }
            }
            Flower::Live => {
                let touched = home.is_some_and(|player| {
                    within(player + Vec3::new(0.0, FLOWER_HEIGHT, 0.0), actor.pos, FLOWER_RADIUS)
                });
                if touched {
                    let prm = Crawler::make(START_FUSED, !flower.carrying);
                    commands.spawn((
                        Actor {
                            name: BOMB_ACTOR.to_owned(),
                            parameters: prm,
                            pos: actor.pos,
                            angle: actor.angle,
                            set_id: 0,
                            room: actor.room,
                        },
                        Crawler::spawn(prm),
                    ));
                    flower.state = Flower::Spent;
                    flower.idle = IDLE_SHORT_BASE + IDLE_SHORT;
                    let _ = entity;
                }
            }
        }
    }
}

pub fn crawler_control(
    collision: Option<Res<Collision>>,
    mut zones: ResMut<ZoneActors>,
    mut health: ResMut<super::health::Health>,
    mut crawlers: Query<(Entity, &mut Actor, &mut Crawler)>,
    players: Query<&Player>,
    mut foes: Query<(Entity, &Actor, &mut HitPoints, Option<&mut Hurt>)>,
    mut events: ResMut<super::events::SimEvents>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let owner = players.iter().next().map(|player| player.pos);
    let mut burst: Vec<(Entity, Vec3)> = Vec::new();
    for (entity, mut actor, mut crawler) in &mut crawlers {
        if crawler.spent {
            continue;
        }
        if crawler.fuse > 0 {
            crawler.fuse -= 1;
        }
        if crawler.start == START_INSTANT || crawler.fuse <= 0 {
            crawler.detonate();
            burst.push((entity, actor.pos));
            continue;
        }
        if crawler.settled {
            continue;
        }
        crawler.speed_y = (crawler.speed_y + GRAVITY).max(MAX_FALL);
        actor.pos.y += crawler.speed_y;
        actor.pos.x += crawler.heading.x * crawler.speed;
        actor.pos.z += crawler.heading.z * crawler.speed;
        crawler.speed *= ROLL_DAMP;
        let Some(floor) = ground::surface(solid, actor.pos) else {
            continue;
        };
        if actor.pos.y > floor {
            continue;
        }
        let bounce = crawler.speed_y * BOUNCE_DAMP;
        if bounce.abs() < SETTLE_SPEED {
            crawler.settled = true;
            crawler.speed = 0.0;
            crawler.speed_y = 0.0;
            actor.pos.y = floor;
            continue;
        }
        crawler.speed *= LAND_DAMP;
        crawler.speed_y = bounce.min(HOP_SPEED);
        crawler.rush();
    }
    for (entity, origin) in burst {
        events.list.push(super::events::SimEvent::Explode);
        blast(
            origin,
            &mut foes,
            owner,
            &mut zones,
            &mut health,
            &mut commands,
        );
        let _ = commands.entity(entity).despawn();
    }
}

pub fn blast(
    origin: Vec3,
    foes: &mut Query<(Entity, &Actor, &mut HitPoints, Option<&mut Hurt>)>,
    owner: Option<Vec3>,
    zones: &mut ZoneActors,
    health: &mut super::health::Health,
    commands: &mut Commands,
) {
    let mut slain: Vec<(Entity, i8, u16)> = Vec::new();
    for (entity, actor, mut hp, mut hurt) in foes.iter_mut() {
        if !within(origin, actor.pos, BLAST_RADIUS) {
            continue;
        }
        hp.0 -= damage::BOMB;
        if let Some(hurt) = hurt.as_deref_mut() {
            hurt.0 = FLINCH;
        }
        if hp.0 <= 0 {
            slain.push((entity, actor.room, actor.set_id));
        }
    }
    if owner.is_some_and(|pos| within(origin, pos, BLAST_RADIUS)) {
        health.damage(damage::BOMB as u8);
    }
    for (entity, room, set_id) in slain {
        zones.mark_done(room, set_id);
        let _ = commands.entity(entity).despawn();
    }
}

pub const FLINCH: u32 = 10;
