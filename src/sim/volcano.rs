use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::arrow::{Arrow, CAPSULE_RADIUS, CAPSULE_STRETCH, Phase, TYPE_ICE};
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::walk;

pub const VOLCANO_ACTOR: &str = "Ykzyg";

pub const CYL_RADIUS_TG: f32 = 300.0;
pub const CYL_HEIGHT: f32 = 3000.0;

pub const LAVA_SUNK: f32 = -2000.0;
pub const LAVA_CREST: f32 = 2000.0;

pub const ERUPT_CREST: f32 = 2300.0;
pub const ERUPT_STEP: f32 = 100.0;

pub const COVERAGE_STEP: f32 = 0.016666667;
pub const ALPHA_SCALE: f32 = 255.5;

pub const FREEZE_CHILL: i32 = 10;
pub const FREEZE_BREAK: i32 = 0x96;

pub const SE_ERUPTION: u32 = 0x61CD;
pub const SE_MAGMA: u32 = 0x1073;
pub const SE_CHILL: u32 = 0x69CE;
pub const SE_ICE_BRK: u32 = 0x69CF;
pub const SE_RIDDLE: u32 = 0x0806;

pub const FREEZE_EVENT: &str = "FREEZE_VOLCANO";
pub const FIRE_EVENT: &str = "FIRE_VOLCANO";

pub const MODE_FIRE_MAIN: u8 = 0;
pub const MODE_FREEZE_DEMO_WAIT: u8 = 1;
pub const MODE_FREEZE_DEMO_MAIN: u8 = 2;
pub const MODE_FREEZE_MAIN: u8 = 3;
pub const MODE_FIRE_DEMO_WAIT: u8 = 4;
pub const MODE_FIRE_DEMO_MAIN: u8 = 5;

const SWITCH_SHIFT: u32 = 8;
const SWITCH_MASK: u32 = 0xFF;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Volcano {
    pub switch_no: u8,
    pub mode: u8,
    pub lava: Vec3,
    pub coverage: f32,
    pub frames: i32,
}

impl Volcano {
    pub fn spawn(parameters: u32, home: Vec3, switches: &Switches) -> Self {
        let switch_no = ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8;
        let (mode, level, coverage) = if switches.test(switch_no) {
            (MODE_FREEZE_MAIN, LAVA_SUNK, 0.0)
        } else {
            (MODE_FIRE_MAIN, LAVA_CREST, 1.0)
        };
        Self {
            switch_no,
            mode,
            lava: Vec3::new(home.x, home.y + level, home.z),
            coverage,
            frames: 0,
        }
    }

    pub fn visible(&self) -> bool {
        self.coverage * ALPHA_SCALE >= 1.0
    }

    pub fn hull(&self) -> (f32, f32) {
        (CYL_HEIGHT, CYL_RADIUS_TG)
    }

    fn clear_timer(&mut self) {
        self.frames = 0;
    }

    fn cover(&mut self, target: f32) {
        walk::chase_f(&mut self.coverage, target, COVERAGE_STEP);
    }

    fn sink(&mut self, home: Vec3) {
        self.lava.y = home.y + LAVA_SUNK;
    }

    fn erupt(&mut self, home: Vec3) {
        if self.lava.y < home.y + ERUPT_CREST {
            self.lava.y += ERUPT_STEP;
        }
    }

    fn struck(&self, arrow: &Arrow) -> bool {
        if arrow.kind != TYPE_ICE || arrow.phase != Phase::Flying {
            return false;
        }
        let tip = arrow.pos + arrow.speed * CAPSULE_STRETCH;
        let floor = self.lava.y;
        let crest = floor + CYL_HEIGHT;
        if arrow.pos.y.max(tip.y) + CAPSULE_RADIUS <= floor
            || arrow.pos.y.min(tip.y) - CAPSULE_RADIUS >= crest
        {
            return false;
        }
        walk::swept(
            Vec3::new(arrow.pos.x, 0.0, arrow.pos.z),
            Vec3::new(tip.x, 0.0, tip.z),
            Vec3::new(self.lava.x, 0.0, self.lava.z),
            CYL_RADIUS_TG + CAPSULE_RADIUS,
        )
    }
}

