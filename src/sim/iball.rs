use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::boomerang;
use super::collision::Collision;
use super::combat::Combat;
use super::events::{SimEvent, SimEvents};
use super::flags::ZoneActors;
use super::ground;
use super::hammer;
use super::hammerhit;
use super::hookshot;
use super::player::Player;
use super::sea::Sea;
use super::stream;
use super::walk;
use super::water;

pub const IBALL_ACTORS: [&str; 1] = ["Iball"];

pub const MODE_WAIT: u8 = 0x0;
pub const MODE_WATER: u8 = 0x1;

pub const RADIUS: f32 = 30.0;
pub const HEIGHT: f32 = 80.0;
pub const GRAVITY: f32 = -7.0;
pub const MAX_FALL: f32 = -100.0;
pub const BOUNCE: f32 = 0.5;
pub const MOVE_DELAY: i32 = 32;
pub const LAND_GATE: f32 = 0.5;

const LIFT: f32 = 40.0;
const SWORD_OFFSET: f32 = 20.0;
const SWORD_REACH: f32 = 20.0;
const CUT_FIRST: u32 = 1;
const CUT_LAST: u32 = 6;

const SE_START: u32 = 0x5802;
const SE_BREAK: u32 = 0x5816;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum At {
    Idle,
    Sword,
    Hammer,
    Hook,
    Arrow,
    FireArrow,
    IceArrow,
    LightArrow,
    Boomerang,
}

impl At {
    fn reach(self) -> f32 {
        match self {
            Self::Sword => SWORD_REACH,
            Self::Hammer => hammer::AT_RADIUS,
            Self::Hook => hookshot::CAPSULE_RADIUS,
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow => {
                arrow::CAPSULE_RADIUS
            }
            Self::Boomerang => boomerang::CAPSULE_RADIUS,
            Self::Idle => 0.0,
        }
    }

