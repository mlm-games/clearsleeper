//! `daObjItnak::Act_c` (`tww/src/d/actor/d_a_obj_Itnak.cpp`), the unused Darknut statue.
//! `_create` (`:87-100`) reads the visibility mode from parameter bits 0-7 and the switch from
//! bits 8-15 (`d_a_obj_Itnak.h:17-18`), sets `mDrawFlag = mVisibilityMode` (`:93`) and the cull
//! box is `:95`. `manage_draw_flag` (`:150-160`) hides mode 1 on the switch, shows mode 0 on the
//! switch, and forces every other mode on.
use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::flags::Switches;

pub const ITNAK_ACTOR: &str = "Itnak";

const ARG0_MASK: u32 = 0xFF;
const SWBIT_SHIFT: u32 = 8;
const SWBIT_MASK: u32 = 0xFF;
const NO_SWITCH: u8 = 0xFF;
const CULL_MIN: [f32; 3] = [-120.0, 0.0, -100.0];
const CULL_MAX: [f32; 3] = [120.0, 280.0, 150.0];
const VIS_HIDE_ON_SWITCH: u8 = 1;
const VIS_SHOW_ON_SWITCH: u8 = 0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Itnak {
    pub drawn: bool,
    visibility: u8,
    switch: u8,
}

fn switch_bit(parameters: u32) -> u8 {
    ((parameters >> SWBIT_SHIFT) & SWBIT_MASK) as u8
}

impl Itnak {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let visibility = (parameters & ARG0_MASK) as u8;
        let switch = switch_bit(parameters);
        let closed = switch != NO_SWITCH && switches.test(switch);
        let drawn = match visibility {
            VIS_HIDE_ON_SWITCH => !closed,
            VIS_SHOW_ON_SWITCH => closed,
            _ => true,
        };
        Self {
            drawn,
            visibility,
            switch,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.drawn
    }

    fn manage_draw_flag(&mut self, switches: &Switches) {
        if self.switch == NO_SWITCH {
            self.drawn = true;
            return;
        }
        let closed = switches.test(self.switch);
        match self.visibility {
            VIS_HIDE_ON_SWITCH => {
                if self.drawn && closed {
                    self.drawn = false;
                }
            }
            VIS_SHOW_ON_SWITCH => {
                if !self.drawn && closed {
                    self.drawn = true;
                }
            }
            _ => self.drawn = true,
        }
    }
}

pub fn is_itnak(name: &str) -> bool {
    name == ITNAK_ACTOR
}

pub fn control(switches: Res<Switches>, mut statues: Query<&mut Itnak>) {
    for mut statue in &mut statues {
        statue.manage_draw_flag(&switches);
    }
}

pub fn spawn(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let actor = Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: glam::Vec3::from_array(source.pos),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
        ..Default::default()
    };
    let statue = Itnak::spawn(source.parameters, world.resource::<Switches>());
    world.spawn((actor, statue)).id()
}
