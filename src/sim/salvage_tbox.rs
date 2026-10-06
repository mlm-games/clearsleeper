use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::event_manager::{EventManager, StaffActor};
use super::sea::Sea;
use super::ship::Crane;
use super::water::probe;
use crate::text::msgproc::MsgGameInfo;

pub const SALVAGE_TBOX_ACTORS: [&str; 1] = ["STBox"];

pub fn is_salvage_tbox(name: &str) -> bool {
    SALVAGE_TBOX_ACTORS.contains(&name)
}

pub const TALK_COMMAND: bool = false;

const BOX_TYPE_SHIFT: u32 = 8;
const BOX_TYPE_MASK: u32 = 0xF;
const ITEM_NO_MASK: u32 = 0xFF;
const CRANE_OFFSET: [f32; 3] = [80.0, 125.0, 125.0];
const DROP_GRAVITY: f32 = -4.0;
const RIPPLE_OFFSET_Y: f32 = 2500.0;
const RIPPLE_MARGIN: f32 = 10.0;
const RIPPLE_RATE: f32 = 12.0;
const SINK_MARGIN: f32 = 50.0;
const SHADOW_ALPHA: u8 = 0x78;
const SHADOW_DEPTH: f32 = 2000.0;
const SHADOW_SCROLL: f32 = -0.1;
const SHADOW_SCALE: f32 = 4.0;
const ROPE_MAX_LENGTH: f32 = 2500.0;
const HOP: [f32; 3] = [3.5, 3.0, 3.0];
const RISE: [f32; 3] = [-20.0, 20.0, 20.0];
const EMITTER_GLOBAL: f32 = 1.5;
const EMITTER_COUNT: usize = 3;
const DROP_WAIT: i32 = 20;

const ACTIONS: [&str; 5] = ["WAIT", "WAIT02", "WAIT_GETITEM", "WAIT_DUMMY", "DROP"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxAct {
    Wait,
    Wait02,
    WaitGetItem,
    WaitDummy,
    Drop,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct SalvageTbox {
    pub act: BoxAct,
    pub box_type: u32,
    pub item_no: u32,
    pub wait: i32,
    pub drawn: bool,
    pub splashed: bool,
    pub jingle: bool,
    pub ripple: bool,
    pub attention: glam::Vec3,
    pub shadow_alpha: u8,
    pub shadow_water_y: f32,
    pub shadow_flat_y: f32,
    pub shadow_depth: f32,
    pub shadow_pos: glam::Vec3,
    pub shadow_scroll: f32,
    pub gravity: f32,
}

impl SalvageTbox {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            act: BoxAct::Wait,
            box_type: (parameters >> BOX_TYPE_SHIFT) & BOX_TYPE_MASK,
            item_no: parameters & ITEM_NO_MASK,
            wait: 0,
            drawn: true,
            splashed: false,
            jingle: false,
            ripple: false,
            attention: glam::Vec3::ZERO,
            shadow_alpha: SHADOW_ALPHA,
            shadow_water_y: 0.0,
            shadow_flat_y: 0.0,
            shadow_depth: SHADOW_DEPTH,
            shadow_pos: glam::Vec3::ZERO,
            shadow_scroll: SHADOW_SCROLL,
            gravity: 0.0,
        }
    }

    fn init(&mut self, act: BoxAct) {
        self.act = act;
        match act {
            BoxAct::Wait | BoxAct::WaitDummy => {}
            BoxAct::Wait02 => self.wait = DROP_WAIT,
            BoxAct::WaitGetItem => {
                self.drawn = false;
                self.ripple = false;
            }
            BoxAct::Drop => self.gravity = DROP_GRAVITY,
        }
    }

    fn step(&mut self, actor: &mut Actor, crane: Option<&Crane>, water_y: f32) -> bool {
        match self.act {
            BoxAct::Wait => {
                let Some(crane) = crane else {
                    return false;
                };
                let Some(top) = crane.top else {
                    return false;
                };
                actor.angle[1] = crane.angle;
                actor.pos = top - glam::Vec3::new(0.0, CRANE_OFFSET[self.box_type as usize], 0.0);
                self.attention = actor.pos;
                if !self.jingle && (self.box_type == 1 || self.box_type == 2) {
                    self.jingle = true;
                }
                if !self.ripple {
                    let mut ripple = actor.pos;
                    ripple.y += RIPPLE_OFFSET_Y;
                    ripple.y = water_y;
                    if actor.pos.y > ripple.y - RIPPLE_MARGIN {
                        self.ripple = true;
                    }
                }
                false
            }
            BoxAct::Wait02 => {
                if let Some(crane) = crane {
                    if let Some(top) = crane.top {
                        actor.pos =
                            top - glam::Vec3::new(0.0, CRANE_OFFSET[self.box_type as usize], 0.0);
                    }
                }
                false
            }
            BoxAct::WaitGetItem | BoxAct::WaitDummy => true,
            BoxAct::Drop => {
                actor.pos.y += self.gravity;
                let surface = water_y;
                if actor.pos.y < surface - SINK_MARGIN {
                    return true;
                }
                if actor.pos.y < surface {
                    if !self.splashed {
                        self.splashed = true;
                    }
                    self.ripple = false;
                }
                false
            }
        }
    }
}