    fn shaft(self) -> bool {
        matches!(
            self,
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow
        )
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Iball {
    pub mode: u8,
    pub timer: i32,
    pub speed: Vec3,
    pub speed_f: f32,
    pub prev_speed_y: f32,
    pub gravity: f32,
    pub grounded: bool,
    pub played_se: bool,
    pub placed: bool,
}

impl Iball {
    pub fn spawn() -> Self {
        Self {
            mode: MODE_WAIT,
            timer: 0,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            prev_speed_y: 0.0,
            gravity: GRAVITY,
            grounded: false,
            played_se: false,
            placed: false,
        }
    }

    fn centre(&self, pos: Vec3) -> Vec3 {
        Vec3::new(pos.x, pos.y + LIFT, pos.z)
    }

    fn place(&mut self, actor: &mut Actor) {
        self.placed = true;
        actor.pos.y -= LIFT;
    }

    fn check_geo(&mut self, actor: &mut Actor, solid: Option<&Collision>, sea: Option<&Sea>) {
        self.timer = self.timer.saturating_add(1);
        self.prev_speed_y = self.speed.y;
        if self.timer > MOVE_DELAY {
            let (speed, _) = stream::calc_speed(
                self.speed_f,
                actor.angle[1],
                self.speed.y,
                self.gravity,
                MAX_FALL,
            );
            self.speed = speed;
            let from = actor.pos;
            actor.pos += self.speed;
            if let Some(solid) = solid {
                actor.pos = solid.slide(from, actor.pos);
            }
        }
        if let Some(floor) = ground::surface(solid, actor.pos)
            && actor.pos.y < floor
        {
            actor.pos.y = floor;
        }
        let resting = ground::grounded(solid, actor.pos);
        let landing = resting && !self.grounded;
        self.grounded = resting;
        if self.mode == MODE_WATER {
            self.mode_water(actor, sea);
        } else {
            self.mode_wait(actor, sea, landing);
        }
    }

    fn mode_wait(&mut self, actor: &mut Actor, sea: Option<&Sea>, landing: bool) {
        if landing {
            self.prev_speed_y *= BOUNCE;
            if self.prev_speed_y > self.gravity - LAND_GATE {
                self.speed_f = 0.0;
            } else {
                self.speed = Vec3::new(0.0, -self.prev_speed_y, 0.0);
            }
        }
        let (wet, sea_y) = water::probe(sea, actor.pos.x, actor.pos.z);
        if wet && sea_y > actor.pos.y {
            self.mode = MODE_WATER;
            actor.pos.y = sea_y;
        }
    }

    fn mode_water(&mut self, actor: &mut Actor, sea: Option<&Sea>) {
        let (wet, sea_y) = water::probe(sea, actor.pos.x, actor.pos.z);
        if !wet || sea_y < actor.pos.y {
            self.mode = MODE_WAIT;
        }
        if wet {
            actor.pos.y = sea_y;
        }
    }

    fn swept_reach(&self, from: Vec3, to: Vec3, pos: Vec3, reach: f32) -> bool {
        walk::swept(from, to, self.centre(pos), RADIUS + reach)
    }
}

fn cyl_touches(base: Vec3, radius: f32, height: f32, at: Vec3, reach: f32) -> bool {
    let dx = at.x - base.x;
    let dz = at.z - base.z;
    let span = radius + reach;
    if span * span < dx * dx + dz * dz {
        return false;
    }
    at.y + reach >= base.y && at.y - reach <= base.y + height
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    sea: Option<Res<'w, Sea>>,
    combat: Res<'w, Combat>,
    cut: Res<'w, hammer::Cut>,
    players: Query<'w, 's, &'static Player>,
    arrows: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
    chains: Query<'w, 's, &'static hookshot::Hookshot>,
    hammers: Query<'w, 's, &'static hammer::Hammer>,
}

fn source_at(watch: &Watch<'_, '_>) -> At {
    if watch
        .chains
        .iter()
        .any(|chain| chain.phase != hookshot::Phase::Wait)
    {
        return At::Hook;
    }
    for shaft in watch.arrows.iter() {
        if shaft.phase != arrow::Phase::Flying {
            continue;
        }
        return match shaft.kind {
            arrow::TYPE_FIRE => At::FireArrow,
            arrow::TYPE_ICE => At::IceArrow,
            arrow::TYPE_LIGHT => At::LightArrow,
            _ => At::Arrow,
        };
    }
    if watch
        .bolts
        .iter()
        .any(|bolt| bolt.parameter == boomerang::MODE_THROW)
    {
        return At::Boomerang;
    }
    if watch.cut.kind != hammer::CUT_NONE && watch.hammers.iter().any(|hammer| hammer.striking()) {
        return At::Hammer;
    }
    if watch.combat.active && watch.combat.frame >= CUT_FIRST && watch.combat.frame < CUT_LAST {
        return At::Sword;
    }
    At::Idle
}

fn struck(ball: &Iball, actor: &Actor, at: At, watch: &Watch<'_, '_>, player: &Player) -> bool {
    match at {
        At::Sword => {
            let tip = Vec3::new(
                player.pos.x + SWORD_OFFSET * player.yaw.sin(),
                player.pos.y,
                player.pos.z + SWORD_OFFSET * player.yaw.cos(),
            );
            cyl_touches(actor.pos, RADIUS, HEIGHT, tip, at.reach())
        }
        At::Hammer => {
            hammerhit::reaches(actor.pos, player.pos)
                || hammerhit::reaches(actor.pos, hammer::strike_origin(player))
        }
        At::Hook => watch.chains.iter().any(|chain| {
            if chain.phase == hookshot::Phase::Wait {
                return false;
            }
            if walk::within(chain.pos, ball.centre(actor.pos), RADIUS + at.reach()) {
                return true;
            }
            chain.phase == hookshot::Phase::Shot
                && ball.swept_reach(
                    chain.pos,
                    chain.pos + chain.heading * hookshot::SHOT_STEP,
                    actor.pos,
                    at.reach(),
                )
        }),
        At::Boomerang => watch.bolts.iter().any(|bolt| {
            bolt.parameter == boomerang::MODE_THROW
                && walk::within(bolt.pos, ball.centre(actor.pos), RADIUS + at.reach())
        }),
        _ if at.shaft() => watch.arrows.iter().any(|shaft| {
            shaft.phase == arrow::Phase::Flying
                && ball.swept_reach(shaft.pos - shaft.speed, shaft.pos, actor.pos, at.reach())
        }),
        _ => false,
    }
}

pub fn is_iball(name: &str) -> bool {
    IBALL_ACTORS.contains(&name)
}

pub fn control(
    watch: Watch<'_, '_>,
    mut zones: ResMut<ZoneActors>,
    mut events: ResMut<SimEvents>,
    mut balls: Query<(Entity, &mut Actor, &mut Iball), Without<Player>>,
    mut commands: Commands,
) {
    let solid = watch.collision.as_deref();
    let sea = watch.sea.as_deref();
    let Some(player) = watch.players.iter().next().copied() else {
        return;
    };
    let at = source_at(&watch);
    let mut gone: Vec<(Entity, i8, u16)> = Vec::new();
    for (entity, mut actor, mut ball) in &mut balls {
        let actor = &mut *actor;
        let ball = &mut *ball;
        if !ball.placed {
            ball.place(actor);
        }
        if !ball.played_se {
            ball.played_se = true;
            events.list.push(SimEvent::Se(SE_START));
        }
        ball.check_geo(actor, solid, sea);
        if struck(ball, actor, at, &watch, &player) {
            events.list.push(SimEvent::Se(SE_BREAK));
            gone.push((entity, actor.room, actor.set_id));
        }
    }
    for (entity, room, set_id) in gone {
        zones.mark_done(room, set_id);
        let _ = commands.entity(entity).despawn();
    }
}