fn order(list: &EventList, index: Option<usize>, entity: Entity, active: &mut ActiveEvent) {
    if let Some(index) = index && active.stage_of(index).is_none() {
        active.order(list, index, entity);
    }
}

fn started(active: &ActiveEvent, index: Option<usize>) -> bool {
    index.is_some_and(|index| active.start_check(index))
}

fn ended(active: &ActiveEvent, index: Option<usize>) -> bool {
    index.is_some_and(|index| active.end_check(index))
}

fn reset(active: &mut ActiveEvent, index: Option<usize>) {
    if let Some(index) = index {
        active.end(index);
    }
}

pub fn is_volcano(name: &str) -> bool {
    name == VOLCANO_ACTOR
}

pub fn control(
    list: Res<EventList>,
    mut switches: ResMut<Switches>,
    mut active: ResMut<ActiveEvent>,
    mut events: ResMut<SimEvents>,
    mut volcanos: Query<(Entity, &Actor, &mut Volcano)>,
    arrows: Query<&Arrow>,
) {
    let freeze = list.event_index(FREEZE_EVENT);
    let fire = list.event_index(FIRE_EVENT);
    for (entity, actor, mut volcano) in &mut volcanos {
        let home = actor.pos;
        match volcano.mode {
            MODE_FIRE_MAIN => {
                events.list.push(SimEvent::Se(SE_ERUPTION));
                events.list.push(SimEvent::Se(SE_MAGMA));
                if arrows.iter().any(|arrow| volcano.struck(arrow)) {
                    volcano.mode = MODE_FREEZE_DEMO_WAIT;
                    order(&list, freeze, entity, &mut active);
                }
                volcano.cover(1.0);
            }
            MODE_FREEZE_DEMO_WAIT => {
                if started(&active, freeze) {
                    volcano.clear_timer();
                    switches.set(volcano.switch_no);
                    volcano.mode = MODE_FREEZE_DEMO_MAIN;
                } else {
                    order(&list, freeze, entity, &mut active);
                }
            }
            MODE_FREEZE_DEMO_MAIN => {
                volcano.frames += 1;
                volcano.cover(0.0);
                match volcano.frames {
                    FREEZE_CHILL => events.list.push(SimEvent::Se(SE_CHILL)),
                    FREEZE_BREAK => {
                        events.list.push(SimEvent::Se(SE_ICE_BRK));
                        events.list.push(SimEvent::Se(SE_RIDDLE));
                    }
                    _ => {}
                }
                if ended(&active, freeze) {
                    reset(&mut active, freeze);
                    volcano.mode = MODE_FREEZE_MAIN;
                }
            }
            MODE_FREEZE_MAIN => {
                if !switches.test(volcano.switch_no) {
                    order(&list, fire, entity, &mut active);
                    volcano.sink(home);
                    volcano.mode = MODE_FIRE_DEMO_WAIT;
                }
            }
            MODE_FIRE_DEMO_WAIT => {
                if started(&active, fire) {
                    volcano.clear_timer();
                    volcano.mode = MODE_FIRE_DEMO_MAIN;
                } else {
                    order(&list, fire, entity, &mut active);
                }
            }
            MODE_FIRE_DEMO_MAIN => {
                events.list.push(SimEvent::Se(SE_ERUPTION));
                events.list.push(SimEvent::Se(SE_MAGMA));
                volcano.erupt(home);
                if ended(&active, fire) {
                    reset(&mut active, fire);
                    volcano.mode = MODE_FIRE_MAIN;
                }
                volcano.cover(1.0);
            }
            _ => {}
        }
    }
}