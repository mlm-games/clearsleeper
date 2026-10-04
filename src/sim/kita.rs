use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::events::{SimEvent, SimEvents};
use super::ground;
use super::stream;
use super::walk;

pub const KITA_ACTORS: [&str; 1] = ["Kita"];

pub const PHASE_HANG: u8 = 0;
pub const PHASE_DROP: u8 = 1;

pub const WET_DRY: u8 = 0;
pub const WET_POOL: u8 = 1;
pub const WET_LANDED: u8 = 2;

pub const KIND_NONE: u8 = 0xFF;
pub const KIND_SMALL: u8 = 1;
pub const KIND_LARGE: u8 = 2;

pub const VARIANT_POOL: u8 = 1;

pub const MASK_ALL: u8 = 0x0F;

pub const SCALE_SMALL: f32 = 1.25;
pub const SCALE_LARGE: f32 = 1.5;
pub const SCALE_UNIT: f32 = 1.0;

pub const CULL_HALF: f32 = 200.0;
pub const CULL_HEIGHT: f32 = 200.0;

pub const POOL_LIFT: f32 = 70.0;

pub const GROUND_DROP: f32 = 200.0;

pub const HEIGHT_SCALE: f32 = 0.3;
pub const HEIGHT_RATE: f32 = 0.05;
pub const HEIGHT_STEP: f32 = 250.0;

pub const GROW_TOP: f32 = 1.0;
pub const GROW_RATE: f32 = 1.0;
pub const GROW_STEP: f32 = 0.001;

pub const LEVEL_SCALE: i16 = 10;
pub const LEVEL_MAX: i16 = 0x200;
pub const FALL_LEVEL_MAX: i16 = 0x300;

pub const WOBBLE_X: i16 = 750;
pub const WOBBLE_Z: i16 = 900;
pub const SWAY_RATE: f32 = 1.0;
pub const SWAY_MAX: f32 = 0.25;

pub const RISE_RATE: f32 = 0.05;
pub const RISE_MAX: f32 = 2.0;

pub const RETURN_SCALE: f32 = 0.5;
pub const RETURN_MAX: f32 = 10.0;

pub const FALL_STEP: f32 = 5.0;
pub const FALL_LIMIT: f32 = -150.0;
pub const LAND_IMPACT: f32 = -50.0;

pub const SE_FLOWER_LAND_W: u32 = 0x6953;
pub const SE_KOKIRI_LANDING: u32 = 0x6951;

pub const HIMO_CHECK: [u8; 4] = [1, 2, 4, 8];
pub const HIMO_YP: [i16; 16] = [
    0, 0x96, 0x96, 0x190, 0x96, 0x190, 0x190, 0x1F4, 0x96, 0x190, 0x190, 0x1F4, 0x190, 0x1F4, 0x1F4,
    0,
];

const KIND_SHIFT: u32 = 0x00;
const KIND_MASK: u32 = 0xFF;
const VARIANT_SHIFT: u32 = 0x08;
const VARIANT_MASK: u32 = 0xFF;

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let step = (scale * (target - *value)).clamp(-max_step, max_step);
        *value += step;
    }
}

fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let step = (scale * *value).clamp(-max_step, max_step);
    *value -= step;
}

fn wobble(units: i16) -> f32 {
    walk::from_s16(units).sin()
}

