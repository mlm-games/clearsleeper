use bevy_ecs::prelude::*;

use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;

pub const HAMI3_ACTORS: [&str; 1] = ["Hami3"];

const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_hami3.h:11-12 (PRM_SWSAVE_W = 0x08, PRM_SWSAVE_S = 0x00)
const SWSAVE2_SHIFT: u32 = 0x08; // <- d_a_obj_hami3.h:14 (PRM_SWSAVE2_S = 0x08)
const SOUND_SHIFT: u32 = 0x10; // <- d_a_obj_hami3.h:17-18 (PRM_SOUND_W = 0x01, PRM_SOUND_S = 0x10)
const SOUND_MASK: u32 = 0x01; // <- d_a_obj_hami3.h:17 (PRM_SOUND_W = 0x01)
const LIFT_OPEN: i16 = 0x4000; // <- d_a_obj_hami3.cpp:68, :143, :144
const LIFT_STEP: i16 = 0x100; // <- d_a_obj_hami3.cpp:141, :175
const EVENT_AMI3: &str = "AMI3"; // <- d_a_obj_hami3.cpp:77 (dComIfGp_evmng_getEventIdx("AMI3"))
const SE_READ_RIDDLE_1: u32 = 0x0806; // <- JAZelAudio_SE.h:18
const SE_OBJ_KAITEN_AMI3_MOVE: u32 = 0x61CB; // <- JAZelAudio_SE.h:1702

pub const CULL_MIN: [f32; 3] = [-1200.0, -1200.0, -1200.0]; // <- d_a_obj_hami3.cpp:76
pub const CULL_MAX: [f32; 3] = [1200.0, 1200.0, 1200.0]; // <- d_a_obj_hami3.cpp:76

/// `daObjHami3::Act_c::field_0x2D8` (`d_a_obj_hami3.cpp:190`), the six-state open/close machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    CloseStop,
    OpenDemoWait,
    OpenDemo,
    OpenStop,
    CloseDemoWait,
    CloseDemo,
}

/// The Great Sea temple rotating platform that opens when the pictogram is returned. The stage
/// string is `Hami3` (`d_stage.cpp:872`) and the archive is `Hami3`
/// (`d_a_obj_hami3.cpp:14`).
///
/// `M_evname` (`:17`, `d_a_obj_hami3.h:43`) is the `"ami_cam"` string. Nothing in
/// `d_a_obj_hami3.cpp` ever reads it - it is declared and never used, so it is not ported.
///
/// `daObjHami3::Act_c` registers no `dCcD` collider: `Create` (`:62-79`) touches only the switch,
/// `shape_angle`, the cull box and the event index, and `MoveBGCreate` (`:86`) is the moving
/// background. So this actor has no inbound damage path.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hami3 {
    /// `prm_get_swSave` (`d_a_obj_hami3.h:21`): `(param >> 0) & 0xFF`.
    pub switch_no: u8,
    /// `prm_get_swSave2` (`d_a_obj_hami3.h:22`): `(param >> 8) & 0xFF`, the switch `open_demo`
    /// (`:149-150`) sets once the platform is fully up.
    pub switch_no_2: u8,
    /// `prm_get_sound` (`d_a_obj_hami3.h:20`): `(param >> 0x10) & 0x01`; `open_demo_wait` plays
    /// the riddle only when it is 0 (`:130-132`).
    pub sound: u8,
    /// `field_0x2D8` (`d_a_obj_hami3.h:51`).
    pub mode: Mode,
    /// `field_0x2C8` (`d_a_obj_hami3.h:47`), the arm's Y rotation in 16-bit units.
    pub lift: i16,
    /// `field_0x2DC` (`d_a_obj_hami3.h:52`), the `AMI3` event index; `None` is `getEventIdx`'s -1.
    pub event_slot: Option<usize>,
    pub entered: bool,
}

