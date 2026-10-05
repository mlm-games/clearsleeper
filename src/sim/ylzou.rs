use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{Charts, EventFlags, Switches};
use super::stream;
use super::walk;

pub const YLZOU_ACTORS: [&str; 1] = ["YLzou"];

pub const NO_SWITCH: u8 = 0xFF;

pub const STEP_UNSET: u8 = 0xFF;
pub const STEP_MOVE_START_WAIT: u8 = 0x00;
pub const STEP_MOVE_REGIST_WAIT: u8 = 0x01;
pub const STEP_MOVE_VIB_START_WAIT: u8 = 0x02;
pub const STEP_MOVE_VIB: u8 = 0x03;
pub const STEP_MOVE: u8 = 0x04;
pub const STEP_MOVE_END_WAIT: u8 = 0x05;
pub const STEP_OPEN_WAIT: u8 = 0x06;
pub const STAIRS_REGIST_WAIT: u8 = 0x07;
pub const STAIRS_MOVE_START_WAIT: u8 = 0x08;
pub const STAIRS_MOVE: u8 = 0x09;
pub const STAIRS_END_WAIT: u8 = 0x0A;
pub const STEP_CLOSE_WAIT_1: u8 = 0x0B;
pub const STEP_CLOSE_WAIT_2: u8 = 0x0C;
pub const STAIRS2_REGIST_WAIT: u8 = 0x0D;
pub const STAIRS2_END_WAIT: u8 = 0x0E;

pub const DEMO_NONE: u8 = 0;
pub const DEMO_MOVE_YLZOU: u8 = 1;
pub const DEMO_GO_UP_STAIRS: u8 = 2;
pub const DEMO_GO_UP_STAIRS2: u8 = 3;

pub const DEMO_MOVE_NAME: &str = "move_YLzou";
pub const DEMO_STAIRS_NAME: &str = "go_up_stairs";
pub const DEMO_STAIRS2_NAME: &str = "go_up_stairs2";

pub const NEXT_ACTION: [u8; 4] = [
    STEP_UNSET,
    STEP_OPEN_WAIT,
    STEP_CLOSE_WAIT_1,
    STEP_CLOSE_WAIT_2,
];

pub const REVERSE_YAW: i16 = i16::MIN;

pub const MOVE_STOP: f32 = 680.0;
pub const MOVE_ACCEL: f32 = 0.1;
pub const MOVE_TOP: f32 = 6.0;

pub const OPEN_Z: f32 = -680.0;

pub const VIB_NONE: u8 = 0;
pub const VIB_RISE: u8 = 1;
pub const VIB_MOVE: u8 = 2;

pub const VIB_STEP: i16 = 0x4000;
pub const VIB_RISE_STEP: f32 = 0.05;
pub const VIB_RISE_TOP: f32 = 0.8;
pub const VIB_FALL_STEP: f32 = 0.05;
pub const VIB_MOVE_FLOOR: f32 = 0.4;
pub const VIB_DECAY: f32 = 0.1;

pub const MASTER_SWORD_CUTSCENE: u32 = 0x2D04;
pub const SWORD_SWINGING_CUTSCENE: u32 = 0x3A04;
pub const COURTYARD_CUTSCENE: u32 = 0x3804;
pub const MOVED_HYRULE_STATUE: u32 = 0x3820;
pub const ZELDA_AWAKENED: u32 = 0x2D02;
pub const FLAG_2C01: u32 = 0x2C01;
pub const STAIRS2_DONE: u32 = 0x3980;

pub const TRIFORCE_ALL: u8 = 8;

pub const SE_VIB: u32 = 0x6225;
pub const SE_MOVE: u32 = 0x6226;
pub const SE_STOP: u32 = 0x6A27;

const SWSAVE_SHIFT: u32 = 0;
const SWSAVE_MASK: u32 = 0xFF;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ylzou {
    pub switch_no: u8,
    pub step: u8,
    pub demo: u8,
    pub event: Option<usize>,
    pub speed: f32,
    pub home_z: f32,
    pub phase: i16,
    pub amp: f32,
    pub bob: f32,
    pub vib: u8,
}

