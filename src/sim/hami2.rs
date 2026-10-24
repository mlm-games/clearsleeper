use bevy_ecs::prelude::*;
use glam::Vec3;

use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::vibration::Vibration;

pub const HAMI2_ACTORS: [&str; 1] = ["Hami2"];

/// `M_evname[] = "ami_cam"` (`d_a_obj_hami2.cpp:22`) is declared but never read by the TU; the
/// two events the actor actually opens and closes are the named lookups at `:93-94`.
const EVENT_OPEN: &str = "AMI2_OPEN"; // <- d_a_obj_hami2.cpp:93
const EVENT_CLOSE: &str = "AMI2_CLOSE"; // <- d_a_obj_hami2.cpp:94

/// `enum Prm_e { PRM_SWSAVE_W = 8, PRM_SWSAVE_S = 0 }` (`d_a_obj_hami2.h:9`), fed through
/// `daObj::PrmAbstract` (`d_a_obj.h:14-17`).
const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_hami2.h:9

/// `mGateAngle = 0x4000` (`:37`) - the fully open quarter turn, in 16-bit angle units.
const GATE_OPEN: i16 = 0x4000; // <- d_a_obj_hami2.cpp:37
/// `mGateAngle += 0x100` (`:112`) / `mGateAngle -= 0x100` (`:141`) - one tick of the swing.
const GATE_STEP: i16 = 0x100; // <- d_a_obj_hami2.cpp:112
/// `fopAcM_setCullSizeBox(this, -1200.0f, -40000.0f, -1200.0f, 1200.0f, 40000.0f, 1200.0f)`
/// (`:92`).
const CULL_MIN: [f32; 3] = [-1200.0, -40000.0, -1200.0]; // <- d_a_obj_hami2.cpp:92
const CULL_MAX: [f32; 3] = [1200.0, 40000.0, 1200.0]; // <- d_a_obj_hami2.cpp:92

/// `JA_SE_OBJ_KAITEN_AMI_OPEN = 0x69C4` (`JAZelAudio_SE.h:1973`), started at `:104`.
const SE_KAITEN_AMI_OPEN: u32 = 0x69C4; // <- JAZelAudio_SE.h:1973
/// `JA_SE_READ_RIDDLE_1 = 0x0806` (`JAZelAudio_SE.h:18`), started at `:105`.
const SE_READ_RIDDLE: u32 = 0x0806; // <- JAZelAudio_SE.h:18
/// `dComIfGp_getVibration().StartShock(4, -33, cXyz(0.0f, 1.0f, 0.0f))` (`:115`, `:144`).
const SHOCK_PATTERN: i32 = 4; // <- d_a_obj_hami2.cpp:115
const SHOCK_FLAGS: i32 = -33; // <- d_a_obj_hami2.cpp:115
const SHOCK_COORD: [f32; 3] = [0.0, 1.0, 0.0]; // <- d_a_obj_hami2.cpp:115

/// `mState` (`d_a_obj_hami2.h:50`): the six labels of `Execute`'s switch (`:155-184`).
const STATE_CLOSE_STOP: u8 = 0; // <- d_a_obj_hami2.cpp:160
const STATE_OPEN_WAIT: u8 = 1; // <- d_a_obj_hami2.cpp:161
const STATE_OPEN_DEMO: u8 = 2; // <- d_a_obj_hami2.cpp:162
const STATE_OPEN_STOP: u8 = 3; // <- d_a_obj_hami2.cpp:163
const STATE_CLOSE_WAIT: u8 = 4; // <- d_a_obj_hami2.cpp:164
const STATE_CLOSE_DEMO: u8 = 5; // <- d_a_obj_hami2.cpp:165

/// A rotating gate - `daObjHami2::Act_c` (`d_a_obj_hami2.h:5-55`), the stage string `Hami2`
/// (`d_stage.cpp:873`). Its `mono1` joint rotates by `mGateAngle` in the model callback
/// (`d_a_obj_hami2.cpp:25-40`); the sim carries that angle and leaves the joint matrix to the
/// renderer.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `CreateHeap` (`:48-89`) builds a
/// `J3DModel` and two `dBgW`s; there is no `dCcD` collider anywhere in the TU and the profile
/// `Group` is `fopAc_ACTOR_e` (`:224`). Neither `HitPoints` nor `Hurt` is attached.
///
/// `Mthd_Delete` (`:127-138`) releases the background and the archive; `Delete` (`:124-126`)
/// returns `TRUE`. There is no refusal path in `Mthd_Create` beyond resource/heap load
/// (`:107-121`), so `spawn` never refuses.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Hami2 {
    /// `prm_get_swSave()` (`d_a_obj_hami2.h:10`), the switch that opens the gate.
    pub switch_no: u8,
    /// `mState` (`d_a_obj_hami2.h:50`).
    pub state: u8,
    /// `mGateAngle` (`d_a_obj_hami2.h:45`), the gate's Y rotation in 16-bit units.
    pub gate_angle: i16,
}