impl Hami3 {
    /// `Create` (`d_a_obj_hami3.cpp:62-79`), minus `fopAcM_SetMtx`, `init_mtx`,
    /// `fopAcM_setCullSizeBox` and the `field_0x2DC` lookup, which needs the stage event list and
    /// so happens on the first tick. The parameter decode is `daObj::PrmAbstract`
    /// (`d_a_obj.h:12-15`), i.e. `((1 << W) - 1) & (param >> S)`.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = (parameters & SWSAVE_MASK) as u8;
        let (mode, lift) = if switches.test(switch_no) {
            (Mode::OpenStop, LIFT_OPEN)
        } else {
            (Mode::CloseStop, 0)
        };
        Self {
            switch_no,
            switch_no_2: ((parameters >> SWSAVE2_SHIFT) & SWSAVE_MASK) as u8,
            sound: ((parameters >> SOUND_SHIFT) & SOUND_MASK) as u8,
            mode,
            lift,
            event_slot: None,
            entered: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `field_0x2DC = dComIfGp_evmng_getEventIdx("AMI3")` (`:77`). `getEventIdx` is called with the
    /// default `evNo = -1` (`d_com_inf_game.h:3611`), which is not the `0xff` of the stage
    /// event-info redirect at `d_event_manager.cpp:347`, so the lookup is by name alone.
    fn create_init(&mut self, list: &EventList) {
        self.event_slot = list.event_index(EVENT_AMI3);
    }

    /// `fopAcM_orderOtherEventId(this, field_0x2DC, 0xff)` (`:122`, `:135`, `:159`, `:169`), guarded
    /// as in `vfan::Vfan::order`.
    fn order(&self, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        let Some(slot) = self.event_slot else {
            return;
        };
        if active.stage_of(slot).is_none() {
            let _ = active.order(list, slot, entity);
        }
    }

    /// `Execute` (`d_a_obj_hami3.cpp:189-214`), minus `shape_angle.x = home.angle.x +
    /// field_0x2C8` (`:210`), which feeds only `set_mtx`'s `ZXYrotM` (`:107`), and `set_mtx`
    /// (`:211`) and the `*mtx` store (`:212`). The switch has no `default` arm and `field_0x2D8`
    /// only ever holds 0..=5, so the fall-through arm is exactly `case 5`.
    fn execute(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        switches: &mut Switches,
        events: &mut SimEvents,
        entity: Entity,
    ) {
        match self.mode {
            Mode::CloseStop => self.close_stop(list, active, switches, entity),
            Mode::OpenDemoWait => self.open_demo_wait(list, active, events, entity),
            Mode::OpenDemo => self.open_demo(switches, events),
            Mode::OpenStop => self.open_stop(list, active, switches, entity),
            Mode::CloseDemoWait => self.close_demo_wait(list, active, entity),
            Mode::CloseDemo => self.close_demo(events),
        }
    }

    /// `daObjHami3_close_stop` (`d_a_obj_hami3.cpp:119-125`).
    fn close_stop(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        switches: &Switches,
        entity: Entity,
    ) {
        if switches.test(self.switch_no) {
            self.order(list, active, entity);
            self.mode = Mode::OpenDemoWait;
        }
    }

    /// `daObjHami3_open_demo_wait` (`:128-137`). `eventInfo.checkCommandDemoAccrpt()` (`:129`) is
    /// `mCommand == dEvtCmd_INDEMO_e` (`f_op_actor.h:177`), which `dEvt_control_c::demoCheck` sets
    /// on both ordered actors when the event starts (`d_event.cpp:314-316`) - `ActiveEvent::
    /// start_check`, the mapping `hami4.rs:108-110` documents.
    fn open_demo_wait(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        events: &mut SimEvents,
        entity: Entity,
    ) {
        if self.event_slot.is_some_and(|slot| active.start_check(slot)) {
            if self.sound == 0 {
                events.list.push(SimEvent::Se(SE_READ_RIDDLE_1));
            }
            self.mode = Mode::OpenDemo;
        } else {
            self.order(list, active, entity);
        }
    }

    /// `daObjHami3_open_demo` (`:140-153`), minus the `StartShock` rumble (`:146`) and
    /// `dComIfGp_event_reset()` (`:148`).
    fn open_demo(&mut self, switches: &mut Switches, events: &mut SimEvents) {
        self.lift = self.lift.wrapping_add(LIFT_STEP);
        events.list.push(SimEvent::Se(SE_OBJ_KAITEN_AMI3_MOVE));
        if self.lift >= LIFT_OPEN {
            self.lift = LIFT_OPEN;
            switches.set(self.switch_no_2);
            self.mode = Mode::OpenStop;
        }
    }

    /// `daObjHami3_open_stop` (`:156-162`): the platform stays up until the pictogram switch is
    /// cleared, at which point it orders the demo again and starts closing.
    fn open_stop(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        switches: &Switches,
        entity: Entity,
    ) {
        if !switches.test(self.switch_no) {
            self.order(list, active, entity);
            self.mode = Mode::CloseDemoWait;
        }
    }

    /// `daObjHami3_close_demo_wait` (`:165-171`).
    fn close_demo_wait(&mut self, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        if self.event_slot.is_some_and(|slot| active.start_check(slot)) {
            self.mode = Mode::CloseDemo;
        } else {
            self.order(list, active, entity);
        }
    }

    /// `daObjHami3_close_demo` (`:174-186`), minus the `StartShock` rumble (`:181`) and
    /// `dComIfGp_event_reset()` (`:183`).
    fn close_demo(&mut self, events: &mut SimEvents) {
        self.lift = self.lift.wrapping_sub(LIFT_STEP);
        events.list.push(SimEvent::Se(SE_OBJ_KAITEN_AMI3_MOVE));
        if self.lift <= 0 {
            self.lift = 0;
            self.mode = Mode::CloseStop;
        }
    }
}

pub fn is_hami3(name: &str) -> bool {
    HAMI3_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut temples: Query<(Entity, &mut Hami3)>,
) {
    for (entity, mut temple) in &mut temples {
        let temple = &mut *temple;
        if !temple.entered {
            temple.entered = true;
            temple.create_init(&list);
        }
        temple.execute(&list, &mut active, &mut switches, &mut events, entity);
    }
}
