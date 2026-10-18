use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, StageEvents};
use super::flags::Switches;

pub const ANDSW2_ACTORS: [&str; 1] = ["AND_SW2"];

pub fn is_andsw2(name: &str) -> bool {
    ANDSW2_ACTORS.contains(&name)
}

pub const ACT_ON_ALL: u8 = 0;
pub const ACT_TIMER: u8 = 1;
pub const ACT_ORDER: u8 = 2;
pub const ACT_EVENT: u8 = 3;
pub const ACT_OFF: u8 = 4;
pub const ACT_WAIT: u8 = 5;

pub const TYPE_ONE_OFF: u8 = 0;
pub const TYPE_CONTINUOUS: u8 = 1;

pub const NO_SWITCH: u8 = 0xFF;
pub const NO_EVENT: i16 = -1;

pub const TIMER_STEPS: i16 = 15;

pub const DEMO_ACCEPT: bool = false;

pub fn event_slot(stage_events: Option<&StageEvents>, list: &EventList, event_no: u8) -> i16 {
    if event_no == NO_SWITCH {
        return NO_EVENT;
    }
    stage_events
        .and_then(|events| events.name(event_no))
        .and_then(|name| list.event_index(name))
        .and_then(|index| i16::try_from(index).ok())
        .unwrap_or(NO_EVENT)
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Andsw2 {
    pub action: u8,
    pub kind: u8,
    pub count: u8,
    pub switch_to_set: u8,
    pub first_to_check: u8,
    pub timer_src: u8,
    pub event_no: u8,
    pub event_idx: i16,
    pub timer: i16,
    pub entered: bool,
}

impl Andsw2 {
    pub fn spawn(parameters: u32, angle: [i16; 3]) -> Self {
        Self {
            action: ACT_ON_ALL,
            kind: ((parameters >> 8) & 0xFF) as u8,
            count: (parameters & 0xFF) as u8,
            switch_to_set: ((parameters >> 16) & 0xFF) as u8,
            first_to_check: ((parameters >> 24) & 0xFF) as u8,
            timer_src: angle[2] as u8,
            event_no: angle[0] as u8,
            event_idx: NO_EVENT,
            timer: 0,
            entered: false,
        }
    }

    pub fn top_switch(&self) -> u8 {
        if self.first_to_check != NO_SWITCH {
            return self.first_to_check;
        }
        if self.switch_to_set != NO_SWITCH {
            return self.switch_to_set.wrapping_add(1);
        }
        NO_SWITCH
    }

    pub fn slot(&self) -> usize {
        usize::from(self.event_idx.max(0) as u16)
    }

    pub fn all_switches(&self, switches: &Switches) -> bool {
        let top = self.top_switch();
        if self.count == NO_SWITCH || self.count == 0 || top == NO_SWITCH {
            return false;
        }
        for i in 0..self.count {
            if !switches.test(top.wrapping_add(i)) {
                return false;
            }
        }
        true
    }

    pub fn create_init(
        &mut self,
        switches: &Switches,
        stage_events: Option<&StageEvents>,
        list: &EventList,
    ) {
        let sw = self.switch_to_set;
        self.action = match self.kind {
            TYPE_ONE_OFF => {
                if sw == NO_SWITCH || switches.test(sw) {
                    ACT_WAIT
                } else {
                    ACT_ON_ALL
                }
            }
            TYPE_CONTINUOUS => {
                if sw == NO_SWITCH {
                    ACT_WAIT
                } else if switches.test(sw) {
                    ACT_OFF
                } else {
                    ACT_ON_ALL
                }
            }
            _ => ACT_WAIT,
        };
        if self.kind == TYPE_ONE_OFF || self.kind == TYPE_CONTINUOUS {
            self.event_idx = event_slot(stage_events, list, self.event_no);
        }
    }

    pub fn action_on_all(&mut self, switches: &mut Switches) {
        if !self.all_switches(switches) {
            return;
        }
        if self.timer_src != NO_SWITCH {
            self.timer = i16::from(self.timer_src) * TIMER_STEPS;
            self.action = ACT_TIMER;
        } else if self.event_idx != NO_EVENT {
            self.action = ACT_ORDER;
        } else {
            switches.set(self.switch_to_set);
            self.action = if self.kind == TYPE_CONTINUOUS {
                ACT_OFF
            } else {
                ACT_WAIT
            };
        }
    }

    pub fn action_timer(&mut self, switches: &mut Switches) {
        if self.kind == TYPE_CONTINUOUS && !self.all_switches(switches) {
            self.action = ACT_ON_ALL;
        } else if self.timer > 0 {
            self.timer -= 1;
        } else if self.event_idx != NO_EVENT {
            self.action = ACT_ORDER;
        } else {
            switches.set(self.switch_to_set);
            if self.kind == TYPE_CONTINUOUS {
                self.action = ACT_WAIT;
            }
        }
    }

    pub fn action_order(
        &mut self,
        entity: Entity,
        switches: &mut Switches,
        list: &EventList,
        active: &mut ActiveEvent,
    ) {
        if DEMO_ACCEPT {
            self.action = ACT_EVENT;
            switches.set(self.switch_to_set);
        } else if self.kind == TYPE_CONTINUOUS && !self.all_switches(switches) {
            self.action = ACT_ON_ALL;
        } else {
            active.order(list, self.slot(), entity);
        }
    }

    pub fn action_event(&mut self, active: &mut ActiveEvent) {
        if active.end_check(self.slot()) {
            self.action = if self.kind == TYPE_CONTINUOUS {
                ACT_OFF
            } else {
                ACT_WAIT
            };
            active.end(self.slot());
        }
    }

    pub fn action_off(&mut self, switches: &mut Switches) {
        if !self.all_switches(switches) {
            self.action = ACT_ON_ALL;
            switches.clear(self.switch_to_set);
        }
    }

    pub fn execute(
        &mut self,
        entity: Entity,
        switches: &mut Switches,
        list: &EventList,
        active: &mut ActiveEvent,
    ) {
        match self.action {
            ACT_ON_ALL => self.action_on_all(switches),
            ACT_TIMER => self.action_timer(switches),
            ACT_ORDER => self.action_order(entity, switches, list, active),
            ACT_EVENT => self.action_event(active),
            ACT_OFF => self.action_off(switches),
            _ => {}
        }
    }
}

pub fn control(
    list: Res<EventList>,
    stage_events: Option<Res<StageEvents>>,
    mut active: ResMut<ActiveEvent>,
    mut switches: ResMut<Switches>,
    mut tags: Query<(Entity, &mut Actor, &mut Andsw2)>,
) {
    let stage_ref = stage_events.as_deref();
    for (entity, mut actor, mut tag) in &mut tags {
        if !tag.entered {
            tag.entered = true;
            tag.create_init(&switches, stage_ref, &list);
            actor.angle[0] = 0;
            actor.angle[2] = 0;
        }
        tag.execute(entity, &mut switches, &list, &mut active);
    }
}

pub fn spawn(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let mut actor = Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: glam::Vec3::from_array(source.pos),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
        ..Default::default()
    };
    actor.angle[0] = 0;
    actor.angle[2] = 0;
    let tag = Andsw2::spawn(source.parameters, source.angle);
    world.spawn((actor, tag)).id()
}
