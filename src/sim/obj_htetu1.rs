//! `daObjHtetu1_c` (`tww/src/d/actor/d_a_obj_htetu1.cpp`), the Tower of the Gods gate.
//! `_create` (`:61-80`) starts the gate lowered when its switch is already on (`:69-71`) and
//! otherwise `_execute` (`:167-220`) waits for the switch, shakes for 70 frames with a 50-unit
//! amplitude (`:176-177`, `:126-130`), then drops 5 units per frame to `spawn.y - 2300`
//! (`:213-218`). The cull box is `:78`.
use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::flags::Switches;
use super::stream;
use super::walk::add_calc;

pub const HTETU1_ACTOR: &str = "Htetu1";

const SWITCH_MASK: u32 = 0xFF;
const NO_SWITCH: u8 = 0xFF;
const DROP: f32 = 2300.0;
const FALL_STEP: f32 = 5.0;
const SHAKE_FRAMES: u16 = 70;
const SHAKE_AMPLITUDE: f32 = 50.0;
const SHAKE_ANGLE_STEP: u16 = 0x859;
const QUAKE_FRAMES: i16 = 200;
const CULL_MIN: [f32; 3] = [-950.0, -1000.0, -100.0];
const CULL_MAX: [f32; 3] = [950.0, 1300.0, 100.0];

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Htetu1 {
    switch: u8,
    bottom_y: f32,
    next_y: f32,
    shake_offset: f32,
    shake_amplitude: f32,
    shake_timer: u16,
    move_state: u8,
    quake_timer: i16,
    started: bool,
}

impl Htetu1 {
    pub fn spawn(parameters: u32, pos: glam::Vec3, switches: &Switches) -> Self {
        let switch = (parameters & SWITCH_MASK) as u8;
        let bottom_y = pos.y - DROP;
        let open = switch != NO_SWITCH && switches.test(switch);
        Self {
            switch,
            bottom_y,
            next_y: if open { bottom_y } else { pos.y },
            shake_offset: 0.0,
            shake_amplitude: 0.0,
            shake_timer: 0,
            move_state: 0,
            quake_timer: -1,
            started: open,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    fn unlock(&mut self) {
        let angle = self.shake_timer.wrapping_mul(SHAKE_ANGLE_STEP) as i16;
        let offset = (self.shake_amplitude * stream::sin_s(angle)).trunc().abs();
        self.next_y -= self.shake_offset;
        self.next_y += offset;
        self.shake_offset = offset;
        add_calc(&mut self.shake_amplitude, 0.0, 0.13, 50.0, 1.0);
    }

    fn execute(&mut self, switches: &Switches, y: &mut f32) {
        if !self.started {
            if self.switch != NO_SWITCH && switches.test(self.switch) {
                self.shake_amplitude = SHAKE_AMPLITUDE;
                self.shake_timer = SHAKE_FRAMES;
                self.shake_offset = SHAKE_AMPLITUDE;
                self.move_state = 1;
                self.started = true;
            }
        } else {
            match self.move_state {
                0 => {}
                1 => {
                    self.unlock();
                    if self.shake_timer != 0 {
                        self.shake_timer -= 1;
                    } else {
                        self.move_state = 2;
                        self.quake_timer = QUAKE_FRAMES;
                    }
                }
                2 => {
                    self.next_y -= FALL_STEP;
                    if self.next_y <= self.bottom_y {
                        self.next_y = self.bottom_y;
                        self.move_state = 0;
                    }
                }
                _ => {}
            }
            *y = self.next_y;
        }
        if self.quake_timer == 0 {
            self.quake_timer = -1;
        } else if self.quake_timer > 0 {
            self.quake_timer -= 1;
        }
    }
}

pub fn is_htetu1(name: &str) -> bool {
    name == HTETU1_ACTOR
}

pub fn control(switches: Res<Switches>, mut gates: Query<(&mut Actor, &mut Htetu1)>) {
    for (mut actor, mut gate) in &mut gates {
        let mut y = actor.pos.y;
        gate.execute(&switches, &mut y);
        actor.pos.y = y;
    }
}

pub fn spawn(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let pos = glam::Vec3::from_array(source.pos);
    let gate = Htetu1::spawn(source.parameters, pos, world.resource::<Switches>());
    let actor = Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: glam::Vec3::new(pos.x, gate.next_y, pos.z),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
        ..Default::default()
    };
    world.spawn((actor, gate)).id()
}
