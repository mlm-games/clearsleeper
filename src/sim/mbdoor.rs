use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::combat::HitPoints;
use super::event::{ActiveEvent, EventList};
use super::flags::Switches;

pub const MBDOOR_ACTORS: [&str; 2] = ["SMBdor", "KGBdor"];

pub const NO_SWITCH: u8 = 0xFF;

pub const SWBIT_SHIFT: u32 = 0;
pub const SWBIT_MASK: u32 = 0xFF;

pub const TYPE_MASK: i16 = 0x3F;

pub const TYPE_SWITCH: u8 = 0;
pub const TYPE_CLEAR: u8 = 1;
pub const TYPE_SEALED: u8 = 2;

pub const UNLOCK_DELAY: i32 = 30;

pub const ACT_LOCK_WAIT: u8 = 1;
pub const ACT_LOCK_OFF: u8 = 2;
pub const ACT_CLOSE_WAIT: u8 = 4;

pub const STOP_OPEN_EVENT: &str = "MBDOOR_STOP_OPEN";

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Mbdoor {
    pub swbit: u8,
    pub kind: u8,
    pub action: u8,
    pub timer: i32,
    pub stop_open: Option<usize>,
}

impl Mbdoor {
    pub fn spawn(parameters: u32, angle: [i16; 3], switches: &Switches) -> Self {
        let swbit = ((parameters >> SWBIT_SHIFT) & SWBIT_MASK) as u8;
        let kind = (angle[2] & TYPE_MASK) as u8;
        let latched = swbit != NO_SWITCH && switches.test(swbit);
        let action = match kind {
            TYPE_SEALED => ACT_LOCK_WAIT,
            TYPE_SWITCH if swbit != NO_SWITCH && !latched => ACT_LOCK_WAIT,
            TYPE_CLEAR if swbit == NO_SWITCH || latched => ACT_LOCK_WAIT,
            _ => ACT_CLOSE_WAIT,
        };
        Self {
            swbit,
            kind,
            action,
            timer: UNLOCK_DELAY,
            stop_open: None,
        }
    }

    fn check_unlock(&mut self, switches: &mut Switches, cleared: bool) -> bool {
        match self.kind {
            TYPE_SWITCH => switches.test(self.swbit),
            TYPE_CLEAR => {
                if !cleared {
                    return false;
                }
                if self.timer > 0 {
                    self.timer -= 1;
                    return false;
                }
                if self.swbit != NO_SWITCH {
                    switches.set(self.swbit);
                }
                true
            }
            _ => false,
        }
    }

    fn order(&mut self, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        if self.stop_open.is_none() {
            self.stop_open = list.event_index(STOP_OPEN_EVENT);
        }
        let Some(cut) = self.stop_open else {
            return;
        };
        if active.stage_of(cut).is_none() {
            active.order(list, cut, entity);
        }
    }
}

pub fn is_mbdoor(name: &str) -> bool {
    MBDOOR_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut switches: ResMut<Switches>,
    enemies: Query<(&Actor, Option<&HitPoints>)>,
    mut doors: Query<(Entity, &Actor, &mut Mbdoor)>,
) {
    for (entity, actor, mut door) in &mut doors {
        match door.action {
            ACT_LOCK_WAIT => {
                let room = actor.room;
                let cleared = room >= 0
                    && !enemies
                        .iter()
                        .any(|(other, hp)| hp.is_some() && other.room == room);
                if door.check_unlock(&mut switches, cleared) {
                    door.action = ACT_LOCK_OFF;
                    door.order(&list, &mut active, entity);
                }
            }
            ACT_LOCK_OFF => door.order(&list, &mut active, entity),
            _ => {}
        }
    }
}