impl Ylzou {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            switch_no: ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8,
            step: STEP_UNSET,
            demo: DEMO_NONE,
            event: None,
            speed: 0.0,
            home_z: 0.0,
            phase: 0,
            amp: 0.0,
            bob: 0.0,
            vib: VIB_NONE,
        }
    }

    fn begin(
        &mut self,
        actor: &mut Actor,
        switches: &Switches,
        charts: &Charts,
        flags: &mut EventFlags,
    ) {
        let pieces = (0..TRIFORCE_ALL)
            .filter(|slot| charts.triforce(*slot))
            .count();
        let (step, demo, moved) = if !flags.test(MASTER_SWORD_CUTSCENE) {
            if self.switch_no != NO_SWITCH && !switches.test(self.switch_no) {
                (STEP_MOVE_START_WAIT, DEMO_MOVE_YLZOU, false)
            } else {
                (STEP_OPEN_WAIT, DEMO_NONE, true)
            }
        } else if !flags.test(SWORD_SWINGING_CUTSCENE) {
            (STEP_OPEN_WAIT, DEMO_NONE, true)
        } else if !flags.test(COURTYARD_CUTSCENE) {
            if flags.test(MOVED_HYRULE_STATUE) {
                (STAIRS_REGIST_WAIT, DEMO_GO_UP_STAIRS, false)
            } else {
                (STEP_CLOSE_WAIT_1, DEMO_NONE, false)
            }
        } else if !flags.test(ZELDA_AWAKENED) {
            (STEP_OPEN_WAIT, DEMO_NONE, true)
        } else if pieces < usize::from(TRIFORCE_ALL) {
            if flags.test(MOVED_HYRULE_STATUE) {
                (STAIRS_REGIST_WAIT, DEMO_GO_UP_STAIRS, false)
            } else {
                (STEP_CLOSE_WAIT_1, DEMO_NONE, false)
            }
        } else if !flags.test(FLAG_2C01) {
            (STEP_CLOSE_WAIT_2, DEMO_NONE, false)
        } else if !flags.test(STAIRS2_DONE) {
            (STAIRS2_REGIST_WAIT, DEMO_GO_UP_STAIRS2, false)
        } else {
            (STEP_CLOSE_WAIT_2, DEMO_NONE, false)
        };
        self.demo = demo;
        if moved {
            flags.set(MOVED_HYRULE_STATUE);
        } else {
            flags.clear(MOVED_HYRULE_STATUE);
        }
        self.enter(actor, step);
    }

    fn enter(&mut self, actor: &mut Actor, step: u8) {
        self.step = step;
        match step {
            STEP_MOVE_START_WAIT => {
                self.event = None;
                self.home_z = 0.0;
                actor.pos = Vec3::ZERO;
            }
            STEP_MOVE_VIB_START_WAIT => {}
            STEP_MOVE_VIB => {
                self.vib = VIB_RISE;
            }
            STEP_MOVE => {
                self.vib = VIB_MOVE;
                actor.angle[1] = REVERSE_YAW;
            }
            STEP_MOVE_END_WAIT | STAIRS_END_WAIT | STAIRS2_END_WAIT => {
                self.speed = 0.0;
                self.vib = VIB_NONE;
            }
            STEP_OPEN_WAIT => {
                self.home_z = OPEN_Z;
                actor.pos = Vec3::new(0.0, 0.0, OPEN_Z);
            }
            STAIRS_MOVE_START_WAIT => {
                actor.angle[1] = 0;
                self.home_z = 0.0;
                actor.pos = Vec3::new(0.0, 0.0, OPEN_Z);
                self.vib = VIB_RISE;
            }
            STAIRS_MOVE => {
                self.vib = VIB_MOVE;
            }
            STEP_CLOSE_WAIT_1 | STEP_CLOSE_WAIT_2 => {
                self.home_z = 0.0;
                actor.pos = Vec3::ZERO;
            }
            _ => {}
        }
    }

    fn drift(&mut self, actor: &mut Actor) {
        let (speed, _) = stream::calc_speed(self.speed, actor.angle[1], 0.0, 0.0, 0.0);
        actor.pos.x += speed.x;
        actor.pos.z += speed.z;
    }

    fn advance(&mut self) {
        self.speed += MOVE_ACCEL;
        if self.speed > MOVE_TOP {
            self.speed = MOVE_TOP;
        }
    }

    fn running(&self, active: &ActiveEvent) -> bool {
        self.event
            .is_some_and(|index| active.start_check(index))
    }

    fn regist(
        &mut self,
        actor: &mut Actor,
        entity: Entity,
        list: &EventList,
        active: &mut ActiveEvent,
    ) {
        let Some(index) = self.event else {
            self.event = demo_event(list, self.demo);
            return;
        };
        if active.start_check(index) {
            self.enter(actor, self.step + 1);
        } else if active.stage_of(index).is_none() {
            active.order(list, index, entity);
        }
    }

    fn close(&mut self, actor: &mut Actor, flags: &mut EventFlags, active: &mut ActiveEvent) {
        let Some(index) = self.event else {
            return;
        };
        if !active.end_check(index) {
            return;
        }
        active.end(index);
        if self.step == STAIRS2_END_WAIT {
            flags.set(STAIRS2_DONE);
        }
        if let Some(next) = NEXT_ACTION.get(usize::from(self.demo)) {
            self.enter(actor, *next);
        }
    }

    fn vibrate(&mut self) {
        self.phase = self.phase.wrapping_add(VIB_STEP);
        self.amp = match self.vib {
            VIB_RISE => (self.amp + VIB_RISE_STEP).min(VIB_RISE_TOP),
            VIB_MOVE => (self.amp - VIB_FALL_STEP).max(VIB_MOVE_FLOOR),
            _ => (self.amp - VIB_DECAY).max(0.0),
        };
        self.bob = self.amp * walk::from_s16(self.phase).sin();
    }
}

