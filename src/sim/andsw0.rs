use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, StageEvents};
use super::flags::Switches;

pub const ANDSW0_ACTORS: [&str; 1] = ["AND_SW0"];

pub fn is_andsw0(name: &str) -> bool {
    ANDSW0_ACTORS.contains(&name)
}

pub const ACT_ON_ALL: i32 = 0;
pub const ACT_OFF_ALL: i32 = 1;
pub const ACT_WAIT: i32 = 10;
pub const ACT_TIMER: i32 = 20;
pub const ACT_TIMER2: i32 = 21;
pub const ACT_TIMER_SET: i32 = 30;

pub const FOREST_PARTS: u8 = 0xFF;
pub const FOREST_SUPPORTED: bool = false;

pub const NO_SWITCH: u8 = 0xFF;
pub const NO_EVENT: i16 = -1;

pub const BEHAVIOR_PLAIN: u8 = 0;
pub const BEHAVIOR_TIMER: u8 = 2;
pub const BEHAVIOR_TIMER_SET: u8 = 3;

pub const TIMER_STEPS: u32 = 15;
pub const TIMER_DELAY: u16 = 65;

pub const EVENT_RUN: u8 = 0;
pub const EVENT_END: u8 = 1;
pub const EVENT_DONE: u8 = 2;

pub const DEMO_ACCEPT: bool = false;

pub fn event_slot(
    stage_events: Option<&StageEvents>,
    list: &EventList,
    event_no: u8,
) -> i16 {
    if event_no == NO_SWITCH {
        return NO_EVENT;
    }
    stage_events
        .and_then(|events| events.name(event_no))
        .and_then(|name| list.event_index(name))
        .and_then(|index| i16::try_from(index).ok())
        .unwrap_or(NO_EVENT)
}

fn bit_of(value: u32) -> u8 {
    u8::try_from(value).unwrap_or(NO_SWITCH)
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Andsw0 {
    pub action: i32,
    pub num_to_check: u8,
    pub behavior: u8,
    pub switch_to_set: u8,
    pub first_to_check: u8,
    pub timer: u16,
    pub timer_src: u8,
    pub event_no: u8,
    pub event_idx: i16,
    pub event_state: u8,
    pub entered: bool,
}

impl Andsw0 {
    pub fn spawn(parameters: u32, angle: [i16; 3]) -> Self {
        Self {
            action: ACT_ON_ALL,
            num_to_check: (parameters & 0xFF) as u8,
            behavior: ((parameters >> 8) & 0xFF) as u8,
            switch_to_set: ((parameters >> 24) & 0xFF) as u8,
            first_to_check: ((parameters >> 16) & 0xFF) as u8,
            timer: u16::from(angle[2] as u8) * TIMER_STEPS as u16,
            timer_src: angle[2] as u8,
            event_no: angle[0] as u8,
            event_idx: NO_EVENT,
            event_state: EVENT_RUN,
            entered: false,
        }
    }

    pub fn create_init(&mut self, stage_events: Option<&StageEvents>, list: &EventList) {
        self.event_idx = event_slot(stage_events, list, self.event_no);
        if self.behavior == BEHAVIOR_TIMER {
            self.action = ACT_TIMER;
        }
        if self.first_to_check == NO_SWITCH {
            self.first_to_check = 0;
        }
        if self.num_to_check == FOREST_PARTS {
            self.behavior = BEHAVIOR_PLAIN;
            self.switch_to_set = 0;
        }
    }

    pub fn top_switch(&self) -> u32 {
        if self.first_to_check != 0 {
            u32::from(self.first_to_check)
        } else {
            u32::from(self.switch_to_set) + 1
        }
    }

    pub fn slot(&self) -> usize {
        usize::from(self.event_idx.max(0) as u16)
    }

    pub fn event_start_check(
        &mut self,
        entity: Entity,
        switches: &mut Switches,
        list: &EventList,
        active: &mut ActiveEvent,
    ) {
        match self.event_state {
            EVENT_RUN => {
                if self.event_idx != NO_EVENT && switches.test(self.switch_to_set) {
                    if DEMO_ACCEPT {
                        self.event_state = EVENT_END;
                    } else {
                        active.order(list, self.slot(), entity);
                    }
                }
            }
            EVENT_END => {
                if active.end_check(self.slot()) {
                    active.end(self.slot());
                    self.event_state = EVENT_DONE;
                }
            }
            _ => {}
        }
    }

    pub fn switch_check(&mut self, switches: &mut Switches) {
        let count = usize::from(self.num_to_check);
        let mut check = self.top_switch();
        match self.action {
            ACT_ON_ALL => {
                for i in 0..count {
                    if !switches.test(bit_of(check)) {
                        break;
                    }
                    if i + 1 == count {
                        if self.behavior != BEHAVIOR_TIMER_SET {
                            switches.set(self.switch_to_set);
                        }
                        self.action = match self.behavior {
                            BEHAVIOR_PLAIN => ACT_WAIT,
                            BEHAVIOR_TIMER_SET => {
                                self.timer = TIMER_DELAY;
                                ACT_TIMER_SET
                            }
                            _ => ACT_OFF_ALL,
                        };
                    }
                    check += 1;
                }
            }
            ACT_OFF_ALL => {
                for _ in 0..count {
                    if !switches.test(bit_of(check)) {
                        switches.clear(self.switch_to_set);
                        self.action = ACT_ON_ALL;
                        break;
                    }
                    check += 1;
                }
            }
            ACT_TIMER => {
                self.timer = u16::from(self.timer_src) * TIMER_STEPS as u16;
                if switches.test(self.switch_to_set) {
                    self.action = ACT_WAIT;
                } else {
                    for _ in 0..count {
                        if switches.test(bit_of(check)) {
                            self.action += 1;
                            break;
                        }
                        check += 1;
                    }
                }
            }
            ACT_TIMER2 => {
                self.timer = self.timer.wrapping_sub(1);
                if self.timer == 0 {
                    for _ in 0..count {
                        switches.clear(bit_of(check));
                        check += 1;
                    }
                    self.action = ACT_TIMER;
                } else {
                    check = self.top_switch();
                    for i in 0..count {
                        if !switches.test(bit_of(check)) {
                            break;
                        }
                        if i + 1 == count {
                            switches.set(self.switch_to_set);
                            self.action = ACT_WAIT;
                        }
                        check += 1;
                    }
                }
            }
            ACT_TIMER_SET => {
                self.timer = self.timer.wrapping_sub(1);
                if self.timer == 0 {
                    switches.set(self.switch_to_set);
                    self.action = ACT_WAIT;
                }
            }
            _ => {}
        }
    }

    pub fn execute(
        &mut self,
        entity: Entity,
        switches: &mut Switches,
        list: &EventList,
        active: &mut ActiveEvent,
    ) {
        self.event_start_check(entity, switches, list, active);
        if self.num_to_check == FOREST_PARTS && !FOREST_SUPPORTED {
            return;
        }
        self.switch_check(switches);
    }
}

pub fn control(
    list: Res<EventList>,
    stage_events: Option<Res<StageEvents>>,
    mut active: ResMut<ActiveEvent>,
    mut switches: ResMut<Switches>,
    mut tags: Query<(Entity, &Actor, &mut Andsw0)>,
) {
    let stage_ref = stage_events.as_deref();
    for (entity, _actor, mut tag) in &mut tags {
        if !tag.entered {
            tag.entered = true;
            tag.create_init(stage_ref, &list);
        }
        tag.execute(entity, &mut switches, &list, &mut active);
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
    let tag = Andsw0::spawn(source.parameters, source.angle);
    world.spawn((actor, tag)).id()
}