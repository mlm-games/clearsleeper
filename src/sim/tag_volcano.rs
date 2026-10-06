use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::flags::{EventFlags, Switches};

pub const TAG_VOLCANO_ACTORS: [&str; 1] = ["VolTag"];

pub fn is_tag_volcano(name: &str) -> bool {
    TAG_VOLCANO_ACTORS.contains(&name)
}

const TYPE_W: u32 = 0x02;
const TYPE_S: u32 = 0x06;
const SWSAVE_W: u32 = 0x08;
const SWSAVE_S: u32 = 0x08;
const TRB_W: u32 = 0x06;
const TRB_S: u32 = 0x00;
const TIMER_W: u32 = 0x08;
const TIMER_S: u32 = 0x10;
const FIELD_2A0_VALUE: i32 = 10;
const EVENT_MELT_ISLE: u32 = 0x1901;
const EVENT_MELT_FIRE: u32 = 0x1902;
const START_POINT: i32 = 2;

#[derive(Component, Debug, Clone, Copy)]
pub struct TagVolcano {
    pub kind: u32,
    pub sw_save: u8,
    pub trb: u32,
    pub timer: u32,
    pub fire_room: bool,
    pub pending: i32,
    pub armed: u8,
}

impl TagVolcano {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: (parameters >> TYPE_S) & ((1 << TYPE_W) - 1),
            sw_save: ((parameters >> SWSAVE_S) & ((1 << SWSAVE_W) - 1)) as u8,
            trb: (parameters >> TRB_S) & ((1 << TRB_W) - 1),
            timer: (parameters >> TIMER_S) & ((1 << TIMER_W) - 1),
            fire_room: false,
            pending: 0,
            armed: 0,
        }
    }

    pub fn setup(
        &mut self,
        room: i8,
        fire_room_no: i8,
        start_point: i32,
        restart_room: i8,
        events: &EventFlags,
        switches: &mut Switches,
    ) {
        if self.kind != 0 {
            return;
        }
        let bit = if room == fire_room_no {
            EVENT_MELT_FIRE
        } else {
            EVENT_MELT_ISLE
        };
        if events.test(bit) {
            switches.set(self.sw_save);
        } else if start_point == START_POINT && room == restart_room {
            self.pending = FIELD_2A0_VALUE;
            switches.set(self.sw_save);
        } else {
            switches.clear(self.sw_save);
        }
        self.fire_room = room == fire_room_no;
    }
}

pub fn control(
    mut tags: Query<(&Actor, &mut TagVolcano)>,
    events: Res<EventFlags>,
    mut switches: ResMut<Switches>,
) {
    let start_point = 0;
    let restart_room = -1;
    let fire_room_no = -1;
    for (actor, mut tag) in tags.iter_mut() {
        if tag.kind != 0 || tag.armed != 0 {
            continue;
        }
        tag.setup(
            actor.room,
            fire_room_no,
            start_point,
            restart_room,
            &events,
            &mut switches,
        );
        tag.armed = 1;
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
    };
    let tag = TagVolcano::spawn(source.parameters);
    world.spawn((actor, tag)).id()
}