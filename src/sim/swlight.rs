use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::taglight::TagLight;
use super::walk;

pub const SWLIGHT_ACTORS: [&str; 2] = ["MsuSW", "MsuSWB"];

pub const MODE_NORM_MOON: u8 = 0;
pub const MODE_NORM_SUN: u8 = 1;
pub const MODE_ACTIVE_MOON: u8 = 2;
pub const MODE_ACTIVE_SUN: u8 = 3;

pub const NO_SWITCH: u8 = 0xFF;

pub const LAMP_BOX: u8 = 0;

pub const LAMP_HALF_X: f32 = 50.0;
pub const LAMP_HALF_Z: f32 = 50.0;
pub const LAMP_FLOOR: f32 = 0.0;
pub const LAMP_TOP: f32 = 100.0;
pub const LAMP_UNSCALE: f32 = 0.5;

pub const EYE_LIFT: f32 = 40.0;

pub const HOLD_FRAMES: i16 = 20;

pub const POWER_ON: f32 = 1.0;
pub const POWER_OFF: f32 = 0.0;
pub const POWER_STEP: f32 = 0.033333335;
pub const POWER_SOUND: f32 = 0.5;

pub const SE_SW_ON: u32 = 0x6962;
pub const SE_SUNSHINE: u32 = 0x7029;

const SWITCH_SHIFT: u32 = 0x00;
const SWITCH2_SHIFT: u32 = 0x08;
const TYPE_SHIFT: u32 = 0x10;
const FIELD_MASK: u32 = 0xFF;
const TYPE_MASK: u32 = 0x01;

fn xrot(v: Vec3, units: i16) -> Vec3 {
    let (s, c) = walk::from_s16(units).sin_cos();
    Vec3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}

fn yrot(v: Vec3, units: i16) -> Vec3 {
    let (s, c) = walk::from_s16(units).sin_cos();
    Vec3::new(v.x * c + v.z * s, v.y, v.z * c - v.x * s)
}

fn zrot(v: Vec3, units: i16) -> Vec3 {
    let (s, c) = walk::from_s16(units).sin_cos();
    Vec3::new(v.x * c - v.y * s, v.x * s + v.y * c, v.z)
}

