use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::ground;
use super::stream;

pub const FALLROCK_ACTOR: &str = "fallrck";

pub const HEAP: u32 = 0xB80;
pub const GRAVITY: f32 = -5.0;
pub const MAX_FALL: f32 = -70.0;
pub const FALL_BUDGET: f32 = 7000.0;
pub const TUMBLE: i16 = 0x3E8;
pub const HIT_RADIUS: f32 = 90.0;
pub const HIT_HEIGHT: f32 = 120.0;
pub const LAVA_LIFT: f32 = 120.0;
pub const PARTICLE_SCALE: f32 = 5.0;
pub const HEAVY: f32 = 1.3;

pub const AT_ROCK: u32 = 1 << 3;
pub const SE_GROUND: u32 = 0x692C;
pub const SE_WATER_SMALL: u32 = 0x6919;
pub const SE_WATER_LARGE: u32 = 0x691A;
pub const SE_MAGMA_SMALL: u32 = 0x691C;
pub const SE_MAGMA_LARGE: u32 = 0x691D;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Impact {
    None,
    Ground,
    Water,
    Magma,
    Spent,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct FallRock {
    pub scale: f32,
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub max_fall: f32,
    pub budget: f32,
    pub tumble: i16,
    pub tilt: i16,
    pub frames: u32,
    pub gone: bool,
}

impl FallRock {
    pub fn spawn(scale: f32) -> Self {
        Self {
            scale,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: GRAVITY,
            max_fall: MAX_FALL,
            budget: 0.0,
            tumble: TUMBLE,
            tilt: 0,
            frames: 0,
            gone: false,
        }
    }

    pub fn hit_radius(&self) -> f32 {
        self.scale * HIT_RADIUS
    }

    pub fn hit_height(&self) -> f32 {
        self.scale * HIT_HEIGHT
    }

    pub fn heavy(&self) -> bool {
        self.scale >= HEAVY
    }

    pub fn sound(&self, impact: Impact) -> Option<u32> {
        match impact {
            Impact::Ground => Some(SE_GROUND),
            Impact::Water if self.heavy() => Some(SE_WATER_LARGE),
            Impact::Water => Some(SE_WATER_SMALL),
            Impact::Magma if self.heavy() => Some(SE_MAGMA_LARGE),
            Impact::Magma => Some(SE_MAGMA_SMALL),
            Impact::None | Impact::Spent => None,
        }
    }

    pub fn integrate(&mut self, heading: i16) -> Vec3 {
        let (speed, speed_f) = stream::calc_speed(
            self.speed_f,
            heading,
            self.speed.y,
            self.gravity,
            self.max_fall,
        );
        self.speed = speed;
        self.speed_f = speed_f;
        speed
    }

    pub fn accumulate(&mut self) -> bool {
        self.budget += self.speed.y.abs();
        self.budget > FALL_BUDGET
    }

    pub fn lava_probe(&self, pos: Vec3) -> Vec3 {
        Vec3::new(pos.x, pos.y + self.scale * LAVA_LIFT, pos.z)
    }

    pub fn drowned(&self, floor: f32) -> bool {
        floor <= 0.0
    }

    pub fn tumble_step(&mut self) {
        self.tilt = self.tilt.wrapping_add(self.tumble);
    }
}

pub fn is_fallrock(name: &str) -> bool {
    name == FALLROCK_ACTOR
}

pub fn control(
    collision: Option<Res<Collision>>,
    water: Option<Res<super::water::Water>>,
    mut rocks: Query<(Entity, &mut Actor, &mut FallRock)>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let wet = water.as_deref().is_some_and(|water| water.in_area);
    for (entity, mut actor, mut rock) in &mut rocks {
        if rock.gone {
            continue;
        }
        let before = actor.pos;
        rock.frames += 1;
        actor.pos += rock.integrate(actor.angle[1]);
        let spent = rock.accumulate();
        let landed = ground::grounded(solid, actor.pos);
        let splash = wet && rock.drowned(actor.pos.y);
        let lava = ground::surface(solid, rock.lava_probe(before))
            .is_some_and(|floor| floor > actor.pos.y + rock.scale * LAVA_LIFT);
        rock.gone = spent || landed || splash || lava;
        rock.tumble_step();
        actor.angle[0] = rock.tilt;
        if rock.gone {
            let _ = commands.entity(entity).despawn();
        }
    }
}
