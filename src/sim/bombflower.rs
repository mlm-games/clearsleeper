use bevy_ecs::prelude::*;

use glam::Vec3;

use super::actor::Actor;
use super::bomb::Bomb;
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


pub const SWITCH_SHIFT: u32 = 8;
pub const SWITCH_MASK: u32 = 0xFF;

pub const FLOWER_RADIUS: f32 = 30.0;
pub const SPENT_RADIUS: f32 = 15.0;
pub const FLOWER_HEIGHT: f32 = 10.0;
pub const FLOWER_TOP: f32 = 100.0;
pub const CRAWLER_TOP: f32 = 66.0;
pub const CRAWLER_HALF: f32 = 36.0;
pub const REVIVE_FRAMES: u8 = 0x46;
pub const IDLE_MAX: u32 = 150;
pub const IDLE_BASE: u32 = 60;
pub const IDLE_SHORT: u32 = 100;
pub const IDLE_SHORT_BASE: u32 = 100;

pub const START_INSTANT: u8 = 0;
pub const START_FUSED: u8 = 1;
pub const START_MASK: u32 = 0x3;
pub const STICK_SHIFT: u32 = 8;

pub const FUSE: i16 = 0x96;
pub const FUSE_CAP: i16 = 0x1E;
pub const GRAVITY: f32 = -2.9;
pub const MAX_FALL: f32 = -100.0;
pub const BOUNCE_DAMP: f32 = -0.6;
pub const ROLL_DAMP: f32 = 0.8;
pub const LAND_DAMP: f32 = 0.9;
pub const SETTLE_SPEED: f32 = 19.5;
pub const HOP_SPEED: f32 = 13.0;
pub const BLAST_RADIUS: f32 = 200.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flower {
    Live,
    Spent,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bombflower {
    pub switch_no: u8,
    pub state: Flower,
    pub armed: bool,
    pub idle: u32,
    pub revive: u8,
}

impl Bombflower {
    pub fn spawn(parameters: u32, spent: bool) -> Self {
        Self {
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
            state: if spent { Flower::Spent } else { Flower::Live },
            armed: spent,
            idle: IDLE_BASE,
            revive: 0,
        }
    }

    pub fn radius(&self) -> f32 {
        match self.state {
            Flower::Live => FLOWER_RADIUS,
            Flower::Spent => SPENT_RADIUS,
        }
    }

    pub fn revived(&mut self) {
        self.state = Flower::Live;
        self.armed = true;
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
        }
    }

    pub fn make(start: u8, stick: bool) -> u32 {
        u32::from(start) | (u32::from(stick) << STICK_SHIFT)
    }

    pub fn rush(&mut self) {
        self.fuse = self.fuse.min(FUSE_CAP);
    }

    pub fn detonate(&mut self) {
        self.spent = true;
    }
}

pub fn is_flower(name: &str) -> bool {
    FLOWER_ACTORS.contains(&name)
}

pub fn flower_control(
    wet: Option<Res<super::water::Water>>,
    mut switches: ResMut<Switches>,
    mut flowers: Query<(&Actor, &mut Bombflower)>,
    players: Query<&Player>,
    mut commands: Commands,
) {
    let soaked = wet.as_deref().is_some_and(|water| water.in_area);
    let home = players.iter().next().map(|player| player.pos);
    for (actor, mut flower) in &mut flowers {
        if !flower.armed {
            flower.state = match switches.test(flower.switch_no) {
                true => Flower::Live,
                false => Flower::Spent,
            };
            flower.armed = true;
        }
        if flower.idle > 0 {
            flower.idle -= 1;
        } else {
            flower.idle = IDLE_BASE + IDLE_MAX;
        }
        match flower.state {
            Flower::Spent => {
                flower.revive = flower.revive.saturating_sub(1);
                if soaked && flower.revive == 0 {
                    flower.revived();
                    switches.set(flower.switch_no);
                }
            }
            Flower::Live => {
                let touched = home.is_some_and(|player| {
                    within(player + Vec3::new(0.0, FLOWER_HEIGHT, 0.0), actor.pos, FLOWER_RADIUS)
                });
                if touched {
                    let prm = Crawler::make(START_FUSED, false);
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
                }
            }
        }
    }
}

pub fn crawler_control(
    collision: Option<Res<Collision>>,
    mut zones: ResMut<ZoneActors>,
    mut crawlers: Query<(Entity, &mut Actor, &mut Crawler), Without<Bombflower>>,
    players: Query<&Player>,
    mut foes: Query<
        (Entity, &Actor, &mut HitPoints, Option<&mut Hurt>),
        (Without<Bombflower>, Without<Crawler>),
    >,
    mut events: ResMut<super::events::SimEvents>,
    mut fences: Query<
        (&Actor, &mut super::barricade::Barricade),
        (Without<Crawler>, Without<Bomb>),
    >,
    mut floats: Query<
        (Entity, &Actor, &mut super::buoybarrel::BuoyBarrel),
        (Without<Crawler>, Without<Bomb>),
    >,
    mut switches: ResMut<super::flags::Switches>,
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
            &mut events,
            &mut fences,
            &mut floats,
            &mut switches,
            &mut commands,
        );
        let _ = commands.entity(entity).despawn();
    }
}

pub fn blast(
    origin: Vec3,
    foes: &mut Query<
        (Entity, &Actor, &mut HitPoints, Option<&mut Hurt>),
        (Without<Bombflower>, Without<Crawler>, Without<super::bomb::Bomb>),
    >,
    owner: Option<Vec3>,
    zones: &mut ZoneActors,
    events: &mut super::events::SimEvents,
    fences: &mut Query<(&Actor, &mut super::barricade::Barricade)>,
    floats: &mut Query<(Entity, &Actor, &mut super::buoybarrel::BuoyBarrel)>,
    switches: &mut ResMut<super::flags::Switches>,
    commands: &mut Commands,
) {
    let heat = super::barricade::AT_BOMB;
    for (actor, mut fence) in fences.iter_mut() {
        if within(origin, actor.pos, BLAST_RADIUS) {
            super::barricade::strike(&mut fence, heat, 0, switches);
        }
    }
    for (entity, actor, mut float) in floats.iter_mut() {
        if float.has_buoy() && within(origin, actor.pos, BLAST_RADIUS) {
            float.pop();
            let _ = commands.entity(entity).despawn();
        }
    }
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
        events.list.push(super::events::SimEvent::Hurt);
    }
    for (entity, room, set_id) in slain {
        zones.mark_done(room, set_id);
        let _ = commands.entity(entity).despawn();
    }
}

pub const FLINCH: u32 = 10;