fn lamp_inside(pos: Vec3, angle: [i16; 3], at: Vec3) -> bool {
    let mut v = at - pos;
    v = yrot(v, angle[1].wrapping_neg());
    v = xrot(v, angle[0].wrapping_neg());
    v = zrot(v, angle[2].wrapping_neg());
    v *= LAMP_UNSCALE;
    v.x.abs() <= LAMP_HALF_X
        && v.y >= LAMP_FLOOR
        && v.y <= LAMP_TOP
        && v.z.abs() <= LAMP_HALF_Z
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Swlight {
    pub switch_no: u8,
    pub switch2_no: u8,
    pub mode: u8,
    pub hold: i16,
    pub power: f32,
    pub eye: Vec3,
    pub placed: bool,
}

impl Swlight {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = ((parameters >> SWITCH_SHIFT) & FIELD_MASK) as u8;
        let switch2_no = ((parameters >> SWITCH2_SHIFT) & FIELD_MASK) as u8;
        let kind = ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8;
        let lit = switches.test(switch_no);
        let lit2 = switch2_no != NO_SWITCH && switches.test(switch2_no);
        let (mode, power) = if kind == 0 {
            if lit {
                (MODE_NORM_SUN, POWER_ON)
            } else {
                (MODE_NORM_MOON, POWER_OFF)
            }
        } else if lit2 || lit {
            (MODE_ACTIVE_SUN, POWER_ON)
        } else {
            (MODE_ACTIVE_MOON, POWER_OFF)
        };
        Self {
            switch_no,
            switch2_no,
            mode,
            hold: HOLD_FRAMES,
            power,
            eye: Vec3::ZERO,
            placed: false,
        }
    }

    fn init_eye_pos(&mut self, actor: &Actor) {
        self.placed = true;
        let mut v = Vec3::new(0.0, 0.0, EYE_LIFT);
        v = zrot(v, actor.angle[2]);
        v = xrot(v, actor.angle[0]);
        v = yrot(v, actor.angle[1]);
        self.eye = actor.pos + v;
    }

    fn is_switch(&self, switches: &Switches) -> bool {
        switches.test(self.switch_no)
    }

    fn is_switch2(&self, switches: &Switches) -> bool {
        self.switch2_no != NO_SWITCH && switches.test(self.switch2_no)
    }

    fn on_switch(&self, switches: &mut Switches) {
        switches.set(self.switch_no);
    }

    fn off_switch(&self, switches: &mut Switches) {
        switches.clear(self.switch_no);
    }

    fn chk_light(&self, lamps: &Query<(&Actor, &TagLight)>) -> bool {
        lamps.iter().any(|(actor, spot)| {
            spot.lit && spot.kind == LAMP_BOX && lamp_inside(actor.pos, actor.angle, self.eye)
        })
    }

    fn power_up(&mut self) -> bool {
        walk::chase_f(&mut self.power, POWER_ON, POWER_STEP)
    }

    fn power_down(&mut self) -> bool {
        walk::chase_f(&mut self.power, POWER_OFF, POWER_STEP)
    }

    fn mode_norm_moon_init(&mut self) {
        self.mode = MODE_NORM_MOON;
        self.hold = HOLD_FRAMES;
    }

    fn mode_norm_sun_init(&mut self) {
        self.mode = MODE_NORM_SUN;
    }

    fn mode_active_moon_init(&mut self) {
        self.mode = MODE_ACTIVE_MOON;
        self.hold = HOLD_FRAMES;
    }

    fn mode_active_sun_init(&mut self) {
        self.mode = MODE_ACTIVE_SUN;
    }

    fn hold_step(&mut self, lit: bool) {
        if lit {
            if self.hold > 0 {
                self.hold -= 1;
            }
        } else {
            self.hold = HOLD_FRAMES;
        }
    }

    fn se_on(&self, events: &mut SimEvents) {
        events.list.push(SimEvent::Se(SE_SW_ON));
    }

    fn se_shine(&self, events: &mut SimEvents) {
        events.list.push(SimEvent::Se(SE_SUNSHINE));
    }

    fn mode_norm_moon(
        &mut self,
        switches: &mut Switches,
        lamps: &Query<(&Actor, &TagLight)>,
        events: &mut SimEvents,
    ) {
        let lit = self.chk_light(lamps);
        self.hold_step(lit);
        let done = if self.is_switch(switches) || self.hold == 0 {
            self.power_up()
        } else {
            self.power_down()
        };
        if done {
            self.se_on(events);
            self.on_switch(switches);
            self.mode_norm_sun_init();
        }
    }

    fn mode_norm_sun(&mut self, switches: &Switches, events: &mut SimEvents) {
        if self.is_switch(switches) {
            self.power_up();
            self.se_shine(events);
        } else if self.power_down() {
            self.mode_norm_moon_init();
        }
    }

    fn mode_active_moon(
        &mut self,
        switches: &mut Switches,
        lamps: &Query<(&Actor, &TagLight)>,
        events: &mut SimEvents,
    ) {
        let lit = self.chk_light(lamps);
        self.hold_step(lit);
        let done = if self.hold == 0 {
            self.power_up()
        } else {
            self.power_down()
        };
        if done {
            self.se_on(events);
            self.on_switch(switches);
            self.mode_active_sun_init();
        }
    }

    fn mode_active_sun(
        &mut self,
        switches: &mut Switches,
        lamps: &Query<(&Actor, &TagLight)>,
        events: &mut SimEvents,
    ) {
        if self.power > POWER_SOUND {
            self.se_shine(events);
        }
        if self.is_switch2(switches) || self.chk_light(lamps) {
            self.power_up();
        } else if self.power_down() {
            self.off_switch(switches);
            self.mode_active_moon_init();
        }
    }

    fn execute(
        &mut self,
        switches: &mut Switches,
        lamps: &Query<(&Actor, &TagLight)>,
        events: &mut SimEvents,
    ) {
        match self.mode {
            MODE_NORM_MOON => self.mode_norm_moon(switches, lamps, events),
            MODE_NORM_SUN => self.mode_norm_sun(switches, events),
            MODE_ACTIVE_MOON => self.mode_active_moon(switches, lamps, events),
            MODE_ACTIVE_SUN => self.mode_active_sun(switches, lamps, events),
            _ => {}
        }
    }
}

pub fn is_swlight(name: &str) -> bool {
    SWLIGHT_ACTORS.contains(&name)
}

pub fn control(
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    lamps: Query<(&Actor, &TagLight)>,
    mut moons: Query<(&Actor, &mut Swlight)>,
) {
    for (actor, mut moon) in &mut moons {
        let moon = &mut *moon;
        if !moon.placed {
            moon.init_eye_pos(actor);
        }
        moon.execute(&mut switches, &lamps, &mut events);
    }
}