pub fn control(
    mut boxes: Query<(&mut Actor, &mut SalvageTbox)>,
    cranes: Query<&Crane>,
    sea: Option<Res<Sea>>,
    info: Res<MsgGameInfo>,
    mut manager: ResMut<EventManager>,
) {
    let crane = cranes.iter().next();
    let staff = manager.get_my_staff_id("STBox", None::<StaffActor>, 0);
    let running = info.event_run_check() && !TALK_COMMAND && staff != -1;
    for (mut actor, mut box_state) in boxes.iter_mut() {
        let ripple_pos = crane.map(|crane| crane.segment);
        let water_y = match ripple_pos {
            Some(pos) => probe(sea.as_deref(), pos.x, pos.z).1,
            None => 0.0,
        };
        if running {
            let idx = manager.get_my_act_idx(staff, &ACTIONS, false, 0);
            if idx == -1 {
                manager.cut_end(staff);
            } else {
                if manager.get_is_advance(staff) {
                    box_state.init(ACTIONS[idx as usize].into());
                }
                let done = box_state.step(&mut actor, crane, water_y);
                if done {
                    manager.cut_end(staff);
                }
            }
        } else {
            box_state.step(&mut actor, crane, water_y);
        }
        if actor.pos.y < water_y {
            box_state.shadow_water_y = water_y + 2.0;
            box_state.shadow_flat_y = water_y + 2.0;
            box_state.shadow_depth = water_y - actor.pos.y;
            if let Some(pos) = ripple_pos {
                box_state.shadow_pos = pos;
            }
        }
        if box_state.ripple {
            box_state.shadow_scroll = SHADOW_SCROLL;
        }
    }
}

impl From<&str> for BoxAct {
    fn from(name: &str) -> Self {
        match name {
            "WAIT02" => BoxAct::Wait02,
            "WAIT_GETITEM" => BoxAct::WaitGetItem,
            "WAIT_DUMMY" => BoxAct::WaitDummy,
            "DROP" => BoxAct::Drop,
            _ => BoxAct::Wait,
        }
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
    let state = SalvageTbox::spawn(source.parameters);
    world.spawn((actor, state)).id()
}

pub fn hook_length() -> f32 {
    ROPE_MAX_LENGTH
}

pub fn emitter_hop(box_type: u32) -> f32 {
    HOP[box_type.min(2) as usize]
}

pub fn emitter_rise(box_type: u32) -> f32 {
    RISE[box_type.min(2) as usize]
}

pub fn emitter_global() -> f32 {
    EMITTER_GLOBAL
}
