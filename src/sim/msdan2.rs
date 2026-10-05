use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::flags::Switches;
use super::walk;

pub const MSDAN2_ACTORS: [&str; 1] = ["Msdan2"];
pub const MSDAN_SUB2_ACTOR: &str = "MsdanS2"; // <- d_stage.cpp:956

pub const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_msdan2.h:11 (PRM_SWSAVE_W)
pub const SWSAVE_SHIFT: u32 = 0x00; // <- d_a_obj_msdan2.h:12 (PRM_SWSAVE_S)
pub const HALF_TURN: i16 = -0x8000; // <- d_a_obj_msdan2.cpp:15
pub const RISE: f32 = 400.0; // <- d_a_obj_msdan2.cpp:16
pub const RING: u32 = 16; // <- d_a_obj_msdan2.cpp:17
pub const STEP: f32 = 50.0; // <- d_a_obj_msdan2.cpp:18
pub const SLOT_SHIFT: u32 = 8; // <- d_a_obj_msdan2.cpp:20 (i << 8)
pub const NO_SET_ID: u16 = 0xFFFF; // <- f_op_actor_mng.cpp:82
pub const EVENT_NAME: &str = "Msdan2"; // <- d_a_obj_msdan2.cpp:23

pub const MODE_WAIT: u8 = 0; // <- d_a_obj_msdan2.cpp:27
pub const MODE_DEMO_WAIT: u8 = 1; // <- d_a_obj_msdan2.cpp:38
pub const MODE_DEMO_RUN: u8 = 2; // <- d_a_obj_msdan2.cpp:43
pub const MODE_DONE: u8 = 3; // <- d_a_obj_msdan2.cpp:26

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Msdan2 {
    pub switch_no: u8,
    pub mode: u8,
    pub entered: bool,
}

impl Msdan2 {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8;
        let mode = if switches.test(switch_no) {
            MODE_DONE
        } else {
            MODE_WAIT
        };
        Self {
            switch_no,
            mode,
            entered: false,
        }
    }

    fn ring(&self, actor: &Actor, commands: &mut Commands) {
        let (sin, cos) = walk::from_s16(actor.angle[1]).sin_cos();
        let mut pos = Vec3::new(actor.pos.x, actor.pos.y + RISE, actor.pos.z);
        let mut angle = actor.angle;
        angle[1] = angle[1].wrapping_add(HALF_TURN);
        for slot in 0..RING {
            pos.x += STEP * sin;
            pos.z += STEP * cos;
            commands.spawn(Actor {
                name: MSDAN_SUB2_ACTOR.to_owned(),
                parameters: (slot << SLOT_SHIFT) + u32::from(self.switch_no),
                pos,
                angle,
                set_id: NO_SET_ID,
                room: actor.room,
            });
        }
    }

    fn execute(
        &mut self,
        list: &EventList,
        index: Option<usize>,
        active: &mut ActiveEvent,
        switches: &Switches,
        entity: Entity,
    ) {
        match self.mode {
            MODE_WAIT => {
                if switches.test(self.switch_no) {
                    if let Some(slot) = index {
                        let _ = active.order(list, slot, entity);
                    }
                    self.mode = MODE_DEMO_WAIT;
                }
            }
            MODE_DEMO_WAIT => {
                if index.is_some_and(|slot| active.start_check(slot)) {
                    self.mode = MODE_DEMO_RUN;
                }
            }
            MODE_DEMO_RUN => {
                if index.is_some_and(|slot| active.end_check(slot)) {
                    if let Some(slot) = index {
                        let _ = active.end(slot);
                    }
                    self.mode = MODE_DONE;
                }
            }
            _ => {}
        }
    }
}

pub fn is_msdan2(name: &str) -> bool {
    MSDAN2_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    switches: Res<Switches>,
    mut rings: Query<(Entity, &Actor, &mut Msdan2)>,
    mut commands: Commands,
) {
    let index = list.event_index(EVENT_NAME);
    for (entity, actor, mut ring) in &mut rings {
        let ring = &mut *ring;
        if !ring.entered {
            ring.entered = true;
            ring.ring(actor, &mut commands);
        }
        ring.execute(&list, index, &mut active, &switches, entity);
    }
}