impl Hami2 {
    /// `Create` (`d_a_obj_hami2.cpp:91-106`), minus `fopAcM_SetMtx`, `init_mtx`, the cull box
    /// (carried by [`Hami2::cull`]) and the two event lookups, which happen on use.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = (parameters & SWSAVE_MASK) as u8;
        if switches.test(switch_no) {
            Self {
                switch_no,
                state: STATE_OPEN_STOP,
                gate_angle: GATE_OPEN,
            }
        } else {
            Self {
                switch_no,
                state: STATE_CLOSE_STOP,
                gate_angle: 0,
            }
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `fopAcM_orderOtherEventId(this, mOpenEvent)` / `mCloseEvent` (`:96`, `:109`, `:122`,
    /// `:130`). Guarded like `hami4.rs`: re-ordering a slot that already holds a stage rewrites
    /// its state (`d_event_manager.cpp:367`).
    fn order(&self, name: &str, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        let Some(slot) = list.event_index(name) else {
            return;
        };
        if active.stage_of(slot).is_none() {
            let _ = active.order(list, slot, entity);
        }
    }

    /// `eventInfo.checkCommandDemoAccrpt()` (`:102`, `:128`) - the event this actor ordered has
    /// begun, which is `ActiveEvent::start_check` on its slot (`hami4.rs:100-103`).
    fn running(name: &str, list: &EventList, active: &ActiveEvent) -> Option<usize> {
        let slot = list.event_index(name)?;
        active.start_check(slot).then_some(slot)
    }

    fn execute(
        &mut self,
        entity: Entity,
        switches: &Switches,
        list: &EventList,
        active: &mut ActiveEvent,
        events: &mut SimEvents,
        vibration: &mut Vibration,
    ) {
        match self.state {
            STATE_CLOSE_STOP => {
                // `daObjHami2_close_stop` (`:145-151`)
                if switches.test(self.switch_no) {
                    self.order(EVENT_OPEN, list, active, entity);
                    self.state = STATE_OPEN_WAIT;
                }
            }
            STATE_OPEN_WAIT => {
                // `daObjHami2_open_demo_wait` (`:154-164`)
                if Self::running(EVENT_OPEN, list, active).is_some() {
                    self.state = STATE_OPEN_DEMO;
                    events.list.push(SimEvent::Se(SE_KAITEN_AMI_OPEN));
                    events.list.push(SimEvent::Se(SE_READ_RIDDLE));
                } else {
                    self.order(EVENT_OPEN, list, active, entity);
                }
            }
            STATE_OPEN_DEMO => {
                // `daObjHami2_open_demo` (`:167-178`)
                self.gate_angle = self.gate_angle.wrapping_add(GATE_STEP);
                if self.gate_angle >= GATE_OPEN {
                    self.gate_angle = GATE_OPEN;
                    self.state = STATE_OPEN_STOP;
                    vibration.start_shock(
                        SHOCK_PATTERN,
                        SHOCK_FLAGS,
                        Vec3::from_array(SHOCK_COORD),
                    );
                    if let Some(slot) = list.event_index(EVENT_OPEN) {
                        active.end(slot);
                    }
                }
            }
            STATE_OPEN_STOP => {
                // `daObjHami2_open_stop` (`:181-186`)
                if !switches.test(self.switch_no) {
                    self.order(EVENT_CLOSE, list, active, entity);
                    self.state = STATE_CLOSE_WAIT;
                }
            }
            STATE_CLOSE_WAIT => {
                // `daObjHami2_close_demo_wait` (`:189-196`)
                if Self::running(EVENT_CLOSE, list, active).is_some() {
                    self.state = STATE_CLOSE_DEMO;
                } else {
                    self.order(EVENT_CLOSE, list, active, entity);
                }
            }
            STATE_CLOSE_DEMO => {
                // `daObjHami2_close_demo` (`:199-211`)
                self.gate_angle = self.gate_angle.wrapping_sub(GATE_STEP);
                if self.gate_angle <= 0 {
                    self.gate_angle = 0;
                    vibration.start_shock(
                        SHOCK_PATTERN,
                        SHOCK_FLAGS,
                        Vec3::from_array(SHOCK_COORD),
                    );
                    if let Some(slot) = list.event_index(EVENT_CLOSE) {
                        active.end(slot);
                    }
                    self.state = STATE_CLOSE_STOP;
                }
            }
            _ => {}
        }
    }
}

pub fn is_hami2(name: &str) -> bool {
    HAMI2_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut events: ResMut<SimEvents>,
    mut vibration: ResMut<Vibration>,
    mut gates: Query<(Entity, &mut Hami2)>,
) {
    for (entity, mut gate) in &mut gates {
        gate.execute(
            entity,
            &switches,
            &list,
            &mut active,
            &mut events,
            &mut vibration,
        );
    }
}