fn himo_drop(mask: u8) -> f32 {
    -(f32::from(HIMO_YP.get(usize::from(mask)).copied().unwrap_or(0))) * HEIGHT_SCALE
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Kita {
    pub counter: i16,
    pub phase: u8,
    pub kind: u8,
    pub wet: u8,
    pub himo: [u8; 4],
    pub pos_rel: Vec3,
    pub sway: Vec3,
    pub height: f32,
    pub sway_grow: f32,
    pub land_y: f32,
    pub speed_y: f32,
    pub home: Vec3,
    pub start: Vec3,
    pub placed: bool,
}

impl Kita {
    pub fn spawn(parameters: u32, home: Vec3, angle: [i16; 3]) -> Self {
        let mut kind = ((parameters >> KIND_SHIFT) & KIND_MASK) as u8;
        let variant = ((parameters >> VARIANT_SHIFT) & VARIANT_MASK) as u8;
        if kind == KIND_NONE {
            kind = 0;
        }
        let mut state = Self {
            counter: 0,
            phase: PHASE_HANG,
            kind,
            wet: WET_DRY,
            himo: [0; 4],
            pos_rel: Vec3::ZERO,
            sway: Vec3::ZERO,
            height: 0.0,
            sway_grow: 0.0,
            land_y: 0.0,
            speed_y: 0.0,
            home,
            start: home,
            placed: false,
        };
        if variant == VARIANT_POOL {
            state.phase = PHASE_DROP;
            state.wet = WET_POOL;
            state.land_y = home.y + POOL_LIFT;
        }
        let mut pos = home;
        let mut angle = angle;
        let mut events = SimEvents::default();
        state.move_proc(&mut pos, &mut angle, &mut events, None);
        state.move_proc(&mut pos, &mut angle, &mut events, None);
        state.start = pos;
        state
    }

    pub fn cull(&self) -> (Vec3, Vec3) {
        let half = self.extent() * CULL_HALF;
        (
            Vec3::new(-half, -CULL_HEIGHT, -half),
            Vec3::new(half, CULL_HEIGHT, half),
        )
    }

    fn extent(&self) -> f32 {
        match self.kind {
            KIND_SMALL => SCALE_SMALL,
            KIND_LARGE => SCALE_LARGE,
            _ => SCALE_UNIT,
        }
    }

    fn himo_mask(&mut self) -> u8 {
        let mut mask = 0_u8;
        for (index, state) in self.himo.iter_mut().enumerate() {
            if *state == 2 {
                *state = 0;
                self.sway_grow = 0.0;
            }
            if *state != 0 {
                if *state == 1 {
                    self.sway_grow = 0.0;
                    *state = 5;
                }
                mask |= HIMO_CHECK[index];
            }
        }
        mask
    }

    fn release(&mut self, pos: Vec3, solid: Option<&Collision>) {
        self.phase = PHASE_DROP;
        if let Some(cross) = ground::surface(solid, pos + Vec3::new(0.0, -GROUND_DROP, 0.0))
            && self.land_y < cross
        {
            self.land_y = cross;
        }
    }

    fn hang(&mut self, pos: &mut Vec3, angle: &mut [i16; 3], solid: Option<&Collision>) {
        let mask = self.himo_mask();
        add_calc2(
            &mut self.height,
            himo_drop(mask),
            HEIGHT_RATE,
            self.sway_grow * HEIGHT_STEP,
        );
        if mask == MASK_ALL {
            self.release(*pos, solid);
        }
        add_calc2(&mut self.sway_grow, GROW_TOP, GROW_RATE, GROW_STEP);
        stream::add_calc_angle_s2(&mut angle[0], 0, LEVEL_SCALE, LEVEL_MAX);
        stream::add_calc_angle_s2(&mut angle[2], 0, LEVEL_SCALE, LEVEL_MAX);
        self.pos_rel.x = self.sway.x * wobble(self.counter.wrapping_mul(WOBBLE_X));
        self.pos_rel.z = self.sway.z * wobble(self.counter.wrapping_mul(WOBBLE_Z));
        add_calc0(&mut self.sway.x, SWAY_RATE, SWAY_MAX);
        add_calc0(&mut self.sway.z, SWAY_RATE, SWAY_MAX);
        *pos = self.home + self.pos_rel;
        pos.y += self.height;
        add_calc0(&mut self.pos_rel.y, RISE_RATE, RISE_MAX);
    }

    fn fall(&mut self, pos: &mut Vec3, angle: &mut [i16; 3], events: &mut SimEvents) {
        if self.wet != WET_LANDED {
            add_calc2(&mut pos.x, self.home.x, RETURN_SCALE, RETURN_MAX);
            add_calc2(&mut pos.z, self.home.z, RETURN_SCALE, RETURN_MAX);
        }
        stream::add_calc_angle_s2(&mut angle[0], 0, LEVEL_SCALE, FALL_LEVEL_MAX);
        stream::add_calc_angle_s2(&mut angle[2], 0, LEVEL_SCALE, FALL_LEVEL_MAX);
        pos.y += self.speed_y;
        self.speed_y -= FALL_STEP;
        if self.speed_y < FALL_LIMIT {
            self.speed_y = FALL_LIMIT;
        }
        if pos.y <= self.land_y {
            pos.y = self.land_y;
            if self.speed_y < LAND_IMPACT {
                if self.wet != WET_DRY {
                    self.wet = WET_LANDED;
                    events.list.push(SimEvent::Se(SE_FLOWER_LAND_W));
                } else {
                    events.list.push(SimEvent::Se(SE_KOKIRI_LANDING));
                }
            }
            self.speed_y = 0.0;
        }
    }

    fn move_proc(
        &mut self,
        pos: &mut Vec3,
        angle: &mut [i16; 3],
        events: &mut SimEvents,
        solid: Option<&Collision>,
    ) {
        self.counter = self.counter.wrapping_add(1);
        match self.phase {
            PHASE_HANG => self.hang(pos, angle, solid),
            PHASE_DROP => self.fall(pos, angle, events),
            _ => {}
        }
    }
}

pub fn is_kita(name: &str) -> bool {
    KITA_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    mut events: ResMut<SimEvents>,
    mut flowers: Query<(&mut Actor, &mut Kita)>,
) {
    let solid = collision.as_deref();
    for (actor, mut flower) in &mut flowers {
        let actor = &mut *actor;
        let flower = &mut *flower;
        if !flower.placed {
            flower.placed = true;
            actor.pos = flower.start;
        }
        flower.move_proc(&mut actor.pos, &mut actor.angle, &mut events, solid);
    }
}