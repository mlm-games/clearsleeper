use bevy_ecs::prelude::*;

use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;

pub const HAMI4_ACTORS: [&str; 1] = ["Hami4"];

/// `fopAcM_orderOtherEvent(this, "AMI4_OPEN")` (`d_a_obj_hami4.cpp:84`, `:96`).
const EVENT_OPEN: &str = "AMI4_OPEN";
const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_hami4.h:12 (PRM_SWSAVE_W = 8, PRM_SWSAVE_S = 0)
const STATE_CLOSE_STOP: u8 = 0; // <- d_a_obj_hami4.cpp:171
const STATE_OPEN_WAIT: u8 = 1; // <- d_a_obj_hami4.cpp:174
const STATE_OPEN_DEMO: u8 = 2; // <- d_a_obj_hami4.cpp:177
const STATE_OPEN_STOP: u8 = 3; // <- d_a_obj_hami4.cpp:180
const LIFT_OPEN: f32 = 1500.0; // <- d_a_obj_hami4.cpp:24, :27, :103
const LIFT_STEP: f32 = 10.0; // <- d_a_obj_hami4.cpp:102
const SE_READ_RIDDLE_1: u32 = 0x0806; // <- JAZelAudio_SE.h:18
const SE_SLIDE_AMI_OPEN: u32 = 0x69C7; // <- JAZelAudio_SE.h:1976

/// `fopAcM_orderOtherEvent(this, name)` (`d_a_obj_hami4.cpp:84`, `:96`) reaches
/// `fopAcM_orderOtherEvent2` (`f_op_actor_mng.h:702`) and then
/// `dComIfGp_event_orderOld(dEvtType_OTHER_e, ...)` (`f_op_actor_mng.cpp:692-698`).
fn order(entity: Entity, list: &EventList, active: &mut ActiveEvent, name: &str) {
    let Some(slot) = list.event_index(name) else {
        return;
    };
    // Guarded: re-ordering every tick rewrites mEventState back to ORDER and the
    // demo never starts. Only order when this event holds no stage yet.
    if active.stage_of(slot).is_none() {
        let _ = active.order(list, slot, entity);
    }
}

/// The Outset Island stone slab shutter under the shop of the bombs, split into four quarter
/// models. The stage string is `Hami4` (`d_stage.cpp:873`) and the archive is `Hami4`
/// (`d_a_obj_hami4.cpp:19`, `:44`, `:121`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Hami4 {
    pub switch_no: u8,
    /// `field_0x378` (`d_a_obj_hami4.h:0x378`), the `_execute` switch subject (`:170`).
    pub state: u8,
    /// `field_0x37C` (`d_a_obj_hami4.h:0x37C`), how far the four quarter slabs have travelled.
    pub lift: f32,
}

impl Hami4 {
    /// `CreateHeap` (`d_a_obj_hami4.cpp:22-29`): a switch that is already set at create time
    /// opens the shutter outright, so `field_0x378` starts at 3 and `close_stop` never runs.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = (parameters & SWSAVE_MASK) as u8;
        if switches.test(switch_no) {
            Self {
                switch_no,
                state: STATE_OPEN_STOP,
                lift: LIFT_OPEN,
            }
        } else {
            Self {
                switch_no,
                state: STATE_CLOSE_STOP,
                lift: 0.0,
            }
        }
    }

    /// `_execute` (`d_a_obj_hami4.cpp:169-186`) minus `set_mtx` (`:184`) and the trailing
    /// `return true`. The switch there has no `default` arm; `field_0x378` only ever holds
    /// 0..=3, so the fall-through arm is exactly `case 3`.
    fn step(
        &mut self,
        entity: Entity,
        switches: &Switches,
        list: &EventList,
        active: &mut ActiveEvent,
        events: &mut SimEvents,
    ) {
        match self.state {
            STATE_CLOSE_STOP => self.close_stop(entity, switches, list, active),
            STATE_OPEN_WAIT => self.open_demo_wait(entity, list, active, events),
            STATE_OPEN_DEMO => self.open_demo(),
            _ => self.open_stop(),
        }
    }

    /// `daObjHami4_close_stop` (`d_a_obj_hami4.cpp:81-87`).
    fn close_stop(
        &mut self,
        entity: Entity,
        switches: &Switches,
        list: &EventList,
        active: &mut ActiveEvent,
    ) {
        if switches.test(self.switch_no) {
            order(entity, list, active, EVENT_OPEN);
            self.state = STATE_OPEN_WAIT;
        }
    }

    /// `daObjHami4_open_demo_wait` (`d_a_obj_hami4.cpp:90-98`). `eventInfo.checkCommandDemoAccrpt()`
    /// (`:91`) is `mCommand == dEvtCmd_INDEMO_e` (`f_op_actor.h:177`), which
    /// `dEvt_control_c::demoStart` sets on both actors of the ordered event
    /// (`d_event.cpp:314-316`) - i.e. the AMI4_OPEN event this actor ordered has actually begun,
    /// which is `ActiveEvent::start_check` (the `auction.rs:848-850` mapping).
    fn open_demo_wait(
        &mut self,
        entity: Entity,
        list: &EventList,
        active: &mut ActiveEvent,
        events: &mut SimEvents,
    ) {
        if list
            .event_index(EVENT_OPEN)
            .is_some_and(|slot| active.start_check(slot))
        {
            self.state = STATE_OPEN_DEMO;
            events.list.push(SimEvent::Se(SE_READ_RIDDLE_1));
            events.list.push(SimEvent::Se(SE_SLIDE_AMI_OPEN));
        } else {
            order(entity, list, active, EVENT_OPEN);
        }
    }

    /// `daObjHami4_open_demo` (`d_a_obj_hami4.cpp:101-111`), minus the `StartShock` vibration
    /// (`:107`) and `dComIfGp_event_reset()` (`:109`).
    fn open_demo(&mut self) {
        let lift = self.lift + LIFT_STEP;
        self.lift = lift;
        if lift >= LIFT_OPEN {
            self.lift = LIFT_OPEN;
            self.state = STATE_OPEN_STOP;
        }
    }

    /// `daObjHami4_open_stop` (`d_a_obj_hami4.cpp:114`) has an empty body: once the slab is up it
    /// stays up for the rest of the stage.
    fn open_stop(&self) {}
}

pub fn is_hami4(name: &str) -> bool {
    HAMI4_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut events: ResMut<SimEvents>,
    mut shutters: Query<(Entity, &mut Hami4)>,
) {
    for (entity, mut shutter) in &mut shutters {
        shutter.step(entity, &switches, &list, &mut active, &mut events);
    }
}