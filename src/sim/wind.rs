use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};

use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::player::Player;
use super::schedule::Schedule;
use super::walk;

pub const WIND_TAG_ACTORS: [&str; 2] = ["Yaflw00", "Ybgaf00"];

pub const NO_SWITCH: u8 = 0xFF;

pub const TYPE_BIG: u8 = 1;

pub const DATA_XZ: [f32; 2] = [150.0, 1000.0];
pub const DATA_Y: [f32; 2] = [1000.0, 7000.0];

pub const DRAW_MIN: f32 = 50.0;

pub const SE_WIND_TAG: u32 = 0x701D;
pub const SE_WIND_TAG_L: u32 = 0x701E;

pub const STRENGTH_TOP: f32 = 1.0;
pub const STRENGTH_SCALE: f32 = 0.5;
pub const STRENGTH_MAX_STEP: f32 = 0.5;
pub const STRENGTH_MIN_STEP: f32 = 0.01;

pub const RISE_SCALE: f32 = 0.1;
pub const CHASE_DIVISOR: f32 = 20.0;

pub const GUST_SPEED: f32 = 30.0;
pub const GUST_FLOOR: f32 = 1.0;
pub const DRAG_SCALE: f32 = 0.5;
pub const DRAG_MIN_STEP: f32 = 0.5;
pub const DRAG_STEP_WIND: f32 = 1.0;
pub const DRAG_STEP_CLEAR: f32 = 3.0;

const SPLASH_MASK: i16 = 0x0F;
const SPLASH_FLAG: i16 = 1;
const DRAW_SHIFT: i32 = 4;
const DRAW_FLAG: i32 = 1;

const MASK_SCH: u32 = 0xFF;
const MASK_SW_TYPE: u32 = 0x01;
const MASK_SWITCH: u32 = 0xFF;
const SHIFT_SW_TYPE: u32 = 23;
const SHIFT_SWITCH: u32 = 24;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct WindTag {
    pub kind: u8,
    pub mask: u8,
    pub switch_type: bool,
    pub switch_no: u8,
    pub splash: bool,
    pub hidden: bool,
    pub cur: f32,
    pub span: f32,
    pub strength: f32,
}

impl WindTag {
    pub fn spawn(parameters: u32, angle_z: i16) -> Self {
        let kind = ((parameters >> 21) & 0x03) as u8;
        let index = usize::from(kind) % 2;
        Self {
            kind,
            mask: (parameters & MASK_SCH) as u8,
            switch_type: (parameters >> SHIFT_SW_TYPE) & MASK_SW_TYPE != 0,
            switch_no: ((parameters >> SHIFT_SWITCH) & MASK_SWITCH) as u8,
            splash: (angle_z & SPLASH_MASK) == SPLASH_FLAG,
            hidden: (i32::from(angle_z) >> DRAW_SHIFT) & DRAW_FLAG == DRAW_FLAG,
            cur: 0.0,
            span: DATA_Y[index],
            strength: 0.0,
        }
    }

    pub fn radius(&self) -> f32 {
        DATA_XZ[usize::from(self.kind) % 2]
    }

    pub fn primed(&self, set: bool) -> bool {
        if self.switch_no == NO_SWITCH {
            return true;
        }
        set == self.switch_type
    }

    pub fn scheduled(&self, schedule: &Schedule) -> bool {
        (self.mask & schedule.bit) != 0 || self.mask == 0
    }

    pub fn ratio(&self) -> f32 {
        if self.span == 0.0 {
            return 0.0;
        }
        self.cur / self.span
    }

    pub fn sound(&self) -> u32 {
        if self.kind == TYPE_BIG {
            SE_WIND_TAG_L
        } else {
            SE_WIND_TAG
        }
    }

    pub fn column(&self, base: Vec3) -> (Vec3, Vec3) {
        (base, base + Vec3::new(0.0, self.cur, 0.0))
    }

    pub fn inside(&self, base: Vec3, at: Vec3) -> bool {
        let (start, end) = self.column(base);
        let radius = self.radius();
        let height = end.y - start.y;
        if height <= 0.0 || at.y < start.y || at.y > end.y {
            return false;
        }
        let dx = at.x - start.x;
        let dz = at.z - start.z;
        dx * dx + dz * dz <= radius * radius
    }