fn demo_event(list: &EventList, demo: u8) -> Option<usize> {
    let name = match demo {
        DEMO_MOVE_YLZOU => DEMO_MOVE_NAME,
        DEMO_GO_UP_STAIRS => DEMO_STAIRS_NAME,
        DEMO_GO_UP_STAIRS2 => DEMO_STAIRS2_NAME,
        _ => return None,
    };
    list.event_index(name)
}

pub fn is_ylzou(name: &str) -> bool {
    YLZOU_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    switches: Res<Switches>,
    charts: Res<Charts>,
    mut flags: ResMut<EventFlags>,
    mut active: ResMut<ActiveEvent>,
    mut events: ResMut<SimEvents>,
    mut statues: Query<(Entity, &mut Actor, &mut Ylzou)>,
) {
    for (entity, mut actor, mut statue) in &mut statues {
        let actor = &mut *actor;
        let statue = &mut *statue;
        if statue.step == STEP_UNSET {
            statue.begin(actor, &switches, &charts, &mut flags);
        }
        statue.drift(actor);
        let step = statue.step;
        match step {
            STEP_MOVE_START_WAIT => {
                if statue.switch_no != NO_SWITCH && switches.test(statue.switch_no) {
                    statue.enter(actor, STEP_MOVE_REGIST_WAIT);
                }
            }
            STEP_MOVE_REGIST_WAIT | STAIRS_REGIST_WAIT | STAIRS2_REGIST_WAIT => {
                statue.regist(actor, entity, &list, &mut active);
            }
            STEP_MOVE_VIB_START_WAIT => {
                if statue.running(&active) {
                    statue.enter(actor, STEP_MOVE_VIB);
                }
            }
            STEP_MOVE_VIB | STAIRS_MOVE_START_WAIT => {
                events.list.push(SimEvent::Se(SE_VIB));
                if statue.running(&active) {
                    statue.enter(actor, step + 1);
                }
            }
            STEP_MOVE => {
                if actor.pos.z < statue.home_z + MOVE_STOP {
                    events.list.push(SimEvent::Se(SE_STOP));
                    statue.enter(actor, STEP_MOVE_END_WAIT);
                } else {
                    statue.advance();
                    events.list.push(SimEvent::Se(SE_MOVE));
                }
            }
            STAIRS_MOVE => {
                if actor.pos.z > statue.home_z {
                    events.list.push(SimEvent::Se(SE_STOP));
                    statue.enter(actor, STAIRS_END_WAIT);
                } else {
                    statue.advance();
                    events.list.push(SimEvent::Se(SE_MOVE));
                }
            }
            STEP_MOVE_END_WAIT | STAIRS_END_WAIT | STAIRS2_END_WAIT => {
                statue.close(actor, &mut flags, &mut active);
            }
            STEP_OPEN_WAIT | STEP_CLOSE_WAIT_1 | STEP_CLOSE_WAIT_2 => {}
            _ => {}
        }
        statue.vibrate();
    }
}