    pub fn blowing(&mut self) {
        walk::add_calc(
            &mut self.strength,
            STRENGTH_TOP,
            STRENGTH_SCALE,
            STRENGTH_MAX_STEP,
            STRENGTH_MIN_STEP,
        );
        let step = RISE_SCALE * self.span;
        walk::add_calc(&mut self.cur, self.span, RISE_SCALE, step, step * 0.5);
    }

    pub fn calming(&mut self) {
        walk::add_calc(
            &mut self.strength,
            0.0,
            STRENGTH_SCALE,
            STRENGTH_MAX_STEP,
            STRENGTH_MIN_STEP,
        );
        if self.switch_type {
            walk::chase_f(&mut self.cur, 0.0, self.span / CHASE_DIVISOR);
        } else {
            let step = RISE_SCALE * self.span;
            walk::add_calc(&mut self.cur, 0.0, RISE_SCALE, step, step * 0.5);
        }
    }

    pub fn step(&mut self, blowing: bool) {
        if blowing {
            self.blowing();
        } else {
            self.calming();
        }
    }

    pub fn visible(&self) -> bool {
        !self.hidden && self.cur >= DRAW_MIN
    }

}

pub fn is_wind_tag(name: &str) -> bool {
    WIND_TAG_ACTORS.contains(&name)
}

fn gust(tag: &WindTag, base: Vec3, player: &mut Player) {
    let (_, end) = tag.column(base);
    let mut target = player.pos - Vec3::new(end.x, 0.0, end.z);
    let mut speed = Vec2::new(target.x, target.z).length();
    if speed < GUST_FLOOR {
        target = player.pos - Vec3::new(base.x, 0.0, base.z);
        speed = Vec2::new(target.x, target.z).length();
    }
    if speed > GUST_SPEED {
        target *= GUST_SPEED / speed;
    }
    walk::add_calc(
        &mut player.drift.x,
        target.x,
        DRAG_SCALE,
        DRAG_STEP_WIND,
        DRAG_MIN_STEP,
    );
    walk::add_calc(
        &mut player.drift.z,
        target.z,
        DRAG_SCALE,
        DRAG_STEP_WIND,
        DRAG_MIN_STEP,
    );
    player.pos.x += player.drift.x;
    player.pos.z += player.drift.z;
}

fn settle(player: &mut Player) {
    walk::add_calc(
        &mut player.drift.x,
        0.0,
        DRAG_SCALE,
        DRAG_STEP_CLEAR,
        DRAG_MIN_STEP,
    );
    walk::add_calc(
        &mut player.drift.z,
        0.0,
        DRAG_SCALE,
        DRAG_STEP_CLEAR,
        DRAG_MIN_STEP,
    );
    player.pos.x += player.drift.x;
    player.pos.z += player.drift.z;
}

pub fn control(
    schedule: Res<Schedule>,
    switches: Res<Switches>,
    mut tags: Query<(&super::actor::Actor, &mut WindTag)>,
    mut players: Query<&mut Player>,
    mut wind: ResMut<super::windmill::Wind>,
    mut events: ResMut<SimEvents>,
) {
    for mut player in &mut players {
        let player = &mut *player;
        let mut blown = false;
        for (actor, tag) in &tags {
            if tag.inside(actor.pos, player.pos) {
                gust(tag, actor.pos, player);
                blown = true;
            }
        }
        if !blown {
            settle(player);
        }
    }
    let mut strongest = 0.0f32;
    let mut audible = false;
    for (_, mut tag) in &mut tags {
        let set = tag.switch_no != NO_SWITCH && switches.test(tag.switch_no);
        tag.step(tag.primed(set) && tag.scheduled(&schedule));
        if tag.visible() {
            audible = true;
            strongest = strongest.max(tag.ratio());
        }
    }
    if audible {
        events.list.push(SimEvent::Se(SE_WIND_TAG));
    }
    wind.strength = strongest;
}
